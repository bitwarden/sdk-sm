//! Deterministic, non-agentic secret-scanning engine for Bitwarden Secrets
//! Manager.
//!
//! Detects hardcoded secrets in a codebase and produces a [`ScanReport`]:
//! location, rule id, and a masked preview only — **never the matched
//! value**. `bws scan` (see the `bws` crate) is the producer: it runs a scan
//! and writes the report to a findings artifact
//! ([`report::write_artifact`]) that MCP servers (e.g. Bitwarden
//! agent-access) read read-only; the model that reads it never triggers a
//! scan itself (see the secret-scanning design doc, §1, §2, §6).
//!
//! This crate has no dependency on the vault or any encryption logic, and
//! does not link `git2`/`gix` — all git access goes through the `git`
//! binary via subprocess, tolerating git being absent or the scan root not
//! being a repository.

pub mod baseline;
pub mod detect;
mod diffparse;
pub mod finding;
pub mod git;
pub mod report;
pub mod rules;
pub mod staged;
pub mod walk;

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::SystemTime,
};

pub use baseline::Baseline;
pub use finding::{Finding, compute_fingerprint, normalize_line};
pub use report::{ScanMode, ScanReport, artifact_path, load_artifact, write_artifact};
pub use rules::Severity;

/// 1 MiB — the default per-file size cap ([`ScanOptions::max_file_size`]).
pub const DEFAULT_MAX_FILE_SIZE: u64 = 1024 * 1024;
/// Default cap on findings per scan ([`ScanOptions::max_findings`]); the
/// report's `truncated` flag is set when this is hit.
pub const DEFAULT_MAX_FINDINGS: usize = 1000;

/// Input to every scan entry point.
#[derive(Debug, Clone)]
pub struct ScanOptions {
    /// Scan root: a repo root or a plain directory. Does not need to be a
    /// git repository for [`scan_worktree`] — only [`scan_staged`] requires
    /// one.
    pub root: PathBuf,
    /// Files larger than this are skipped by the walk.
    pub max_file_size: u64,
    /// Stop after this many findings; sets [`ScanReport::truncated`].
    pub max_findings: usize,
}

impl ScanOptions {
    /// Options for `root` with the documented defaults for everything
    /// else.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            max_file_size: DEFAULT_MAX_FILE_SIZE,
            max_findings: DEFAULT_MAX_FINDINGS,
        }
    }
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self::new(".")
    }
}

/// Errors from any scan or artifact operation. Never carries a matched
/// secret value — only paths, error text from the OS/serde/git, and
/// artifact metadata.
#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("I/O error at {}: {source}", path.display())]
    IoPath {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to (de)serialize the findings artifact: {0}")]
    Json(#[from] serde_json::Error),

    #[error(
        "findings artifact schema_version {found} is not supported by this build (expected {expected})"
    )]
    UnsupportedSchemaVersion { found: u32, expected: u32 },

    #[error("scan root does not exist or is not a directory: {}", .0.display())]
    InvalidRoot(PathBuf),

    #[error("`{command}` failed: {message}")]
    GitCommand { command: String, message: String },
}

/// Canonicalize and validate a scan root. Every entry point starts here so
/// findings always carry an absolute, symlink-resolved `repo_root` and
/// [`walk::to_rel_path`] can reliably strip it as a prefix.
fn normalize_root(root: &Path) -> Result<PathBuf, ScanError> {
    let canonical =
        fs::canonicalize(root).map_err(|_source| ScanError::InvalidRoot(root.to_path_buf()))?;
    if !canonical.is_dir() {
        return Err(ScanError::InvalidRoot(root.to_path_buf()));
    }
    Ok(canonical)
}

fn generated_at_now() -> String {
    humantime::format_rfc3339_seconds(SystemTime::now()).to_string()
}

/// `git -C <root> rev-parse HEAD`, trimmed. `None` if git is missing, the
/// command fails, or `root` is not a git repository (or has no commits
/// yet) — never panics.
pub fn git_head_commit(root: &Path) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if commit.is_empty() {
        None
    } else {
        Some(commit)
    }
}

/// `git -C <root> status --porcelain` is non-empty. `None` if git is
/// missing, the command fails, or `root` is not a git repository — never
/// panics.
pub fn git_is_dirty(root: &Path) -> Option<bool> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["status", "--porcelain"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(!output.stdout.is_empty())
}

/// Scan every file on disk under `opts.root` (respecting `.gitignore`, the
/// path allowlist, and `.bitwardenignore`). Findings are sorted by
/// `(path, line, rule_id)` for deterministic output.
pub fn scan_worktree(opts: &ScanOptions) -> Result<ScanReport, ScanError> {
    let root = normalize_root(&opts.root)?;
    let baseline = Baseline::load(&root)?;
    let files = walk::walk(&root, opts.max_file_size)?;

    let mut findings = Vec::new();
    let mut truncated = false;

    'files: for file in &files {
        let Ok(bytes) = fs::read(&file.abs_path) else {
            // Unreadable file (permission denied, race with a delete):
            // skip it rather than aborting the whole scan.
            continue;
        };
        let text = String::from_utf8_lossy(&bytes);

        for (idx, line) in text.lines().enumerate() {
            let line_no = (idx as u64).saturating_add(1);
            for detected in detect::scan_line(line) {
                if findings.len() >= opts.max_findings {
                    truncated = true;
                    break 'files;
                }
                let normalized = normalize_line(line);
                let fingerprint =
                    compute_fingerprint(detected.rule_id, &file.rel_path, &normalized);
                if baseline.contains(&fingerprint) {
                    continue;
                }
                findings.push(Finding {
                    fingerprint,
                    rule_id: detected.rule_id.to_string(),
                    severity: detected.severity,
                    path: file.rel_path.clone(),
                    line: line_no,
                    column: detected.column,
                    preview: detected.preview,
                    origin: "worktree".to_string(),
                    commit: None,
                    author: None,
                    first_seen: None,
                });
            }
        }
    }

    findings.sort_by(|a, b| (&a.path, a.line, &a.rule_id).cmp(&(&b.path, b.line, &b.rule_id)));

    Ok(ScanReport {
        schema_version: report::SCHEMA_VERSION,
        generated_at: generated_at_now(),
        repo_root: root.to_string_lossy().into_owned(),
        head_commit: git_head_commit(&root),
        dirty: git_is_dirty(&root),
        scan_mode: ScanMode::Worktree,
        truncated,
        findings,
    })
}

/// Scan only the added (`+`) lines of `git diff --cached` under
/// `opts.root`. Requires `opts.root` to be inside a git repository — unlike
/// [`scan_worktree`], staged-mode has no meaning without git.
pub fn scan_staged(opts: &ScanOptions) -> Result<ScanReport, ScanError> {
    let root = normalize_root(&opts.root)?;
    let baseline = Baseline::load(&root)?;

    let diff = staged::diff_cached(&root)?;
    let added_lines = staged::parse_staged_diff(&diff);
    let (findings, truncated) =
        staged::scan_added_lines(&added_lines, &baseline, opts.max_findings);

    Ok(ScanReport {
        schema_version: report::SCHEMA_VERSION,
        generated_at: generated_at_now(),
        repo_root: root.to_string_lossy().into_owned(),
        head_commit: git_head_commit(&root),
        dirty: git_is_dirty(&root),
        scan_mode: ScanMode::Staged,
        truncated,
        findings,
    })
}

/// Scan git history over the patch stream (`git log -p`), attributing each
/// finding to the oldest commit that introduced it (plan §1.3, §2.1).
/// `range` is a git revision range (e.g. `"main..HEAD"`, `"abc123.."`)
/// passed through to `git log`; `None` scans the full history of `HEAD`.
/// Requires `opts.root` to be inside a git repository.
///
/// Unlike [`scan_worktree`]/[`scan_staged`], findings dedupe by
/// fingerprint before the `max_findings` cap and sort are applied: the same
/// secret reintroduced across commits is one [`Finding`], not one per
/// commit (see [`git::scan_patch_stream`] for the dedupe mechanics).
pub fn scan_history(opts: &ScanOptions, range: Option<&str>) -> Result<ScanReport, ScanError> {
    let root = normalize_root(&opts.root)?;
    let baseline = Baseline::load(&root)?;

    let (mut findings, truncated) =
        git::scan_patch_stream(&root, range, &baseline, opts.max_findings)?;
    findings.sort_by(|a, b| (&a.path, a.line, &a.rule_id).cmp(&(&b.path, b.line, &b.rule_id)));

    Ok(ScanReport {
        schema_version: report::SCHEMA_VERSION,
        generated_at: generated_at_now(),
        repo_root: root.to_string_lossy().into_owned(),
        head_commit: git_head_commit(&root),
        dirty: git_is_dirty(&root),
        scan_mode: ScanMode::History,
        truncated,
        findings,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn write_file(root: &Path, rel: &str, contents: &str) {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("creating fixture parent dir must succeed");
        }
        fs::write(path, contents).expect("writing fixture file must succeed");
    }

    #[test]
    fn scan_worktree_finds_a_real_secret_with_correct_location() {
        let dir = tempfile::tempdir().expect("tempdir creation must succeed in test");
        let aws_example_key = "AKIA".to_string() + "IOSFODNN7EXAMPLE";
        write_file(
            dir.path(),
            "src/config.ts",
            &format!("const unrelated = 1;\nconst key = \"{aws_example_key}\";\n"),
        );

        let opts = ScanOptions::new(dir.path());
        let report = scan_worktree(&opts).expect("scan_worktree must succeed");

        assert_eq!(report.findings.len(), 1);
        let finding = &report.findings[0];
        assert_eq!(finding.rule_id, "aws-access-key-id");
        assert_eq!(finding.path, "src/config.ts");
        assert_eq!(finding.line, 2);
        assert_eq!(
            finding.column, 14,
            "column should point at the start of the match, not the line"
        );
        assert!(!finding.preview.contains(&aws_example_key));
    }

    #[test]
    fn scan_worktree_is_none_outside_a_git_repo() {
        let dir = tempfile::tempdir().expect("tempdir creation must succeed in test");
        write_file(dir.path(), "a.txt", "hello\n");

        let report =
            scan_worktree(&ScanOptions::new(dir.path())).expect("scan_worktree must succeed");
        assert_eq!(report.head_commit, None);
        assert_eq!(report.dirty, None);
    }

    #[test]
    fn scan_worktree_errors_on_missing_root() {
        let missing = std::env::temp_dir().join("bitwarden-scan-does-not-exist-xyz");
        let err = scan_worktree(&ScanOptions::new(missing)).expect_err("missing root must error");
        assert!(matches!(err, ScanError::InvalidRoot(_)));
    }

    #[test]
    fn fingerprints_are_stable_across_two_scans_of_the_same_tree() {
        let dir = tempfile::tempdir().expect("tempdir creation must succeed in test");
        let aws_example_key = "AKIA".to_string() + "IOSFODNN7EXAMPLE";
        write_file(dir.path(), "a.txt", &format!("key=\"{aws_example_key}\"\n"));

        let opts = ScanOptions::new(dir.path());
        let first = scan_worktree(&opts).expect("first scan must succeed");
        let second = scan_worktree(&opts).expect("second scan must succeed");

        assert_eq!(first.findings.len(), 1);
        assert_eq!(
            first.findings[0].fingerprint,
            second.findings[0].fingerprint
        );
    }

    #[test]
    fn unrelated_edit_to_another_line_keeps_the_fingerprint() {
        let dir = tempfile::tempdir().expect("tempdir creation must succeed in test");
        let aws_example_key = "AKIA".to_string() + "IOSFODNN7EXAMPLE";
        write_file(
            dir.path(),
            "a.txt",
            &format!("first line unrelated\nkey=\"{aws_example_key}\"\n"),
        );

        let opts = ScanOptions::new(dir.path());
        let before = scan_worktree(&opts).expect("scan must succeed");

        write_file(
            dir.path(),
            "a.txt",
            &format!("first line CHANGED\nkey=\"{aws_example_key}\"\n"),
        );
        let after = scan_worktree(&opts).expect("scan must succeed");

        assert_eq!(
            before.findings[0].fingerprint,
            after.findings[0].fingerprint
        );
    }

    #[test]
    fn baseline_entry_suppresses_a_finding() {
        let dir = tempfile::tempdir().expect("tempdir creation must succeed in test");
        let aws_example_key = "AKIA".to_string() + "IOSFODNN7EXAMPLE";
        write_file(dir.path(), "a.txt", &format!("key=\"{aws_example_key}\"\n"));

        let opts = ScanOptions::new(dir.path());
        let before = scan_worktree(&opts).expect("scan must succeed");
        assert_eq!(before.findings.len(), 1);

        write_file(
            dir.path(),
            ".bitwardenignore",
            &format!("# baseline\n{}\n", before.findings[0].fingerprint),
        );
        let after = scan_worktree(&opts).expect("scan must succeed");
        assert!(after.findings.is_empty());
    }

    #[test]
    fn gitignored_file_is_not_reported() {
        let dir = tempfile::tempdir().expect("tempdir creation must succeed in test");
        let aws_example_key = "AKIA".to_string() + "IOSFODNN7EXAMPLE";
        write_file(dir.path(), ".gitignore", "ignored.txt\n");
        write_file(
            dir.path(),
            "ignored.txt",
            &format!("key=\"{aws_example_key}\"\n"),
        );
        write_file(
            dir.path(),
            "kept.txt",
            &format!("key=\"{aws_example_key}\"\n"),
        );

        let report = scan_worktree(&ScanOptions::new(dir.path())).expect("scan must succeed");
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].path, "kept.txt");
    }

    #[test]
    fn lockfile_is_skipped() {
        let dir = tempfile::tempdir().expect("tempdir creation must succeed in test");
        let aws_example_key = "AKIA".to_string() + "IOSFODNN7EXAMPLE";
        write_file(
            dir.path(),
            "Cargo.lock",
            &format!("checksum = \"{aws_example_key}\"\n"),
        );

        let report = scan_worktree(&ScanOptions::new(dir.path())).expect("scan must succeed");
        assert!(report.findings.is_empty());
    }

    #[test]
    fn binary_file_is_skipped() {
        let dir = tempfile::tempdir().expect("tempdir creation must succeed in test");
        let aws_example_key = "AKIA".to_string() + "IOSFODNN7EXAMPLE";
        let mut contents = format!("key=\"{aws_example_key}\"\n").into_bytes();
        contents.insert(0, 0u8);
        fs::write(dir.path().join("binary.dat"), contents)
            .expect("writing fixture file must succeed");

        let report = scan_worktree(&ScanOptions::new(dir.path())).expect("scan must succeed");
        assert!(report.findings.is_empty());
    }

    #[test]
    fn max_findings_sets_truncated() {
        let dir = tempfile::tempdir().expect("tempdir creation must succeed in test");
        let aws_example_key = "AKIA".to_string() + "IOSFODNN7EXAMPLE";
        let mut contents = String::new();
        for i in 0..5 {
            contents.push_str(&format!("key{i}=\"{aws_example_key}\"\n"));
        }
        write_file(dir.path(), "many.txt", &contents);

        let mut opts = ScanOptions::new(dir.path());
        opts.max_findings = 2;
        let report = scan_worktree(&opts).expect("scan must succeed");
        assert_eq!(report.findings.len(), 2);
        assert!(report.truncated);
    }
}
