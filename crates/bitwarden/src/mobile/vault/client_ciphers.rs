use super::client_vault::ClientVault;
use crate::{
    crypto::{Decryptable, Encryptable},
    error::Result,
    vault::{Cipher, CipherListView, CipherView},
    Client,
};

pub struct ClientCiphers<'a> {
    pub(crate) client: &'a Client,
}

impl<'a> ClientCiphers<'a> {
    pub async fn encrypt(&self, cipher_view: CipherView) -> Result<Cipher> {
        let enc = self.client.get_encryption_settings()?;

        let cipher = cipher_view.encrypt(enc)?;

        Ok(cipher)
    }

    pub async fn decrypt(&self, cipher: Cipher) -> Result<CipherView> {
        let enc = self.client.get_encryption_settings()?;

        let cipher_view = cipher.decrypt(enc)?;

        Ok(cipher_view)
    }

    pub async fn decrypt_list(&self, ciphers: Vec<Cipher>) -> Result<Vec<CipherListView>> {
        let enc = self.client.get_encryption_settings()?;

        let cipher_views: Result<_> = ciphers.into_iter().map(|c| c.decrypt(enc)).collect();

        cipher_views
    }
}

impl<'a> ClientVault<'a> {
    pub fn ciphers(&'a self) -> ClientCiphers<'a> {
        ClientCiphers {
            client: self.client,
        }
    }
}
