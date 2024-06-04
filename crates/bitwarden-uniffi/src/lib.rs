uniffi::setup_scaffolding!();

use std::sync::Arc;

use async_lock::RwLock;
use auth::ClientAuth;
use bitwarden::ClientSettings;

pub mod auth;
pub mod crypto;
mod error;
pub mod platform;
pub mod tool;
mod uniffi_support;
pub mod vault;

#[cfg(feature = "docs")]
pub mod docs;

use crypto::ClientCrypto;
use error::Result;
use platform::ClientPlatform;
use tool::{ClientExporters, ClientGenerators, ClientSends};
use vault::ClientVault;

#[derive(uniffi::Object)]
pub struct Client(RwLock<bitwarden::Client>);

#[uniffi::export]
impl Client {
    /// Initialize a new instance of the SDK client
    #[uniffi::constructor]
    pub fn new(settings: Option<ClientSettings>) -> Arc<Self> {
        init_logger();
        Arc::new(Self(RwLock::new(bitwarden::Client::new(settings))))
    }

    /// Crypto operations
    pub fn crypto(self: Arc<Self>) -> Arc<ClientCrypto> {
        Arc::new(ClientCrypto(self))
    }

    /// Vault item operations
    pub fn vault(self: Arc<Self>) -> Arc<ClientVault> {
        Arc::new(ClientVault(self))
    }

    pub fn platform(self: Arc<Self>) -> Arc<ClientPlatform> {
        Arc::new(ClientPlatform(self))
    }

    /// Generator operations
    pub fn generators(self: Arc<Self>) -> Arc<ClientGenerators> {
        Arc::new(ClientGenerators(self))
    }

    /// Exporters
    pub fn exporters(self: Arc<Self>) -> Arc<ClientExporters> {
        Arc::new(ClientExporters(self))
    }

    /// Sends operations
    pub fn sends(self: Arc<Self>) -> Arc<ClientSends> {
        Arc::new(ClientSends(self))
    }

    /// Auth operations
    pub fn auth(self: Arc<Self>) -> Arc<ClientAuth> {
        Arc::new(ClientAuth(self))
    }

    /// Test method, echoes back the input
    pub fn echo(&self, msg: String) -> String {
        msg
    }
}

fn init_logger() {
    #[cfg(not(target_os = "android"))]
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init();

    #[cfg(target_os = "android")]
    android_logger::init_once(
        android_logger::Config::default().with_max_level(uniffi::deps::log::LevelFilter::Info),
    );
}
