//! Gitignore-aware filesystem walk with binary/size/path skips.

use std::{fs, io::Read, path::Path, path::PathBuf};

use ignore::WalkBuilder;

use crate::ScanError;

/// Files with these basenames are dependency lockfiles: high-entropy by
/// nature, never hand-authored, and never where a secret is introduced.
const ALLOWLIST_BASENAMES: &[&str] = &[
    "package-lock.json",
    "Cargo.lock",
    "yarn.lock",
    "pnpm-lock.yaml",
    "composer.lock",
    "Gemfile.lock",
    "poetry.lock",
    "go.sum",
];

/// Files with these suffixes are skipped: documented examples/samples
/// (placeholder values, not real secrets) and minified/bundled/sourcemap
/// assets (see [`crate::detect::MAX_LINE_LEN`] for the line-length half of
/// this same guard).
const ALLOWLIST_SUFFIXES: &[&str] = &[".example", ".sample", ".min.js", ".min.css", ".map"];

/// Always skipped, root-relative, regardless of gitignore state: the
/// scanner's own artifact and baseline file. Never scan your own output.
const ALWAYS_SKIP_RELATIVE: &[&str] = &[".bitwarden/secret-findings.json", ".bitwardenignore"];

/// How many leading bytes of a file are sniffed for a NUL byte to decide
/// whether it is binary.
const BINARY_SNIFF_LEN: usize = 8192;

/// One file selected for scanning.
#[derive(Debug, Clone)]
pub struct WalkedFile {
    pub abs_path: PathBuf,
    /// Root-relative, forward-slash path (stored in [`crate::Finding::path`]
    /// as-is).
    pub rel_path: String,
}

/// Whether an already root-relative, forward-slash path is skipped by the
/// path allowlist, independent of gitignore/size/binary checks. Exposed so
/// [`crate::staged`] can apply the identical allowlist to diff paths.
pub fn is_allowlisted(rel_path: &str) -> bool {
    if ALWAYS_SKIP_RELATIVE.contains(&rel_path) {
        return true;
    }
    let basename = rel_path.rsplit('/').next().unwrap_or(rel_path);
    if ALLOWLIST_BASENAMES.contains(&basename) {
        return true;
    }
    ALLOWLIST_SUFFIXES
        .iter()
        .any(|suffix| rel_path.ends_with(suffix))
}

/// Convert an absolute path under `root` into a root-relative,
/// forward-slash path string. `None` if `path` is not under `root`.
pub fn to_rel_path(root: &Path, path: &Path) -> Option<String> {
    let rel = path.strip_prefix(root).ok()?;
    let parts: Vec<String> = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    Some(parts.join("/"))
}

fn is_binary(bytes: &[u8]) -> bool {
    bytes.contains(&0)
}

/// Walk `root`, applying `.gitignore`/`.git/info/exclude`/global-gitignore
/// rules (via [`ignore::WalkBuilder`]), the size cap, binary sniffing, and
/// the path allowlist. Symlinks are not followed.
///
/// Test/fixture directories are deliberately **not** skipped — real leaks
/// live in test configs; false positives there are what the baseline
/// (`.bitwardenignore`, see [`crate::baseline`]) is for.
///
/// Individual entries this process cannot stat or read (permission denied,
/// races) are silently skipped rather than aborting the whole scan.
pub fn walk(root: &Path, max_file_size: u64) -> Result<Vec<WalkedFile>, ScanError> {
    if !root.is_dir() {
        return Err(ScanError::InvalidRoot(root.to_path_buf()));
    }

    let mut files = Vec::new();
    let mut builder = WalkBuilder::new(root);
    // Honor .gitignore even when `root` isn't inside a git repository, and
    // never follow symlinks out of the scan root. Hidden files ARE scanned
    // (`hidden(false)`): dotfiles like `.env` are the single most common
    // home of hardcoded secrets, and skipping them would hollow out the
    // scanner's core use case. `.git` itself is excluded explicitly below.
    builder.follow_links(false).require_git(false).hidden(false);
    builder.filter_entry(|entry| entry.file_name() != ".git");

    for entry in builder.build() {
        let Ok(entry) = entry else {
            continue;
        };
        let Some(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_file() {
            continue;
        }

        let abs_path = entry.path().to_path_buf();
        let Some(rel_path) = to_rel_path(root, &abs_path) else {
            continue;
        };
        if is_allowlisted(&rel_path) {
            continue;
        }

        let Ok(metadata) = fs::metadata(&abs_path) else {
            continue;
        };
        if metadata.len() > max_file_size {
            continue;
        }

        let Ok(mut file) = fs::File::open(&abs_path) else {
            continue;
        };
        let mut sniff = vec![0u8; BINARY_SNIFF_LEN];
        let bytes_read = file.read(&mut sniff).unwrap_or(0);
        sniff.truncate(bytes_read);
        if is_binary(&sniff) {
            continue;
        }

        files.push(WalkedFile { abs_path, rel_path });
    }

    files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lockfiles_and_examples_are_allowlisted() {
        assert!(is_allowlisted("Cargo.lock"));
        assert!(is_allowlisted("nested/dir/package-lock.json"));
        assert!(is_allowlisted("config.env.example"));
        assert!(is_allowlisted("bundle.min.js"));
        assert!(is_allowlisted("bundle.min.js.map"));
        assert!(!is_allowlisted("src/config.ts"));
    }

    #[test]
    fn own_artifact_and_baseline_are_always_skipped() {
        assert!(is_allowlisted(".bitwarden/secret-findings.json"));
        assert!(is_allowlisted(".bitwardenignore"));
    }

    #[test]
    fn hidden_files_are_scanned_but_git_dir_is_not() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        fs::write(root.join(".env"), "A=1\n").expect("write .env");
        fs::create_dir_all(root.join(".git")).expect("mkdir .git");
        fs::write(root.join(".git").join("config"), "[core]\n").expect("write git config");

        let files = walk(root, 1024 * 1024).expect("walk");
        let rels: Vec<&str> = files.iter().map(|f| f.rel_path.as_str()).collect();
        assert!(rels.contains(&".env"), "dotfiles must be scanned: {rels:?}");
        assert!(
            !rels.iter().any(|r| r.starts_with(".git/")),
            ".git contents must never be scanned: {rels:?}"
        );
    }

    #[test]
    fn to_rel_path_uses_forward_slashes() {
        let root = Path::new("/a/b");
        let path = Path::new("/a/b/c/d.rs");
        assert_eq!(to_rel_path(root, path).as_deref(), Some("c/d.rs"));
    }
}
