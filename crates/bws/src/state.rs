use std::path::PathBuf;

use directories::BaseDirs;

use crate::{DEFAULT_CONFIG_DIRECTORY, error::UserError};

pub(crate) const DEFAULT_STATE_DIRECTORY: &str = "state";

pub(crate) fn get_state_file(
    state_dir: Option<PathBuf>,
    access_token_id: String,
) -> Result<PathBuf, UserError> {
    let mut state_dir = match state_dir {
        Some(state_dir) => state_dir,
        None => BaseDirs::new()
            .map(|base_dirs| {
                base_dirs
                    .home_dir()
                    .join(DEFAULT_CONFIG_DIRECTORY)
                    .join(DEFAULT_STATE_DIRECTORY)
            })
            .ok_or(UserError::NoStateDir)?,
    };

    std::fs::create_dir_all(&state_dir).map_err(|e| UserError::state_dir(&state_dir, e))?;
    state_dir.push(access_token_id);

    Ok(state_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::STATE_HINT;

    #[cfg(unix)]
    #[test]
    fn unwritable_state_dir() {
        let file = tempfile::NamedTempFile::new().expect("temp file to be created");

        // A file as the state directory itself, and a file as its parent.
        for dir in [file.path().to_path_buf(), file.path().join("state")] {
            let e = get_state_file(Some(dir.clone()), "id".to_string())
                .expect_err("a file cannot be a state directory");

            assert_eq!(
                e.to_string(),
                format!(
                    "Could not use state directory '{}': not a directory.",
                    dir.display()
                )
            );
            assert_eq!(e.hint().as_deref(), Some(STATE_HINT));
        }
    }
}
