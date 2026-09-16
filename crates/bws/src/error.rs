use std::{any::Any, error::Error, io, path::Path, sync::LazyLock};

use bitwarden_core::{ApiError, auth::login::LoginError};
use bitwarden_crypto::CryptoError;
use color_eyre::eyre;
use regex::Regex;
use uuid::Uuid;

const HINT_COPY_TOKEN: &str = "Copy the full access token from the Bitwarden web app.";
const SERVER_URL_HINT: &str = "Check the server URL (--server-url or server-base in the config).";
const NETWORK_HINT: &str = "Check the server URL and your network connection.";
const TLS_HINT: &str = "Check the server URL or install the server's CA certificate.";
const UNKNOWN_ERROR: &str = "An unknown error occurred.";
const MAX_MESSAGE_CHARS: usize = 160;
const HINT_CHECK_ACCESS: &str = "Check the ID and that the machine account has access to it.";
const HINT_NOTHING_DELETED: &str =
    "Nothing was deleted. Check the IDs and that the machine account has access to them.";

static OS_ERROR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r" ?\(os error -?[0-9]+\)").expect("OS_ERROR_RE to be valid"));
static USERINFO_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([A-Za-z][A-Za-z0-9+.-]*://)[^/?#@ ]+@").expect("USERINFO_RE to be valid")
});
static VALIDATION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(key|value|note|name) must not (be empty|contain only whitespaces|exceed ([0-9]+) characters in length)$",
    )
    .expect("VALIDATION_RE to be valid")
});
static HTTP_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)^Received error message from server: \[(([0-9]{3})[^\]]*)\] ?(.*)$")
        .expect("HTTP_RE to be valid")
});

/// An error with a message and optional hint that are shown to the user as they are.
#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub(crate) struct UserError {
    message: String,
    hint: Option<String>,
    #[source]
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
    pub(crate) fn io(action: &str, path: &Path, e: io::Error) -> Self {
        let message = format!("{action} '{}': {}.", path.display(), io_reason(&e));
        Self::new(message).source(e)
    }

    /// Like [`UserError::io`], but for operations with no path to name: `<action>: <io reason>.`
    pub(crate) fn io_action(action: &str, e: io::Error) -> Self {
        let message = format!("{action}: {}.", io_reason(&e));
        Self::new(message).source(e)
    }

    /// Like [`UserError::io`], but reports an existing file as `not a directory` instead of
    /// `already exists`, which is how `create_dir_all` surfaces a file in the path.
    pub(crate) fn create_dir(action: &str, path: &Path, e: io::Error) -> Self {
        if e.kind() == io::ErrorKind::AlreadyExists && !path.is_dir() {
            return Self::io(action, path, io::ErrorKind::NotADirectory.into()).source(e);
        }
        Self::io(action, path, e)
    }

    pub(crate) fn hint_text(&self) -> Option<&str> {
        self.hint.as_deref()
    }
}

/// `<label>: <body>` followed by an optional `Hint: <hint>` line.
fn message_lines(label: &str, body: &str, hint: Option<&str>) -> String {
    match hint {
        Some(hint) => format!("{label}: {body}\nHint: {hint}\n"),
        None => format!("{label}: {body}\n"),
    }
}

/// Renders an error as `Error: <message>` with an optional `Hint: <hint>` line.
pub(crate) fn render(report: &eyre::Report) -> String {
    let (message, hint) = classify(report);
    message_lines("Error", &message, hint.as_deref())
}

/// Renders the full report, followed by the `Hint: <hint>` line [`render`] would show.
pub(crate) fn render_verbose(report: &eyre::Report) -> String {
    let (_, hint) = classify(report);
    message_lines("Error", &format!("{report:?}"), hint.as_deref())
}

/// Renders `Error: <action>: <io reason>.` for failures that cannot be propagated as a report,
/// such as a write to stdout that has to be reported on stderr instead.
pub(crate) fn render_io_error(action: &str, e: &io::Error) -> String {
    message_lines("Error", &format!("{action}: {}.", io_reason(e)), None)
}

/// Reduces a report to the user-facing `(message, hint)` pair.
fn classify(report: &eyre::Report) -> (String, Option<String>) {
    if let Some(e) = report.downcast_ref::<UserError>() {
        return (e.message.clone(), e.hint.clone());
    }
    let err: &(dyn Error + 'static) = report.as_ref();
    match generic(err) {
        Some(e) => (e.message, e.hint),
        None => (sentence(&report.to_string()), None),
    }
}

/// Classifies network and HTTP errors that need no call-site context.
fn generic(err: &(dyn Error + 'static)) -> Option<UserError> {
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
        return Some(
            UserError::new(format!("The TLS certificate of {} is not trusted.", host()))
                .hint(TLS_HINT),
        );
    }
    if io_kind(io::ErrorKind::TimedOut) || texts.iter().any(|t| t.contains("timed out")) {
        return Some(
            UserError::new(format!("Connection to {} timed out.", host()))
                .hint("Check your network connection."),
        );
    }
    if top.starts_with("builder error") {
        // Release builds of the SDK only allow https, and reqwest reports a plain http URL as a
        // builder error.
        if texts.iter().any(|t| t == "URL scheme is not allowed")
            && texts.iter().any(|t| t.contains("for url (http://"))
        {
            return Some(UserError::new("The server URL must use https://.").hint(SERVER_URL_HINT));
        }
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
            if server_message(body).is_some() {
                e
            } else {
                e.hint(SERVER_URL_HINT)
            }
        }
        429 => UserError::new(format!("Too many requests ({reason})."))
            .hint("Wait a moment and try again."),
        500..=599 => UserError::new(format!("The server returned an error ({reason})."))
            .hint("Try again later."),
        status => {
            let e = UserError::new(format!("Unexpected response from the server ({reason})."));
            // A generic web server rejecting the request shape suggests a wrong server URL.
            if matches!(status, 405 | 406 | 415) {
                e.hint(SERVER_URL_HINT)
            } else {
                e
            }
        }
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
            serde_json::Value::Array(items) => items.as_slice(),
            other => std::slice::from_ref(other),
        })
        .filter_map(|v| v.as_str())
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
            let (_, rest) = t.split_once("for url (")?;
            let url = rest.split_once(')').map_or(rest, |(u, _)| u);
            url_host(url)
        })
        .unwrap_or_else(|| "the server".to_string())
}

/// Returns `host[:port]` of a URL, without scheme, userinfo, path, query or fragment.
fn url_host(url: &str) -> Option<String> {
    let url = url.split_once("://").map_or(url, |(_, r)| r);
    let authority = url.split(['/', '?', '#']).next().unwrap_or(url);
    let host = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    (!host.is_empty()).then(|| host.to_string())
}

pub(crate) fn malformed_token() -> UserError {
    UserError::new("The access token is malformed.").hint(HINT_COPY_TOKEN)
}

/// Classifies an access token login failure against the identity server at `identity_url`.
pub(crate) fn login_error(e: LoginError, identity_url: &str) -> UserError {
    let host = url_host(identity_url).unwrap_or_else(|| "the server".to_string());

    let error = match &e {
        LoginError::Api(ApiError::Response(rc)) => {
            login_response_error(rc.status.as_u16(), &rc.message, &host)
        }
        LoginError::IdentityFail(r) => Some(invalid_access_token(&r.error).unwrap_or_else(|| {
            login_failed(&[&r.error_model.message, &r.error_description, &r.error])
        })),
        LoginError::Crypto(CryptoError::Decrypt) => Some(
            UserError::new("The access token's decryption key is invalid.").hint(HINT_COPY_TOKEN),
        ),
        LoginError::Crypto(_)
        | LoginError::Serde(_)
        | LoginError::MissingField(_)
        | LoginError::JwtTokenParse(_)
        | LoginError::InvalidResponse
        | LoginError::InvalidOrganizationId => Some(UserError::new(format!(
            "Unexpected login response from the server at {host}."
        ))),
        LoginError::AccessTokenInvalid(_) => Some(malformed_token()),
        _ => None,
    };

    error
        .or_else(|| generic(&e))
        .unwrap_or_else(|| UserError::new(sentence(&e.to_string())))
        .source(e)
}

/// Classifies an identity server response that could not be parsed as a login result.
/// Returns `None` for statuses that [`generic`] handles.
fn login_response_error(status: u16, body: &str, host: &str) -> Option<UserError> {
    match status {
        400 => invalid_access_token(
            serde_json::from_str::<serde_json::Value>(body)
                .ok()?
                .get("error")?
                .as_str()?,
        ),
        429 => Some(
            UserError::new("Too many login attempts.")
                .hint("Wait a few minutes and retry; keep state enabled to reuse sessions."),
        ),
        // Identity answers 400, 401, 403 and 429 itself; any other 4xx comes from a wrong URL or
        // a non-Bitwarden server, as does a 200 that is not JSON.
        402 | 404..=499 => Some(unexpected_login_endpoint(host)),
        200 if serde_json::from_str::<serde::de::IgnoredAny>(body).is_err() => {
            Some(unexpected_login_endpoint(host))
        }
        _ => None,
    }
}

/// Classifies the OAuth `error` code of a rejected access token.
fn invalid_access_token(code: &str) -> Option<UserError> {
    matches!(code, "invalid_client" | "invalid_grant").then(|| {
        UserError::new("The access token is invalid, expired, or revoked.")
            .hint("Create a new access token in the Bitwarden web app.")
    })
}

fn unexpected_login_endpoint(host: &str) -> UserError {
    UserError::new(format!(
        "Unexpected response from the login endpoint at {host}."
    ))
    .hint(SERVER_URL_HINT)
}

/// Uses the first non-empty candidate as the server's reason for a failed login.
fn login_failed(candidates: &[&str]) -> UserError {
    match candidates.iter().map(|c| c.trim()).find(|c| !c.is_empty()) {
        Some(reason) => UserError::new(format!(
            "Login failed: {}.",
            sentence(reason).trim_end_matches('.')
        )),
        None => UserError::new("Login failed."),
    }
}

/// What a Secrets Manager request operated on, used to name it in error messages.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Target {
    Secret(Uuid),
    Project(Uuid),
    /// Several secrets being deleted at once.
    Secrets,
    /// Several projects being deleted at once.
    Projects,
    /// A secret being updated and moved to a project.
    SecretOrProject(Uuid, Uuid),
    None,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Op {
    Read,
    Write,
}

/// Classifies a Secrets Manager error for a request on `target`.
///
/// The SDK error type can't be named from bws, so it is classified by its Display and Debug text.
pub(crate) fn sm_error<E>(e: E, target: Target, op: Op) -> UserError
where
    E: Error + Send + Sync + 'static,
{
    let display = e.to_string();
    let debug = format!("{e:?}");
    let http = HTTP_RE.captures(&display);
    let status = http
        .as_ref()
        .and_then(|c| c.get(2)?.as_str().parse::<u16>().ok());
    // A 404 without a Bitwarden error body likely means a wrong server URL, which `generic`
    // reports.
    let bitwarden_body = http
        .as_ref()
        .and_then(|c| server_message(c.get(3)?.as_str()))
        .is_some();

    let error = validation_error(&display)
        .or_else(|| match (status, op) {
            (Some(404), _) if bitwarden_body => not_found(target),
            (Some(403), Op::Read) => not_found(target),
            (Some(403), _) => no_write_access(target),
            _ => None,
        })
        .or_else(|| decoding_error(&debug, &display))
        .or_else(|| generic(&e))
        .unwrap_or_else(|| UserError::new(sentence(&display)));
    error.source(e)
}

/// Classifies SDK errors that mean the response could not be turned into data.
fn decoding_error(debug: &str, display: &str) -> Option<UserError> {
    if debug.starts_with("Crypto(") {
        return Some(UserError::new(
            "Could not decrypt data returned by the server.",
        ));
    }
    let undecodable = ["MissingField(", "Chrono(", "Api(Serde("]
        .iter()
        .any(|p| debug.starts_with(p))
        // A wrong content type is left to `generic`, which adds the server URL hint.
        && !display.contains("content type response when JSON was expected");
    undecodable.then(|| UserError::new("Unexpected response from the server."))
}

fn validation_error(display: &str) -> Option<UserError> {
    if display.starts_with("Unknown validation error") {
        return Some(UserError::new("The input is invalid."));
    }
    let c = VALIDATION_RE.captures(display)?;
    let subject = match c.get(1)?.as_str() {
        "key" => "Secret key",
        "value" => "Secret value",
        "note" => "Secret note",
        _ => "Project name",
    };
    let predicate = match (c.get(2)?.as_str(), c.get(3)) {
        (_, Some(max)) => format!("exceed {} characters", max.as_str()),
        ("contain only whitespaces", _) => "contain only whitespace".to_string(),
        (other, _) => other.to_string(),
    };
    Some(UserError::new(format!("{subject} must not {predicate}.")))
}

fn not_found(target: Target) -> Option<UserError> {
    let (message, hint) = match target {
        Target::Secret(id) => (
            format!("Secret {id} not found or not accessible."),
            HINT_CHECK_ACCESS,
        ),
        Target::SecretOrProject(id, project_id) => (
            format!("Secret {id} or project {project_id} not found or not accessible."),
            HINT_CHECK_ACCESS,
        ),
        Target::Project(id) => (
            format!("Project {id} not found or not accessible."),
            HINT_CHECK_ACCESS,
        ),
        Target::Secrets => (
            "One or more of the given secrets were not found or not accessible.".to_string(),
            HINT_NOTHING_DELETED,
        ),
        Target::Projects => (
            "One or more of the given projects were not found or not accessible.".to_string(),
            HINT_NOTHING_DELETED,
        ),
        Target::None => return None,
    };
    Some(UserError::new(message).hint(hint))
}

fn no_write_access(target: Target) -> Option<UserError> {
    let subject = match target {
        Target::Secret(id) => format!("secret {id}"),
        Target::SecretOrProject(id, project_id) => {
            format!("secret {id} or project {project_id}")
        }
        Target::Project(id) => format!("project {id}"),
        Target::Secrets => "these secrets".to_string(),
        Target::Projects => "these projects".to_string(),
        Target::None => return None,
    };
    Some(UserError::new(format!(
        "The machine account does not have write access to {subject}."
    )))
}

/// `Failed to delete <failed> of <requested> <noun>s.`
pub(crate) fn partial_delete(failed: usize, requested: usize, noun: &str) -> UserError {
    let plural = if requested == 1 { "" } else { "s" };
    UserError::new(format!(
        "Failed to delete {failed} of {requested} {noun}{plural}."
    ))
}

/// Whether to show full error reports. Read once; the environment cannot change mid-process.
pub(crate) fn is_verbose() -> bool {
    static VERBOSE: LazyLock<bool> = LazyLock::new(|| {
        let var = |name| std::env::var_os(name).map(|v| v.to_string_lossy().into_owned());
        verbose_from(
            var("BWS_DEBUG").as_deref(),
            var("RUST_BACKTRACE").as_deref(),
        )
    });
    *VERBOSE
}

fn verbose_from(bws_debug: Option<&str>, rust_backtrace: Option<&str>) -> bool {
    let debug = bws_debug.is_some_and(|v| {
        let v = v.trim().to_ascii_lowercase();
        !matches!(v.as_str(), "" | "0" | "false")
    });
    debug || rust_backtrace.is_some_and(|v| v != "0")
}

pub(crate) fn warn(message: &str, hint: Option<&str>) {
    eprint!("{}", message_lines("Warning", message, hint));
}

// `PanicHookInfo::payload_as_str` would replace the downcasts, but it is only stable since 1.91
// and the workspace MSRV is 1.88.
pub(crate) fn render_panic(payload: &(dyn Any + Send)) -> String {
    let text = payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("unknown error");
    let message = sentence(text);
    message_lines(
        "Error",
        &format!(
            "bws crashed unexpectedly: {}.",
            message.trim_end_matches('.')
        ),
        Some("Rerun with BWS_DEBUG=1 and report it at https://github.com/bitwarden/sdk-sm/issues"),
    )
}

fn sentence(s: &str) -> String {
    let line = s
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("");
    let line = line.split_whitespace().collect::<Vec<_>>().join(" ");
    // Server-supplied text reaches this function, and escape sequences would let a hostile server
    // move the cursor or clear the terminal. `split_whitespace` only drops whitespace controls.
    let line: String = line.chars().filter(|c| !c.is_control()).collect();
    let line = OS_ERROR_RE.replace_all(&line, "");
    let line = USERINFO_RE.replace_all(&line, "$1");

    let truncated = line.chars().count() > MAX_MESSAGE_CHARS;
    let text = uppercase_first(&line.chars().take(MAX_MESSAGE_CHARS).collect::<String>());

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

/// Formats text to follow a colon: like [`sentence`] but without the trailing period, and
/// lowercased unless it starts with an acronym.
pub(crate) fn clause(s: &str) -> String {
    let sentence = sentence(s);
    let text = sentence.strip_suffix('.').unwrap_or(&sentence);
    // Keep the case of a leading acronym, like `TOML`.
    if text.chars().nth(1).is_some_and(char::is_uppercase) {
        return text.to_string();
    }
    lowercase_first(text)
}

fn uppercase_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn lowercase_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().chain(chars).collect(),
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
        _ => lowercase_first(&OS_ERROR_RE.replace_all(&e.to_string(), "")),
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::{self, Display};

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

        // Control characters are inert text, so escape sequences cannot reach the terminal.
        assert_eq!(
            sentence("Bad request\x1b[2K\x1b[1G\x07"),
            "Bad request[2K[1G."
        );
        assert_eq!(sentence("\x1b[31mred\x1b[0m"), "[31mred[0m.");
        assert_eq!(sentence("a\x00b\x08c"), "Abc.");
        assert_eq!(sentence("\x1b\x07\x08"), UNKNOWN_ERROR);

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
            lines(&generic(&tls).expect("classified")),
            (
                "The TLS certificate of example.com:8443 is not trusted.".to_string(),
                Some(TLS_HINT.to_string())
            )
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
        let bad_scheme = ChainErr(
            "builder error for url (http://example.com/identity/connect/token)".to_string(),
            Some(Box::new(ChainErr(
                "URL scheme is not allowed".to_string(),
                None,
            ))),
        );
        assert_eq!(
            lines(&generic(&bad_scheme).expect("classified")),
            (
                "The server URL must use https://.".to_string(),
                Some(SERVER_URL_HINT.to_string())
            )
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

        // The server's own text is promoted into the top-level line, so it must not be able to
        // carry terminal escapes there.
        let e = http(
            "400 Bad Request",
            r#"{"message":"Bad request\u001b[2K\u001b[1G","validationErrors":{"Name":["\u001b[31mRed\u001b[0m"]}}"#,
        );
        assert_eq!(e.message, "Bad request[2K[1G: [31mRed[0m.");

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
            lines(&http("418 I'm a teapot", "")),
            (
                "Unexpected response from the server (418 I'm a teapot).".to_string(),
                None
            )
        );
        assert_eq!(
            lines(&http("405 Method Not Allowed", "<html></html>")),
            (
                "Unexpected response from the server (405 Method Not Allowed).".to_string(),
                Some(SERVER_URL_HINT.to_string())
            )
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
        assert!(!render_verbose(&report).contains("Hint:"));

        let report = eyre::Report::new(UserError::new("Something failed.").hint("Do this."));
        let verbose = render_verbose(&report);
        assert!(verbose.starts_with("Error: "), "{verbose}");
        assert!(verbose.ends_with("\nHint: Do this.\n"), "{verbose}");

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
    fn render_io_error_reports_reason() {
        assert_eq!(
            render_io_error(
                "Could not write to stdout",
                &io::Error::other("No space left on device (os error 28)")
            ),
            "Error: Could not write to stdout: no space left on device.\n"
        );
        assert_eq!(
            render_io_error(
                "Could not write to stdout",
                &io::Error::from(io::ErrorKind::PermissionDenied)
            ),
            "Error: Could not write to stdout: permission denied.\n"
        );
    }

    #[test]
    fn login_response_errors() {
        let host = "vault.example.com:8443";
        for body in [
            r#"{"error":"invalid_client"}"#,
            r#"{"error":"invalid_grant"}"#,
        ] {
            let e = login_response_error(400, body, host).expect("classified");
            assert_eq!(
                lines(&e),
                (
                    "The access token is invalid, expired, or revoked.".to_string(),
                    Some("Create a new access token in the Bitwarden web app.".to_string())
                )
            );
        }
        assert!(login_response_error(400, r#"{"error":"invalid_scope"}"#, host).is_none());
        assert!(login_response_error(400, "<html>", host).is_none());

        assert_eq!(
            lines(&login_response_error(429, "", host).expect("classified")),
            (
                "Too many login attempts.".to_string(),
                Some(
                    "Wait a few minutes and retry; keep state enabled to reuse sessions."
                        .to_string()
                )
            )
        );

        let unexpected = (
            "Unexpected response from the login endpoint at vault.example.com:8443.".to_string(),
            Some(SERVER_URL_HINT.to_string()),
        );
        for status in [404, 405, 415] {
            assert_eq!(
                lines(&login_response_error(status, "<html></html>", host).expect("classified")),
                unexpected
            );
        }
        for status in [401, 403] {
            assert!(login_response_error(status, "{}", host).is_none());
        }
        assert!(invalid_access_token("invalid_scope").is_none());
        assert_eq!(
            lines(&login_response_error(200, "<html>login</html>", host).expect("classified")),
            unexpected
        );
        assert!(login_response_error(200, "{}", host).is_none());
        assert!(login_response_error(500, "<html>oops</html>", host).is_none());
    }

    #[test]
    fn login_errors() {
        const IDENTITY_URL: &str = "https://u:p@identity.example.com/identity";

        let e = login_error(LoginError::Crypto(CryptoError::Decrypt), IDENTITY_URL);
        assert_eq!(
            lines(&e),
            (
                "The access token's decryption key is invalid.".to_string(),
                Some(HINT_COPY_TOKEN.to_string())
            )
        );
        assert!(Error::source(&e).is_some());

        let e = login_error(
            LoginError::MissingField(bitwarden_core::MissingFieldError("organization")),
            IDENTITY_URL,
        );
        assert_eq!(
            lines(&e),
            (
                "Unexpected login response from the server at identity.example.com.".to_string(),
                None
            )
        );

        // Without a profile, bws logs in against the SDK's default identity server.
        let e = login_error(
            LoginError::InvalidResponse,
            &bitwarden_core::ClientSettings::default().identity_url,
        );
        assert_eq!(
            e.message,
            "Unexpected login response from the server at identity.bitwarden.com."
        );

        let Err(token_error) = "not-a-token".parse::<bitwarden::secrets_manager::AccessToken>()
        else {
            panic!("token to be malformed");
        };
        let e = login_error(LoginError::AccessTokenInvalid(token_error), IDENTITY_URL);
        assert_eq!(
            lines(&e),
            (
                "The access token is malformed.".to_string(),
                Some(HINT_COPY_TOKEN.to_string())
            )
        );

        let e = login_error(LoginError::AuthenticationFailed, IDENTITY_URL);
        assert_eq!(lines(&e), ("Failed to authenticate.".to_string(), None));
    }

    #[test]
    fn login_failed_reasons() {
        assert_eq!(
            login_failed(&["", " invalid client credentials ", "invalid_client"]).message,
            "Login failed: Invalid client credentials."
        );
        assert_eq!(
            login_failed(&["Username or password is incorrect. Try again.", "", ""]).message,
            "Login failed: Username or password is incorrect. Try again."
        );
        assert_eq!(login_failed(&["", " ", ""]).message, "Login failed.");
    }

    #[test]
    fn clause_formats_text() {
        assert_eq!(
            clause("Key with no value, expected `=`\n"),
            "key with no value, expected `=`"
        );
        assert_eq!(clause("TOML is invalid."), "TOML is invalid");
    }

    struct SdkErr {
        display: String,
        debug: String,
    }

    impl SdkErr {
        fn new(display: &str, debug: &str) -> Self {
            Self {
                display: display.to_string(),
                debug: debug.to_string(),
            }
        }

        fn http(status: u16, reason: &str) -> Self {
            Self::new(
                &format!(
                    "Received error message from server: [{status} {reason}] {{\"message\": \"Resource not found.\", \"validationErrors\": null, \"object\": \"error\"}}"
                ),
                &format!("Api(Response(ResponseContent {{ status: {status}, message: \"...\" }}))"),
            )
        }
    }

    impl Display for SdkErr {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(&self.display)
        }
    }

    impl fmt::Debug for SdkErr {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(&self.debug)
        }
    }

    impl Error for SdkErr {}

    const SECRET_ID: Uuid = Uuid::from_u128(0x15744a66_0000_0000_0000_000000000001);
    const PROJECT_ID: Uuid = Uuid::from_u128(0x15744a66_0000_0000_0000_000000000002);

    fn sm(e: SdkErr, target: Target, op: Op) -> (String, Option<String>) {
        lines(&sm_error(e, target, op))
    }

    #[test]
    fn sm_validation_errors() {
        let validation = |display: &str| {
            sm(
                SdkErr::new(display, "Validation(...)"),
                Target::None,
                Op::Write,
            )
            .0
        };
        assert_eq!(
            validation("name must not be empty"),
            "Project name must not be empty."
        );
        assert_eq!(
            validation("name must not contain only whitespaces"),
            "Project name must not contain only whitespace."
        );
        assert_eq!(
            validation("name must not exceed 500 characters in length"),
            "Project name must not exceed 500 characters."
        );
        assert_eq!(
            validation("key must not be empty"),
            "Secret key must not be empty."
        );
        assert_eq!(
            validation("value must not be empty"),
            "Secret value must not be empty."
        );
        assert_eq!(
            validation("key must not contain only whitespaces"),
            "Secret key must not contain only whitespace."
        );
        assert_eq!(
            validation("note must not exceed 7000 characters in length"),
            "Secret note must not exceed 7000 characters."
        );
        assert_eq!(
            validation("Unknown validation error: ValidationErrors {}"),
            "The input is invalid."
        );
    }

    #[test]
    fn sm_not_found() {
        let hint = Some(HINT_CHECK_ACCESS.to_string());
        assert_eq!(
            sm(
                SdkErr::http(404, "Not Found"),
                Target::Secret(SECRET_ID),
                Op::Read
            ),
            (
                format!("Secret {SECRET_ID} not found or not accessible."),
                hint.clone()
            )
        );
        assert_eq!(
            sm(
                SdkErr::http(403, "Forbidden"),
                Target::Secret(SECRET_ID),
                Op::Read
            ),
            (
                format!("Secret {SECRET_ID} not found or not accessible."),
                hint.clone()
            )
        );
        assert_eq!(
            sm(
                SdkErr::http(404, "Not Found"),
                Target::Project(PROJECT_ID),
                Op::Write
            ),
            (
                format!("Project {PROJECT_ID} not found or not accessible."),
                hint.clone()
            )
        );
        assert_eq!(
            sm(
                SdkErr::http(404, "Not Found"),
                Target::SecretOrProject(SECRET_ID, PROJECT_ID),
                Op::Write
            ),
            (
                format!("Secret {SECRET_ID} or project {PROJECT_ID} not found or not accessible."),
                hint.clone()
            )
        );
        assert_eq!(
            sm(
                SdkErr::http(404, "Not Found"),
                Target::Secret(SECRET_ID),
                Op::Write
            ),
            (
                format!("Secret {SECRET_ID} not found or not accessible."),
                hint
            )
        );
        assert_eq!(
            sm(SdkErr::http(404, "Not Found"), Target::Secrets, Op::Write),
            (
                "One or more of the given secrets were not found or not accessible.".to_string(),
                Some(HINT_NOTHING_DELETED.to_string())
            )
        );
        assert_eq!(
            sm(SdkErr::http(404, "Not Found"), Target::Projects, Op::Write),
            (
                "One or more of the given projects were not found or not accessible.".to_string(),
                Some(HINT_NOTHING_DELETED.to_string())
            )
        );
        assert_eq!(
            sm(
                SdkErr::new(
                    "Received error message from server: [404 Not Found] <html></html>",
                    "Api(Response(...))"
                ),
                Target::Secret(SECRET_ID),
                Op::Read
            ),
            (
                "The requested item was not found or is not accessible.".to_string(),
                Some(SERVER_URL_HINT.to_string())
            )
        );
        assert_eq!(
            sm(SdkErr::http(404, "Not Found"), Target::None, Op::Read).0,
            "The requested item was not found or is not accessible."
        );
    }

    #[test]
    fn sm_no_write_access() {
        let write = |target| sm(SdkErr::http(403, "Forbidden"), target, Op::Write);
        let denied = |subject: &str| {
            (
                format!("The machine account does not have write access to {subject}."),
                None,
            )
        };
        assert_eq!(
            write(Target::Secret(SECRET_ID)),
            denied(&format!("secret {SECRET_ID}"))
        );
        assert_eq!(
            write(Target::SecretOrProject(SECRET_ID, PROJECT_ID)),
            denied(&format!("secret {SECRET_ID} or project {PROJECT_ID}"))
        );
        assert_eq!(
            write(Target::Project(PROJECT_ID)),
            denied(&format!("project {PROJECT_ID}"))
        );
        assert_eq!(write(Target::Secrets), denied("these secrets"));
        assert_eq!(write(Target::Projects), denied("these projects"));
        assert_eq!(write(Target::None).0, "Access denied (403 Forbidden).");
    }

    #[test]
    fn sm_unexpected_data() {
        assert_eq!(
            sm(
                SdkErr::new("The decryption operation failed", "Crypto(Decrypt)"),
                Target::Secret(SECRET_ID),
                Op::Read
            ),
            (
                "Could not decrypt data returned by the server.".to_string(),
                None
            )
        );
        for debug in [
            "MissingField(MissingFieldError(\"response.id\"))",
            "Chrono(ParseError(Invalid))",
            "Api(Serde(Error(\"expected value\", line: 1, column: 1)))",
        ] {
            assert_eq!(
                sm(SdkErr::new("anything", debug), Target::None, Op::Read).0,
                "Unexpected response from the server."
            );
        }
        assert_eq!(
            sm(
                SdkErr::new(
                    "Received unexpected content type response when JSON was expected",
                    "Api(Serde(...))"
                ),
                Target::None,
                Op::Read
            ),
            (
                "Unexpected response from the server.".to_string(),
                Some(SERVER_URL_HINT.to_string())
            )
        );
        let e = sm_error(
            SdkErr::new("Something odd happened", "Other"),
            Target::None,
            Op::Read,
        );
        assert_eq!(e.message, "Something odd happened.");
        assert!(Error::source(&e).is_some());
    }

    #[test]
    fn partial_delete_message() {
        assert_eq!(
            partial_delete(1, 3, "secret").message,
            "Failed to delete 1 of 3 secrets."
        );
        assert_eq!(
            partial_delete(1, 1, "project").message,
            "Failed to delete 1 of 1 project."
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
