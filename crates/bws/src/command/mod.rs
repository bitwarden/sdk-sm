pub(crate) mod project;
pub(crate) mod run;
pub(crate) mod secret;

use std::{path::PathBuf, str::FromStr};

use bitwarden::secrets_manager::AccessToken;
use clap::{CommandFactory, ValueEnum};
use clap_complete::Shell;
use color_eyre::eyre::{Result, bail};

use crate::{Cli, ProfileKey, config, error::UserError, render, util};

/// The name of `key` as typed on the command line, e.g. `server-base`.
fn key_name(key: ProfileKey) -> String {
    key.to_possible_value()
        .expect("ProfileKey to have a clap value name")
        .get_name()
        .to_string()
}

/// Rejects values that `key` cannot hold, e.g. a server URL without a scheme.
fn validate_value(key: ProfileKey, value: &str) -> Result<()> {
    // HTTP only allowed in debug builds
    #[cfg(debug_assertions)]
    let valid = value.starts_with("http://") || value.starts_with("https://");
    #[cfg(debug_assertions)]
    let message = "a URL starting with http:// or https://";

    #[cfg(not(debug_assertions))]
    let valid = value.starts_with("https://");
    #[cfg(not(debug_assertions))]
    let message = "a URL starting with https://";

    let expected = match key {
        ProfileKey::server_base | ProfileKey::server_api | ProfileKey::server_identity
            if !valid =>
        {
            message
        }
        ProfileKey::state_opt_out if util::string_to_bool(value).is_err() => "true, false, 1, or 0",
        _ => return Ok(()),
    };
    Err(UserError::InvalidConfigValue {
        key: key_name(key),
        expected,
    }
    .into())
}

pub(crate) fn completions(shell: Option<Shell>) -> Result<()> {
    let Some(shell) = shell.or_else(Shell::from_env) else {
        bail!("Couldn't autodetect a valid shell. Run `bws completions --help` for more info.");
    };

    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    let mut script = Vec::new();
    clap_complete::generate(shell, &mut cmd, name, &mut script);
    render::write_stdout(script);

    Ok(())
}

pub(crate) fn config(
    name: Option<ProfileKey>,
    value: Option<String>,
    delete: bool,
    profile: Option<String>,
    access_token: Option<String>,
    config_file: Option<PathBuf>,
) -> Result<()> {
    let profile = if let Some(profile) = profile {
        profile
    } else if let Some(access_token) = access_token {
        AccessToken::from_str(&access_token)
            .map_err(|e| {
                UserError::MalformedConfigToken {
                    source: Box::new(e),
                }
            })?
            .access_token_id
            .to_string()
    } else {
        String::from("default")
    };

    if delete {
        config::delete_profile(config_file.as_deref(), profile)?;
        render::write_stdout("Profile deleted successfully!\n");
    } else {
        let (name, value) = match (name, value) {
            (None, _) => return Err(UserError::MissingConfigName.into()),
            (Some(name), None) => {
                return Err(UserError::MissingConfigValue {
                    key: key_name(name),
                }
                .into());
            }
            (Some(name), Some(value)) => {
                validate_value(name, &value)?;
                (name, value)
            }
        };

        config::update_profile(config_file.as_deref(), profile, name, value)?;
        render::write_stdout("Profile updated successfully!\n");
    };

    Ok(())
}
