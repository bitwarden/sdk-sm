use std::{path::PathBuf, str::FromStr};

use bitwarden::{
    ClientSettings,
    auth::{AccessToken, login::AccessTokenLoginRequest},
};
use bitwarden_cli::install_color_eyre;
use clap::{CommandFactory, Parser};
use color_eyre::eyre::{Result, bail};
use config::Profile;
use log::error;
use render::OutputSettings;

mod cli;
mod command;
mod config;
mod render;
mod state;
mod util;

use crate::cli::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    process_commands().await
}

async fn process_commands() -> Result<()> {
    let cli = Cli::parse();
    let color = cli.color;

    install_color_eyre(color)?;

    let Some(command) = cli.command else {
        let mut cmd = Cli::command();
        eprintln!("{}", cmd.render_help().ansi());
        std::process::exit(1);
    };

    // These commands don't require authentication, so we process them first
    match command {
        Commands::Completions { shell } => {
            return command::completions(shell);
        }
        Commands::Config {
            name,
            value,
            delete,
        } => {
            return command::config(
                name,
                value,
                delete,
                cli.profile,
                cli.access_token,
                cli.config_file,
            );
        }
        Commands::Generate {
            include_lowercase,
            include_uppercase,
            include_numbers,
            length,
            include_special,
            include_ambiguous,
            min_lowercase,
            min_uppercase,
            min_number,
            min_special,
        } => {
            return command::generate::generate_secret(
                include_lowercase,
                include_uppercase,
                include_numbers,
                length,
                include_special,
                include_ambiguous,
                min_lowercase,
                min_uppercase,
                min_number,
                min_special,
            );
        }
        _ => (),
    }

    let access_token = match cli.access_token {
        Some(key) => key,
        None => bail!("Missing access token"),
    };
    let access_token_obj: AccessToken = access_token.parse()?;

    let profile = get_config_profile(
        &cli.server_url,
        &cli.profile,
        &cli.config_file,
        &access_token,
    )?;

    let settings = profile
        .clone()
        .map(|p| -> Result<_> {
            Ok(ClientSettings {
                identity_url: p.identity_url()?,
                api_url: p.api_url()?,
                ..Default::default()
            })
        })
        .transpose()?;

    let state_file = match get_state_opt_out(&profile) {
        true => None,
        false => match state::get_state_file(
            profile.and_then(|p| p.state_dir).map(Into::into),
            access_token_obj.access_token_id.to_string(),
        ) {
            Ok(state_file) => Some(state_file),
            Err(e) => {
                eprintln!(
                    "Warning: {}\nRetrieving the state file failed. Attempting to continue without using state. Please set \"state_dir\" in your config file to avoid authentication limits.",
                    e
                );
                None
            }
        },
    };

    let client = bitwarden::Client::new(settings);

    // Load session or return if no session exists
    let _ = client
        .auth()
        .login_access_token(&AccessTokenLoginRequest {
            access_token,
            state_file,
        })
        .await?;

    let organization_id = match client.internal.get_access_token_organization() {
        Some(id) => id,
        None => {
            error!("Access token isn't associated to an organization.");
            return Ok(());
        }
    };

    let output_settings = OutputSettings::new(cli.output, color);

    // And finally we process all the commands which require authentication
    match command {
        Commands::Project { cmd } => {
            command::project::process_command(cmd, client, organization_id, output_settings).await
        }

        Commands::Secret { cmd } => {
            command::secret::process_command(cmd, client, organization_id, output_settings).await
        }

        Commands::Run {
            command,
            shell,
            no_inherit_env,
            project_id,
            uuids_as_keynames,
        } => {
            let exit_code = command::run::run(
                client,
                organization_id,
                project_id,
                uuids_as_keynames,
                no_inherit_env,
                shell,
                command,
            )
            .await?;

            // exit with the exit code from the child process
            std::process::exit(exit_code);
        }

        Commands::Config { .. } | Commands::Completions { .. } | Commands::Generate { .. } => {
            unreachable!()
        }
    }
}

fn get_config_profile(
    server_url: &Option<String>,
    profile: &Option<String>,
    config_file: &Option<PathBuf>,
    access_token: &str,
) -> Result<Option<config::Profile>, color_eyre::Report> {
    let profile = if let Some(server_url) = server_url {
        Some(config::Profile::from_url(server_url)?)
    } else {
        let profile_defined = profile.is_some();

        let profile_key = if let Some(profile) = profile {
            profile.to_owned()
        } else {
            AccessToken::from_str(access_token)?
                .access_token_id
                .to_string()
        };

        let config = config::load_config(config_file.as_deref(), config_file.is_some())?;
        config.select_profile(&profile_key, profile_defined)?
    };
    Ok(profile)
}

fn get_state_opt_out(profile: &Option<Profile>) -> bool {
    if let Some(profile) = profile {
        if let Some(state_opt_out) = &profile.state_opt_out {
            return util::string_to_bool(state_opt_out).unwrap_or(false);
        }
    }

    false
}
