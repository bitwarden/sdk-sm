//! `bws scan`: local, offline secret scanning (worktree / staged / history),
//! backed by the `bitwarden-scan` engine.
//!
//! Deliberately bypasses the access-token gate in `main.rs` — a scan is
//! purely local (it never talks to Secrets Manager), so it must work for a
//! developer who hasn't configured `bws` yet.

use std::{path::Path, process::Command as ProcessCommand};

use bitwarden_cli::Color;
use bitwarden_scan::{ScanOptions, ScanReport, Severity, report};
use color_eyre::eyre::{Result, bail};

use crate::{cli::Output, render::pretty_print};

/// No findings.
const EXIT_CLEAN: i32 = 0;
/// One or more findings present.
const EXIT_FINDINGS: i32 = 1;
/// The scan itself failed (bad path, git error, I/O error, ...).
const EXIT_ERROR: i32 = 2;

/// Parsed `bws scan` arguments (mirrors the `Commands::Scan` clap variant;
/// kept separate so the formatting/exit-code logic below can be unit-tested
/// without going through clap).
pub(crate) struct ScanArgs {
    pub(crate) path: std::path::PathBuf,
    pub(crate) staged: bool,
    /// `None`: worktree mode. `Some(None)`: `--history` with no range
    /// (full history). `Some(Some(range))`: `--history=<range>`.
    pub(crate) history: Option<Option<String>>,
    pub(crate) no_write: bool,
}

/// Run `bws scan` and return the process exit code (never panics on scan
/// failure — errors are printed to stderr and mapped to [`EXIT_ERROR`]).
pub(crate) fn run(args: ScanArgs, output: Output, color: Color) -> i32 {
    match run_inner(args, output, color) {
        Ok(exit_code) => exit_code,
        Err(err) => {
            eprintln!("Error: {err:#}");
            EXIT_ERROR
        }
    }
}

fn run_inner(args: ScanArgs, output: Output, color: Color) -> Result<i32> {
    let opts = ScanOptions::new(&args.path);

    let (scan_report, may_write_artifact) = if args.staged {
        (bitwarden_scan::scan_staged(&opts)?, false)
    } else if let Some(range) = &args.history {
        (
            bitwarden_scan::scan_history(&opts, range.as_deref())?,
            false,
        )
    } else {
        (bitwarden_scan::scan_worktree(&opts)?, true)
    };

    if may_write_artifact && !args.no_write {
        let root = Path::new(&scan_report.repo_root);
        report::write_artifact(root, &scan_report)?;
        warn_if_artifact_not_gitignored(root);
    }

    print_report(&scan_report, output, color)?;

    Ok(exit_code_for(&scan_report))
}

fn print_report(scan_report: &ScanReport, output: Output, color: Color) -> Result<()> {
    match output {
        Output::JSON => {
            let mut text = serde_json::to_string_pretty(scan_report)
                .expect("ScanReport serialization is infallible");
            text.push('\n');
            pretty_print("json", &text, color);
        }
        Output::YAML => {
            let text =
                serde_yaml::to_string(scan_report).expect("ScanReport serialization is infallible");
            pretty_print("yaml", &text, color);
        }
        Output::Table => {
            print!("{}", format_human(scan_report));
        }
        Output::TSV => {
            print!("{}", format_tsv(scan_report));
        }
        Output::Env => {
            bail!(
                "`--output env` is not supported for `bws scan` (there is no key/value pair to \
                 emit for a finding); use --output json, --output yaml, --output table, or \
                 --output tsv instead"
            );
        }
        Output::None => {}
    }
    Ok(())
}

/// Exit code for a completed scan: [`EXIT_FINDINGS`] whenever the report has
/// at least one finding, [`EXIT_CLEAN`] otherwise. A scan that failed
/// outright never reaches this function — see [`run`].
fn exit_code_for(scan_report: &ScanReport) -> i32 {
    if scan_report.findings.is_empty() {
        EXIT_CLEAN
    } else {
        EXIT_FINDINGS
    }
}

fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Low => "LOW",
        Severity::Medium => "MEDIUM",
        Severity::High => "HIGH",
    }
}

/// The `--output table` (human) rendering: one block per finding, then a
/// footer with the finding count, a truncation notice if applicable, and
/// provenance.
fn format_human(scan_report: &ScanReport) -> String {
    let mut out = String::new();
    for finding in &scan_report.findings {
        out.push_str(&format!(
            "{severity} {path}:{line}:{column} {rule_id} {preview}\n",
            severity = severity_label(finding.severity),
            path = finding.path,
            line = finding.line,
            column = finding.column,
            rule_id = finding.rule_id,
            preview = finding.preview,
        ));
        out.push_str(&format!("    fingerprint: {}\n", finding.fingerprint));
        if let Some(commit) = &finding.commit {
            out.push_str(&format!("    commit: {commit}"));
            if let Some(first_seen) = &finding.first_seen {
                out.push_str(&format!("  first_seen: {first_seen}"));
            }
            out.push('\n');
        }
    }
    out.push_str(&format_footer(scan_report));
    out
}

fn format_footer(scan_report: &ScanReport) -> String {
    let mut footer = String::new();

    footer.push_str(&match scan_report.findings.len() {
        1 => "1 finding".to_string(),
        n => format!("{n} findings"),
    });
    if scan_report.truncated {
        footer.push_str(
            " (truncated — the max-findings cap was hit; narrow the scan to see the rest)",
        );
    }
    footer.push('\n');

    let head_commit = scan_report
        .head_commit
        .as_deref()
        .map(short_commit)
        .unwrap_or_else(|| "none".to_string());
    let dirty = match scan_report.dirty {
        Some(true) => "true",
        Some(false) => "false",
        None => "unknown",
    };
    footer.push_str(&format!(
        "generated_at: {}  head_commit: {head_commit}  dirty: {dirty}\n",
        scan_report.generated_at
    ));

    if scan_report.findings.iter().any(|f| f.commit.is_some()) {
        footer.push_str(
            "remediation: rotate leaked secrets at the provider — do not rewrite git history\n",
        );
    }

    footer
}

/// First 7 characters of a commit hash, `git`'s traditional short form.
fn short_commit(commit: &str) -> String {
    commit.chars().take(7).collect()
}

/// Tab-separated, one line per finding — no header, no footer, matching the
/// data-only shape of the other `bws` commands' TSV output.
fn format_tsv(scan_report: &ScanReport) -> String {
    let mut out = String::new();
    for finding in &scan_report.findings {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            severity_label(finding.severity),
            finding.path,
            finding.line,
            finding.column,
            finding.rule_id,
            finding.preview,
            finding.fingerprint,
            finding.commit.as_deref().unwrap_or(""),
            finding.first_seen.as_deref().unwrap_or(""),
        ));
    }
    out
}

/// Warn on stderr when the findings artifact isn't gitignored — it records
/// finding locations (path/line/rule id/preview) and shouldn't be committed.
/// Uses `git check-ignore -q`; only warns on exit code 1 (definitely not
/// ignored). Any other outcome (git missing, `root` not a repository, the
/// path *is* ignored) is silent — we only want a confident, actionable
/// warning, never a false alarm outside a git repo.
fn warn_if_artifact_not_gitignored(root: &Path) {
    let status = ProcessCommand::new("git")
        .arg("-C")
        .arg(root)
        .args(["check-ignore", "-q", report::ARTIFACT_RELATIVE_PATH])
        // `-q` only suppresses the matched-path output on stdout; without a
        // git repository at all, `check-ignore` still writes a `fatal: not
        // a git repository` line to stderr. That outcome must stay silent
        // (see the doc comment above), so both streams are discarded rather
        // than inherited from this process (`Command::status`'s default).
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    if let Ok(status) = status
        && status.code() == Some(1)
    {
        eprintln!(
            "Warning: {} is not gitignored. It records finding locations (not secret values), \
             but should still not be committed — add it to .gitignore.",
            report::ARTIFACT_RELATIVE_PATH
        );
    }
}

#[cfg(test)]
mod tests {
    use bitwarden_scan::{Finding, ScanMode};

    use super::*;

    fn sample_finding(rule_id: &str, severity: Severity) -> Finding {
        Finding {
            fingerprint: "abc123fingerprint".to_string(),
            rule_id: rule_id.to_string(),
            severity,
            path: "src/config.ts".to_string(),
            line: 2,
            column: 14,
            preview: "AKIA****".to_string(),
            origin: "worktree".to_string(),
            commit: None,
            author: None,
            first_seen: None,
        }
    }

    fn sample_report(findings: Vec<Finding>, truncated: bool) -> ScanReport {
        ScanReport {
            schema_version: 1,
            generated_at: "2026-08-11T12:00:00Z".to_string(),
            repo_root: "/tmp/example".to_string(),
            head_commit: Some("0123456789abcdef".to_string()),
            dirty: Some(false),
            scan_mode: ScanMode::Worktree,
            truncated,
            findings,
        }
    }

    #[test]
    fn exit_code_is_clean_when_no_findings() {
        let report = sample_report(Vec::new(), false);
        assert_eq!(exit_code_for(&report), EXIT_CLEAN);
    }

    #[test]
    fn exit_code_is_findings_when_findings_present() {
        let report = sample_report(
            vec![sample_finding("aws-access-key-id", Severity::High)],
            false,
        );
        assert_eq!(exit_code_for(&report), EXIT_FINDINGS);
    }

    #[test]
    fn human_format_includes_location_rule_and_fingerprint() {
        let report = sample_report(
            vec![sample_finding("aws-access-key-id", Severity::High)],
            false,
        );
        let text = format_human(&report);
        assert!(text.contains("HIGH src/config.ts:2:14 aws-access-key-id AKIA****"));
        assert!(text.contains("fingerprint: abc123fingerprint"));
        assert!(text.contains("1 finding\n"));
    }

    #[test]
    fn human_format_never_contains_a_full_secret_value() {
        let aws_example_key = "AKIA".to_string() + "IOSFODNN7EXAMPLE";
        let report = sample_report(
            vec![sample_finding("aws-access-key-id", Severity::High)],
            false,
        );
        let text = format_human(&report);
        assert!(!text.contains(&aws_example_key));
    }

    #[test]
    fn human_format_shows_history_provenance_and_remediation_note() {
        let mut finding = sample_finding("aws-access-key-id", Severity::High);
        finding.origin = "history".to_string();
        finding.commit = Some("deadbeefcafe".to_string());
        finding.first_seen = Some("2020-01-01T00:00:00Z".to_string());
        let mut report = sample_report(vec![finding], false);
        report.scan_mode = ScanMode::History;

        let text = format_human(&report);
        assert!(text.contains("commit: deadbeefcafe  first_seen: 2020-01-01T00:00:00Z"));
        assert!(text.contains("remediation: rotate leaked secrets at the provider"));
    }

    #[test]
    fn footer_omits_remediation_note_when_no_history_findings() {
        let report = sample_report(
            vec![sample_finding("aws-access-key-id", Severity::High)],
            false,
        );
        assert!(!format_footer(&report).contains("remediation"));
    }

    #[test]
    fn footer_shows_truncation_notice_when_truncated() {
        let report = sample_report(
            vec![sample_finding("aws-access-key-id", Severity::High)],
            true,
        );
        assert!(format_footer(&report).contains("truncated"));
    }

    #[test]
    fn footer_shortens_the_head_commit() {
        let report = sample_report(Vec::new(), false);
        assert!(format_footer(&report).contains("head_commit: 0123456"));
        assert!(!format_footer(&report).contains("0123456789abcdef"));
    }

    #[test]
    fn tsv_format_is_one_tab_separated_line_per_finding() {
        let report = sample_report(
            vec![sample_finding("aws-access-key-id", Severity::High)],
            false,
        );
        let text = format_tsv(&report);
        let mut fields = text.trim_end().split('\t');
        assert_eq!(fields.next(), Some("HIGH"));
        assert_eq!(fields.next(), Some("src/config.ts"));
    }
}
