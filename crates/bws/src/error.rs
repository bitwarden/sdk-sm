use std::{
    any::Any,
    error::Error,
    io,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use bitwarden_core::{ApiError, auth::login::LoginError};
use bitwarden_crypto::CryptoError;
use color_eyre::eyre;
use regex::Regex;

const HINT_COPY_TOKEN: &str = "Copy the full access token from the Bitwarden web app.";
const SERVER_URL_HINT: &str = "Check the server URL (--server-url or server-base in the config).";
const NETWORK_HINT: &str = "Check the server URL and your network connection.";
const TLS_HINT: &str = "Check the server URL or install the server's CA certificate.";
const CONFIG_USAGE_HINT: &str = "Usage: bws config <name> <value>";
pub(crate) const STATE_HINT: &str =
    "Continuing without state; set a writable dir with: bws config state-dir <dir>";
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

/// A user-facing error: a message and optional `Hint:` line, with the underlying error as the
/// source for the verbose report. Variants exist for classification and message templates; the
/// fix-it text is centralized in [`UserError::hint`].
#[derive(Debug, thiserror::Error)]
pub(crate) enum UserError {
    // ---- Configuration ----
    #[error("Could not determine the home directory.")]
    NoHomeDirectory,

    #[error("Config file '{path}' does not exist.")]
    ConfigFileMissing { path: PathBuf },

    #[error("Config file '{path}' does not exist; nothing to delete.")]
    NothingToDelete { path: PathBuf },

    #[error("Config file '{path}' is a directory.")]
    ConfigFileIsDir { path: PathBuf },

    /// `location` is `" at line N, column M"` when toml reports a span, else `""`.
    #[error("Invalid config file '{path}'{location}: {reason}.")]
    InvalidConfig {
        path: PathBuf,
        location: String,
        reason: String,
        #[source]
        source: Box<toml::de::Error>,
    },

    #[error("Profile '{name}' not found in '{file}'.")]
    ProfileNotFound { name: String, file: PathBuf },

    #[error("Invalid value for '{key}'; expected {expected}.")]
    InvalidConfigValue { key: String, expected: &'static str },

    #[error("Missing config name.")]
    MissingConfigName,

    #[error("Missing value for '{key}'.")]
    MissingConfigValue { key: String },

    #[error("The access token in BWS_ACCESS_TOKEN or --access-token is malformed.")]
    MalformedConfigToken {
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },

    /// The passphrase cannot access the server because it is not associated with a known
    /// profile, and the profile has neither an identity nor an API URL.
    #[error("Profile '{name}' has no server URL.")]
    ProfileNoUrl { name: String },

    // ---- Auth and state ----
    #[error("No access token provided.")]
    NoAccessToken,

    #[error("The access token is malformed.")]
    MalformedToken {
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },

    #[error("The access token is invalid, expired, or revoked.")]
    InvalidAccessToken,

    #[error("Too many auth attempts.")]
    TooManyAuthAttempts,

    #[error("The access token's decryption key is invalid.")]
    InvalidDecryptionKey,

    #[error("Unexpected response from the auth endpoint at {host}.")]
    UnexpectedAuthEndpoint { host: String },

    #[error("Unexpected auth response from the server at {host}.")]
    UnexpectedAuthResponse { host: String },

    #[error("Auth failed.")]
    AuthFailed,

    #[error("Auth failed: {reason}.")]
    AuthFailedReason { reason: String },

    /// An unclassified auth failure; the SDK's `LoginError` text is the message.
    #[error("{message}")]
    UnclassifiedAuth {
        message: String,
        #[source]
        source: Box<LoginError>,
    },

    #[error("Could not determine a state directory (no home directory).")]
    NoStateDir,

    #[error("Could not use state directory '{path}': {reason}.")]
    StateDirUnusable {
        path: PathBuf,
        reason: String,
        #[source]
        source: io::Error,
    },

    /// `<action> '<path>': <io reason>.`
    #[error("{action} '{path}': {reason}.")]
    Io {
        action: String,
        path: PathBuf,
        reason: String,
        #[source]
        source: io::Error,
    },

    // ---- Run ----
    #[error("Could not find the shell '{name}'.")]
    ShellNotFound { name: String },

    #[error("No command provided.")]
    NoCommand,

    #[error("Multiple secrets have the name '{name}'.")]
    DuplicateSecretName { name: String },

    // ---- Delete ----
    #[error(
        "Failed to delete {count} secret{s}.",
        s = if *count == 1 { "" } else { "s" }
    )]
    FailedToDeleteSecrets { count: usize },

    #[error(
        "Failed to delete {count} project{s}.",
        s = if *count == 1 { "" } else { "s" }
    )]
    FailedToDeleteProjects { count: usize },

    // ---- Network and HTTP ----
    #[error("Could not connect to {host}: connection refused.")]
    ConnectionRefused { host: String },

    #[error("Could not resolve host {host}.")]
    Dns { host: String },

    #[error("The TLS certificate of {host} is not trusted.")]
    Tls { host: String },

    #[error("Connection to {host} timed out.")]
    Timeout { host: String },

    #[error("The server URL must use https://.")]
    ServerUrlHttps,

    #[error("The server URL is invalid.")]
    ServerUrlInvalid,

    #[error("Could not reach {host}.")]
    ServerUnreachable { host: String },

    #[error("Unexpected response from the server.")]
    UnexpectedResponse,

    /// A 400 rejected by the server, whose reason text was already sanitized by `sentence`.
    #[error("{message}")]
    ServerMessage { message: String },

    #[error("The server rejected the request ({reason}).")]
    BadRequest { reason: String },

    #[error("The server rejected the access token ({reason}).")]
    Unauthorized { reason: String },

    #[error("Access denied ({reason}).")]
    Forbidden { reason: String },

    /// A 404 whose URL hint depends on whether the body was a Bitwarden error.
    #[error("The requested item was not found or is not accessible.")]
    NotFound { check_url: bool },

    #[error("Too many requests ({reason}).")]
    TooManyRequests { reason: String },

    #[error("The server returned an error ({reason}).")]
    ServerError { reason: String },

    /// Any other status; a 405/406/415 suggests a wrong server URL.
    #[error("Unexpected response from the server ({reason}).")]
    UnexpectedHttpStatus { reason: String, check_url: bool },
}

impl UserError {
    /// The `Hint:` line shown under the message, if any.
    pub(crate) fn hint(&self) -> Option<String> {
        let hint = match self {
            UserError::NoHomeDirectory => "Set --config-file or BWS_CONFIG_FILE.",
            UserError::ConfigFileIsDir { .. } => {
                "In a container, create the host file first (e.g. `touch`) so it mounts as a file."
            }
            UserError::MissingConfigName | UserError::MissingConfigValue { .. } => {
                CONFIG_USAGE_HINT
            }
            UserError::MalformedConfigToken { .. } => "Fix or unset it, or pass --profile.",
            UserError::ProfileNoUrl { name } => {
                return Some(format!(
                    "Run: bws config --profile {name} server-base <url>"
                ));
            }
            UserError::NoAccessToken => "Pass --access-token or set BWS_ACCESS_TOKEN.",
            UserError::MalformedToken { .. } => HINT_COPY_TOKEN,
            UserError::InvalidDecryptionKey => HINT_COPY_TOKEN,
            UserError::InvalidAccessToken => "Create a new access token in the Bitwarden web app.",
            UserError::TooManyAuthAttempts => {
                "Wait a few minutes and retry; keep state enabled to reuse sessions."
            }
            UserError::UnexpectedAuthEndpoint { .. } => SERVER_URL_HINT,
            UserError::NoStateDir | UserError::StateDirUnusable { .. } => STATE_HINT,
            UserError::ConnectionRefused { .. } => SERVER_URL_HINT,
            UserError::Dns { .. } => NETWORK_HINT,
            UserError::Tls { .. } => TLS_HINT,
            UserError::Timeout { .. } => "Check your network connection.",
            UserError::ServerUrlHttps | UserError::ServerUrlInvalid => SERVER_URL_HINT,
            UserError::ServerUnreachable { .. } => NETWORK_HINT,
            UserError::UnexpectedResponse => SERVER_URL_HINT,
            UserError::Unauthorized { .. } => {
                "The token may be expired or revoked; create a new one in the Bitwarden web app."
            }
            UserError::Forbidden { .. } => "Check the machine account's permissions.",
            UserError::NotFound { check_url: true } => SERVER_URL_HINT,
            UserError::TooManyRequests { .. } => "Wait a moment and try again.",
            UserError::ServerError { .. } => "Try again later.",
            UserError::UnexpectedHttpStatus {
                check_url: true, ..
            } => SERVER_URL_HINT,
            UserError::ShellNotFound { .. } => {
                "Install the shell, or set the full path with --shell <shell>."
            }
            UserError::NoCommand => "Pass a command, or pipe one via stdin.",
            UserError::DuplicateSecretName { .. } => {
                "Use unique secret names, or pass --uuids-as-keynames."
            }
            UserError::FailedToDeleteSecrets { .. } | UserError::FailedToDeleteProjects { .. } => {
                "See the errors listed above."
            }
            _ => return None,
        };
        Some(hint.to_string())
    }

    /// `<action> '<path>': <io reason>.`
    pub(crate) fn io(action: &str, path: &Path, e: io::Error) -> Self {
        UserError::Io {
            action: action.to_string(),
            path: path.to_owned(),
            reason: io_reason(&e),
            source: e,
        }
    }

    /// Like [`UserError::io`], but reports an existing file as `not a directory` instead of
    /// `already exists`, which is how `create_dir_all` surfaces a file in the path.
    pub(crate) fn create_dir(action: &str, path: &Path, e: io::Error) -> Self {
        if e.kind() == io::ErrorKind::AlreadyExists && !path.is_dir() {
            return UserError::Io {
                action: action.to_string(),
                path: path.to_owned(),
                reason: io_reason(&io::Error::from(io::ErrorKind::NotADirectory)),
                source: e,
            };
        }
        UserError::io(action, path, e)
    }

    /// `<could not use state directory> '<path>': <io reason>.`
    pub(crate) fn state_dir(path: &Path, e: io::Error) -> Self {
        UserError::StateDirUnusable {
            path: path.to_owned(),
            reason: if e.kind() == io::ErrorKind::AlreadyExists && !path.is_dir() {
                io_reason(&io::Error::from(io::ErrorKind::NotADirectory))
            } else {
                io_reason(&e)
            },
            source: e,
        }
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
        return (e.to_string(), e.hint());
    }
    let err: &(dyn Error + 'static) = report.as_ref();
    match generic(err) {
        Some(e) => (e.to_string(), e.hint()),
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
        return Some(UserError::ConnectionRefused { host: host() });
    }
    if texts
        .iter()
        .any(|t| t == "dns error" || t.starts_with("failed to lookup address"))
    {
        return Some(UserError::Dns { host: host() });
    }
    if texts.iter().any(|t| {
        t.contains("invalid peer certificate") || (t.contains("certificate") && t.contains("verif"))
    }) {
        return Some(UserError::Tls { host: host() });
    }
    if io_kind(io::ErrorKind::TimedOut) || texts.iter().any(|t| t.contains("timed out")) {
        return Some(UserError::Timeout { host: host() });
    }
    if top.starts_with("builder error") {
        // Release builds of the SDK only allow https, and reqwest reports a plain http URL as a
        // builder error.
        if texts.iter().any(|t| t == "URL scheme is not allowed")
            && texts.iter().any(|t| t.contains("for url (http://"))
        {
            return Some(UserError::ServerUrlHttps);
        }
        return Some(UserError::ServerUrlInvalid);
    }
    if top.starts_with("error sending request") || top.starts_with("error decoding response body") {
        return Some(UserError::ServerUnreachable { host: host() });
    }
    if top.contains("content type response when JSON was expected") {
        return Some(UserError::UnexpectedResponse);
    }
    None
}

fn http_error(status: u16, reason: &str, body: &str) -> UserError {
    match status {
        400 => match server_message(body) {
            Some(message) => UserError::ServerMessage { message },
            None => UserError::BadRequest {
                reason: reason.to_string(),
            },
        },
        401 => UserError::Unauthorized {
            reason: reason.to_string(),
        },
        403 => UserError::Forbidden {
            reason: reason.to_string(),
        },
        404 => UserError::NotFound {
            check_url: server_message(body).is_none(),
        },
        429 => UserError::TooManyRequests {
            reason: reason.to_string(),
        },
        500..=599 => UserError::ServerError {
            reason: reason.to_string(),
        },
        // A generic web server rejecting the request shape suggests a wrong server URL.
        status => UserError::UnexpectedHttpStatus {
            reason: reason.to_string(),
            check_url: matches!(status, 405 | 406 | 415),
        },
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

/// Returns `host[:port]` of a URL, without scheme, userinfo, path, query or fragment.
fn url_host(url: &str) -> Option<String> {
    let url = url.split_once("://").map_or(url, |(_, r)| r);
    let authority = url.split(['/', '?', '#']).next().unwrap_or(url);
    let host = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    (!host.is_empty()).then(|| host.to_string())
}

/// Returns `host[:port]` of the first URL mentioned in the chain, without scheme or userinfo.
fn host_from_chain(texts: &[String]) -> String {
    texts
        .iter()
        .find_map(|t| {
            let (_, rest) = t.split_once("for url (")?;
            url_host(rest.split_once(')').map_or(rest, |(u, _)| u))
        })
        .unwrap_or_else(|| "the server".to_string())
}

/// `The access token is malformed.` with the parse error as the source for the verbose report.
pub(crate) fn malformed_token<E: Error + Send + Sync + 'static>(source: E) -> UserError {
    UserError::MalformedToken {
        source: Box::new(source),
    }
}

/// Classifies an access token auth failure against the identity server at `identity_url`.
pub(crate) fn auth_error(e: LoginError, identity_url: &str) -> UserError {
    let host = url_host(identity_url).unwrap_or_else(|| "the server".to_string());

    let classified = match &e {
        LoginError::Api(ApiError::Response(rc)) => {
            auth_response_error(rc.status.as_u16(), &rc.message, &host)
        }
        LoginError::IdentityFail(r) => invalid_access_token(&r.error)
            .or_else(|| auth_failed(&[&r.error_model.message, &r.error_description, &r.error])),
        LoginError::Crypto(CryptoError::Decrypt) => Some(UserError::InvalidDecryptionKey),
        LoginError::Crypto(_)
        | LoginError::Serde(_)
        | LoginError::MissingField(_)
        | LoginError::JwtTokenParse(_)
        | LoginError::InvalidResponse
        | LoginError::InvalidOrganizationId => Some(UserError::UnexpectedAuthResponse { host }),
        _ => None,
    };

    match classified {
        Some(error) => error,
        None if matches!(&e, LoginError::AccessTokenInvalid(_)) => malformed_token(e),
        None => generic(&e).unwrap_or_else(|| UserError::UnclassifiedAuth {
            message: sentence(&e.to_string()),
            source: Box::new(e),
        }),
    }
}

/// Classifies an identity server response that could not be parsed as an auth result.
/// Returns `None` for statuses that [`generic`] handles.
fn auth_response_error(status: u16, body: &str, host: &str) -> Option<UserError> {
    match status {
        400 => invalid_access_token(
            serde_json::from_str::<serde_json::Value>(body)
                .ok()?
                .get("error")?
                .as_str()?,
        ),
        429 => Some(UserError::TooManyAuthAttempts),
        // Identity answers 400, 401, 403 and 429 itself; any other 4xx comes from a wrong URL or
        // a non-Bitwarden server, as does a 200 that is not JSON.
        402 | 404..=499 => Some(unexpected_auth_endpoint(host)),
        200 if serde_json::from_str::<serde::de::IgnoredAny>(body).is_err() => {
            Some(unexpected_auth_endpoint(host))
        }
        _ => None,
    }
}

/// Classifies the OAuth `error` code of a rejected access token.
fn invalid_access_token(code: &str) -> Option<UserError> {
    matches!(code, "invalid_client" | "invalid_grant").then_some(UserError::InvalidAccessToken)
}

fn unexpected_auth_endpoint(host: &str) -> UserError {
    UserError::UnexpectedAuthEndpoint {
        host: host.to_string(),
    }
}

/// Uses the first non-empty candidate as the server's reason for a failed auth.
fn auth_failed(candidates: &[&str]) -> Option<UserError> {
    match candidates.iter().map(|c| c.trim()).find(|c| !c.is_empty()) {
        Some(reason) => Some(UserError::AuthFailedReason {
            reason: sentence(reason).trim_end_matches('.').to_string(),
        }),
        None => Some(UserError::AuthFailed),
    }
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
        (e.to_string(), e.hint())
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
    fn clause_formats_text() {
        assert_eq!(
            clause("Key with no value, expected `=`\n"),
            "key with no value, expected `=`"
        );
        assert_eq!(clause("TOML is invalid."), "TOML is invalid");
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
        assert_eq!(e.to_string(), "Could not resolve host nonexistent.invalid.");
        assert_eq!(e.hint().as_deref(), Some(NETWORK_HINT));
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
            classify_text("builder error").to_string(),
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
            classify_text("error sending request").to_string(),
            "Could not reach the server."
        );
        assert_eq!(
            classify_text("Received unexpected content type response when JSON was expected")
                .to_string(),
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
            http("400 Bad Request", "").to_string(),
            "The server rejected the request (400 Bad Request)."
        );
        assert_eq!(
            http("400 Bad Request", "<html>bad</html>").to_string(),
            "The server rejected the request (400 Bad Request)."
        );

        // The server's own text is promoted into the top-level line, so it must not be able to
        // carry terminal escapes there.
        let e = http(
            "400 Bad Request",
            r#"{"message":"Bad request\u001b[2K\u001b[1G","validationErrors":{"Name":["\u001b[31mRed\u001b[0m"]}}"#,
        );
        assert_eq!(e.to_string(), "Bad request[2K[1G: [31mRed[0m.");

        let e = http("401 Unauthorized", "{}");
        assert_eq!(
            e.to_string(),
            "The server rejected the access token (401 Unauthorized)."
        );
        assert!(e.hint().is_some());

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
        assert!(!e.to_string().contains("<html"));

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
            e.to_string(),
            "The server returned an error (504 Gateway Timeout)."
        );

        let e = classify_text("Received error message from server: [404 Not Found] <html></html>");
        assert_eq!(e.hint().as_deref(), Some(SERVER_URL_HINT));
    }

    #[test]
    fn generic_ignores_unknown_errors() {
        assert!(generic(&ChainErr("Missing access token".to_string(), None)).is_none());
    }

    #[test]
    fn run_errors() {
        let failures = [
            (
                UserError::ShellNotFound {
                    name: "sh".to_string(),
                },
                "Could not find the shell 'sh'.",
                "Install the shell, or set the full path with --shell <shell>.",
            ),
            (
                UserError::NoCommand,
                "No command provided.",
                "Pass a command, or pipe one via stdin.",
            ),
            (
                UserError::DuplicateSecretName {
                    name: "api_key".to_string(),
                },
                "Multiple secrets have the name 'api_key'.",
                "Use unique secret names, or pass --uuids-as-keynames.",
            ),
        ];
        for (e, message, hint) in failures {
            assert_eq!(lines(&e), (message.to_string(), Some(hint.to_string())));
        }
    }

    #[test]
    fn delete_errors() {
        let failures = [
            (
                UserError::FailedToDeleteSecrets { count: 1 },
                "Failed to delete 1 secret.",
                "See the errors listed above.",
            ),
            (
                UserError::FailedToDeleteSecrets { count: 2 },
                "Failed to delete 2 secrets.",
                "See the errors listed above.",
            ),
            (
                UserError::FailedToDeleteProjects { count: 1 },
                "Failed to delete 1 project.",
                "See the errors listed above.",
            ),
            (
                UserError::FailedToDeleteProjects { count: 3 },
                "Failed to delete 3 projects.",
                "See the errors listed above.",
            ),
        ];
        for (e, message, hint) in failures {
            assert_eq!(lines(&e), (message.to_string(), Some(hint.to_string())));
        }
    }

    #[test]
    fn auth_response_errors() {
        let host = "vault.example.com:8443";
        for body in [
            r#"{"error":"invalid_client"}"#,
            r#"{"error":"invalid_grant"}"#,
        ] {
            let e = auth_response_error(400, body, host).expect("classified");
            assert_eq!(
                lines(&e),
                (
                    "The access token is invalid, expired, or revoked.".to_string(),
                    Some("Create a new access token in the Bitwarden web app.".to_string())
                )
            );
        }
        assert!(auth_response_error(400, r#"{"error":"invalid_scope"}"#, host).is_none());
        assert!(auth_response_error(400, "<html>", host).is_none());

        assert_eq!(
            lines(&auth_response_error(429, "", host).expect("classified")),
            (
                "Too many auth attempts.".to_string(),
                Some(
                    "Wait a few minutes and retry; keep state enabled to reuse sessions."
                        .to_string()
                )
            )
        );

        let unexpected = (
            "Unexpected response from the auth endpoint at vault.example.com:8443.".to_string(),
            Some(SERVER_URL_HINT.to_string()),
        );
        for status in [404, 405, 415] {
            assert_eq!(
                lines(&auth_response_error(status, "<html></html>", host).expect("classified")),
                unexpected
            );
        }
        for status in [401, 403] {
            assert!(auth_response_error(status, "{}", host).is_none());
        }
        assert!(invalid_access_token("invalid_scope").is_none());
        assert_eq!(
            lines(&auth_response_error(200, "<html>auth</html>", host).expect("classified")),
            unexpected
        );
        assert!(auth_response_error(200, "{}", host).is_none());
        assert!(auth_response_error(500, "<html>oops</html>", host).is_none());
    }

    #[test]
    fn auth_errors() {
        const IDENTITY_URL: &str = "https://u:p@identity.example.com/identity";

        let e = auth_error(LoginError::Crypto(CryptoError::Decrypt), IDENTITY_URL);
        assert_eq!(
            lines(&e),
            (
                "The access token's decryption key is invalid.".to_string(),
                Some(HINT_COPY_TOKEN.to_string())
            )
        );

        let e = auth_error(
            LoginError::MissingField(bitwarden_core::MissingFieldError("organization")),
            IDENTITY_URL,
        );
        assert_eq!(
            lines(&e),
            (
                "Unexpected auth response from the server at identity.example.com.".to_string(),
                None
            )
        );

        // Without a profile, bws logs in against the SDK's default identity server.
        let e = auth_error(
            LoginError::InvalidResponse,
            &bitwarden_core::ClientSettings::default().identity_url,
        );
        assert_eq!(
            e.to_string(),
            "Unexpected auth response from the server at identity.bitwarden.com."
        );

        let Err(token_error) = "not-a-token".parse::<bitwarden::secrets_manager::AccessToken>()
        else {
            panic!("token to be malformed");
        };
        let e = auth_error(LoginError::AccessTokenInvalid(token_error), IDENTITY_URL);
        assert_eq!(
            lines(&e),
            (
                "The access token is malformed.".to_string(),
                Some(HINT_COPY_TOKEN.to_string())
            )
        );

        let e = auth_error(LoginError::AuthenticationFailed, IDENTITY_URL);
        assert_eq!(lines(&e), ("Failed to authenticate.".to_string(), None));
        assert!(std::error::Error::source(&e).is_some());
    }

    #[test]
    fn auth_failed_reasons() {
        assert_eq!(
            auth_failed(&["", " invalid client credentials ", "invalid_client"])
                .expect("classified")
                .to_string(),
            "Auth failed: Invalid client credentials."
        );
        assert_eq!(
            auth_failed(&["Username or password is incorrect. Try again.", "", ""])
                .expect("classified")
                .to_string(),
            "Auth failed: Username or password is incorrect. Try again."
        );
        assert_eq!(
            auth_failed(&["", " ", ""]).expect("classified").to_string(),
            "Auth failed."
        );
    }

    #[test]
    fn render_formats_lines() {
        let report = eyre::Report::new(UserError::Dns {
            host: "example.com".to_string(),
        });
        assert_eq!(
            render(&report),
            "Error: Could not resolve host example.com.\nHint: Check the server URL and your network connection.\n"
        );

        let report = eyre::Report::new(UserError::BadRequest {
            reason: "400 Bad Request".to_string(),
        });
        assert_eq!(
            render(&report),
            "Error: The server rejected the request (400 Bad Request).\n"
        );
        assert!(!render_verbose(&report).contains("Hint:"));

        let report = eyre::Report::new(UserError::Dns {
            host: "example.com".to_string(),
        });
        let verbose = render_verbose(&report);
        assert!(verbose.starts_with("Error: "), "{verbose}");
        assert!(
            verbose.ends_with("\nHint: Check the server URL and your network connection.\n"),
            "{verbose}"
        );

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
