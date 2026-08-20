//! The [`Finding`] shape written into the artifact (see [`crate::report`])
//! and its stable fingerprint.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub use crate::rules::Severity;

/// One detected secret. Never carries the matched value itself — only
/// location, rule id, and a masked [`preview`](Finding::preview).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// `sha256(rule_id + "\0" + path + "\0" + normalized_line)`, hex
    /// encoded. Stable across rescans of an unchanged line; deliberately
    /// has no commit component (see plan §2.1) so a single
    /// `.bitwardenignore` entry silences a finding in every commit that
    /// introduced it, not just one.
    pub fingerprint: String,
    pub rule_id: String,
    pub severity: Severity,
    /// Root-relative path, forward slashes regardless of platform.
    pub path: String,
    /// 1-based line number.
    pub line: u64,
    /// 1-based column, counted in characters.
    pub column: u64,
    /// Masked preview of the matched text. Never the full value.
    pub preview: String,
    /// `"worktree"` or `"history"`.
    pub origin: String,
    /// History-mode only.
    pub commit: Option<String>,
    /// History-mode only.
    pub author: Option<String>,
    /// History-mode only.
    pub first_seen: Option<String>,
}

/// The line text used to key a fingerprint: leading/trailing whitespace
/// trimmed, otherwise verbatim.
pub fn normalize_line(line: &str) -> String {
    line.trim().to_string()
}

/// Compute a finding fingerprint. See [`Finding::fingerprint`] for the
/// stability contract.
pub fn compute_fingerprint(rule_id: &str, path: &str, normalized_line: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(rule_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(path.as_bytes());
    hasher.update(b"\0");
    hasher.update(normalized_line.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_is_stable_for_the_same_inputs() {
        let a = compute_fingerprint("aws-access-key-id", "src/config.ts", "const x = 1;");
        let b = compute_fingerprint("aws-access-key-id", "src/config.ts", "const x = 1;");
        assert_eq!(a, b);
    }

    #[test]
    fn fingerprint_changes_with_rule_path_or_line() {
        let base = compute_fingerprint("rule-a", "path/a", "line");
        assert_ne!(base, compute_fingerprint("rule-b", "path/a", "line"));
        assert_ne!(base, compute_fingerprint("rule-a", "path/b", "line"));
        assert_ne!(base, compute_fingerprint("rule-a", "path/a", "other"));
    }

    #[test]
    fn normalize_line_trims_whitespace_only() {
        assert_eq!(normalize_line("  const x = 1;  \n"), "const x = 1;");
        assert_eq!(normalize_line("const   x = 1;"), "const   x = 1;");
    }
}
