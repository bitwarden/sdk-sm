//! The findings artifact: schema, atomic write, and load.
//!
//! Written by `bws scan` (worktree mode only), read by MCP servers (e.g.
//! Bitwarden agent-access). "Never scanned" (file absent) and "scanned,
//! zero findings" (empty `findings`) are distinct states end-to-end —
//! [`load_artifact`] returns `Ok(None)` only for the former.

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{ScanError, finding::Finding};

/// Current artifact schema version. Bump when the [`ScanReport`] shape
/// changes in a way old readers can't tolerate.
pub const SCHEMA_VERSION: u32 = 1;

/// Artifact location, relative to the scan root.
pub const ARTIFACT_RELATIVE_PATH: &str = ".bitwarden/secret-findings.json";

/// Which input a scan ran over. Serializes as the lowercase strings in the
/// §2 artifact schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScanMode {
    Worktree,
    Staged,
    History,
}

/// The findings artifact (plan §2). Provenance (`generated_at`,
/// `head_commit`, `dirty`) travels with every response so a consumer never
/// acts on stale findings without knowing it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub schema_version: u32,
    /// RFC3339, seconds precision.
    pub generated_at: String,
    pub repo_root: String,
    /// `None` outside a git repository.
    pub head_commit: Option<String>,
    /// `None` outside a git repository.
    pub dirty: Option<bool>,
    pub scan_mode: ScanMode,
    /// `true` when `max_findings` was hit and the run stopped early.
    pub truncated: bool,
    pub findings: Vec<Finding>,
}

/// `<root>/.bitwarden/secret-findings.json`.
pub fn artifact_path(root: &Path) -> PathBuf {
    root.join(ARTIFACT_RELATIVE_PATH)
}

fn tmp_path_for(path: &Path) -> PathBuf {
    let mut os_string = path.as_os_str().to_os_string();
    os_string.push(".tmp");
    PathBuf::from(os_string)
}

/// Atomically write `report` to `<root>/.bitwarden/secret-findings.json`:
/// write to a sibling `.tmp` file, then rename over the final path. Creates
/// `.bitwarden/` if needed.
pub fn write_artifact(root: &Path, report: &ScanReport) -> Result<(), ScanError> {
    let path = artifact_path(root);
    let dir = path
        .parent()
        .ok_or_else(|| ScanError::InvalidRoot(root.to_path_buf()))?;
    fs::create_dir_all(dir).map_err(|source| ScanError::IoPath {
        path: dir.to_path_buf(),
        source,
    })?;

    let tmp_path = tmp_path_for(&path);
    let contents = serde_json::to_vec_pretty(report)?;

    let mut file = fs::File::create(&tmp_path).map_err(|source| ScanError::IoPath {
        path: tmp_path.clone(),
        source,
    })?;
    file.write_all(&contents)
        .map_err(|source| ScanError::IoPath {
            path: tmp_path.clone(),
            source,
        })?;
    file.sync_all().map_err(|source| ScanError::IoPath {
        path: tmp_path.clone(),
        source,
    })?;
    drop(file);

    fs::rename(&tmp_path, &path).map_err(|source| ScanError::IoPath { path, source })?;
    Ok(())
}

/// Load the findings artifact.
///
/// - `Ok(None)`: no artifact file — "never scanned", distinct from "scanned,
///   zero findings" (an empty `findings` list inside `Ok(Some(_))`).
/// - `Err`: file present but unreadable, malformed JSON, or an unsupported
///   `schema_version`.
pub fn load_artifact(root: &Path) -> Result<Option<ScanReport>, ScanError> {
    let path = artifact_path(root);
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(ScanError::IoPath { path, source }),
    };

    let report: ScanReport = serde_json::from_str(&contents)?;
    if report.schema_version != SCHEMA_VERSION {
        return Err(ScanError::UnsupportedSchemaVersion {
            found: report.schema_version,
            expected: SCHEMA_VERSION,
        });
    }
    Ok(Some(report))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_report() -> ScanReport {
        ScanReport {
            schema_version: SCHEMA_VERSION,
            generated_at: "2026-08-11T12:00:00Z".to_string(),
            repo_root: "/tmp/example".to_string(),
            head_commit: Some("abc123".to_string()),
            dirty: Some(false),
            scan_mode: ScanMode::Worktree,
            truncated: false,
            findings: Vec::new(),
        }
    }

    #[test]
    fn load_returns_none_when_artifact_absent() {
        let dir = tempfile::tempdir().expect("tempdir creation must succeed in test");
        let result = load_artifact(dir.path()).expect("loading an absent artifact must not error");
        assert!(result.is_none());
    }

    #[test]
    fn write_then_load_round_trips() {
        let dir = tempfile::tempdir().expect("tempdir creation must succeed in test");
        let report = sample_report();
        write_artifact(dir.path(), &report).expect("write_artifact must succeed");

        let loaded = load_artifact(dir.path())
            .expect("load_artifact must succeed")
            .expect("artifact must be present after write_artifact");
        assert_eq!(loaded.repo_root, report.repo_root);
        assert_eq!(loaded.head_commit, report.head_commit);
        assert_eq!(loaded.schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn write_creates_bitwarden_directory() {
        let dir = tempfile::tempdir().expect("tempdir creation must succeed in test");
        write_artifact(dir.path(), &sample_report()).expect("write_artifact must succeed");
        assert!(dir.path().join(".bitwarden").is_dir());
        assert!(artifact_path(dir.path()).is_file());
    }

    #[test]
    fn write_leaves_no_tmp_file_behind() {
        let dir = tempfile::tempdir().expect("tempdir creation must succeed in test");
        write_artifact(dir.path(), &sample_report()).expect("write_artifact must succeed");
        let tmp = tmp_path_for(&artifact_path(dir.path()));
        assert!(!tmp.exists());
    }

    #[test]
    fn load_errors_on_unsupported_schema_version() {
        let dir = tempfile::tempdir().expect("tempdir creation must succeed in test");
        let path = artifact_path(dir.path());
        fs::create_dir_all(path.parent().expect("artifact path always has a parent"))
            .expect("mkdir must succeed");
        fs::write(&path, r#"{"schema_version":999,"generated_at":"x","repo_root":"/","head_commit":null,"dirty":null,"scan_mode":"worktree","truncated":false,"findings":[]}"#).expect("write must succeed");

        let err = load_artifact(dir.path()).expect_err("schema_version 999 must error");
        assert!(matches!(
            err,
            ScanError::UnsupportedSchemaVersion {
                found: 999,
                expected: 1
            }
        ));
    }

    #[test]
    fn load_errors_on_malformed_json() {
        let dir = tempfile::tempdir().expect("tempdir creation must succeed in test");
        let path = artifact_path(dir.path());
        fs::create_dir_all(path.parent().expect("artifact path always has a parent"))
            .expect("mkdir must succeed");
        fs::write(&path, "not json").expect("write must succeed");

        assert!(load_artifact(dir.path()).is_err());
    }
}
