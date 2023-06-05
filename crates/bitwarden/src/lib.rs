//! # Bitwarden
//!
//! A Rust client SDK to interact with the Bitwarden Secrets Manager.
//! This is a beta release and might be missing some functionality.
//!
//! To use this crate, add it to your `Cargo.toml`:
//!
//! ```ini
//! [dependencies]
//! bitwarden = "*"
//! ```
//!
//! # Basic setup
//!
//! All operations in this crate are done via a [Client](crate::client::Client):
//!
//! ```rust
//! use bitwarden::{
//!     Client,
//!     error::Result,
//!     sdk::{
//!         auth::request::AccessTokenLoginRequest,
//!         request::{
//!             client_settings::{ClientSettings, DeviceType},
//!             secrets_request::SecretIdentifiersRequest
//!         },
//!     },
//! };
//!
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
//!     };
//!     let mut client = Client::new(Some(settings));
//!
//!     // Before we operate, we need to authenticate with a token
//!     let token = AccessTokenLoginRequest { access_token: String::from("") };
//!     client.access_token_login(&token).await.unwrap();
//!
//!     let org_id = SecretIdentifiersRequest { organization_id: Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap() };
//!     println!("Stored secrets: {:#?}", client.secrets().list(&org_id).await.unwrap());
//!     Ok(())
//! }
//! ```
//!

mod api;
pub mod client;
mod commands;
pub mod crypto;
pub mod error;
pub mod sdk;
mod util;
pub mod wordlist;

pub use client::Client;
