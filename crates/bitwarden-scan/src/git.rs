//! `git log -p` patch-stream history scan.
//!
//! Scans the *patch stream* rather than per-commit blob trees (plan §1.3):
//! each version of a line is examined exactly once as it appears in a diff,
//! instead of re-reading every unchanged file across every commit. Findings
//! dedupe by [`crate::finding::compute_fingerprint`], which has no commit
//! component (plan §2.1) — the same secret reappearing across commits
//! collapses into a single [`Finding`] whose `commit`/`author`/`first_seen`
//! point at the *oldest* commit that introduced it.
//!
//! Never links `git2`/`gix` — shells out to the `git` binary and streams
//! its stdout line-by-line (never buffers the whole patch stream), so this
//! scales to large histories.

use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
    path::Path,
    process::{Command, Stdio},
};

use crate::{
    ScanError,
    baseline::Baseline,
    detect, diffparse,
    finding::{Finding, compute_fingerprint, normalize_line},
    walk,
};

/// Prefixes every commit-header line emitted by [`COMMIT_FORMAT`]. Diff
/// content lines (context, `+`/`-`, hunk headers, `diff --git`, `Binary
/// files ... differ`, etc.) are all plain text and never start with a
/// control character, so this sentinel can't collide with real diff text.
const HEADER_SENTINEL: char = '\u{1}';

/// Separates fields within a commit-header line. A plain space would break
/// on author names that contain spaces, so this uses the ASCII unit
/// separator instead — never legitimately present in a name/email/date.
const FIELD_SEP: char = '\u{1f}';

/// `git log --format` string: sentinel, full commit hash, author name,
/// author email, author date (strict ISO 8601 / RFC 3339), each field
/// separated by [`FIELD_SEP`].
const COMMIT_FORMAT: &str = "%x01%H%x1f%an%x1f%ae%x1f%aI";

/// A parsed commit-header line: the introducing commit's identity, with
/// `author` already formatted as `"Name <email>"` for [`Finding::author`].
struct CommitHeader {
    hash: String,
    author: String,
    date: String,
}

/// Parse one line as a [`COMMIT_FORMAT`] commit header. `None` if `line`
/// doesn't start with [`HEADER_SENTINEL`] or doesn't have all four fields
/// (defensive — should never happen against real `git log` output).
fn parse_commit_header(line: &str) -> Option<CommitHeader> {
    let rest = line.strip_prefix(HEADER_SENTINEL)?;
    let mut parts = rest.splitn(4, FIELD_SEP);
    let hash = parts.next()?.to_string();
    let author_name = parts.next()?;
    let author_email = parts.next()?;
    let date = parts.next()?.to_string();
    Some(CommitHeader {
        hash,
        author: format!("{author_name} <{author_email}>"),
        date,
    })
}

/// The state machine driving both the streaming scan ([`scan_patch_stream`])
/// and the pure, buffer-free parser tests ([`feed_text`] in tests below) —
/// one line in, zero or more findings out.
struct HistoryScanState<'a> {
    baseline: &'a Baseline,
    max_findings: usize,
    findings: HashMap<String, Finding>,
    truncated: bool,
    current_commit: Option<CommitHeader>,
    current_path: Option<String>,
    next_new_line: u64,
    in_hunk: bool,
}

impl<'a> HistoryScanState<'a> {
    fn new(baseline: &'a Baseline, max_findings: usize) -> Self {
        Self {
            baseline,
            max_findings,
            findings: HashMap::new(),
            truncated: false,
            current_commit: None,
            current_path: None,
            next_new_line: 0,
            in_hunk: false,
        }
    }

    /// Feed one line of `git log -p` output (no trailing newline). Returns
    /// `false` once the `max_findings` cap has been hit — the caller should
    /// stop feeding lines (and, in the streaming case, kill the git
    /// subprocess early rather than reading the rest of a possibly huge
    /// history).
    fn feed_line(&mut self, line: &str) -> bool {
        if self.truncated {
            return false;
        }

        if let Some(header) = parse_commit_header(line) {
            self.current_commit = Some(header);
            self.current_path = None;
            self.in_hunk = false;
            return true;
        }
        // A new `diff --git` block starts: reset file state unconditionally
        // so a binary-file diff (which prints no `+++`/`@@` lines at all)
        // can never inherit the previous file's path or hunk state.
        if line.starts_with("diff --git ") {
            self.current_path = None;
            self.in_hunk = false;
            return true;
        }
        if let Some(new_path) = diffparse::parse_new_file_line(line) {
            self.in_hunk = false;
            self.current_path = new_path.filter(|p| !walk::is_allowlisted(p));
            return true;
        }
        if line.starts_with("--- ") {
            return true;
        }
        if line.starts_with("@@ ") {
            match diffparse::parse_hunk_new_start(line) {
                Some(start) => {
                    self.next_new_line = start;
                    self.in_hunk = true;
                }
                None => self.in_hunk = false,
            }
            return true;
        }
        if !self.in_hunk {
            return true;
        }
        let Some(path) = self.current_path.clone() else {
            return true;
        };
        let Some(commit) = self.current_commit.as_ref() else {
            return true;
        };

        if let Some(content) = line.strip_prefix('+') {
            let commit_hash = commit.hash.clone();
            let author = commit.author.clone();
            let first_seen = commit.date.clone();
            for detected in detect::scan_line(content) {
                let normalized = normalize_line(content);
                let fingerprint = compute_fingerprint(detected.rule_id, &path, &normalized);
                if self.baseline.contains(&fingerprint) {
                    continue;
                }
                if !self.findings.contains_key(&fingerprint)
                    && self.findings.len() >= self.max_findings
                {
                    self.truncated = true;
                    break;
                }
                // `git log` emits commits newest-first, so each further
                // occurrence of the same fingerprint is *older* than the
                // one already stored. Unconditionally overwriting means the
                // last write — reached only once we've walked all the way
                // back to the oldest introducing commit — is what survives
                // (plan §2.1: earliest commit wins as `first_seen`).
                self.findings.insert(
                    fingerprint.clone(),
                    Finding {
                        fingerprint,
                        rule_id: detected.rule_id.to_string(),
                        severity: detected.severity,
                        path: path.clone(),
                        line: self.next_new_line,
                        column: detected.column,
                        preview: detected.preview,
                        origin: "history".to_string(),
                        commit: Some(commit_hash.clone()),
                        author: Some(author.clone()),
                        first_seen: Some(first_seen.clone()),
                    },
                );
            }
            self.next_new_line += 1;
        } else if line.starts_with('-') {
            // Removed line: doesn't exist in the new file, doesn't consume
            // a new-file line number (matches `staged::parse_staged_diff`).
        } else {
            self.next_new_line += 1;
        }

        !self.truncated
    }

    fn into_result(self) -> (Vec<Finding>, bool) {
        (self.findings.into_values().collect(), self.truncated)
    }
}

/// Run `git -C <root> log -p --unified=0 --no-color --diff-filter=AM
/// --format=<sentinel> [range] -- .`, calling `on_line` for every line of
/// stdout as it is produced. Never buffers the whole output (histories can
/// be huge) — reads through a `BufReader` one line at a time.
///
/// If `on_line` returns `false` (the `max_findings` cap was hit), the git
/// subprocess is killed rather than read to completion, and this returns
/// `Ok(())` — that's an intentional early stop, not a git failure.
///
/// Errors carry a stderr snippet — never file content — for a missing git
/// binary, a non-repository root, or an invalid `range`.
fn run_log_patch_stream(
    root: &Path,
    range: Option<&str>,
    mut on_line: impl FnMut(&str) -> bool,
) -> Result<(), ScanError> {
    const COMMAND_LABEL: &str = "git log -p";

    let mut cmd = Command::new("git");
    cmd.arg("-C")
        .arg(root)
        .args(["log", "-p", "--unified=0", "--no-color", "--diff-filter=AM"]);
    cmd.arg(format!("--format={COMMIT_FORMAT}"));
    if let Some(range) = range {
        cmd.arg(range);
    }
    cmd.arg("--").arg(".");
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|source| ScanError::GitCommand {
        command: COMMAND_LABEL.to_string(),
        message: source.to_string(),
    })?;

    let stdout = child.stdout.take().ok_or_else(|| ScanError::GitCommand {
        command: COMMAND_LABEL.to_string(),
        message: "failed to capture subprocess stdout".to_string(),
    })?;
    let mut reader = BufReader::new(stdout);
    let mut buf = String::new();
    loop {
        buf.clear();
        let bytes_read = reader
            .read_line(&mut buf)
            .map_err(|source| ScanError::GitCommand {
                command: COMMAND_LABEL.to_string(),
                message: source.to_string(),
            })?;
        if bytes_read == 0 {
            break;
        }
        let line = buf.trim_end_matches(['\n', '\r']);
        if !on_line(line) {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(());
        }
    }

    let status = child.wait().map_err(|source| ScanError::GitCommand {
        command: COMMAND_LABEL.to_string(),
        message: source.to_string(),
    })?;
    if !status.success() {
        let mut stderr_text = String::new();
        if let Some(mut stderr) = child.stderr.take() {
            let _ = stderr.read_to_string(&mut stderr_text);
        }
        let snippet: String = stderr_text.trim().chars().take(500).collect();
        let message = if snippet.is_empty() {
            format!("exited with {status}")
        } else {
            snippet
        };
        return Err(ScanError::GitCommand {
            command: COMMAND_LABEL.to_string(),
            message,
        });
    }

    Ok(())
}

/// Stream a `git log -p` patch stream and scan every added line, applying
/// the same path allowlist and baseline as [`crate::scan_worktree`].
/// `range` is a git revision range (e.g. `"main..HEAD"`, `"abc123.."`);
/// `None` scans the full history of `HEAD`. Returns findings (unsorted —
/// [`crate::scan_history`] sorts them) and whether `max_findings` was hit.
pub(crate) fn scan_patch_stream(
    root: &Path,
    range: Option<&str>,
    baseline: &Baseline,
    max_findings: usize,
) -> Result<(Vec<Finding>, bool), ScanError> {
    let mut state = HistoryScanState::new(baseline, max_findings);
    run_log_patch_stream(root, range, |line| state.feed_line(line))?;
    Ok(state.into_result())
}

#[cfg(test)]
mod tests {
    use std::{fs, process::Command};

    use super::*;

    /// Pure, subprocess-free entry point for parser tests: drives
    /// [`HistoryScanState`] over canned `git log -p`-shaped text instead of
    /// a real git subprocess.
    fn feed_text(text: &str, baseline: &Baseline, max_findings: usize) -> (Vec<Finding>, bool) {
        let mut state = HistoryScanState::new(baseline, max_findings);
        for line in text.lines() {
            if !state.feed_line(line) {
                break;
            }
        }
        state.into_result()
    }

    fn header(hash: &str, name: &str, email: &str, date: &str) -> String {
        format!("{HEADER_SENTINEL}{hash}{FIELD_SEP}{name}{FIELD_SEP}{email}{FIELD_SEP}{date}")
    }

    #[test]
    fn parses_sentinel_commit_header() {
        let line = header(
            "abc123",
            "Ada Lovelace",
            "ada@example.com",
            "2020-01-01T00:00:00Z",
        );
        let parsed = parse_commit_header(&line).expect("header must parse");
        assert_eq!(parsed.hash, "abc123");
        assert_eq!(parsed.author, "Ada Lovelace <ada@example.com>");
        assert_eq!(parsed.date, "2020-01-01T00:00:00Z");
    }

    #[test]
    fn non_sentinel_line_is_not_a_commit_header() {
        assert!(parse_commit_header("diff --git a/x b/x").is_none());
        assert!(parse_commit_header("+const a = 1;").is_none());
    }

    #[test]
    fn single_commit_single_file_finds_secret_at_correct_line() {
        let aws_example_key = "AKIA".to_string() + "IOSFODNN7EXAMPLE";
        let text = format!(
            "{header}\n\
             diff --git a/src/config.ts b/src/config.ts\n\
             new file mode 100644\n\
             index 000..111\n\
             --- /dev/null\n\
             +++ b/src/config.ts\n\
             @@ -0,0 +1,2 @@\n\
             +const a = 1;\n\
             +const key = \"{aws_example_key}\";\n",
            header = header("c1", "Dev", "dev@example.com", "2020-01-01T00:00:00Z"),
        );

        let (findings, truncated) = feed_text(&text, &Baseline::default(), 1000);
        assert!(!truncated);
        assert_eq!(findings.len(), 1);
        let finding = &findings[0];
        assert_eq!(finding.rule_id, "aws-access-key-id");
        assert_eq!(finding.path, "src/config.ts");
        assert_eq!(finding.line, 2, "secret is on the second added line");
        assert_eq!(finding.origin, "history");
        assert_eq!(finding.commit.as_deref(), Some("c1"));
        assert_eq!(finding.author.as_deref(), Some("Dev <dev@example.com>"));
        assert_eq!(finding.first_seen.as_deref(), Some("2020-01-01T00:00:00Z"));
    }

    #[test]
    fn multi_file_hunk_line_numbers_tracked_independently_per_file() {
        let aws_example_key = "AKIA".to_string() + "IOSFODNN7EXAMPLE";
        let text = format!(
            "{header}\n\
             diff --git a/a.txt b/a.txt\n\
             index 111..222 100644\n\
             --- a/a.txt\n\
             +++ b/a.txt\n\
             @@ -5,0 +6,1 @@\n\
             +unrelated in a\n\
             diff --git a/b.txt b/b.txt\n\
             index 333..444 100644\n\
             --- a/b.txt\n\
             +++ b/b.txt\n\
             @@ -20,0 +22,1 @@\n\
             +const key = \"{aws_example_key}\";\n",
            header = header("c1", "Dev", "dev@example.com", "2020-01-01T00:00:00Z"),
        );

        let (findings, _) = feed_text(&text, &Baseline::default(), 1000);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].path, "b.txt");
        assert_eq!(findings[0].line, 22);
    }

    #[test]
    fn newer_commit_processed_first_older_commit_wins_first_seen() {
        // git log emits newest-first: c2 (newer) appears before c1 (older)
        // in the stream, same fingerprint (same rule/path/normalized line).
        let aws_example_key = "AKIA".to_string() + "IOSFODNN7EXAMPLE";
        let text = format!(
            "{header_c2}\n\
             diff --git a/a.txt b/a.txt\n\
             index 111..222 100644\n\
             --- a/a.txt\n\
             +++ b/a.txt\n\
             @@ -0,0 +1,1 @@\n\
             +const key = \"{aws_example_key}\";\n\
             {header_c1}\n\
             diff --git a/a.txt b/a.txt\n\
             new file mode 100644\n\
             index 000..111\n\
             --- /dev/null\n\
             +++ b/a.txt\n\
             @@ -0,0 +1,1 @@\n\
             +const key = \"{aws_example_key}\";\n",
            header_c2 = header("c2-newer", "Dev", "dev@example.com", "2021-01-01T00:00:00Z"),
            header_c1 = header("c1-older", "Dev", "dev@example.com", "2020-01-01T00:00:00Z"),
        );

        let (findings, _) = feed_text(&text, &Baseline::default(), 1000);
        assert_eq!(
            findings.len(),
            1,
            "same fingerprint across two commits dedupes to one finding"
        );
        assert_eq!(
            findings[0].commit.as_deref(),
            Some("c1-older"),
            "the oldest introducing commit must win, not the first one seen in the newest-first stream"
        );
        assert_eq!(
            findings[0].first_seen.as_deref(),
            Some("2020-01-01T00:00:00Z")
        );
    }

    #[test]
    fn binary_file_diff_contributes_no_findings() {
        let text = format!(
            "{header}\n\
             diff --git a/blob.bin b/blob.bin\n\
             index 111..222 100644\n\
             Binary files a/blob.bin and b/blob.bin differ\n",
            header = header("c1", "Dev", "dev@example.com", "2020-01-01T00:00:00Z"),
        );
        let (findings, _) = feed_text(&text, &Baseline::default(), 1000);
        assert!(findings.is_empty());
    }

    #[test]
    fn allowlisted_path_is_skipped() {
        let aws_example_key = "AKIA".to_string() + "IOSFODNN7EXAMPLE";
        let text = format!(
            "{header}\n\
             diff --git a/Cargo.lock b/Cargo.lock\n\
             index 111..222 100644\n\
             --- a/Cargo.lock\n\
             +++ b/Cargo.lock\n\
             @@ -1,0 +2,1 @@\n\
             +checksum = \"{aws_example_key}\"\n",
            header = header("c1", "Dev", "dev@example.com", "2020-01-01T00:00:00Z"),
        );
        let (findings, _) = feed_text(&text, &Baseline::default(), 1000);
        assert!(findings.is_empty());
    }

    #[test]
    fn baseline_suppresses_a_history_finding_and_fingerprint_matches_worktree() {
        let aws_example_key = "AKIA".to_string() + "IOSFODNN7EXAMPLE";
        let content = format!("const key = \"{aws_example_key}\";");
        let normalized = normalize_line(&content);
        let worktree_fingerprint = compute_fingerprint("aws-access-key-id", "a.txt", &normalized);

        let text = format!(
            "{header}\n\
             diff --git a/a.txt b/a.txt\n\
             new file mode 100644\n\
             index 000..111\n\
             --- /dev/null\n\
             +++ b/a.txt\n\
             @@ -0,0 +1,1 @@\n\
             +{content}\n",
            header = header("c1", "Dev", "dev@example.com", "2020-01-01T00:00:00Z"),
        );

        let (findings, _) = feed_text(&text, &Baseline::default(), 1000);
        assert_eq!(findings.len(), 1);
        assert_eq!(
            findings[0].fingerprint, worktree_fingerprint,
            "history and worktree findings of the same (rule, path, line) share one fingerprint"
        );

        let mut suppressing = Baseline::default();
        suppressing.insert_for_test(worktree_fingerprint);
        let (suppressed, _) = feed_text(&text, &suppressing, 1000);
        assert!(suppressed.is_empty());
    }

    #[test]
    fn max_findings_truncates_and_sets_truncated() {
        let aws_example_key = "AKIA".to_string() + "IOSFODNN7EXAMPLE";
        let text = format!(
            "{header}\n\
             diff --git a/a.txt b/a.txt\n\
             new file mode 100644\n\
             index 000..111\n\
             --- /dev/null\n\
             +++ b/a.txt\n\
             @@ -0,0 +1,2 @@\n\
             +const one = \"{aws_example_key}\";\n\
             +const two = \"{aws_example_key}\";\n",
            header = header("c1", "Dev", "dev@example.com", "2020-01-01T00:00:00Z"),
        );
        let (findings, truncated) = feed_text(&text, &Baseline::default(), 1);
        assert!(truncated);
        assert_eq!(findings.len(), 1);
    }

    // --- Integration tests against a real temporary git repository ---

    fn git(root: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .env("GIT_AUTHOR_NAME", "Test Dev")
            .env("GIT_AUTHOR_EMAIL", "dev@example.com")
            .env("GIT_COMMITTER_NAME", "Test Dev")
            .env("GIT_COMMITTER_EMAIL", "dev@example.com")
            .output()
            .expect("git subprocess must spawn in test");
        assert!(
            output.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    fn commit(root: &Path, rel: &str, contents: &str, message: &str) -> String {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("mkdir must succeed in test");
        }
        fs::write(&path, contents).expect("writing fixture file must succeed in test");
        git(root, &["add", rel]);
        git(root, &["commit", "-m", message]);
        git(root, &["rev-parse", "HEAD"])
    }

    fn init_repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir creation must succeed in test");
        git(dir.path(), &["init", "-q"]);
        dir
    }

    #[test]
    fn scan_history_finds_reintroduced_secret_at_its_oldest_commit() {
        let aws_example_key = "AKIA".to_string() + "IOSFODNN7EXAMPLE";
        let dir = init_repo();
        let root = dir.path();

        let secret_line = format!("const key = \"{aws_example_key}\";");
        let commit_a = commit(root, "config.ts", &secret_line, "A: add secret");
        let _commit_b = commit(
            root,
            "config.ts",
            "const key = \"removed\";",
            "B: remove secret",
        );
        let commit_c = commit(root, "config.ts", &secret_line, "C: re-add secret");

        let baseline = Baseline::default();
        let (findings, truncated) =
            scan_patch_stream(root, None, &baseline, 1000).expect("scan_patch_stream must succeed");
        assert!(!truncated);
        assert_eq!(
            findings.len(),
            1,
            "the re-added line must dedupe to a single finding, not one per commit"
        );
        assert_eq!(findings[0].origin, "history");
        assert_eq!(
            findings[0].commit.as_deref(),
            Some(commit_a.as_str()),
            "the oldest commit that introduced the secret must win, not the most recent re-add"
        );
        assert!(findings[0].first_seen.is_some());

        // A range scan excluding commit A must see only the re-add (C).
        let range = format!("{commit_a}..{commit_c}");
        let (range_findings, _) = scan_patch_stream(root, Some(&range), &baseline, 1000)
            .expect("ranged scan_patch_stream must succeed");
        assert_eq!(range_findings.len(), 1);
        assert_eq!(range_findings[0].commit.as_deref(), Some(commit_c.as_str()));
    }

    #[test]
    fn scan_history_public_entry_point_assembles_report_metadata() {
        let aws_example_key = "AKIA".to_string() + "IOSFODNN7EXAMPLE";
        let dir = init_repo();
        let root = dir.path();
        commit(
            root,
            "config.ts",
            &format!("const key = \"{aws_example_key}\";"),
            "add secret",
        );

        let opts = crate::ScanOptions::new(root);
        let report = crate::scan_history(&opts, None).expect("scan_history must succeed");

        assert_eq!(report.scan_mode, crate::ScanMode::History);
        assert!(!report.truncated);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].origin, "history");
        assert!(
            report.head_commit.is_some(),
            "history mode must carry head_commit metadata, same as worktree mode"
        );
        assert!(
            report.dirty.is_some(),
            "history mode must carry dirty metadata, same as worktree mode"
        );
    }

    #[test]
    fn scan_patch_stream_errors_on_non_repository() {
        let dir = tempfile::tempdir().expect("tempdir creation must succeed in test");
        let baseline = Baseline::default();
        let err = scan_patch_stream(dir.path(), None, &baseline, 1000)
            .expect_err("a non-repository root must error");
        assert!(matches!(err, ScanError::GitCommand { .. }));
    }

    #[test]
    fn scan_patch_stream_errors_on_bad_range() {
        let dir = init_repo();
        let root = dir.path();
        commit(root, "a.txt", "hello\n", "initial");

        let baseline = Baseline::default();
        let err = scan_patch_stream(root, Some("not-a-real-range..HEAD"), &baseline, 1000)
            .expect_err("an invalid range must error");
        assert!(matches!(err, ScanError::GitCommand { .. }));
    }

    /// Not run by default (`cargo test`): scans this repository's own
    /// history end-to-end and prints wall time, for a human to run with
    /// `cargo test -p bitwarden-scan --release -- --ignored history_scan_large_repo_smoke`.
    #[test]
    #[ignore]
    fn history_scan_large_repo_smoke() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("crates/bitwarden-scan is always two levels under the workspace root")
            .to_path_buf();
        let baseline = Baseline::default();

        let start = std::time::Instant::now();
        let (findings, truncated) = scan_patch_stream(&repo_root, None, &baseline, 100_000)
            .expect("scanning this repo's own history must succeed");
        let elapsed = start.elapsed();

        println!(
            "history_scan_large_repo_smoke: {} findings, truncated={truncated}, wall time {elapsed:?}",
            findings.len()
        );
    }
}
