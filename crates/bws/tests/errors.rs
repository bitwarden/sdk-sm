mod common;

use std::net::TcpListener;

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

fn auth_with(server: &MockServer, token: &str) -> (i32, String) {
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

fn closed_port_url() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener to bind");
    let port = listener.local_addr().expect("listener address").port();
    drop(listener);
    format!("http://127.0.0.1:{port}")
}

#[test]
fn debug_env_shows_full_report() {
    let url = closed_port_url();

    let (code, _, stderr) = bws(
        &["secret", "list", "-u", &url],
        &[("BWS_ACCESS_TOKEN", TEST_TOKEN), ("BWS_DEBUG", "1")],
    );

    assert_eq!(code, 1);
    assert!(stderr.contains("Location:"), "stderr:\n{stderr}");
}

#[test]
fn server_error_hides_html_body() {
    let server = MockServer::start(vec![Route {
        method: "POST",
        path_prefix: "/identity/connect/token",
        status: 500,
        content_type: "text/html",
        body: "<html>oops</html>",
    }]);

    let (code, _, stderr) = bws(
        &["secret", "list", "-u", &server.url()],
        &[("BWS_ACCESS_TOKEN", TEST_TOKEN)],
    );

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &[
            "Error: The server returned an error (500 Internal Server Error).",
            "Hint: Try again later.",
        ],
    );
}

#[test]
fn server_message_cannot_inject_escape_sequences() {
    let server = MockServer::start(vec![Route {
        method: "POST",
        path_prefix: "/identity/connect/token",
        status: 400,
        content_type: "application/json",
        // Escapes that would clear the screen and rewrite the line the user already saw.
        body: r#"{"message":"Bad request\u001b[2J\u001b[1G","object":"error"}"#,
    }]);

    let (code, _, stderr) = bws(
        &["secret", "list", "-u", &server.url()],
        &[("BWS_ACCESS_TOKEN", TEST_TOKEN)],
    );

    assert_eq!(code, 1);
    assert!(
        !stderr.chars().any(|c| c.is_control() && c != '\n'),
        "stderr carries control characters:\n{stderr:?}"
    );
    // The escapes survive as inert text rather than reaching the terminal as commands.
    assert_error(&stderr, &["Error: Bad request[2J[1G."]);
}

#[cfg(unix)]
#[test]
fn unwritable_stdout_is_not_reported_as_a_crash() {
    // `/dev/full` fails every write with ENOSPC, the way writing into a full disk does. It only
    // exists on Linux; the other reachable failure, EBADF from a closed descriptor, cannot be
    // tested at all because std turns it into a successful no-op write.
    let Ok(stdout) = std::fs::OpenOptions::new().write(true).open("/dev/full") else {
        return;
    };
    let child = std::process::Command::new(env!("CARGO_BIN_EXE_bws"))
        .args(["completions", "zsh"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::from(stdout))
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("bws binary to run");

    let output = child.wait_with_output().expect("bws to finish");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(1), "stderr:\n{stderr}");
    for forbidden in ["crashed unexpectedly", "report it"] {
        assert!(!stderr.contains(forbidden), "stderr:\n{stderr}");
    }
    assert_error(
        &stderr,
        &["Error: Could not write to stdout: no space left on device."],
    );
}

#[test]
fn bare_bws_prints_plain_help() {
    let (code, _, stderr) = bws(&[], &[]);

    assert_eq!(code, 1);
    assert!(stderr.contains("Usage:"), "stderr:\n{stderr}");
    assert!(!stderr.contains('\x1b'), "stderr:\n{stderr}");
}

#[cfg(unix)]
#[test]
fn closed_stdout_exits_quietly() {
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_bws"))
        .args(["completions", "zsh"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("bws binary to run");
    drop(child.stdout.take());

    let output = child.wait_with_output().expect("bws to finish");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(0), "stderr:\n{stderr}");
    assert!(stderr.is_empty(), "stderr:\n{stderr}");
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

    let (code, stderr) = auth_with(&server, TEST_TOKEN);

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
fn auth_rate_limited() {
    let server = identity(429, "text/plain", "");

    let (code, stderr) = auth_with(&server, TEST_TOKEN);

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &[
            "Error: Too many auth attempts.",
            "Hint: Wait a few minutes and retry; keep state enabled to reuse sessions.",
        ],
    );
}

#[test]
fn auth_endpoint_returns_html() {
    let server = identity(200, "text/html", "<html>auth page</html>");
    let host = server.url().trim_start_matches("http://").to_string();

    let (code, stderr) = auth_with(&server, TEST_TOKEN);

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &[
            &format!("Error: Unexpected response from the auth endpoint at {host}."),
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

    let (code, stderr) = auth_with(&server, &token);

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

    let (code, stderr) = auth_with(&server, &format!(" {TEST_TOKEN}\n"));

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

    let (code, stderr) = auth_with(&server, TEST_TOKEN);

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
fn auth_failed_with_server_message() {
    let server = identity(
        400,
        "application/json",
        r#"{"error":"invalid_request","error_description":"bad","ErrorModel":{"Message":"Machine account is disabled.","Object":"error"}}"#,
    );

    let (code, stderr) = auth_with(&server, TEST_TOKEN);

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &["Error: Auth failed: Machine account is disabled."],
    );
}

#[test]
fn auth_endpoint_method_not_allowed() {
    let server = identity(405, "text/html", "<html>nope</html>");
    let host = server.url().trim_start_matches("http://").to_string();

    let (code, stderr) = auth_with(&server, TEST_TOKEN);

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &[
            &format!("Error: Unexpected response from the auth endpoint at {host}."),
            "Hint: Check the server URL (--server-url or server-base in the config).",
        ],
    );
}

#[test]
fn run_shell_not_found() {
    let server = identity(200, "application/json", IDENTITY_OK_BODY);

    let (code, _, stderr) = bws(
        &[
            "run",
            "-u",
            &server.url(),
            "--shell",
            "definitely-not-a-shell",
            "echo",
            "hi",
        ],
        &[("BWS_ACCESS_TOKEN", TEST_TOKEN)],
    );

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &[
            "Error: Could not find the shell 'definitely-not-a-shell'.",
            "Hint: Install the shell, or set the full path with --shell <shell>.",
        ],
    );
}

#[test]
fn secret_delete_failure() {
    let server = MockServer::start(vec![
        Route {
            method: "POST",
            path_prefix: "/identity/connect/token",
            status: 200,
            content_type: "application/json",
            body: IDENTITY_OK_BODY,
        },
        Route {
            method: "POST",
            path_prefix: "/api/secrets/delete",
            status: 200,
            content_type: "application/json",
            body: r#"{"data":[{"id":"b4bd53be-8ee3-41d7-8c1f-3c0e2bf44a5f","error":"The secret is referenced by an automation"}]}"#,
        },
    ]);

    let (code, _, stderr) = bws(
        &[
            "secret",
            "delete",
            "-u",
            &server.url(),
            "b4bd53be-8ee3-41d7-8c1f-3c0e2bf44a5f",
        ],
        &[("BWS_ACCESS_TOKEN", TEST_TOKEN)],
    );

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &[
            "1 secret had an error:",
            "b4bd53be-8ee3-41d7-8c1f-3c0e2bf44a5f: The secret is referenced by an automation",
            "Error: Failed to delete 1 secret.",
            "Hint: See the errors listed above.",
        ],
    );
}

#[test]
fn project_delete_failure() {
    let server = MockServer::start(vec![
        Route {
            method: "POST",
            path_prefix: "/identity/connect/token",
            status: 200,
            content_type: "application/json",
            body: IDENTITY_OK_BODY,
        },
        Route {
            method: "POST",
            path_prefix: "/api/projects/delete",
            status: 200,
            content_type: "application/json",
            body: r#"{"data":[{"id":"e4bd53be-8ee3-41d7-8c1f-3c0e2bf44a5f","error":"The project is referenced by a secret"}]}"#,
        },
    ]);

    let (code, _, stderr) = bws(
        &[
            "project",
            "delete",
            "-u",
            &server.url(),
            "e4bd53be-8ee3-41d7-8c1f-3c0e2bf44a5f",
        ],
        &[("BWS_ACCESS_TOKEN", TEST_TOKEN)],
    );

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &[
            "1 project had an error:",
            "e4bd53be-8ee3-41d7-8c1f-3c0e2bf44a5f: The project is referenced by a secret",
            "Error: Failed to delete 1 project.",
            "Hint: See the errors listed above.",
        ],
    );
}
