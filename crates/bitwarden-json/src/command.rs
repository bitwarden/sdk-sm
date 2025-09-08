#[cfg(feature = "secrets")]
use bitwarden::{
    auth::login::AccessTokenLoginRequest,
    generators::PasswordGeneratorRequest,
    secrets_manager::{
        projects::{
            ProjectCreateRequest, ProjectGetRequest, ProjectPutRequest, ProjectsDeleteRequest,
            ProjectsListRequest,
        },
        secrets::{
            SecretCreateRequest, SecretGetRequest, SecretIdentifiersRequest, SecretPutRequest,
            SecretsDeleteRequest, SecretsGetRequest, SecretsSyncRequest,
        },
    },
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum Command {
    #[cfg(feature = "secrets")]
    /// Login with Secrets Manager Access Token
    ///
    /// This command is for initiating an authentication handshake with Bitwarden.
    ///
    /// Returns: [ApiKeyLoginResponse](bitwarden::auth::login::ApiKeyLoginResponse)
    LoginAccessToken(AccessTokenLoginRequest),

    #[cfg(feature = "secrets")]
    Secrets(SecretsCommand),
    #[cfg(feature = "secrets")]
    Projects(ProjectsCommand),
    #[cfg(feature = "secrets")]
    Generators(GeneratorsCommand),
    #[cfg(debug_assertions)]
    Debug(DebugCommand),
}

#[cfg(feature = "secrets")]
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum SecretsCommand {
    /// > Requires Authentication
    /// > Requires using an Access Token for login or calling Sync at least once
    /// Retrieve a secret by the provided identifier
    ///
    /// Returns: [SecretResponse](bitwarden::secrets_manager::secrets::SecretResponse)
    Get(SecretGetRequest),

    /// > Requires Authentication
    /// > Requires using an Access Token for login or calling Sync at least once
    /// Retrieve secrets by the provided identifiers
    ///
    /// Returns: [SecretsResponse](bitwarden::secrets_manager::secrets::SecretsResponse)
    GetByIds(SecretsGetRequest),

    /// > Requires Authentication
    /// > Requires using an Access Token for login or calling Sync at least once
    /// Creates a new secret in the provided organization using the given data
    ///
    /// Returns: [SecretResponse](bitwarden::secrets_manager::secrets::SecretResponse)
    Create(SecretCreateRequest),

    /// > Requires Authentication
    /// > Requires using an Access Token for login or calling Sync at least once
    /// Lists all secret identifiers of the given organization, to then retrieve each secret, use
    /// `CreateSecret`
    ///
    /// Returns: [SecretIdentifiersResponse](bitwarden::secrets_manager::secrets::SecretIdentifiersResponse)
    List(SecretIdentifiersRequest),

    /// > Requires Authentication
    /// > Requires using an Access Token for login or calling Sync at least once
    /// Updates an existing secret with the provided ID using the given data
    ///
    /// Returns: [SecretResponse](bitwarden::secrets_manager::secrets::SecretResponse)
    Update(SecretPutRequest),

    /// > Requires Authentication
    /// > Requires using an Access Token for login or calling Sync at least once
    /// Deletes all the secrets whose IDs match the provided ones
    ///
    /// Returns: [SecretsDeleteResponse](bitwarden::secrets_manager::secrets::SecretsDeleteResponse)
    Delete(SecretsDeleteRequest),

    /// > Requires Authentication
    /// > Requires using an Access Token for login
    /// Retrieve the secrets accessible by the authenticated machine account
    /// Optionally, provide the last synced date to assess whether any changes have occurred
    /// If changes are detected, retrieves all the secrets accessible by the authenticated machine
    /// account
    ///
    /// Returns: [SecretsSyncResponse](bitwarden::secrets_manager::secrets::SecretsSyncResponse)
    Sync(SecretsSyncRequest),
}

#[cfg(feature = "secrets")]
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum ProjectsCommand {
    /// > Requires Authentication
    /// > Requires using an Access Token for login or calling Sync at least once
    /// Retrieve a project by the provided identifier
    ///
    /// Returns: [ProjectResponse](bitwarden::secrets_manager::projects::ProjectResponse)
    Get(ProjectGetRequest),

    /// > Requires Authentication
    /// > Requires using an Access Token for login or calling Sync at least once
    /// Creates a new project in the provided organization using the given data
    ///
    /// Returns: [ProjectResponse](bitwarden::secrets_manager::projects::ProjectResponse)
    Create(ProjectCreateRequest),

    /// > Requires Authentication
    /// > Requires using an Access Token for login or calling Sync at least once
    /// Lists all projects of the given organization
    ///
    /// Returns: [ProjectsResponse](bitwarden::secrets_manager::projects::ProjectsResponse)
    List(ProjectsListRequest),

    /// > Requires Authentication
    /// > Requires using an Access Token for login or calling Sync at least once
    /// Updates an existing project with the provided ID using the given data
    ///
    /// Returns: [ProjectResponse](bitwarden::secrets_manager::projects::ProjectResponse)
    Update(ProjectPutRequest),

    /// > Requires Authentication
    /// > Requires using an Access Token for login or calling Sync at least once
    /// Deletes all the projects whose IDs match the provided ones
    ///
    /// Returns: [ProjectsDeleteResponse](bitwarden::secrets_manager::projects::ProjectsDeleteResponse)
    Delete(ProjectsDeleteRequest),
}

#[cfg(feature = "secrets")]
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum GeneratorsCommand {
    /// Generate a password
    ///
    /// Returns: [String]
    GeneratePassword(PasswordGeneratorRequest),
}

#[cfg(debug_assertions)]
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum DebugCommand {
    CancellationTest { duration_millis: u64 },
    ErrorTest {},
}
