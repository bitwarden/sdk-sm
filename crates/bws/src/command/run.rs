use std::{
    collections::HashMap,
    io::{IsTerminal, Read},
    process,
};

use bitwarden::secrets_manager::{
    SecretsManagerClient,
    secrets::{SecretIdentifiersByProjectRequest, SecretIdentifiersRequest, SecretsGetRequest},
};
use color_eyre::eyre::{Result, bail};
use itertools::Itertools;
use uuid::Uuid;
use which::which;

use crate::{
    ACCESS_TOKEN_KEY_VAR_NAME,
    util::{is_valid_posix_name, uuid_to_posix},
};

// Essential environment variables that should be preserved even when `--no-inherit-env` is used
const WINDOWS_ESSENTIAL_VARS: &[&str] = &["SystemRoot", "ComSpec", "windir"];

pub(crate) async fn run(
    client: SecretsManagerClient,
    organization_id: Uuid,
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
        bail!("Shell '{}' not found", shell);
    }

    let user_command = if command.is_empty() {
        if std::io::stdin().is_terminal() {
            bail!("No command provided");
        }

        let mut buffer = String::new();
        std::io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        command.join(" ")
    };

    let res = if let Some(project_id) = project_id {
        client
            .secrets()
            .list_by_project(&SecretIdentifiersByProjectRequest { project_id })
            .await?
    } else {
        client
            .secrets()
            .list(&SecretIdentifiersRequest { organization_id })
            .await?
    };

    let secret_ids = res.data.into_iter().map(|e| e.id).collect();
    let secrets = client
        .secrets()
        .get_by_ids(SecretsGetRequest { ids: secret_ids })
        .await?
        .data;

    if !uuids_as_keynames {
        if let Some(duplicate) = secrets.iter().map(|s| &s.key).duplicates().next() {
            bail!(
                "Multiple secrets with name: '{}'. Use --uuids-as-keynames or use unique names for secrets",
                duplicate
            );
        }
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
                eprintln!(
                    "Warning: secret '{}' does not have a POSIX-compliant name",
                    k
                );
            }
        })
        .collect();

    let mut command = process::Command::new(shell);
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
    match command.spawn() {
        Ok(mut child) => match child.wait() {
            Ok(exit_status) => Ok(exit_status.code().unwrap_or(1)),
            Err(e) => {
                bail!("Failed to wait for process: {}", e)
            }
        },
        Err(e) => {
            bail!("Failed to execute process: {}", e)
        }
    }
}
