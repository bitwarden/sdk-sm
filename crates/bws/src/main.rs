use std::{path::PathBuf, process::ExitCode};

use bitwarden::secrets_manager::{
    AccessToken, AccessTokenLoginRequest, ClientSettings, SecretsManagerClient,
};
use bitwarden_cli::{Color, install_color_eyre};
use clap::{CommandFactory, Parser};
use color_eyre::eyre::Result;
use config::Profile;
use error::UserError;
use render::OutputSettings;

mod cli;
mod command;
mod config;
mod error;
mod render;
mod state;
mod util;

use crate::cli::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    // The TLS verifier logs its own error before bws reports the failure.
    let log_filter = if error::is_verbose() {
        "info"
    } else {
        "info,rustls_platform_verifier=off"
    };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_filter)).init();

    match process_commands().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(report) => {
            eprint!(
                "{}",
                if error::is_verbose() {
                    error::render_verbose(&report)
                } else {
                    error::render(&report)
                }
            );
            ExitCode::from(1)
        }
    }
}

async fn process_commands() -> Result<()> {
    let cli = Cli::parse();
    let color = cli.color;

    install_color_eyre(color)?;
    if !error::is_verbose() {
        std::panic::set_hook(Box::new(|info| {
            eprint!("{}", error::render_panic(info.payload()))
        }));
    }

    let Some(command) = cli.command else {
        let help = Cli::command().render_help();
        // Same as `Color::is_enabled()`, but the help goes to stderr and that method only ever
        // probes stdout.
        let stderr_color = match color {
            Color::Yes => true,
            Color::No => false,
            Color::Auto => supports_color::on(supports_color::Stream::Stderr).is_some(),
        };
        if stderr_color {
            eprintln!("{}", help.ansi());
        } else {
            eprintln!("{help}");
        }
        std::process::exit(1);
    };

    let access_token = cli
        .access_token
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());

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
                access_token,
                cli.config_file,
            );
        }
        _ => (),
    }

    let Some(access_token) = access_token else {
        return Err(UserError::new("No access token provided.")
            .hint("Pass --access-token or set BWS_ACCESS_TOKEN.")
            .into());
    };
    let access_token_obj: AccessToken = access_token
        .parse()
        .map_err(|e| error::malformed_token().source(e))?;

    let profile = get_config_profile(
        &cli.server_url,
        &cli.profile,
        &cli.config_file,
        &access_token_obj,
    )?;

    let settings = profile
        .as_ref()
        .map(|(name, p)| -> Result<_, UserError> {
            let (identity_url, api_url) = p.server_urls(name)?;
            Ok(ClientSettings {
                identity_url,
                api_url,
                ..Default::default()
            })
        })
        .transpose()?;
    let profile = profile.map(|(_, p)| p);

    let state_file = match get_state_opt_out(&profile) {
        true => None,
        false => match state::get_state_file(
            profile.and_then(|p| p.state_dir).map(Into::into),
            access_token_obj.access_token_id.to_string(),
        ) {
            Ok(state_file) => Some(state_file),
            Err(e) => {
                error::warn(&e.to_string(), e.hint_text());
                None
            }
        },
    };

    let identity_url = settings.as_ref().map(|s| s.identity_url.clone());
    let client = SecretsManagerClient::new(settings);

    // Load session or return if no session exists
    let _ = client
        .auth()
        .login_access_token(&AccessTokenLoginRequest {
            access_token,
            state_file,
        })
        .await
        .map_err(|e| error::login_error(e, identity_url.as_deref()))?;

    let Some(organization_id) = client.get_access_token_organization() else {
        return Err(
            UserError::new("The access token is not associated with an organization.").into(),
        );
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

        Commands::Config { .. } | Commands::Completions { .. } => {
            unreachable!()
        }
    }
}

/// Returns the profile to use and its name.
fn get_config_profile(
    server_url: &Option<String>,
    profile: &Option<String>,
    config_file: &Option<PathBuf>,
    access_token: &AccessToken,
) -> Result<Option<(String, config::Profile)>, color_eyre::Report> {
    let path = config::get_config_path(config_file.as_deref(), false)?;
    let config = config::load_config(Some(&path), config_file.is_some())?;

    let profile = if let Some(server_url) = server_url {
        let mut p = config::Profile::from_url(server_url)?;
        if p.state_opt_out.is_none()
            && let Some((_, default)) = config.select_profile("default", false, &path)?
        {
            p.state_opt_out = default.state_opt_out;
        }
        Some((server_url.to_owned(), p))
    } else {
        let profile_defined = profile.is_some();

        let profile_key = if let Some(profile) = profile {
            profile.to_owned()
        } else {
            access_token.access_token_id.to_string()
        };

        config.select_profile(&profile_key, profile_defined, &path)?
    };
    Ok(profile)
}

fn get_state_opt_out(profile: &Option<Profile>) -> bool {
    if let Some(profile) = profile
        && let Some(state_opt_out) = &profile.state_opt_out
    {
        return *state_opt_out;
    }

    false
}
