use std::sync::Arc;

use bitwarden::platform::FingerprintRequest;

use crate::{error::Result, Client};

#[derive(uniffi::Object)]
pub struct ClientPlatform(pub(crate) Arc<Client>);

#[uniffi::export(async_runtime = "tokio")]
impl ClientPlatform {
    /// Fingerprint (public key)
    pub async fn fingerprint(&self, req: FingerprintRequest) -> Result<String> {
        Ok(self
            .0
             .0
            .write()
            .await
            .platform()
            .fingerprint(&req)?
            .fingerprint)
    }

    /// Fingerprint (private key)
    ///
    /// Arguments:
    /// - `fingerprint_material` - The input material, used in the fingerprint generation process.
    /// - `private_key` - The user's encrypted private key.
    pub async fn users_fingerprint(&self, fingerprint_material: String) -> Result<String> {
        Ok(self
            .0
             .0
            .write()
            .await
            .platform()
            .users_fingerprint(fingerprint_material)?)
    }
}
