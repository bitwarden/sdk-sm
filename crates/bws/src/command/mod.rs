pub(crate) mod generate;
pub(crate) mod project;
pub(crate) mod run;
pub(crate) mod secret;

use std::{path::PathBuf, str::FromStr};

use bitwarden::auth::AccessToken;
use clap::CommandFactory;
use clap_complete::Shell;
use color_eyre::eyre::{Result, bail};

use crate::{Cli, ProfileKey, config, util};

pub(crate) fn completions(shell: Option<Shell>) -> Result<()> {
    let Some(shell) = shell.or_else(Shell::from_env) else {
        bail!("Couldn't autodetect a valid shell. Run `bws completions --help` for more info.");
    };

    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    clap_complete::generate(shell, &mut cmd, name, &mut std::io::stdout());

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
        AccessToken::from_str(&access_token)?
            .access_token_id
            .to_string()
    } else {
        String::from("default")
    };

    if delete {
        config::delete_profile(config_file.as_deref(), profile)?;
        println!("Profile deleted successfully!");
    } else {
        let (name, value) = match (name, value) {
            (None, None) => bail!("Missing `name` and `value`"),
            (None, Some(_)) => bail!("Missing `value`"),
            (Some(_), None) => bail!("Missing `name`"),
            (Some(ProfileKey::state_opt_out), Some(value)) => {
                if util::string_to_bool(value.as_str()).is_err() {
                    bail!("Profile key \"state_opt_out\" must be \"true\" or \"false\"");
                } else {
                    (ProfileKey::state_opt_out, value)
                }
            }
            (Some(name), Some(value)) => (name, value),
        };

        config::update_profile(config_file.as_deref(), profile, name, value)?;
        println!("Profile updated successfully!");
    };

    Ok(())
}
