use std::path::PathBuf;

use directories::BaseDirs;

use crate::{DEFAULT_CONFIG_DIRECTORY, error::UserError};

pub(crate) const DEFAULT_STATE_DIRECTORY: &str = "state";

const STATE_HINT: &str =
    "Continuing without state; set a writable dir with: bws config state-dir <dir>";

pub(crate) fn get_state_file(
    state_dir: Option<PathBuf>,
    access_token_id: String,
) -> Result<PathBuf, UserError> {
    let mut state_dir = match state_dir {
        Some(state_dir) => state_dir,
        None => match BaseDirs::new() {
            Some(base_dirs) => base_dirs
                .home_dir()
                .join(DEFAULT_CONFIG_DIRECTORY)
                .join(DEFAULT_STATE_DIRECTORY),
            None => {
                return Err(UserError::new(
                    "Could not determine a state directory (no home directory).",
                )
                .hint(STATE_HINT));
            }
        },
    };

    std::fs::create_dir_all(&state_dir).map_err(|e| {
        UserError::create_dir("Could not use state directory", &state_dir, e).hint(STATE_HINT)
    })?;
    state_dir.push(access_token_id);

    Ok(state_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn unwritable_state_dir() {
        let file = tempfile::NamedTempFile::new().expect("temp file to be created");
        let dir = file.path().join("state");

        let e = get_state_file(Some(dir.clone()), "id".to_string())
            .expect_err("a file cannot be a parent directory");

        assert_eq!(
            e.to_string(),
            format!(
                "Could not use state directory '{}': not a directory.",
                dir.display()
            )
        );
        assert_eq!(e.hint_text(), Some(STATE_HINT));

        let e = get_state_file(Some(file.path().to_path_buf()), "id".to_string())
            .expect_err("a file is not a directory");
        assert_eq!(
            e.to_string(),
            format!(
                "Could not use state directory '{}': not a directory.",
                file.path().display()
            )
        );
    }
}
