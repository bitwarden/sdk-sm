//! `git diff --cached` parsing and scanning of added lines only.
//!
//! The parser is a pure function over `&str` so it is unit-testable without
//! a real git repository; [`diff_cached`] is the only part of this module
//! that shells out.

use std::{path::Path, process::Command};

use crate::{
    ScanError,
    baseline::Baseline,
    detect, diffparse,
    finding::{Finding, compute_fingerprint, normalize_line},
    walk,
};

/// One added (`+`) line from a staged diff, with its 1-based line number in
/// the *new* version of the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagedAddedLine {
    /// Root-relative, forward-slash path (as git already writes it).
    pub path: String,
    pub line: u64,
    pub content: String,
}

/// Parse `git diff --cached --unified=0 --no-color` output into the set of
/// added lines, correctly attributing each to its path and 1-based line
/// number in the new file. Deleted files (`+++ /dev/null`) contribute no
/// added lines. Pure function: takes the diff text, not a path or a repo.
///
/// Hunk-header and `+++` parsing is shared with [`crate::git`]'s patch-
/// stream scan via [`crate::diffparse`].
pub fn parse_staged_diff(diff: &str) -> Vec<StagedAddedLine> {
    let mut results = Vec::new();
    let mut current_path: Option<String> = None;
    let mut next_new_line: u64 = 0;
    let mut in_hunk = false;

    for raw_line in diff.lines() {
        if let Some(new_path) = diffparse::parse_new_file_line(raw_line) {
            in_hunk = false;
            current_path = new_path;
            continue;
        }
        if raw_line.starts_with("--- ") {
            continue;
        }
        if raw_line.starts_with("@@ ") {
            match diffparse::parse_hunk_new_start(raw_line) {
                Some(start) => {
                    next_new_line = start;
                    in_hunk = true;
                }
                None => in_hunk = false,
            }
            continue;
        }
        if !in_hunk {
            continue;
        }
        let Some(path) = current_path.as_ref() else {
            continue;
        };
        if let Some(content) = raw_line.strip_prefix('+') {
            results.push(StagedAddedLine {
                path: path.clone(),
                line: next_new_line,
                content: content.to_string(),
            });
            next_new_line += 1;
        } else if raw_line.starts_with('-') {
            // Removed line: does not exist in the new file, doesn't
            // consume a new-file line number.
        } else {
            // `--unified=0` should only ever produce `+`/`-` lines inside a
            // hunk, but tolerate a stray context line defensively.
            next_new_line += 1;
        }
    }

    results
}

/// Run `git -C <root> diff --cached --unified=0 --no-color` and return its
/// stdout. Errors if git is missing, `root` is not a git repository, or the
/// command otherwise fails — staged-mode has no meaning without git.
pub fn diff_cached(root: &Path) -> Result<String, ScanError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["diff", "--cached", "--unified=0", "--no-color"])
        .output()
        .map_err(|source| ScanError::GitCommand {
            command: "git diff --cached".to_string(),
            message: source.to_string(),
        })?;

    if !output.status.success() {
        return Err(ScanError::GitCommand {
            command: "git diff --cached".to_string(),
            message: format!("exited with {}", output.status),
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Scan every added line from a parsed staged diff, applying the same path
/// allowlist and baseline as [`crate::scan_worktree`]. Returns findings
/// sorted by (path, line, rule_id) and whether the `max_findings` cap was
/// hit.
pub fn scan_added_lines(
    added_lines: &[StagedAddedLine],
    baseline: &Baseline,
    max_findings: usize,
) -> (Vec<Finding>, bool) {
    let mut findings = Vec::new();
    let mut truncated = false;

    for added in added_lines {
        if walk::is_allowlisted(&added.path) {
            continue;
        }
        for detected in detect::scan_line(&added.content) {
            if findings.len() >= max_findings {
                truncated = true;
                break;
            }
            let normalized = normalize_line(&added.content);
            let fingerprint = compute_fingerprint(detected.rule_id, &added.path, &normalized);
            if baseline.contains(&fingerprint) {
                continue;
            }
            findings.push(Finding {
                fingerprint,
                rule_id: detected.rule_id.to_string(),
                severity: detected.severity,
                path: added.path.clone(),
                line: added.line,
                column: detected.column,
                preview: detected.preview,
                origin: "worktree".to_string(),
                commit: None,
                author: None,
                first_seen: None,
            });
        }
        if truncated {
            break;
        }
    }

    findings.sort_by(|a, b| (&a.path, a.line, &a.rule_id).cmp(&(&b.path, b.line, &b.rule_id)));
    (findings, truncated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_hunk_added_lines() {
        let aws_example_key = "AKIA".to_string() + "IOSFODNN7EXAMPLE";
        let diff = format!(
            "diff --git a/src/config.ts b/src/config.ts\n\
             index 111..222 100644\n\
             --- a/src/config.ts\n\
             +++ b/src/config.ts\n\
             @@ -10,0 +11,2 @@\n\
             +const a = 1;\n\
             +const key = \"{aws_example_key}\";\n"
        );
        let added = parse_staged_diff(&diff);
        assert_eq!(added.len(), 2);
        assert_eq!(
            added[0],
            StagedAddedLine {
                path: "src/config.ts".to_string(),
                line: 11,
                content: "const a = 1;".to_string(),
            }
        );
        assert_eq!(added[1].line, 12);
        assert!(added[1].content.contains(&aws_example_key));
    }

    #[test]
    fn skips_deleted_files() {
        let diff = "diff --git a/old.txt b/old.txt\n\
                     index 111..000 100644\n\
                     --- a/old.txt\n\
                     +++ /dev/null\n\
                     @@ -1,2 +0,0 @@\n\
                     -line one\n\
                     -line two\n";
        assert!(parse_staged_diff(diff).is_empty());
    }

    #[test]
    fn new_file_is_attributed_to_its_own_path() {
        let diff = "diff --git a/new.txt b/new.txt\n\
                     new file mode 100644\n\
                     index 000..111\n\
                     --- /dev/null\n\
                     +++ b/new.txt\n\
                     @@ -0,0 +1,2 @@\n\
                     +line one\n\
                     +line two\n";
        let added = parse_staged_diff(diff);
        assert_eq!(added.len(), 2);
        assert_eq!(added[0].path, "new.txt");
        assert_eq!(added[0].line, 1);
        assert_eq!(added[1].line, 2);
    }

    #[test]
    fn multiple_hunks_in_one_file_track_line_numbers_independently() {
        let diff = "diff --git a/f.txt b/f.txt\n\
                     index 111..222 100644\n\
                     --- a/f.txt\n\
                     +++ b/f.txt\n\
                     @@ -5,0 +6,1 @@\n\
                     +added at six\n\
                     @@ -20,0 +22,1 @@\n\
                     +added at twenty-two\n";
        let added = parse_staged_diff(diff);
        assert_eq!(added.len(), 2);
        assert_eq!(added[0].line, 6);
        assert_eq!(added[1].line, 22);
    }

    #[test]
    fn multiple_files_in_one_diff_are_kept_separate() {
        let diff = "diff --git a/a.txt b/a.txt\n\
                     index 111..222 100644\n\
                     --- a/a.txt\n\
                     +++ b/a.txt\n\
                     @@ -1,0 +2,1 @@\n\
                     +in a\n\
                     diff --git a/b.txt b/b.txt\n\
                     index 333..444 100644\n\
                     --- a/b.txt\n\
                     +++ b/b.txt\n\
                     @@ -1,0 +2,1 @@\n\
                     +in b\n";
        let added = parse_staged_diff(diff);
        assert_eq!(added.len(), 2);
        assert_eq!(added[0].path, "a.txt");
        assert_eq!(added[1].path, "b.txt");
    }

    #[test]
    fn scan_added_lines_finds_a_real_secret_and_respects_baseline() {
        let aws_example_key = "AKIA".to_string() + "IOSFODNN7EXAMPLE";
        let lines = vec![StagedAddedLine {
            path: "src/config.ts".to_string(),
            line: 11,
            content: format!("const key = \"{aws_example_key}\";"),
        }];
        let empty_baseline = Baseline::default();
        let (findings, truncated) = scan_added_lines(&lines, &empty_baseline, 1000);
        assert!(!truncated);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "aws-access-key-id");
        assert_eq!(findings[0].line, 11);

        let mut suppressing = Baseline::default();
        suppressing.insert_for_test(findings[0].fingerprint.clone());
        let (findings, _) = scan_added_lines(&lines, &suppressing, 1000);
        assert!(findings.is_empty());
    }

    #[test]
    fn scan_added_lines_skips_allowlisted_paths() {
        let aws_example_key = "AKIA".to_string() + "IOSFODNN7EXAMPLE";
        let lines = vec![StagedAddedLine {
            path: "Cargo.lock".to_string(),
            line: 1,
            content: aws_example_key,
        }];
        let (findings, _) = scan_added_lines(&lines, &Baseline::default(), 1000);
        assert!(findings.is_empty());
    }
}
