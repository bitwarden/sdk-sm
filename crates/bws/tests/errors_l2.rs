mod common;

use common::{MockServer, Route, TEST_TOKEN, assert_error, bws};

/// Identity token response that logs in `TEST_TOKEN`, copied from `crates/fake-server`.
const IDENTITY_OK_BODY: &str = include_str!("fixtures/identity_ok.json");

fn identity(status: u16, content_type: &'static str, body: &'static str) -> MockServer {
    MockServer::start(vec![Route {
        method: "POST",
        path_prefix: "/identity/connect/token",
        status,
        content_type,
        body,
    }])
}

fn login_with(server: &MockServer, token: &str) -> (i32, String) {
    let (code, _, stderr) = bws(
        &["secret", "list", "-u", &server.url()],
        &[("BWS_ACCESS_TOKEN", token)],
    );
    (code, stderr)
}

/// Writes `contents` to a config file in a fresh temp dir, which is removed with the returned
/// guard.
fn config_file(contents: &str) -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().expect("temp dir to be created");
    let path = dir.path().join("config");
    std::fs::write(&path, contents).expect("config to be written");
    let path = path.to_str().expect("utf-8 path").to_string();
    (dir, path)
}

#[test]
fn missing_access_token() {
    let (code, _, stderr) = bws(&["secret", "list"], &[]);

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &[
            "Error: No access token provided.",
            "Hint: Pass --access-token or set BWS_ACCESS_TOKEN.",
        ],
    );
}

#[test]
fn malformed_access_token() {
    let (code, _, stderr) = bws(&["secret", "list"], &[("BWS_ACCESS_TOKEN", "garbage")]);

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &[
            "Error: The access token is malformed.",
            "Hint: Copy the full access token from the Bitwarden web app.",
        ],
    );
}

#[test]
fn missing_config_file() {
    let dir = tempfile::tempdir().expect("temp dir to be created");
    let path = dir.path().join("missing").join("config");
    let path = path.to_str().expect("utf-8 path");

    let (code, _, stderr) = bws(
        &["secret", "list", "-f", path],
        &[("BWS_ACCESS_TOKEN", TEST_TOKEN)],
    );

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &[&format!("Error: Config file '{path}' does not exist.")],
    );
}

#[test]
fn invalid_config_file() {
    let (_dir, path) = config_file("hello");

    let (code, _, stderr) = bws(
        &["secret", "list", "-f", &path],
        &[("BWS_ACCESS_TOKEN", TEST_TOKEN)],
    );

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &[&format!(
            "Error: Invalid config file '{path}' at line 1, column 6: key with no value, expected `=`."
        )],
    );
}

#[test]
fn unknown_profile() {
    let (_dir, path) = config_file("[profiles.default]\nserver_base = \"https://example.com\"\n");

    let (code, _, stderr) = bws(
        &["secret", "list", "-f", &path, "-p", "work"],
        &[("BWS_ACCESS_TOKEN", TEST_TOKEN)],
    );

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &[&format!("Error: Profile 'work' not found in '{path}'.")],
    );
}

#[test]
fn revoked_access_token() {
    let server = identity(400, "application/json", r#"{"error":"invalid_client"}"#);

    let (code, stderr) = login_with(&server, TEST_TOKEN);

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &[
            "Error: The access token is invalid, expired, or revoked.",
            "Hint: Create a new access token in the Bitwarden web app.",
        ],
    );
}

#[test]
fn login_rate_limited() {
    let server = identity(429, "text/plain", "");

    let (code, stderr) = login_with(&server, TEST_TOKEN);

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &[
            "Error: Too many login attempts.",
            "Hint: Wait a few minutes and retry; keep state enabled to reuse sessions.",
        ],
    );
}

#[test]
fn login_endpoint_returns_html() {
    let server = identity(200, "text/html", "<html>login page</html>");
    let host = server.url().trim_start_matches("http://").to_string();

    let (code, stderr) = login_with(&server, TEST_TOKEN);

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &[
            &format!("Error: Unexpected response from the login endpoint at {host}."),
            "Hint: Check the server URL (--server-url or server-base in the config).",
        ],
    );
}

#[test]
fn wrong_decryption_key() {
    let server = identity(200, "application/json", IDENTITY_OK_BODY);
    let (id_and_secret, _) = TEST_TOKEN
        .split_once(':')
        .expect("TEST_TOKEN to contain ':'");
    let token = format!("{id_and_secret}:AAAAAAAAAAAAAAAAAAAAAA==");

    let (code, stderr) = login_with(&server, &token);

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &[
            "Error: The access token's decryption key is invalid.",
            "Hint: Copy the full access token from the Bitwarden web app.",
        ],
    );
}

#[cfg(unix)]
#[test]
fn unwritable_state_dir_warns() {
    let server = identity(200, "application/json", IDENTITY_OK_BODY);
    let dir = tempfile::tempdir().expect("temp dir to be created");
    let blocker = dir.path().join("file");
    std::fs::write(&blocker, "").expect("file to be written");
    let state_dir = blocker.join("state");
    let (_config_dir, path) = config_file(&format!(
        "[profiles.default]\nserver_base = \"{}\"\nstate_dir = \"{}\"\n",
        server.url(),
        state_dir.display()
    ));

    let (_, _, stderr) = bws(
        &["secret", "list", "-f", &path],
        &[("BWS_ACCESS_TOKEN", TEST_TOKEN)],
    );

    let lines: Vec<&str> = stderr.lines().take(2).collect();
    assert_eq!(
        lines,
        [
            format!(
                "Warning: Could not use state directory '{}': not a directory.",
                state_dir.display()
            )
            .as_str(),
            "Hint: Continuing without state; set a writable dir with: bws config state-dir <dir>",
        ],
        "stderr:\n{stderr}"
    );
}

#[test]
fn empty_access_token_is_missing() {
    let (code, _, stderr) = bws(&["secret", "list"], &[("BWS_ACCESS_TOKEN", " ")]);

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &[
            "Error: No access token provided.",
            "Hint: Pass --access-token or set BWS_ACCESS_TOKEN.",
        ],
    );
}

#[test]
fn access_token_whitespace_is_trimmed() {
    let server = identity(400, "application/json", r#"{"error":"invalid_grant"}"#);

    let (code, stderr) = login_with(&server, &format!(" {TEST_TOKEN}\n"));

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &[
            "Error: The access token is invalid, expired, or revoked.",
            "Hint: Create a new access token in the Bitwarden web app.",
        ],
    );
}

#[test]
fn revoked_access_token_identity_fail_shape() {
    let server = identity(
        400,
        "application/json",
        r#"{"error":"invalid_client","error_description":"invalid_client","ErrorModel":{"Message":"","Object":"error"}}"#,
    );

    let (code, stderr) = login_with(&server, TEST_TOKEN);

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &[
            "Error: The access token is invalid, expired, or revoked.",
            "Hint: Create a new access token in the Bitwarden web app.",
        ],
    );
}

#[test]
fn login_failed_with_server_message() {
    let server = identity(
        400,
        "application/json",
        r#"{"error":"invalid_request","error_description":"bad","ErrorModel":{"Message":"Machine account is disabled.","Object":"error"}}"#,
    );

    let (code, stderr) = login_with(&server, TEST_TOKEN);

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &["Error: Login failed: Machine account is disabled."],
    );
}

#[test]
fn login_endpoint_method_not_allowed() {
    let server = identity(405, "text/html", "<html>nope</html>");
    let host = server.url().trim_start_matches("http://").to_string();

    let (code, stderr) = login_with(&server, TEST_TOKEN);

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &[
            &format!("Error: Unexpected response from the login endpoint at {host}."),
            "Hint: Check the server URL (--server-url or server-base in the config).",
        ],
    );
}
