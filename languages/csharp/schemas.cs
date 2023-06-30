// <auto-generated />
//
// To parse this JSON data, add NuGet 'Newtonsoft.Json' then do one of these:
//
//    using Bit.Sdk;
//
//    var clientSettings = ClientSettings.FromJson(jsonString);
//    var command = Command.FromJson(jsonString);
//    var responseForApiKeyLoginResponse = ResponseForApiKeyLoginResponse.FromJson(jsonString);
//    var responseForPasswordLoginResponse = ResponseForPasswordLoginResponse.FromJson(jsonString);
//    var responseForSecretIdentifiersResponse = ResponseForSecretIdentifiersResponse.FromJson(jsonString);
//    var responseForSecretResponse = ResponseForSecretResponse.FromJson(jsonString);
//    var responseForSecretsDeleteResponse = ResponseForSecretsDeleteResponse.FromJson(jsonString);

namespace Bit.Sdk
{
    using System;
    using System.Collections.Generic;

    using System.Globalization;
    using Newtonsoft.Json;
    using Newtonsoft.Json.Converters;

    /// <summary>
    /// Basic client behavior settings. These settings specify the various targets and behavior
    /// of the Bitwarden Client. They are optional and uneditable once the client is
    /// initialized.
    ///
    /// Defaults to
    ///
    /// ``` # use bitwarden::client::client_settings::{ClientSettings, DeviceType}; # use
    /// assert_matches::assert_matches; let settings = ClientSettings { identity_url:
    /// "https://identity.bitwarden.com".to_string(), api_url:
    /// "https://api.bitwarden.com".to_string(), user_agent: "Bitwarden Rust-SDK".to_string(),
    /// device_type: DeviceType::SDK, state_path: None, }; let default =
    /// ClientSettings::default(); assert_matches!(settings, default); ```
    ///
    /// Targets `localhost:8080` for debug builds.
    /// </summary>
    public partial class ClientSettings
    {
        /// <summary>
        /// The api url of the targeted Bitwarden instance. Defaults to `https://api.bitwarden.com`
        /// </summary>
        [JsonProperty("apiUrl")]
        public string ApiUrl { get; set; }

        /// <summary>
        /// Device type to send to Bitwarden. Defaults to SDK
        /// </summary>
        [JsonProperty("deviceType")]
        public DeviceType DeviceType { get; set; }

        /// <summary>
        /// The identity url of the targeted Bitwarden instance. Defaults to
        /// `https://identity.bitwarden.com`
        /// </summary>
        [JsonProperty("identityUrl")]
        public string IdentityUrl { get; set; }

        /// <summary>
        /// Path to the file that stores the SDK's internal state, when not set the state is kept in
        /// memory only This option has no effect when compiling for WebAssembly, in that case
        /// LocalStorage is always used.
        /// </summary>
        [JsonProperty("statePath")]
        public string StatePath { get; set; }

        /// <summary>
        /// The user_agent to sent to Bitwarden. Defaults to `Bitwarden Rust-SDK`
        /// </summary>
        [JsonProperty("userAgent")]
        public string UserAgent { get; set; }
    }

    /// <summary>
    /// Login with username and password
    ///
    /// This command is for initiating an authentication handshake with Bitwarden. Authorization
    /// may fail due to requiring 2fa or captcha challenge completion despite accurate
    /// credentials.
    ///
    /// This command is not capable of handling authentication requiring 2fa or captcha.
    ///
    /// Returns: [PasswordLoginResponse](crate::sdk::auth::response::PasswordLoginResponse)
    ///
    /// Login with API Key
    ///
    /// This command is for initiating an authentication handshake with Bitwarden.
    ///
    /// Returns: [ApiKeyLoginResponse](crate::sdk::auth::response::ApiKeyLoginResponse)
    ///
    /// Login with Secrets Manager Access Token
    ///
    /// This command is for initiating an authentication handshake with Bitwarden.
    ///
    /// Returns: [ApiKeyLoginResponse](crate::sdk::auth::response::ApiKeyLoginResponse)
    ///
    /// Login with a previously saved session
    ///
    /// > Requires Authentication Get the API key of the currently authenticated user
    ///
    /// Returns:
    /// [UserApiKeyResponse](crate::sdk::response::user_api_key_response::UserApiKeyResponse)
    ///
    /// Get the user's passphrase
    ///
    /// Returns: String
    ///
    /// > Requires Authentication Retrieve all user data, ciphers and organizations the user is a
    /// part of
    /// </summary>
    public partial class Command
    {
        [JsonProperty("passwordLogin", NullValueHandling = NullValueHandling.Ignore)]
        public PasswordLoginRequest PasswordLogin { get; set; }

        [JsonProperty("apiKeyLogin", NullValueHandling = NullValueHandling.Ignore)]
        public ApiKeyLoginRequest ApiKeyLogin { get; set; }

        [JsonProperty("accessTokenLogin", NullValueHandling = NullValueHandling.Ignore)]
        public AccessTokenLoginRequest AccessTokenLogin { get; set; }

        [JsonProperty("sessionLogin", NullValueHandling = NullValueHandling.Ignore)]
        public SessionLoginRequest SessionLogin { get; set; }

        [JsonProperty("getUserApiKey", NullValueHandling = NullValueHandling.Ignore)]
        public SecretVerificationRequest GetUserApiKey { get; set; }

        [JsonProperty("fingerprint", NullValueHandling = NullValueHandling.Ignore)]
        public FingerprintRequest Fingerprint { get; set; }

        [JsonProperty("sync", NullValueHandling = NullValueHandling.Ignore)]
        public SyncRequest Sync { get; set; }

        [JsonProperty("secrets", NullValueHandling = NullValueHandling.Ignore)]
        public SecretsCommand Secrets { get; set; }

        [JsonProperty("projects", NullValueHandling = NullValueHandling.Ignore)]
        public ProjectsCommand Projects { get; set; }

        [JsonProperty("folders", NullValueHandling = NullValueHandling.Ignore)]
        public FoldersCommand Folders { get; set; }
    }

    /// <summary>
    /// Login to Bitwarden with access token
    /// </summary>
    public partial class AccessTokenLoginRequest
    {
        /// <summary>
        /// Bitwarden service API access token
        /// </summary>
        [JsonProperty("accessToken")]
        public string AccessToken { get; set; }
    }

    /// <summary>
    /// Login to Bitwarden with Api Key
    /// </summary>
    public partial class ApiKeyLoginRequest
    {
        /// <summary>
        /// Bitwarden account client_id
        /// </summary>
        [JsonProperty("clientId")]
        public string ClientId { get; set; }

        /// <summary>
        /// Bitwarden account client_secret
        /// </summary>
        [JsonProperty("clientSecret")]
        public string ClientSecret { get; set; }

        /// <summary>
        /// Bitwarden account master password
        /// </summary>
        [JsonProperty("password")]
        public string Password { get; set; }
    }

    public partial class FingerprintRequest
    {
        /// <summary>
        /// The input material, used in the fingerprint generation process.
        /// </summary>
        [JsonProperty("fingerprintMaterial")]
        public string FingerprintMaterial { get; set; }

        /// <summary>
        /// The user's public key
        /// </summary>
        [JsonProperty("publicKey")]
        public string PublicKey { get; set; }
    }

    /// <summary>
    /// > Requires Authentication > Requires an unlocked vault Creates a new folder with the
    /// provided data
    ///
    /// > Requires Authentication > Requires an unlocked vault and calling Sync at least once
    /// Lists all folders in the vault
    ///
    /// Returns: [FoldersResponse](bitwarden::platform::folders::FoldersResponse)
    ///
    /// > Requires Authentication > Requires an unlocked vault Updates an existing folder with
    /// the provided data given its ID
    ///
    /// > Requires Authentication > Requires an unlocked vault Deletes the folder associated with
    /// the provided ID
    /// </summary>
    public partial class FoldersCommand
    {
        [JsonProperty("create", NullValueHandling = NullValueHandling.Ignore)]
        public FolderCreateRequest Create { get; set; }

        [JsonProperty("list", NullValueHandling = NullValueHandling.Ignore)]
        public Dictionary<string, object> List { get; set; }

        [JsonProperty("update", NullValueHandling = NullValueHandling.Ignore)]
        public FolderUpdateRequest Update { get; set; }

        [JsonProperty("delete", NullValueHandling = NullValueHandling.Ignore)]
        public FolderDeleteRequest Delete { get; set; }
    }

    public partial class FolderCreateRequest
    {
        /// <summary>
        /// Encrypted folder name
        /// </summary>
        [JsonProperty("name")]
        public string Name { get; set; }
    }

    public partial class FolderDeleteRequest
    {
        /// <summary>
        /// ID of the folder to delete
        /// </summary>
        [JsonProperty("id")]
        public Guid Id { get; set; }
    }

    public partial class FolderUpdateRequest
    {
        /// <summary>
        /// ID of the folder to update
        /// </summary>
        [JsonProperty("id")]
        public Guid Id { get; set; }

        /// <summary>
        /// Encrypted folder name
        /// </summary>
        [JsonProperty("name")]
        public string Name { get; set; }
    }

    public partial class SecretVerificationRequest
    {
        /// <summary>
        /// The user's master password to use for user verification. If supplied, this will be used
        /// for verification purposes.
        /// </summary>
        [JsonProperty("masterPassword")]
        public string MasterPassword { get; set; }

        /// <summary>
        /// Alternate user verification method through OTP. This is provided for users who have no
        /// master password due to use of Customer Managed Encryption. Must be present and valid if
        /// master_password is absent.
        /// </summary>
        [JsonProperty("otp")]
        public string Otp { get; set; }
    }

    /// <summary>
    /// Login to Bitwarden with Username and Password
    /// </summary>
    public partial class PasswordLoginRequest
    {
        /// <summary>
        /// Bitwarden account email address
        /// </summary>
        [JsonProperty("email")]
        public string Email { get; set; }

        /// <summary>
        /// Bitwarden account master password
        /// </summary>
        [JsonProperty("password")]
        public string Password { get; set; }
    }

    /// <summary>
    /// > Requires Authentication > Requires using an Access Token for login or calling Sync at
    /// least once Retrieve a project by the provided identifier
    ///
    /// Returns: [ProjectResponse](crate::sdk::response::projects_response::ProjectResponse)
    ///
    /// > Requires Authentication > Requires using an Access Token for login or calling Sync at
    /// least once Creates a new project in the provided organization using the given data
    ///
    /// Returns: [ProjectResponse](crate::sdk::response::projects_response::ProjectResponse)
    ///
    /// > Requires Authentication > Requires using an Access Token for login or calling Sync at
    /// least once Lists all projects of the given organization
    ///
    /// Returns: [ProjectsResponse](crate::sdk::response::projects_response::ProjectsResponse)
    ///
    /// > Requires Authentication > Requires using an Access Token for login or calling Sync at
    /// least once Updates an existing project with the provided ID using the given data
    ///
    /// Returns: [ProjectResponse](crate::sdk::response::projects_response::ProjectResponse)
    ///
    /// > Requires Authentication > Requires using an Access Token for login or calling Sync at
    /// least once Deletes all the projects whose IDs match the provided ones
    ///
    /// Returns:
    /// [ProjectsDeleteResponse](crate::sdk::response::projects_response::ProjectsDeleteResponse)
    /// </summary>
    public partial class ProjectsCommand
    {
        [JsonProperty("get", NullValueHandling = NullValueHandling.Ignore)]
        public ProjectGetRequest Get { get; set; }

        [JsonProperty("create", NullValueHandling = NullValueHandling.Ignore)]
        public ProjectCreateRequest Create { get; set; }

        [JsonProperty("list", NullValueHandling = NullValueHandling.Ignore)]
        public ProjectsListRequest List { get; set; }

        [JsonProperty("update", NullValueHandling = NullValueHandling.Ignore)]
        public ProjectPutRequest Update { get; set; }

        [JsonProperty("delete", NullValueHandling = NullValueHandling.Ignore)]
        public ProjectsDeleteRequest Delete { get; set; }
    }

    public partial class ProjectCreateRequest
    {
        [JsonProperty("name")]
        public string Name { get; set; }

        /// <summary>
        /// Organization where the project will be created
        /// </summary>
        [JsonProperty("organizationId")]
        public Guid OrganizationId { get; set; }
    }

    public partial class ProjectsDeleteRequest
    {
        /// <summary>
        /// IDs of the projects to delete
        /// </summary>
        [JsonProperty("ids")]
        public Guid[] Ids { get; set; }
    }

    public partial class ProjectGetRequest
    {
        /// <summary>
        /// ID of the project to retrieve
        /// </summary>
        [JsonProperty("id")]
        public Guid Id { get; set; }
    }

    public partial class ProjectsListRequest
    {
        /// <summary>
        /// Organization to retrieve all the projects from
        /// </summary>
        [JsonProperty("organizationId")]
        public Guid OrganizationId { get; set; }
    }

    public partial class ProjectPutRequest
    {
        /// <summary>
        /// ID of the project to modify
        /// </summary>
        [JsonProperty("id")]
        public Guid Id { get; set; }

        [JsonProperty("name")]
        public string Name { get; set; }

        /// <summary>
        /// Organization ID of the project to modify
        /// </summary>
        [JsonProperty("organizationId")]
        public Guid OrganizationId { get; set; }
    }

    /// <summary>
    /// > Requires Authentication > Requires using an Access Token for login or calling Sync at
    /// least once Retrieve a secret by the provided identifier
    ///
    /// Returns: [SecretResponse](crate::sdk::response::secrets_response::SecretResponse)
    ///
    /// > Requires Authentication > Requires using an Access Token for login or calling Sync at
    /// least once Creates a new secret in the provided organization using the given data
    ///
    /// Returns: [SecretResponse](crate::sdk::response::secrets_response::SecretResponse)
    ///
    /// > Requires Authentication > Requires using an Access Token for login or calling Sync at
    /// least once Lists all secret identifiers of the given organization, to then retrieve each
    /// secret, use `CreateSecret`
    ///
    /// Returns:
    /// [SecretIdentifiersResponse](crate::sdk::response::secrets_response::SecretIdentifiersResponse)
    ///
    /// > Requires Authentication > Requires using an Access Token for login or calling Sync at
    /// least once Updates an existing secret with the provided ID using the given data
    ///
    /// Returns: [SecretResponse](crate::sdk::response::secrets_response::SecretResponse)
    ///
    /// > Requires Authentication > Requires using an Access Token for login or calling Sync at
    /// least once Deletes all the secrets whose IDs match the provided ones
    ///
    /// Returns:
    /// [SecretsDeleteResponse](crate::sdk::response::secrets_response::SecretsDeleteResponse)
    /// </summary>
    public partial class SecretsCommand
    {
        [JsonProperty("get", NullValueHandling = NullValueHandling.Ignore)]
        public SecretGetRequest Get { get; set; }

        [JsonProperty("create", NullValueHandling = NullValueHandling.Ignore)]
        public SecretCreateRequest Create { get; set; }

        [JsonProperty("list", NullValueHandling = NullValueHandling.Ignore)]
        public SecretIdentifiersRequest List { get; set; }

        [JsonProperty("update", NullValueHandling = NullValueHandling.Ignore)]
        public SecretPutRequest Update { get; set; }

        [JsonProperty("delete", NullValueHandling = NullValueHandling.Ignore)]
        public SecretsDeleteRequest Delete { get; set; }
    }

    public partial class SecretCreateRequest
    {
        [JsonProperty("key")]
        public string Key { get; set; }

        [JsonProperty("note")]
        public string Note { get; set; }

        /// <summary>
        /// Organization where the secret will be created
        /// </summary>
        [JsonProperty("organizationId")]
        public Guid OrganizationId { get; set; }

        /// <summary>
        /// IDs of the projects that this secret will belong to
        /// </summary>
        [JsonProperty("projectIds")]
        public Guid[] ProjectIds { get; set; }

        [JsonProperty("value")]
        public string Value { get; set; }
    }

    public partial class SecretsDeleteRequest
    {
        /// <summary>
        /// IDs of the secrets to delete
        /// </summary>
        [JsonProperty("ids")]
        public Guid[] Ids { get; set; }
    }

    public partial class SecretGetRequest
    {
        /// <summary>
        /// ID of the secret to retrieve
        /// </summary>
        [JsonProperty("id")]
        public Guid Id { get; set; }
    }

    public partial class SecretIdentifiersRequest
    {
        /// <summary>
        /// Organization to retrieve all the secrets from
        /// </summary>
        [JsonProperty("organizationId")]
        public Guid OrganizationId { get; set; }
    }

    public partial class SecretPutRequest
    {
        /// <summary>
        /// ID of the secret to modify
        /// </summary>
        [JsonProperty("id")]
        public Guid Id { get; set; }

        [JsonProperty("key")]
        public string Key { get; set; }

        [JsonProperty("note")]
        public string Note { get; set; }

        /// <summary>
        /// Organization ID of the secret to modify
        /// </summary>
        [JsonProperty("organizationId")]
        public Guid OrganizationId { get; set; }

        [JsonProperty("value")]
        public string Value { get; set; }
    }

    /// <summary>
    /// Login to Bitwarden using a saved session
    /// </summary>
    public partial class SessionLoginRequest
    {
        /// <summary>
        /// User's master password, used to unlock the vault
        /// </summary>
        [JsonProperty("password")]
        public string Password { get; set; }

        /// <summary>
        /// User's uuid
        /// </summary>
        [JsonProperty("userId")]
        public Guid UserId { get; set; }
    }

    public partial class SyncRequest
    {
        /// <summary>
        /// Exclude the subdomains from the response, defaults to false
        /// </summary>
        [JsonProperty("excludeSubdomains")]
        public bool? ExcludeSubdomains { get; set; }
    }

    public partial class ResponseForApiKeyLoginResponse
    {
        /// <summary>
        /// The response data. Populated if `success` is true.
        /// </summary>
        [JsonProperty("data")]
        public ApiKeyLoginResponse Data { get; set; }

        /// <summary>
        /// A message for any error that may occur. Populated if `success` is false.
        /// </summary>
        [JsonProperty("errorMessage")]
        public string ErrorMessage { get; set; }

        /// <summary>
        /// Whether or not the SDK request succeeded.
        /// </summary>
        [JsonProperty("success")]
        public bool Success { get; set; }
    }

    public partial class ApiKeyLoginResponse
    {
        [JsonProperty("authenticated")]
        public bool Authenticated { get; set; }

        /// <summary>
        /// Whether or not the user is required to update their master password
        /// </summary>
        [JsonProperty("forcePasswordReset")]
        public bool ForcePasswordReset { get; set; }

        /// <summary>
        /// TODO: What does this do?
        /// </summary>
        [JsonProperty("resetMasterPassword")]
        public bool ResetMasterPassword { get; set; }

        [JsonProperty("twoFactor")]
        public ApiKeyLoginResponseTwoFactorProviders TwoFactor { get; set; }
    }

    public partial class ApiKeyLoginResponseTwoFactorProviders
    {
        [JsonProperty("authenticator")]
        public PurpleAuthenticator Authenticator { get; set; }

        /// <summary>
        /// Duo-backed 2fa
        /// </summary>
        [JsonProperty("duo")]
        public PurpleDuo Duo { get; set; }

        /// <summary>
        /// Email 2fa
        /// </summary>
        [JsonProperty("email")]
        public PurpleEmail Email { get; set; }

        /// <summary>
        /// Duo-backed 2fa operated by an organization the user is a member of
        /// </summary>
        [JsonProperty("organizationDuo")]
        public PurpleDuo OrganizationDuo { get; set; }

        /// <summary>
        /// Presence indicates the user has stored this device as bypassing 2fa
        /// </summary>
        [JsonProperty("remember")]
        public PurpleRemember Remember { get; set; }

        /// <summary>
        /// WebAuthn-backed 2fa
        /// </summary>
        [JsonProperty("webAuthn")]
        public PurpleWebAuthn WebAuthn { get; set; }

        /// <summary>
        /// Yubikey-backed 2fa
        /// </summary>
        [JsonProperty("yubiKey")]
        public PurpleYubiKey YubiKey { get; set; }
    }

    public partial class PurpleAuthenticator
    {
    }

    public partial class PurpleDuo
    {
        [JsonProperty("host")]
        public string Host { get; set; }

        [JsonProperty("signature")]
        public string Signature { get; set; }
    }

    public partial class PurpleEmail
    {
        /// <summary>
        /// The email to request a 2fa TOTP for
        /// </summary>
        [JsonProperty("email")]
        public string Email { get; set; }
    }

    public partial class PurpleRemember
    {
    }

    public partial class PurpleWebAuthn
    {
    }

    public partial class PurpleYubiKey
    {
        /// <summary>
        /// Whether the stored yubikey supports near field communication
        /// </summary>
        [JsonProperty("nfc")]
        public bool Nfc { get; set; }
    }

    public partial class ResponseForPasswordLoginResponse
    {
        /// <summary>
        /// The response data. Populated if `success` is true.
        /// </summary>
        [JsonProperty("data")]
        public PasswordLoginResponse Data { get; set; }

        /// <summary>
        /// A message for any error that may occur. Populated if `success` is false.
        /// </summary>
        [JsonProperty("errorMessage")]
        public string ErrorMessage { get; set; }

        /// <summary>
        /// Whether or not the SDK request succeeded.
        /// </summary>
        [JsonProperty("success")]
        public bool Success { get; set; }
    }

    public partial class PasswordLoginResponse
    {
        [JsonProperty("authenticated")]
        public bool Authenticated { get; set; }

        /// <summary>
        /// The information required to present the user with a captcha challenge. Only present when
        /// authentication fails due to requiring validation of a captcha challenge.
        /// </summary>
        [JsonProperty("captcha")]
        public CaptchaResponse Captcha { get; set; }

        /// <summary>
        /// Whether or not the user is required to update their master password
        /// </summary>
        [JsonProperty("forcePasswordReset")]
        public bool ForcePasswordReset { get; set; }

        /// <summary>
        /// TODO: What does this do?
        /// </summary>
        [JsonProperty("resetMasterPassword")]
        public bool ResetMasterPassword { get; set; }

        /// <summary>
        /// The available two factor authentication options. Present only when authentication fails
        /// due to requiring a second authentication factor.
        /// </summary>
        [JsonProperty("twoFactor")]
        public PasswordLoginResponseTwoFactorProviders TwoFactor { get; set; }
    }

    public partial class CaptchaResponse
    {
        /// <summary>
        /// hcaptcha site key
        /// </summary>
        [JsonProperty("siteKey")]
        public string SiteKey { get; set; }
    }

    public partial class PasswordLoginResponseTwoFactorProviders
    {
        [JsonProperty("authenticator")]
        public FluffyAuthenticator Authenticator { get; set; }

        /// <summary>
        /// Duo-backed 2fa
        /// </summary>
        [JsonProperty("duo")]
        public FluffyDuo Duo { get; set; }

        /// <summary>
        /// Email 2fa
        /// </summary>
        [JsonProperty("email")]
        public FluffyEmail Email { get; set; }

        /// <summary>
        /// Duo-backed 2fa operated by an organization the user is a member of
        /// </summary>
        [JsonProperty("organizationDuo")]
        public FluffyDuo OrganizationDuo { get; set; }

        /// <summary>
        /// Presence indicates the user has stored this device as bypassing 2fa
        /// </summary>
        [JsonProperty("remember")]
        public FluffyRemember Remember { get; set; }

        /// <summary>
        /// WebAuthn-backed 2fa
        /// </summary>
        [JsonProperty("webAuthn")]
        public FluffyWebAuthn WebAuthn { get; set; }

        /// <summary>
        /// Yubikey-backed 2fa
        /// </summary>
        [JsonProperty("yubiKey")]
        public FluffyYubiKey YubiKey { get; set; }
    }

    public partial class FluffyAuthenticator
    {
    }

    public partial class FluffyDuo
    {
        [JsonProperty("host")]
        public string Host { get; set; }

        [JsonProperty("signature")]
        public string Signature { get; set; }
    }

    public partial class FluffyEmail
    {
        /// <summary>
        /// The email to request a 2fa TOTP for
        /// </summary>
        [JsonProperty("email")]
        public string Email { get; set; }
    }

    public partial class FluffyRemember
    {
    }

    public partial class FluffyWebAuthn
    {
    }

    public partial class FluffyYubiKey
    {
        /// <summary>
        /// Whether the stored yubikey supports near field communication
        /// </summary>
        [JsonProperty("nfc")]
        public bool Nfc { get; set; }
    }

    public partial class ResponseForSecretIdentifiersResponse
    {
        /// <summary>
        /// The response data. Populated if `success` is true.
        /// </summary>
        [JsonProperty("data")]
        public SecretIdentifiersResponse Data { get; set; }

        /// <summary>
        /// A message for any error that may occur. Populated if `success` is false.
        /// </summary>
        [JsonProperty("errorMessage")]
        public string ErrorMessage { get; set; }

        /// <summary>
        /// Whether or not the SDK request succeeded.
        /// </summary>
        [JsonProperty("success")]
        public bool Success { get; set; }
    }

    public partial class SecretIdentifiersResponse
    {
        [JsonProperty("data")]
        public SecretIdentifierResponse[] Data { get; set; }
    }

    public partial class SecretIdentifierResponse
    {
        [JsonProperty("id")]
        public Guid Id { get; set; }

        [JsonProperty("key")]
        public string Key { get; set; }

        [JsonProperty("organizationId")]
        public Guid OrganizationId { get; set; }
    }

    public partial class ResponseForSecretResponse
    {
        /// <summary>
        /// The response data. Populated if `success` is true.
        /// </summary>
        [JsonProperty("data")]
        public SecretResponse Data { get; set; }

        /// <summary>
        /// A message for any error that may occur. Populated if `success` is false.
        /// </summary>
        [JsonProperty("errorMessage")]
        public string ErrorMessage { get; set; }

        /// <summary>
        /// Whether or not the SDK request succeeded.
        /// </summary>
        [JsonProperty("success")]
        public bool Success { get; set; }
    }

    public partial class SecretResponse
    {
        [JsonProperty("creationDate")]
        public string CreationDate { get; set; }

        [JsonProperty("id")]
        public Guid Id { get; set; }

        [JsonProperty("key")]
        public string Key { get; set; }

        [JsonProperty("note")]
        public string Note { get; set; }

        [JsonProperty("object")]
        public string Object { get; set; }

        [JsonProperty("organizationId")]
        public Guid OrganizationId { get; set; }

        [JsonProperty("projectId")]
        public Guid? ProjectId { get; set; }

        [JsonProperty("revisionDate")]
        public string RevisionDate { get; set; }

        [JsonProperty("value")]
        public string Value { get; set; }
    }

    public partial class ResponseForSecretsDeleteResponse
    {
        /// <summary>
        /// The response data. Populated if `success` is true.
        /// </summary>
        [JsonProperty("data")]
        public SecretsDeleteResponse Data { get; set; }

        /// <summary>
        /// A message for any error that may occur. Populated if `success` is false.
        /// </summary>
        [JsonProperty("errorMessage")]
        public string ErrorMessage { get; set; }

        /// <summary>
        /// Whether or not the SDK request succeeded.
        /// </summary>
        [JsonProperty("success")]
        public bool Success { get; set; }
    }

    public partial class SecretsDeleteResponse
    {
        [JsonProperty("data")]
        public SecretDeleteResponse[] Data { get; set; }
    }

    public partial class SecretDeleteResponse
    {
        [JsonProperty("error")]
        public string Error { get; set; }

        [JsonProperty("id")]
        public Guid Id { get; set; }
    }

    /// <summary>
    /// Device type to send to Bitwarden. Defaults to SDK
    /// </summary>
    public enum DeviceType { Android, AndroidAmazon, ChromeBrowser, ChromeExtension, EdgeBrowser, EdgeExtension, FirefoxBrowser, FirefoxExtension, IOs, IeBrowser, LinuxDesktop, MacOsDesktop, OperaBrowser, OperaExtension, SafariBrowser, SafariExtension, Sdk, UnknownBrowser, Uwp, VivaldiBrowser, VivaldiExtension, WindowsDesktop };

    public partial class ClientSettings
    {
        public static ClientSettings FromJson(string json) => JsonConvert.DeserializeObject<ClientSettings>(json, Bit.Sdk.Converter.Settings);
    }

    public partial class Command
    {
        public static Command FromJson(string json) => JsonConvert.DeserializeObject<Command>(json, Bit.Sdk.Converter.Settings);
    }

    public partial class ResponseForApiKeyLoginResponse
    {
        public static ResponseForApiKeyLoginResponse FromJson(string json) => JsonConvert.DeserializeObject<ResponseForApiKeyLoginResponse>(json, Bit.Sdk.Converter.Settings);
    }

    public partial class ResponseForPasswordLoginResponse
    {
        public static ResponseForPasswordLoginResponse FromJson(string json) => JsonConvert.DeserializeObject<ResponseForPasswordLoginResponse>(json, Bit.Sdk.Converter.Settings);
    }

    public partial class ResponseForSecretIdentifiersResponse
    {
        public static ResponseForSecretIdentifiersResponse FromJson(string json) => JsonConvert.DeserializeObject<ResponseForSecretIdentifiersResponse>(json, Bit.Sdk.Converter.Settings);
    }

    public partial class ResponseForSecretResponse
    {
        public static ResponseForSecretResponse FromJson(string json) => JsonConvert.DeserializeObject<ResponseForSecretResponse>(json, Bit.Sdk.Converter.Settings);
    }

    public partial class ResponseForSecretsDeleteResponse
    {
        public static ResponseForSecretsDeleteResponse FromJson(string json) => JsonConvert.DeserializeObject<ResponseForSecretsDeleteResponse>(json, Bit.Sdk.Converter.Settings);
    }

    public static class Serialize
    {
        public static string ToJson(this ClientSettings self) => JsonConvert.SerializeObject(self, Bit.Sdk.Converter.Settings);
        public static string ToJson(this Command self) => JsonConvert.SerializeObject(self, Bit.Sdk.Converter.Settings);
        public static string ToJson(this ResponseForApiKeyLoginResponse self) => JsonConvert.SerializeObject(self, Bit.Sdk.Converter.Settings);
        public static string ToJson(this ResponseForPasswordLoginResponse self) => JsonConvert.SerializeObject(self, Bit.Sdk.Converter.Settings);
        public static string ToJson(this ResponseForSecretIdentifiersResponse self) => JsonConvert.SerializeObject(self, Bit.Sdk.Converter.Settings);
        public static string ToJson(this ResponseForSecretResponse self) => JsonConvert.SerializeObject(self, Bit.Sdk.Converter.Settings);
        public static string ToJson(this ResponseForSecretsDeleteResponse self) => JsonConvert.SerializeObject(self, Bit.Sdk.Converter.Settings);
    }

    internal static class Converter
    {
        public static readonly JsonSerializerSettings Settings = new JsonSerializerSettings
        {
            MetadataPropertyHandling = MetadataPropertyHandling.Ignore,
            DateParseHandling = DateParseHandling.None,
            Converters =
            {
                DeviceTypeConverter.Singleton,
                new IsoDateTimeConverter { DateTimeStyles = DateTimeStyles.AssumeUniversal }
            },
        };
    }

    internal class DeviceTypeConverter : JsonConverter
    {
        public override bool CanConvert(Type t) => t == typeof(DeviceType) || t == typeof(DeviceType?);

        public override object ReadJson(JsonReader reader, Type t, object existingValue, JsonSerializer serializer)
        {
            if (reader.TokenType == JsonToken.Null) return null;
            var value = serializer.Deserialize<string>(reader);
            switch (value)
            {
                case "Android":
                    return DeviceType.Android;
                case "AndroidAmazon":
                    return DeviceType.AndroidAmazon;
                case "ChromeBrowser":
                    return DeviceType.ChromeBrowser;
                case "ChromeExtension":
                    return DeviceType.ChromeExtension;
                case "EdgeBrowser":
                    return DeviceType.EdgeBrowser;
                case "EdgeExtension":
                    return DeviceType.EdgeExtension;
                case "FirefoxBrowser":
                    return DeviceType.FirefoxBrowser;
                case "FirefoxExtension":
                    return DeviceType.FirefoxExtension;
                case "IEBrowser":
                    return DeviceType.IeBrowser;
                case "LinuxDesktop":
                    return DeviceType.LinuxDesktop;
                case "MacOsDesktop":
                    return DeviceType.MacOsDesktop;
                case "OperaBrowser":
                    return DeviceType.OperaBrowser;
                case "OperaExtension":
                    return DeviceType.OperaExtension;
                case "SDK":
                    return DeviceType.Sdk;
                case "SafariBrowser":
                    return DeviceType.SafariBrowser;
                case "SafariExtension":
                    return DeviceType.SafariExtension;
                case "UWP":
                    return DeviceType.Uwp;
                case "UnknownBrowser":
                    return DeviceType.UnknownBrowser;
                case "VivaldiBrowser":
                    return DeviceType.VivaldiBrowser;
                case "VivaldiExtension":
                    return DeviceType.VivaldiExtension;
                case "WindowsDesktop":
                    return DeviceType.WindowsDesktop;
                case "iOS":
                    return DeviceType.IOs;
            }
            throw new Exception("Cannot unmarshal type DeviceType");
        }

        public override void WriteJson(JsonWriter writer, object untypedValue, JsonSerializer serializer)
        {
            if (untypedValue == null)
            {
                serializer.Serialize(writer, null);
                return;
            }
            var value = (DeviceType)untypedValue;
            switch (value)
            {
                case DeviceType.Android:
                    serializer.Serialize(writer, "Android");
                    return;
                case DeviceType.AndroidAmazon:
                    serializer.Serialize(writer, "AndroidAmazon");
                    return;
                case DeviceType.ChromeBrowser:
                    serializer.Serialize(writer, "ChromeBrowser");
                    return;
                case DeviceType.ChromeExtension:
                    serializer.Serialize(writer, "ChromeExtension");
                    return;
                case DeviceType.EdgeBrowser:
                    serializer.Serialize(writer, "EdgeBrowser");
                    return;
                case DeviceType.EdgeExtension:
                    serializer.Serialize(writer, "EdgeExtension");
                    return;
                case DeviceType.FirefoxBrowser:
                    serializer.Serialize(writer, "FirefoxBrowser");
                    return;
                case DeviceType.FirefoxExtension:
                    serializer.Serialize(writer, "FirefoxExtension");
                    return;
                case DeviceType.IeBrowser:
                    serializer.Serialize(writer, "IEBrowser");
                    return;
                case DeviceType.LinuxDesktop:
                    serializer.Serialize(writer, "LinuxDesktop");
                    return;
                case DeviceType.MacOsDesktop:
                    serializer.Serialize(writer, "MacOsDesktop");
                    return;
                case DeviceType.OperaBrowser:
                    serializer.Serialize(writer, "OperaBrowser");
                    return;
                case DeviceType.OperaExtension:
                    serializer.Serialize(writer, "OperaExtension");
                    return;
                case DeviceType.Sdk:
                    serializer.Serialize(writer, "SDK");
                    return;
                case DeviceType.SafariBrowser:
                    serializer.Serialize(writer, "SafariBrowser");
                    return;
                case DeviceType.SafariExtension:
                    serializer.Serialize(writer, "SafariExtension");
                    return;
                case DeviceType.Uwp:
                    serializer.Serialize(writer, "UWP");
                    return;
                case DeviceType.UnknownBrowser:
                    serializer.Serialize(writer, "UnknownBrowser");
                    return;
                case DeviceType.VivaldiBrowser:
                    serializer.Serialize(writer, "VivaldiBrowser");
                    return;
                case DeviceType.VivaldiExtension:
                    serializer.Serialize(writer, "VivaldiExtension");
                    return;
                case DeviceType.WindowsDesktop:
                    serializer.Serialize(writer, "WindowsDesktop");
                    return;
                case DeviceType.IOs:
                    serializer.Serialize(writer, "iOS");
                    return;
            }
            throw new Exception("Cannot marshal type DeviceType");
        }

        public static readonly DeviceTypeConverter Singleton = new DeviceTypeConverter();
    }
}

