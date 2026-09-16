mod common;

use common::{MockServer, Route, TEST_TOKEN, assert_error, bws};

/// Identity token response that logs in `TEST_TOKEN`, copied from `crates/fake-server`.
const IDENTITY_OK_BODY: &str = include_str!("fixtures/identity_ok.json");
const NOT_FOUND_BODY: &str =
    r#"{"message": "Resource not found.", "validationErrors": null, "object": "error"}"#;
const ID: &str = "15744a66-341a-4c62-af50-b16300fc8b5d";
const ACCESS_HINT: &str = "Hint: Check the ID and that the machine account has access to it.";

fn server(extra: Vec<Route>) -> MockServer {
    let mut routes = vec![Route {
        method: "POST",
        path_prefix: "/identity/connect/token",
        status: 200,
        content_type: "application/json",
        body: IDENTITY_OK_BODY,
    }];
    routes.extend(extra);
    MockServer::start(routes)
}

fn not_found(path_prefix: &'static str) -> Route {
    Route {
        method: "GET",
        path_prefix,
        status: 404,
        content_type: "application/json",
        body: NOT_FOUND_BODY,
    }
}

fn logged_in(server: &MockServer, args: &[&str]) -> (i32, String) {
    let url = server.url();
    let args = [&["-u", url.as_str()], args].concat();
    let (code, _, stderr) = bws(&args, &[("BWS_ACCESS_TOKEN", TEST_TOKEN)]);
    (code, stderr)
}

fn config_path(dir: &tempfile::TempDir) -> String {
    dir.path()
        .join("config")
        .to_str()
        .expect("utf-8 path")
        .to_string()
}

#[test]
fn secret_not_found() {
    let server = server(vec![not_found("/api/secrets/")]);

    let (code, stderr) = logged_in(&server, &["secret", "get", ID]);

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &[
            &format!("Error: Secret {ID} not found or not accessible."),
            ACCESS_HINT,
        ],
    );
}

#[test]
fn project_not_found() {
    let server = server(vec![not_found("/api/projects/")]);

    let (code, stderr) = logged_in(&server, &["project", "get", ID]);

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &[
            &format!("Error: Project {ID} not found or not accessible."),
            ACCESS_HINT,
        ],
    );
}

#[test]
fn secret_key_empty() {
    let server = server(vec![]);

    let (code, stderr) = logged_in(&server, &["secret", "create", "", "value", ID]);

    assert_eq!(code, 1);
    assert_error(&stderr, &["Error: Secret key must not be empty."]);
}

#[test]
fn project_name_whitespace() {
    let server = server(vec![]);

    let (code, stderr) = logged_in(&server, &["project", "create", "   "]);

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &["Error: Project name must not contain only whitespace."],
    );
}

#[test]
fn run_shell_not_found() {
    let server = server(vec![]);

    let (code, stderr) = logged_in(
        &server,
        &["run", "--shell", "/nonexistent/shell", "--", "echo", "hi"],
    );

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &[
            "Error: Shell '/nonexistent/shell' not found.",
            "Hint: Install it or pass a different shell with --shell.",
        ],
    );
}

#[cfg(unix)]
#[test]
fn run_without_command() {
    let server = server(vec![]);

    let (code, stderr) = logged_in(&server, &["run"]);

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &[
            "Error: No command provided.",
            "Hint: Usage: bws run -- <command>",
        ],
    );
}

#[test]
fn config_missing_value() {
    let dir = tempfile::tempdir().expect("temp dir to be created");
    let path = config_path(&dir);

    let (code, _, stderr) = bws(&["config", "server-base", "-f", &path], &[]);

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &[
            "Error: Missing value for 'server-base'.",
            "Hint: Usage: bws config <name> <value>",
        ],
    );
}

#[test]
fn config_invalid_state_opt_out() {
    let dir = tempfile::tempdir().expect("temp dir to be created");
    let path = config_path(&dir);

    let (code, _, stderr) = bws(&["config", "state-opt-out", "maybe", "-f", &path], &[]);

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &["Error: Invalid value for 'state-opt-out'; expected true, false, 1, or 0."],
    );
}

#[test]
fn config_delete_without_file() {
    let dir = tempfile::tempdir().expect("temp dir to be created");
    let path = config_path(&dir);

    let (code, _, stderr) = bws(&["config", "-d", "-f", &path], &[]);

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &[&format!(
            "Error: Config file '{path}' does not exist; nothing to delete."
        )],
    );
}

#[test]
fn completions_without_shell() {
    let (code, _, stderr) = bws(&["completions"], &[]);

    assert_eq!(code, 1);
    assert_error(
        &stderr,
        &[
            "Error: Could not detect your shell.",
            "Hint: Pass it explicitly: bws completions <bash|elvish|fish|powershell|zsh>",
        ],
    );
}
