use std::{
    collections::HashMap,
    io::{IsTerminal, Read},
    process,
};

use bitwarden::{
    OrganizationId,
    secrets_manager::{
        SecretsManagerClient,
        secrets::{SecretIdentifiersByProjectRequest, SecretIdentifiersRequest, SecretsGetRequest},
    },
};
use color_eyre::eyre::Result;
use itertools::Itertools;
use uuid::Uuid;
use which::which;

use crate::{
    ACCESS_TOKEN_KEY_VAR_NAME,
    error::{self, Op, Target, UserError},
    util::{is_valid_posix_name, uuid_to_posix},
};

// Essential environment variables that should be preserved even when `--no-inherit-env` is used
const WINDOWS_ESSENTIAL_VARS: &[&str] = &["SystemRoot", "ComSpec", "windir"];

pub(crate) async fn run(
    client: SecretsManagerClient,
    organization_id: OrganizationId,
    project_id: Option<Uuid>,
    uuids_as_keynames: bool,
    no_inherit_env: bool,
    shell: Option<String>,
    command: Vec<String>,
) -> Result<i32> {
    let is_windows = std::env::consts::OS == "windows";

    let shell = shell.unwrap_or_else(|| {
        if is_windows {
            "powershell".to_string()
        } else {
            "sh".to_string()
        }
    });

    if which(&shell).is_err() {
        return Err(UserError::new(format!("Shell '{shell}' not found."))
            .hint("Install it or pass a different shell with --shell.")
            .into());
    }

    let no_command = || UserError::new("No command provided.").hint("Usage: bws run -- <command>");
    let user_command = if command.is_empty() {
        if std::io::stdin().is_terminal() {
            return Err(no_command().into());
        }

        let mut buffer = String::new();
        std::io::stdin().read_to_string(&mut buffer).map_err(|e| {
            UserError::new(format!(
                "Could not read the command from stdin: {}.",
                error::io_reason(&e)
            ))
            .source(e)
        })?;
        if buffer.trim().is_empty() {
            return Err(no_command().into());
        }
        buffer
    } else {
        command.join(" ")
    };

    let res = if let Some(project_id) = project_id {
        client
            .secrets()
            .list_by_project(&SecretIdentifiersByProjectRequest { project_id })
            .await
            .map_err(|e| error::sm_error(e, Target::Project(project_id), Op::Read))?
    } else {
        client
            .secrets()
            .list(&SecretIdentifiersRequest {
                organization_id: organization_id.into(),
            })
            .await
            .map_err(|e| error::sm_error(e, Target::None, Op::Read))?
    };

    let secret_ids = res.data.into_iter().map(|e| e.id).collect();
    let secrets = client
        .secrets()
        .get_by_ids(SecretsGetRequest { ids: secret_ids })
        .await
        .map_err(|e| error::sm_error(e, Target::None, Op::Read))?
        .data;

    if !uuids_as_keynames
        && let Some(duplicate) = secrets.iter().map(|s| &s.key).duplicates().next()
    {
        return Err(
            UserError::new(format!("Multiple secrets are named '{duplicate}'."))
                .hint("Use unique secret names or pass --uuids-as-keynames.")
                .into(),
        );
    }

    let environment: HashMap<String, String> = secrets
        .into_iter()
        .map(|s| {
            if uuids_as_keynames {
                (uuid_to_posix(&s.id), s.value)
            } else {
                (s.key, s.value)
            }
        })
        .inspect(|(k, _)| {
            if !is_valid_posix_name(k) {
                error::warn(
                    &format!("Secret '{k}' is not a valid environment variable name."),
                    None,
                );
            }
        })
        .collect();

    let mut command = process::Command::new(&shell);
    command
        .arg("-c")
        .arg(&user_command)
        .stdout(process::Stdio::inherit())
        .stderr(process::Stdio::inherit());

    if no_inherit_env {
        let path = std::env::var("PATH").unwrap_or_else(|_| match is_windows {
            true => "C:\\Windows;C:\\Windows\\System32".to_string(),
            false => "/bin:/usr/bin".to_string(),
        });

        command.env_clear();

        // Preserve essential PowerShell environment variables on Windows
        if is_windows {
            for &var in WINDOWS_ESSENTIAL_VARS {
                if let Ok(value) = std::env::var(var) {
                    command.env(var, value);
                }
            }
        }

        command.env("PATH", path); // PATH is always necessary
        command.envs(environment);
    } else {
        command.env_remove(ACCESS_TOKEN_KEY_VAR_NAME);
        command.envs(environment);
    }

    // propagate the exit status from the child process
    let mut child = command.spawn().map_err(|e| {
        UserError::new(format!(
            "Could not start '{shell}': {}.",
            error::io_reason(&e)
        ))
        .source(e)
    })?;
    let exit_status = child.wait().map_err(|e| {
        UserError::new(format!(
            "Could not wait for the command to finish: {}.",
            error::io_reason(&e)
        ))
        .source(e)
    })?;
    Ok(exit_status.code().unwrap_or(1))
}
