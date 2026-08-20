//! `.bitwardenignore` baseline: fingerprint-keyed suppression of findings.
//!
//! Content-keyed (see [`crate::finding::compute_fingerprint`]), so one
//! baseline entry silences a finding in every commit that touches the same
//! line — not just the one it was added in (plan §2.1).

use std::{collections::HashSet, fs, path::Path};

use crate::{ScanError, finding::Finding};

/// Baseline filename, resolved relative to the scan root.
pub const BASELINE_FILENAME: &str = ".bitwardenignore";

/// A loaded `.bitwardenignore`: one fingerprint per non-comment, non-blank
/// line.
#[derive(Debug, Default, Clone)]
pub struct Baseline {
    fingerprints: HashSet<String>,
}

impl Baseline {
    /// Load `<root>/.bitwardenignore`. A missing file is an empty baseline,
    /// not an error — most repos never create one.
    pub fn load(root: &Path) -> Result<Self, ScanError> {
        let path = root.join(BASELINE_FILENAME);
        let contents = match fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(source) => return Err(ScanError::IoPath { path, source }),
        };
        Ok(Self::parse(&contents))
    }

    /// Parse baseline contents: one fingerprint per line, `#` comments and
    /// blank lines ignored. Pure, so it's testable without touching disk.
    pub fn parse(contents: &str) -> Self {
        let fingerprints = contents
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(str::to_string)
            .collect();
        Self { fingerprints }
    }

    pub fn contains(&self, fingerprint: &str) -> bool {
        self.fingerprints.contains(fingerprint)
    }

    pub fn is_empty(&self) -> bool {
        self.fingerprints.is_empty()
    }

    /// Drop every finding whose fingerprint is in the baseline.
    pub fn retain_unsuppressed(&self, findings: Vec<Finding>) -> Vec<Finding> {
        findings
            .into_iter()
            .filter(|f| !self.contains(&f.fingerprint))
            .collect()
    }

    #[cfg(test)]
    pub fn insert_for_test(&mut self, fingerprint: String) {
        self.fingerprints.insert(fingerprint);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fingerprints_ignoring_comments_and_blanks() {
        let baseline =
            Baseline::parse("# a comment\n\nabc123\n  def456  \n# another comment\nghi789\n");
        assert!(baseline.contains("abc123"));
        assert!(baseline.contains("def456"));
        assert!(baseline.contains("ghi789"));
        assert_eq!(baseline.fingerprints.len(), 3);
    }

    #[test]
    fn empty_file_yields_empty_baseline() {
        let baseline = Baseline::parse("");
        assert!(baseline.is_empty());
    }

    #[test]
    fn suppresses_matching_findings_only() {
        let mut baseline = Baseline::default();
        baseline.insert_for_test("keep-me-out".to_string());

        let suppressed = Finding {
            fingerprint: "keep-me-out".to_string(),
            rule_id: "aws-access-key-id".to_string(),
            severity: crate::rules::Severity::High,
            path: "a.txt".to_string(),
            line: 1,
            column: 1,
            preview: "AKIA****".to_string(),
            origin: "worktree".to_string(),
            commit: None,
            author: None,
            first_seen: None,
        };
        let mut survives = suppressed.clone();
        survives.fingerprint = "different".to_string();

        let result = baseline.retain_unsuppressed(vec![suppressed, survives.clone()]);
        assert_eq!(result, vec![survives]);
    }

    #[test]
    fn load_returns_empty_baseline_when_file_absent() {
        let dir = tempfile::tempdir().expect("tempdir creation must succeed in test");
        let baseline =
            Baseline::load(dir.path()).expect("loading an absent baseline must not error");
        assert!(baseline.is_empty());
    }
}
