//! Shared parsing helpers for unified-diff (`--unified=0`) text, used by
//! both [`crate::staged`] (`git diff --cached`) and [`crate::git`] (`git log
//! -p` patch stream). Pure functions over `&str` — no subprocess calls
//! here, so they're unit-testable without a git repository.

use std::sync::OnceLock;

use regex::Regex;

/// Matches a `@@ -a,b +c,d @@` unified-diff hunk header and captures the
/// new-file start line (`c`).
fn hunk_header_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^@@ -\d+(?:,\d+)? \+(\d+)(?:,\d+)? @@")
            .expect("static hunk header pattern always compiles")
    })
}

/// Parse a `+++ ...` diff line into the new-file path.
///
/// - `None`: `line` is not a `+++ ` line at all — the caller should try
///   other line kinds.
/// - `Some(None)`: `+++ /dev/null` — a deleted file, no new-file path.
/// - `Some(Some(path))`: the new-file path, with the `b/` prefix stripped.
pub fn parse_new_file_line(line: &str) -> Option<Option<String>> {
    let rest = line.strip_prefix("+++ ")?;
    if rest == "/dev/null" {
        Some(None)
    } else {
        Some(Some(rest.strip_prefix("b/").unwrap_or(rest).to_string()))
    }
}

/// Parse the new-file start line number (`c` in `@@ -a,b +c,d @@`) from a
/// hunk header line. `None` if `line` doesn't match.
pub fn parse_hunk_new_start(line: &str) -> Option<u64> {
    hunk_header_re()
        .captures(line)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().parse().unwrap_or(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_new_file_path_stripping_b_prefix() {
        assert_eq!(
            parse_new_file_line("+++ b/src/config.ts"),
            Some(Some("src/config.ts".to_string()))
        );
    }

    #[test]
    fn dev_null_new_file_line_is_some_none() {
        assert_eq!(parse_new_file_line("+++ /dev/null"), Some(None));
    }

    #[test]
    fn non_plus_plus_plus_line_is_none() {
        assert_eq!(parse_new_file_line("--- a/src/config.ts"), None);
        assert_eq!(parse_new_file_line("@@ -1,0 +1,1 @@"), None);
    }

    #[test]
    fn parses_hunk_new_start_with_and_without_a_count() {
        assert_eq!(parse_hunk_new_start("@@ -10,0 +11,2 @@"), Some(11));
        assert_eq!(parse_hunk_new_start("@@ -1 +1 @@"), Some(1));
    }

    #[test]
    fn non_hunk_header_line_is_none() {
        assert_eq!(parse_hunk_new_start("+const a = 1;"), None);
    }
}
