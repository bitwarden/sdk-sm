mod common;

use std::net::TcpListener;

use common::{MockServer, Route, TEST_TOKEN, assert_error, bws};

fn closed_port_url() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener to bind");
    let port = listener.local_addr().expect("listener address").port();
    drop(listener);
    format!("http://127.0.0.1:{port}")
}

#[test]
fn connection_refused_is_two_lines() {
    let url = closed_port_url();
    let host = url.trim_start_matches("http://");

    let (code, _, stderr) = bws(
        &["secret", "list", "-u", &url],
        &[("BWS_ACCESS_TOKEN", TEST_TOKEN)],
    );

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &[
            &format!("Error: Could not connect to {host}: connection refused."),
            "Hint: Check the server URL (--server-url or server-base in the config).",
        ],
    );
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
fn clap_errors_are_unchanged() {
    let (code, _, stderr) = bws(&["secret", "get", "not-a-uuid"], &[]);

    assert_eq!(code, 2);
    assert!(
        stderr.starts_with("error: invalid value"),
        "stderr:\n{stderr}"
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
