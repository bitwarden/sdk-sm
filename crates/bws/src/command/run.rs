use std::{
    collections::{HashMap, HashSet},
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

// Environment variables that are always inherited from the current shell and can never be
// set by the run command, even when allowed with `--allow-vars`
const NEVER_SET_VARS: &[&str] = &["PATH"];

// Shell and language runtime environment variables that can inject code into the child
// process, regardless of the OS it's running on
const PORTABLE_DO_NOT_SET_VARS: &[&str] = &[
    // used for testing the `--allow-vars` flag
    "BWS_DO_NOT_SET",
    // shells
    "BASH_ENV",
    "ENV",
    "IFS",
    "PROMPT_COMMAND",
    "PS4",
    "SHELLOPTS",
    "ZDOTDIR",
    // python
    "PYTHONPATH",
    "PYTHONHOME",
    "PYTHONSTARTUP",
    // node
    "NODE_OPTIONS",
    "NODE_REPL_EXTERNAL_MODULE",
    // rust
    "RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER",
    "RUSTDOCFLAGS",
    "RUSTFLAGS",
    // ruby, perl and php
    "RUBYLIB",
    "RUBYOPT",
    "PERL5LIB",
    "PERL5OPT",
    "PHP_INI_SCAN_DIR",
    // jvm
    "CLASSPATH",
    "JAVA_TOOL_OPTIONS",
    "JDK_JAVA_OPTIONS",
    "_JAVA_OPTIONS",
    // dotnet
    "CORECLR_ENABLE_PROFILING",
    "DOTNET_ADDITIONAL_DEPS",
    "DOTNET_STARTUP_HOOKS",
];

// Dynamic linker environment variables, which only apply to the OS the CLI runs on
#[cfg(target_os = "linux")]
const OS_DO_NOT_SET_VARS: &[&str] = &["LD_LIBRARY_PATH", "LD_PRELOAD"];

#[cfg(target_os = "macos")]
const OS_DO_NOT_SET_VARS: &[&str] = &["DYLD_INSERT_LIBRARIES"];

#[cfg(target_os = "windows")]
const OS_DO_NOT_SET_VARS: &[&str] = &["USERPROFILE"];

/// Normalizes variable names for comparison, since environment variable names are
/// case-insensitive on Windows and hand written lists contain stray whitespace
fn normalized_vars<I, S>(vars: I) -> HashSet<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    vars.into_iter()
        .map(|var| var.as_ref().trim().to_lowercase())
        .filter(|var| !var.is_empty())
        .collect()
}

/// Decides which secret keys are injected into the environment of the child process.
///
/// All comparisons are case-insensitive because environment variable names are
/// case-insensitive on Windows.
struct VarFilter {
    allowed: HashSet<String>,
    never_set: HashSet<String>,
    do_not_set: HashSet<String>,
}

impl VarFilter {
    fn new(allow_vars: &[String], extra_do_not_set_vars: &[String]) -> Self {
        Self {
            allowed: normalized_vars(allow_vars),
            never_set: normalized_vars(NEVER_SET_VARS),
            // The built-in lists can only be extended, never replaced
            do_not_set: normalized_vars(
                PORTABLE_DO_NOT_SET_VARS
                    .iter()
                    .copied()
                    .chain(OS_DO_NOT_SET_VARS.iter().copied())
                    .chain(extra_do_not_set_vars.iter().map(String::as_str)),
            ),
        }
    }

    fn allows(&self, key: &str) -> bool {
        let key = key.trim().to_lowercase();

        !self.never_set.contains(&key)
            && (self.allowed.contains(&key) || !self.do_not_set.contains(&key))
    }
}

pub(crate) async fn run(
    client: SecretsManagerClient,
    organization_id: OrganizationId,
    project_id: Option<Uuid>,
    allow_vars: Vec<String>,
    extra_do_not_set_vars: Vec<String>,
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

    let var_filter = VarFilter::new(&allow_vars, &extra_do_not_set_vars);

    let res = if let Some(project_id) = project_id {
        client
            .secrets()
            .list_by_project(&SecretIdentifiersByProjectRequest { project_id })
            .await?
    } else {
        client
            .secrets()
            .list(&SecretIdentifiersRequest {
                organization_id: organization_id.into(),
            })
            .await?
    };

    let secret_ids = res.data.into_iter().map(|e| e.id).collect();
    let secrets = client
        .secrets()
        .get_by_ids(SecretsGetRequest { ids: secret_ids })
        .await?
        .data;

    if !uuids_as_keynames
        && let Some(duplicate) = secrets.iter().map(|s| &s.key).duplicates().next()
    {
        bail!(
            "Multiple secrets with name: '{}'. Use --uuids-as-keynames or use unique names for secrets",
            duplicate
        );
    }

    let environment: HashMap<String, String> = secrets
        .into_iter()
        .filter(|s| var_filter.allows(&s.key))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn vars(values: &[&str]) -> Vec<String> {
        values.iter().map(|v| (*v).to_string()).collect()
    }

    #[test]
    fn allows_unlisted_vars() {
        let filter = VarFilter::new(&[], &[]);

        assert!(filter.allows("TUX"));
        assert!(filter.allows("LD_PRELOAD_VALUE"));
    }

    #[test]
    fn denies_portable_and_os_vars_by_default() {
        let filter = VarFilter::new(&[], &[]);

        assert!(!filter.allows("PYTHONPATH"));
        assert!(!filter.allows("NODE_OPTIONS"));
        assert!(!filter.allows("RUSTFLAGS"));
        assert!(!filter.allows("BWS_DO_NOT_SET"));
        assert!(!filter.allows(OS_DO_NOT_SET_VARS[0]));
    }

    #[test]
    fn allows_denied_vars_when_allow_vars_includes_them() {
        let filter = VarFilter::new(&vars(&["PYTHONPATH"]), &[]);

        assert!(filter.allows("PYTHONPATH"));
        assert!(!filter.allows("NODE_OPTIONS"));
    }

    #[test]
    fn never_sets_path_even_when_allowed() {
        let filter = VarFilter::new(&vars(&["PATH"]), &[]);

        assert!(!filter.allows("PATH"));
    }

    #[test]
    fn extra_do_not_set_vars_extend_the_built_in_list() {
        let filter = VarFilter::new(&[], &vars(&["KUBECONFIG"]));

        assert!(!filter.allows("KUBECONFIG"));
        assert!(!filter.allows("PYTHONPATH"));

        let allowed = VarFilter::new(&vars(&["KUBECONFIG"]), &vars(&["KUBECONFIG"]));
        assert!(allowed.allows("KUBECONFIG"));
    }

    #[test]
    fn comparisons_are_case_insensitive() {
        let filter = VarFilter::new(&vars(&["pythonpath"]), &[]);

        assert!(filter.allows("PYTHONPATH"));
        assert!(!filter.allows("Node_Options"));
        assert!(!filter.allows("path"));
    }

    #[test]
    fn trims_surrounding_whitespace() {
        let filter = VarFilter::new(&[], &vars(&[" KUBECONFIG "]));

        assert!(!filter.allows("KUBECONFIG"));
        assert!(VarFilter::new(&vars(&[" PYTHONPATH "]), &[]).allows("PYTHONPATH"));
    }
}
