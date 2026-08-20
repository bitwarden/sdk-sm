//! Integration tests for `bws scan`, exercised as a real subprocess (`bws`
//! is a binary-only crate, so this is the only way to test the wired-up CLI
//! end to end — unit tests for the formatter and exit-code mapping live in
//! `src/command/scan.rs`).

use std::{fs, process::Command};

/// Never set `BWS_ACCESS_TOKEN`/`-t` here: `scan` must work without one.
fn bws_scan(dir: &std::path::Path, extra_args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bws"))
        .env_remove("BWS_ACCESS_TOKEN")
        .arg("scan")
        .args(extra_args)
        .arg(dir)
        .output()
        .expect("bws scan must spawn")
}

#[test]
fn scan_finds_a_synthetic_secret_and_writes_the_artifact() {
    let dir = tempfile::tempdir().expect("tempdir creation must succeed in test");
    let aws_example_key = "AKIA".to_string() + "IOSFODNN7EXAMPLE";
    fs::write(
        dir.path().join("config.ts"),
        format!("const key = \"{aws_example_key}\";\n"),
    )
    .expect("writing fixture file must succeed");

    let output = bws_scan(dir.path(), &["--output", "table"]);

    assert_eq!(
        output.status.code(),
        Some(1),
        "findings present must exit 1: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("aws-access-key-id"),
        "stdout should name the rule that fired: {stdout}"
    );
    assert!(
        stdout.contains("fingerprint:"),
        "stdout should include a copyable fingerprint line: {stdout}"
    );
    assert!(
        !stdout.contains(&aws_example_key),
        "stdout must never contain the matched secret value: {stdout}"
    );

    let artifact = dir.path().join(".bitwarden/secret-findings.json");
    assert!(
        artifact.is_file(),
        "the findings artifact must be written by default"
    );
    let contents = fs::read_to_string(&artifact).expect("the findings artifact must be readable");
    assert!(
        contents.contains("aws-access-key-id"),
        "artifact should record the rule id: {contents}"
    );
    assert!(
        !contents.contains(&aws_example_key),
        "the artifact must never contain the matched secret value: {contents}"
    );
}

#[test]
fn scan_exits_zero_and_writes_no_findings_on_a_clean_directory() {
    let dir = tempfile::tempdir().expect("tempdir creation must succeed in test");
    fs::write(dir.path().join("a.txt"), "hello\n").expect("writing fixture file must succeed");

    let output = bws_scan(dir.path(), &[]);

    assert_eq!(
        output.status.code(),
        Some(0),
        "a clean directory must exit 0: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let artifact = dir.path().join(".bitwarden/secret-findings.json");
    assert!(
        artifact.is_file(),
        "a clean scan still writes the artifact (scanned, zero findings)"
    );
}

#[test]
fn scan_no_write_skips_the_artifact() {
    let dir = tempfile::tempdir().expect("tempdir creation must succeed in test");
    fs::write(dir.path().join("a.txt"), "hello\n").expect("writing fixture file must succeed");

    let output = bws_scan(dir.path(), &["--no-write"]);

    assert_eq!(output.status.code(), Some(0));
    assert!(
        !dir.path().join(".bitwarden/secret-findings.json").exists(),
        "--no-write must skip the artifact"
    );
}

#[test]
fn scan_works_without_an_access_token() {
    // No BWS_ACCESS_TOKEN and no -t/--access-token anywhere in this test —
    // if scan required auth, this would fail with "Missing access token"
    // and a non-zero/non-one exit code instead of a clean scan result.
    let dir = tempfile::tempdir().expect("tempdir creation must succeed in test");
    fs::write(dir.path().join("a.txt"), "hello\n").expect("writing fixture file must succeed");

    let output = bws_scan(dir.path(), &[]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("access token"),
        "scan must not require an access token: stderr={stderr}"
    );
    assert_eq!(output.status.code(), Some(0));
}
