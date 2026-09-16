use std::{
    any::Any,
    error::Error,
    fmt::{self, Display},
    io,
    path::Path,
    sync::LazyLock,
};

use color_eyre::eyre;
use regex::Regex;

const SERVER_URL_HINT: &str = "Check the server URL (--server-url or server-base in the config).";
const NETWORK_HINT: &str = "Check the server URL and your network connection.";
const UNKNOWN_ERROR: &str = "An unknown error occurred.";
const MAX_MESSAGE_CHARS: usize = 160;

static OS_ERROR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r" ?\(os error -?[0-9]+\)").expect("OS_ERROR_RE to be valid"));
static USERINFO_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([A-Za-z][A-Za-z0-9+.-]*://)[^/?#@ ]+@").expect("USERINFO_RE to be valid")
});
static HTTP_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)^Received error message from server: \[(([0-9]{3})[^\]]*)\] ?(.*)$")
        .expect("HTTP_RE to be valid")
});

/// An error with a message and optional hint that are shown to the user as they are.
#[derive(Debug)]
pub(crate) struct UserError {
    message: String,
    hint: Option<String>,
    source: Option<Box<dyn Error + Send + Sync + 'static>>,
}

impl UserError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            hint: None,
            source: None,
        }
    }

    pub(crate) fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub(crate) fn source(mut self, e: impl Error + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(e));
        self
    }

    /// `<action> '<path>': <io reason>.`
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "For call sites that classify their own errors")
    )]
    pub(crate) fn io(action: &str, path: &Path, e: io::Error) -> Self {
        let message = format!("{action} '{}': {}.", path.display(), io_reason(&e));
        Self::new(message).source(e)
    }
}

impl Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for UserError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_deref().map(|e| e as &(dyn Error + 'static))
    }
}

/// Renders an error as `Error: <message>` with an optional `Hint: <hint>` line.
pub(crate) fn render(report: &eyre::Report) -> String {
    let UserError { message, hint, .. } = classify(report);
    match hint {
        Some(hint) => format!("Error: {message}\nHint: {hint}\n"),
        None => format!("Error: {message}\n"),
    }
}

pub(crate) fn classify(report: &eyre::Report) -> UserError {
    if let Some(e) = report.downcast_ref::<UserError>() {
        return UserError {
            message: e.message.clone(),
            hint: e.hint.clone(),
            source: None,
        };
    }
    let err: &(dyn Error + 'static) = report.as_ref();
    generic(err).unwrap_or_else(|| UserError::new(sentence(&report.to_string())))
}

/// Classifies network and HTTP errors that need no call-site context.
pub(crate) fn generic(err: &(dyn Error + 'static)) -> Option<UserError> {
    let chain: Vec<&(dyn Error + 'static)> =
        std::iter::successors(Some(err), |e| (*e).source()).collect();
    let texts: Vec<String> = chain.iter().map(|e| e.to_string()).collect();
    let top = texts.first().map(String::as_str).unwrap_or_default();
    let io_kind = |kind: io::ErrorKind| {
        chain.iter().any(|e| {
            e.downcast_ref::<io::Error>()
                .is_some_and(|e| e.kind() == kind)
        })
    };
    let host = || host_from_chain(&texts);

    // The response body is part of the message, so match HTTP errors before the text heuristics.
    if let Some(c) = HTTP_RE.captures(top) {
        let reason = c.get(1).map_or("", |m| m.as_str());
        let status: u16 = c.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
        let body = c.get(3).map_or("", |m| m.as_str());
        return Some(http_error(status, reason, body));
    }

    if io_kind(io::ErrorKind::ConnectionRefused)
        || texts.iter().any(|t| t.contains("Connection refused"))
    {
        return Some(
            UserError::new(format!(
                "Could not connect to {}: connection refused.",
                host()
            ))
            .hint(SERVER_URL_HINT),
        );
    }
    if texts
        .iter()
        .any(|t| t == "dns error" || t.starts_with("failed to lookup address"))
    {
        return Some(
            UserError::new(format!("Could not resolve host {}.", host())).hint(NETWORK_HINT),
        );
    }
    if texts.iter().any(|t| {
        t.contains("invalid peer certificate") || (t.contains("certificate") && t.contains("verif"))
    }) {
        return Some(UserError::new(format!(
            "The TLS certificate of {} is not trusted.",
            host()
        )));
    }
    if io_kind(io::ErrorKind::TimedOut) || texts.iter().any(|t| t.contains("timed out")) {
        return Some(
            UserError::new(format!("Connection to {} timed out.", host()))
                .hint("Check your network connection."),
        );
    }
    if top.starts_with("builder error") {
        return Some(UserError::new("The server URL is invalid.").hint(SERVER_URL_HINT));
    }
    if top.starts_with("error sending request") || top.starts_with("error decoding response body") {
        return Some(UserError::new(format!("Could not reach {}.", host())).hint(NETWORK_HINT));
    }
    if top.contains("content type response when JSON was expected") {
        return Some(UserError::new("Unexpected response from the server.").hint(SERVER_URL_HINT));
    }
    None
}

fn http_error(status: u16, reason: &str, body: &str) -> UserError {
    match status {
        400 => match server_message(body) {
            Some(message) => UserError::new(message),
            None => UserError::new(format!("The server rejected the request ({reason}).")),
        },
        401 => UserError::new(format!("The server rejected the access token ({reason}).")).hint(
            "The token may be expired or revoked; create a new one in the Bitwarden web app.",
        ),
        403 => UserError::new(format!("Access denied ({reason})."))
            .hint("Check the machine account's permissions."),
        404 => {
            let e = UserError::new("The requested item was not found or is not accessible.");
            match server_message(body) {
                Some(_) => e,
                None => e.hint(SERVER_URL_HINT),
            }
        }
        429 => UserError::new(format!("Too many requests ({reason})."))
            .hint("Wait a moment and try again."),
        500..=599 => UserError::new(format!("The server returned an error ({reason})."))
            .hint("Try again later."),
        _ => UserError::new(format!("Unexpected response from the server ({reason}).")),
    }
}

/// Extracts `message` and `validationErrors` from a Bitwarden server JSON error body.
fn server_message(body: &str) -> Option<String> {
    let json: serde_json::Value = serde_json::from_str(body).ok()?;
    let message = json.get("message")?.as_str()?.trim();
    if message.is_empty() {
        return None;
    }
    let mut text = message.trim_end_matches(['.', ':', ' ']).to_string();

    let details: Vec<&str> = json
        .get("validationErrors")
        .and_then(|v| v.as_object())
        .into_iter()
        .flat_map(|o| o.values())
        .flat_map(|v| match v {
            serde_json::Value::Array(items) => items.iter().filter_map(|i| i.as_str()).collect(),
            serde_json::Value::String(s) => vec![s.as_str()],
            _ => vec![],
        })
        .map(|s| s.trim().trim_end_matches(['.', ' ']))
        .filter(|s| !s.is_empty())
        .collect();
    if !details.is_empty() {
        text.push_str(": ");
        text.push_str(&details.join("; "));
    }
    Some(sentence(&text))
}

/// Returns `host[:port]` of the first URL mentioned in the chain, without scheme or userinfo.
fn host_from_chain(texts: &[String]) -> String {
    texts
        .iter()
        .find_map(|t| {
            let start = t.find("for url (")? + "for url (".len();
            let rest = &t[start..];
            let url = &rest[..rest.find(')').unwrap_or(rest.len())];
            let url = url.split_once("://").map_or(url, |(_, r)| r);
            let authority = &url[..url.find(['/', '?', '#']).unwrap_or(url.len())];
            let host = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
            (!host.is_empty()).then(|| host.to_string())
        })
        .unwrap_or_else(|| "the server".to_string())
}

pub(crate) fn is_verbose() -> bool {
    let var = |name| std::env::var_os(name).map(|v| v.to_string_lossy().into_owned());
    verbose_from(
        var("BWS_DEBUG").as_deref(),
        var("RUST_BACKTRACE").as_deref(),
    )
}

fn verbose_from(bws_debug: Option<&str>, rust_backtrace: Option<&str>) -> bool {
    let debug = bws_debug.is_some_and(|v| {
        let v = v.trim().to_ascii_lowercase();
        !matches!(v.as_str(), "" | "0" | "false")
    });
    debug || rust_backtrace.is_some_and(|v| v != "0")
}

#[expect(dead_code, reason = "For call sites that report recoverable problems")]
pub(crate) fn warn(message: &str, hint: Option<&str>) {
    eprintln!("Warning: {message}");
    if let Some(hint) = hint {
        eprintln!("Hint: {hint}");
    }
}

pub(crate) fn render_panic(payload: &(dyn Any + Send)) -> String {
    let text = payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("unknown error");
    let message = sentence(text);
    format!(
        "Error: bws crashed unexpectedly: {}.\nHint: Rerun with BWS_DEBUG=1 and report it at https://github.com/bitwarden/sdk-sm/issues\n",
        message.trim_end_matches('.')
    )
}

fn sentence(s: &str) -> String {
    let line = s
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("");
    let line = line.split_whitespace().collect::<Vec<_>>().join(" ");
    let line = OS_ERROR_RE.replace_all(&line, "");
    let line = USERINFO_RE.replace_all(&line, "$1");

    let truncated = line.chars().count() > MAX_MESSAGE_CHARS;
    let mut text: String = line.chars().take(MAX_MESSAGE_CHARS).collect();
    text = uppercase_first(&text);

    if truncated {
        // The ellipsis would be eaten by the trailing punctuation rule below.
        return format!("{}...", text.trim_end());
    }
    let text = text.trim_end_matches(['.', ':', ' ']);
    if text.is_empty() {
        return UNKNOWN_ERROR.to_string();
    }
    format!("{text}.")
}

fn uppercase_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn io_reason(e: &io::Error) -> String {
    use io::ErrorKind::*;
    match e.kind() {
        PermissionDenied => "permission denied".to_string(),
        NotFound => "not found".to_string(),
        IsADirectory => "is a directory".to_string(),
        NotADirectory => "not a directory".to_string(),
        AlreadyExists => "already exists".to_string(),
        ReadOnlyFilesystem => "read-only file system".to_string(),
        _ => {
            let text = OS_ERROR_RE.replace_all(&e.to_string(), "").into_owned();
            let mut chars = text.chars();
            match chars.next() {
                Some(first) => first.to_lowercase().chain(chars).collect(),
                None => text,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct ChainErr(String, Option<Box<dyn Error + Send + Sync>>);

    impl Display for ChainErr {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(&self.0)
        }
    }

    impl Error for ChainErr {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            self.1.as_deref().map(|e| e as &(dyn Error + 'static))
        }
    }

    fn chain(links: &[&str], root: io::Error) -> ChainErr {
        let mut current: Box<dyn Error + Send + Sync> = Box::new(root);
        for link in links.iter().rev() {
            current = Box::new(ChainErr(link.to_string(), Some(current)));
        }
        *current
            .downcast::<ChainErr>()
            .expect("chain to have at least one link")
    }

    fn classify_text(top: &str) -> UserError {
        generic(&ChainErr(top.to_string(), None)).expect("error to be classified")
    }

    fn lines(e: &UserError) -> (String, Option<String>) {
        (e.message.clone(), e.hint.clone())
    }

    #[test]
    fn sentence_formats_text() {
        assert_eq!(sentence("\n  first line  \nsecond"), "First line.");
        assert_eq!(sentence("a   b\tc"), "A b c.");
        assert_eq!(
            sentence("Connection refused (os error 61)"),
            "Connection refused."
        );
        assert_eq!(
            sentence("error for url (https://user:pass@example.com/x)"),
            "Error for url (https://example.com/x)."
        );
        assert_eq!(sentence("done.:  "), "Done.");
        assert_eq!(sentence(""), UNKNOWN_ERROR);
        assert_eq!(sentence(" ... "), UNKNOWN_ERROR);

        let long = "x".repeat(200);
        let out = sentence(&long);
        assert_eq!(out, format!("X{}...", "x".repeat(159)));
    }

    #[test]
    fn io_reason_maps_kinds() {
        let cases = [
            (io::ErrorKind::PermissionDenied, "permission denied"),
            (io::ErrorKind::NotFound, "not found"),
            (io::ErrorKind::IsADirectory, "is a directory"),
            (io::ErrorKind::NotADirectory, "not a directory"),
            (io::ErrorKind::AlreadyExists, "already exists"),
            (io::ErrorKind::ReadOnlyFilesystem, "read-only file system"),
        ];
        for (kind, expected) in cases {
            assert_eq!(io_reason(&io::Error::from(kind)), expected);
        }
        assert_eq!(
            io_reason(&io::Error::other("Disk on fire (os error 5)")),
            "disk on fire"
        );
    }

    #[test]
    fn user_error_io() {
        let e = UserError::io(
            "Could not read config file",
            Path::new("/tmp/config"),
            io::Error::from(io::ErrorKind::PermissionDenied),
        );
        assert_eq!(
            e.to_string(),
            "Could not read config file '/tmp/config': permission denied."
        );
        assert!(Error::source(&e).is_some());
    }

    #[test]
    fn generic_connection_refused() {
        let err = chain(
            &[
                "error sending request for url (http://u:p@127.0.0.1:9/identity/connect/token)",
                "client error (Connect)",
                "tcp connect error",
            ],
            io::Error::new(io::ErrorKind::ConnectionRefused, "boom"),
        );
        let e = generic(&err).expect("classified");
        assert_eq!(
            lines(&e),
            (
                "Could not connect to 127.0.0.1:9: connection refused.".to_string(),
                Some(SERVER_URL_HINT.to_string())
            )
        );
    }

    #[test]
    fn generic_dns() {
        let err = chain(
            &[
                "error sending request for url (https://nonexistent.invalid/identity/connect/token)",
                "client error (Connect)",
                "dns error",
            ],
            io::Error::other(
                "failed to lookup address information: nodename nor servname provided",
            ),
        );
        let e = generic(&err).expect("classified");
        assert_eq!(e.message, "Could not resolve host nonexistent.invalid.");
        assert_eq!(e.hint.as_deref(), Some(NETWORK_HINT));
    }

    #[test]
    fn generic_tls_timeout_builder_content_type() {
        let tls = chain(
            &["error sending request for url (https://example.com:8443/api)"],
            io::Error::other("invalid peer certificate: UnknownIssuer"),
        );
        assert_eq!(
            generic(&tls).expect("classified").message,
            "The TLS certificate of example.com:8443 is not trusted."
        );

        let timeout = chain(
            &["error sending request for url (https://example.com/api?x=1)"],
            io::Error::from(io::ErrorKind::TimedOut),
        );
        assert_eq!(
            lines(&generic(&timeout).expect("classified")),
            (
                "Connection to example.com timed out.".to_string(),
                Some("Check your network connection.".to_string())
            )
        );

        assert_eq!(
            classify_text("builder error").message,
            "The server URL is invalid."
        );
        assert_eq!(
            classify_text("error sending request").message,
            "Could not reach the server."
        );
        assert_eq!(
            classify_text("Received unexpected content type response when JSON was expected")
                .message,
            "Unexpected response from the server."
        );
    }

    #[test]
    fn generic_http() {
        let http = |status: &str, body: &str| {
            classify_text(&format!(
                "Received error message from server: [{status}] {body}"
            ))
        };

        let e = http(
            "400 Bad Request",
            r#"{"message":"The model state is invalid.","validationErrors":{"Name":["The Name field is required."]},"object":"error"}"#,
        );
        assert_eq!(
            lines(&e),
            (
                "The model state is invalid: The Name field is required.".to_string(),
                None
            )
        );
        assert_eq!(
            http("400 Bad Request", "").message,
            "The server rejected the request (400 Bad Request)."
        );
        assert_eq!(
            http("400 Bad Request", "<html>bad</html>").message,
            "The server rejected the request (400 Bad Request)."
        );

        let e = http("401 Unauthorized", "{}");
        assert_eq!(
            e.message,
            "The server rejected the access token (401 Unauthorized)."
        );
        assert!(e.hint.is_some());

        assert_eq!(
            lines(&http("403 Forbidden", "")),
            (
                "Access denied (403 Forbidden).".to_string(),
                Some("Check the machine account's permissions.".to_string())
            )
        );
        assert_eq!(
            lines(&http(
                "404 Not Found",
                r#"{"message":"Resource not found."}"#
            )),
            (
                "The requested item was not found or is not accessible.".to_string(),
                None
            )
        );
        assert_eq!(
            lines(&http("429 Too Many Requests", "")),
            (
                "Too many requests (429 Too Many Requests).".to_string(),
                Some("Wait a moment and try again.".to_string())
            )
        );

        let e = http(
            "500 Internal Server Error",
            "<html>\n<body>oops</body>\n</html>",
        );
        assert_eq!(
            lines(&e),
            (
                "The server returned an error (500 Internal Server Error).".to_string(),
                Some("Try again later.".to_string())
            )
        );
        assert!(!e.message.contains("<html"));

        assert_eq!(
            http("418 I'm a teapot", "").message,
            "Unexpected response from the server (418 I'm a teapot)."
        );
    }

    #[test]
    fn generic_http_body_does_not_trigger_network_heuristics() {
        let e = classify_text(
            "Received error message from server: [504 Gateway Timeout] <html>upstream timed out</html>",
        );
        assert_eq!(
            e.message,
            "The server returned an error (504 Gateway Timeout)."
        );

        let e = classify_text("Received error message from server: [404 Not Found] <html></html>");
        assert_eq!(e.hint.as_deref(), Some(SERVER_URL_HINT));
    }

    #[test]
    fn generic_ignores_unknown_errors() {
        assert!(generic(&ChainErr("Missing access token".to_string(), None)).is_none());
    }

    #[test]
    fn render_formats_lines() {
        let report = eyre::Report::new(UserError::new("Something failed.").hint("Do this."));
        assert_eq!(
            render(&report),
            "Error: Something failed.\nHint: Do this.\n"
        );

        let report = eyre::Report::new(UserError::new("Something failed."));
        assert_eq!(render(&report), "Error: Something failed.\n");

        let report = eyre::eyre!("Doesn't contain a decryption key");
        assert_eq!(
            render(&report),
            "Error: Doesn't contain a decryption key.\n"
        );

        let report = eyre::eyre!(
            "TOML parse error at line 1, column 6\n  |\n1 | hello\n  |      ^\nkey with no value, expected `=`\n"
        );
        assert_eq!(
            render(&report),
            "Error: TOML parse error at line 1, column 6.\n"
        );
    }

    #[test]
    fn render_panic_message() {
        let payload: Box<dyn Any + Send> = Box::new("Input is valid: broken pipe.");
        assert_eq!(
            render_panic(payload.as_ref()),
            "Error: bws crashed unexpectedly: Input is valid: broken pipe.\nHint: Rerun with BWS_DEBUG=1 and report it at https://github.com/bitwarden/sdk-sm/issues\n"
        );
        let payload: Box<dyn Any + Send> = Box::new(42);
        assert!(
            render_panic(payload.as_ref())
                .starts_with("Error: bws crashed unexpectedly: Unknown error.\n")
        );
    }

    #[test]
    fn verbose_env_parsing() {
        assert!(!verbose_from(None, None));
        assert!(!verbose_from(Some(""), None));
        assert!(!verbose_from(Some("0"), None));
        assert!(!verbose_from(Some("FALSE"), None));
        assert!(verbose_from(Some("1"), None));
        assert!(verbose_from(Some("yes"), None));
        assert!(!verbose_from(None, Some("0")));
        assert!(verbose_from(None, Some("1")));
        assert!(verbose_from(None, Some("full")));
    }
}
