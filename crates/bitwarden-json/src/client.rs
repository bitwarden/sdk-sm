use bitwarden::ClientSettings;
#[cfg(feature = "secrets")]
use bitwarden::{
    generators::ClientGeneratorExt,
    secrets_manager::{ClientProjectsExt, ClientSecretsExt},
};

#[cfg(feature = "secrets")]
use crate::command::{GeneratorsCommand, ProjectsCommand, SecretsCommand};
use crate::{
    command::Command,
    response::{Response, ResponseIntoString},
};

pub struct Client(bitwarden::Client);

impl Client {
    pub fn new(settings_input: Option<String>) -> Self {
        let settings = Self::parse_settings(settings_input);
        Self(bitwarden::Client::new(settings))
    }

    pub async fn run_command(&self, input_str: &str) -> String {
        const SUBCOMMANDS_TO_CLEAN: &[&str] = &["Secrets"];
        let mut cmd_value: serde_json::Value = match serde_json::from_str(input_str) {
            Ok(cmd) => cmd,
            Err(e) => {
                return Response::error(format!("Invalid command string: {}", e)).into_string()
            }
        };

        if let Some(cmd_value_map) = cmd_value.as_object_mut() {
            cmd_value_map.retain(|_, v| !v.is_null());

            for &subcommand in SUBCOMMANDS_TO_CLEAN {
                if let Some(cmd_value_secrets) = cmd_value_map
                    .get_mut(subcommand)
                    .and_then(|v| v.as_object_mut())
                {
                    cmd_value_secrets.retain(|_, v| !v.is_null());
                }
            }
        }

        let cmd: Command = match serde_json::from_value(cmd_value) {
            Ok(cmd) => cmd,
            Err(e) => {
                return Response::error(format!("Invalid command value: {}", e)).into_string()
            }
        };

        let client = &self.0;

        match cmd {
            #[cfg(feature = "secrets")]
            Command::LoginAccessToken(req) => {
                client.auth().login_access_token(&req).await.into_string()
            }

            #[cfg(feature = "secrets")]
            Command::Secrets(cmd) => match cmd {
                SecretsCommand::Get(req) => client.secrets().get(&req).await.into_string(),
                SecretsCommand::GetByIds(req) => {
                    client.secrets().get_by_ids(req).await.into_string()
                }
                SecretsCommand::Create(req) => client.secrets().create(&req).await.into_string(),
                SecretsCommand::List(req) => client.secrets().list(&req).await.into_string(),
                SecretsCommand::Update(req) => client.secrets().update(&req).await.into_string(),
                SecretsCommand::Delete(req) => client.secrets().delete(req).await.into_string(),
                SecretsCommand::Sync(req) => client.secrets().sync(&req).await.into_string(),
            },

            #[cfg(feature = "secrets")]
            Command::Projects(cmd) => match cmd {
                ProjectsCommand::Get(req) => client.projects().get(&req).await.into_string(),
                ProjectsCommand::Create(req) => client.projects().create(&req).await.into_string(),
                ProjectsCommand::List(req) => client.projects().list(&req).await.into_string(),
                ProjectsCommand::Update(req) => client.projects().update(&req).await.into_string(),
                ProjectsCommand::Delete(req) => client.projects().delete(req).await.into_string(),
            },

            #[cfg(feature = "secrets")]
            Command::Generators(cmd) => match cmd {
                GeneratorsCommand::GeneratePassword(req) => {
                    client.generator().password(req).into_string()
                }
            },
            #[cfg(debug_assertions)]
            Command::Debug(cmd) => {
                use bitwarden::Error;

                use crate::command::DebugCommand;

                match cmd {
                    DebugCommand::CancellationTest { duration_millis } => {
                        use tokio::time::sleep;
                        let duration = std::time::Duration::from_millis(duration_millis);
                        sleep(duration).await;
                        println!("After wait #1");
                        sleep(duration).await;
                        println!("After wait #2");
                        sleep(duration).await;
                        println!("After wait #3");
                        Ok::<i32, Error>(42).into_string()
                    }
                    DebugCommand::ErrorTest {} => {
                        use bitwarden::Error;

                        Err::<i32, Error>(Error::Internal(std::borrow::Cow::Borrowed(
                            "This is an error.",
                        )))
                        .into_string()
                    }
                }
            }
        }
    }

    fn parse_settings(settings_input: Option<String>) -> Option<ClientSettings> {
        if let Some(input) = settings_input.as_ref() {
            match serde_json::from_str(input) {
                Ok(settings) => return Some(settings),
                Err(e) => {
                    log::error!("Failed to parse settings: {}", e);
                }
            }
        }
        None
    }
}
