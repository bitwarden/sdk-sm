use std::path::PathBuf;

use color_eyre::eyre::{Result, bail};
use directories::BaseDirs;

use crate::DEFAULT_CONFIG_DIRECTORY;

pub(crate) const DEFAULT_STATE_DIRECTORY: &str = "state";

pub(crate) fn get_state_file(
    state_dir: Option<PathBuf>,
    access_token_id: String,
) -> Result<PathBuf> {
    let mut state_dir = match state_dir {
        Some(state_dir) => state_dir,
        None => {
            if let Some(base_dirs) = BaseDirs::new() {
                base_dirs
                    .home_dir()
                    .join(DEFAULT_CONFIG_DIRECTORY)
                    .join(DEFAULT_STATE_DIRECTORY)
            } else {
                bail!("A valid home directory doesn't exist");
            }
        }
    };

    std::fs::create_dir_all(&state_dir)?;
    state_dir.push(access_token_id);

    Ok(state_dir)
}
