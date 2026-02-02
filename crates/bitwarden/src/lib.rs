//! # Bitwarden
//!
//! A Rust client SDK to interact with the Bitwarden Secrets Manager.
//! This is a beta release and might be missing some functionality.
//!
//! To use this crate, add it to your `Cargo.toml`:
//!
//! ```ini
//! [dependencies]
//! bitwarden = { "*", features = ["secrets"] }
//! ```
//!
//! # Basic setup
//!
//! All operations in this crate are done via a [Client]:
//!
//! ```rust
//! use bitwarden::{
//!     auth::login::AccessTokenLoginRequest,
//!     error::Result,
//!     secrets_manager::{secrets::SecretIdentifiersRequest, ClientSecretsExt},
//!     Client, ClientSettings, DeviceType,
//! };
//! use uuid::Uuid;
//!
//! async fn test() -> Result<()> {
//!     // Use the default values
//!     let mut client = Client::new(None);
//!
//!     // Or set your own values
//!     let settings = ClientSettings {
//!         identity_url: "https://identity.bitwarden.com".to_string(),
//!         api_url: "https://api.bitwarden.com".to_string(),
//!         user_agent: "Bitwarden Rust-SDK".to_string(),
//!         device_type: DeviceType::SDK,
//!         bitwarden_client_version: Some(env!("CARGO_PKG_VERSION").to_string()),
//!         ..Default::default()
//!     };
//!     let mut client = Client::new(Some(settings));
//!
//!     // Before we operate, we need to authenticate with a token
//!     let token = AccessTokenLoginRequest {
//!         access_token: String::from(""),
//!         state_file: None,
//!     };
//!     client.auth().login_access_token(&token).await.unwrap();
//!
//!     let org_id = SecretIdentifiersRequest {
//!         organization_id: Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
//!     };
//!     println!(
//!         "Stored secrets: {:#?}",
//!         client.secrets().list(&org_id).await.unwrap()
//!     );
//!     Ok(())
//! }
//! ```

// Ensure the readme docs compile
#[doc = include_str!("../README.md")]
mod readme {}

pub use bitwarden_core::DeviceType;

pub mod error;

#[cfg(feature = "secrets")]
pub mod generators {
    pub use bitwarden_generators::{GeneratorClientsExt, PasswordError, PasswordGeneratorRequest};
}

#[cfg(feature = "secrets")]
pub mod secrets_manager {
    pub use bitwarden_sm::*;

    // These traits are here just for backwards compatibility, as now this functionality is exposed
    // by the client type directly.
    #[deprecated(note = "Using `ClientSecretsExt` is no longer necessary")]
    pub trait ClientSecretsExt {}
    #[deprecated(note = "Using `ClientProjectsExt` is no longer necessary")]
    pub trait ClientProjectsExt {}
    #[deprecated(note = "Using `ClientGeneratorsExt` is no longer necessary")]
    pub trait ClientGeneratorsExt {}
}

#[cfg(feature = "secrets")]
#[deprecated(note = "Use bitwarden_sm::secrets_manager::ClientSettings instead")]
pub use bitwarden_sm::ClientSettings;
#[cfg(feature = "secrets")]
#[deprecated(note = "Use bitwarden_sm::secrets_manager::SecretsManagerClient instead")]
pub use bitwarden_sm::SecretsManagerClient as Client;

#[cfg(feature = "secrets")]
#[deprecated(note = "Use bitwarden_sm::secrets_manager::* instead")]
pub mod auth {
    pub use bitwarden_sm::AccessToken;
    pub mod login {
        pub use bitwarden_sm::{AccessTokenLoginRequest, AccessTokenLoginResponse};
    }
}
