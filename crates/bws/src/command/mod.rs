pub(crate) mod project;
pub(crate) mod run;
pub(crate) mod secret;

use std::{path::PathBuf, str::FromStr};

use bitwarden::secrets_manager::AccessToken;
use clap::{CommandFactory, ValueEnum};
use clap_complete::Shell;
use color_eyre::eyre::Result;

use crate::{Cli, ProfileKey, config, error::UserError, render, util};

const CONFIG_USAGE_HINT: &str = "Usage: bws config <name> <value>";

/// The name of `key` as typed on the command line, e.g. `server-base`.
fn key_name(key: ProfileKey) -> String {
    key.to_possible_value()
        .map(|v| v.get_name().to_string())
        .unwrap_or_default()
}

pub(crate) fn completions(shell: Option<Shell>) -> Result<()> {
    let Some(shell) = shell.or_else(Shell::from_env) else {
        return Err(UserError::new("Could not detect your shell.")
            .hint("Pass it explicitly: bws completions <bash|elvish|fish|powershell|zsh>")
            .into());
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
                UserError::new(
                    "The access token in BWS_ACCESS_TOKEN or --access-token is malformed.",
                )
                .hint("Fix or unset it, or pass --profile.")
                .source(e)
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
            (None, _) => {
                return Err(UserError::new("Missing config name.")
                    .hint(CONFIG_USAGE_HINT)
                    .into());
            }
            (Some(name), None) => {
                return Err(
                    UserError::new(format!("Missing value for '{}'.", key_name(name)))
                        .hint(CONFIG_USAGE_HINT)
                        .into(),
                );
            }
            (
                Some(
                    name @ (ProfileKey::server_base
                    | ProfileKey::server_api
                    | ProfileKey::server_identity),
                ),
                Some(value),
            ) if !(value.starts_with("http://") || value.starts_with("https://")) => {
                return Err(UserError::new(format!(
                    "Invalid value for '{}'; expected a URL starting with http:// or https://.",
                    key_name(name)
                ))
                .into());
            }
            (Some(ProfileKey::state_opt_out), Some(value)) => {
                if util::string_to_bool(value.as_str()).is_err() {
                    return Err(UserError::new(
                        "Invalid value for 'state-opt-out'; expected true, false, 1, or 0.",
                    )
                    .into());
                } else {
                    (ProfileKey::state_opt_out, value)
                }
            }
            (Some(name), Some(value)) => (name, value),
        };

        config::update_profile(config_file.as_deref(), profile, name, value)?;
        render::write_stdout("Profile updated successfully!\n");
    };

    Ok(())
}
