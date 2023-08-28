use std::collections::HashMap;

use rsa::RsaPrivateKey;
use uuid::Uuid;

use crate::{
    crypto::{decrypt, decrypt_aes256, encrypt_aes256, CipherString, SymmetricCryptoKey},
    error::{CryptoError, Error, Result},
};

#[cfg(feature = "internal")]
use {
    crate::client::auth_settings::AuthSettings,
    rsa::{pkcs8::DecodePrivateKey, Oaep},
};

pub struct EncryptionSettings {
    user_key: SymmetricCryptoKey,
    private_key: Option<RsaPrivateKey>,
    org_keys: HashMap<Uuid, SymmetricCryptoKey>,
}

impl std::fmt::Debug for EncryptionSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncryptionSettings").finish()
    }
}

impl EncryptionSettings {
    #[cfg(feature = "internal")]
    pub(crate) fn new(
        auth: &AuthSettings,
        password: &str,
        user_key: CipherString,
        private_key: CipherString,
    ) -> Result<Self> {
        // Stretch keys from the provided password

        let (key, mac_key) = crate::crypto::stretch_key_password(
            password.as_bytes(),
            auth.email.as_bytes(),
            &auth.kdf,
        )?;

        // Decrypt the user key with the stretched key
        let user_key = {
            let (iv, mac, data) = match user_key {
                CipherString::AesCbc256_HmacSha256_B64 { iv, mac, data } => (iv, mac, data),
                _ => return Err(CryptoError::InvalidKey.into()),
            };

            let dec = decrypt_aes256(&iv, &mac, data, Some(mac_key), key)?;
            SymmetricCryptoKey::try_from(dec.as_slice())?
        };

        // Decrypt the private key with the user key
        let private_key = {
            let dec = decrypt(&private_key, &user_key)?;
            Some(rsa::RsaPrivateKey::from_pkcs8_der(&dec).map_err(|_| CryptoError::InvalidKey)?)
        };

        Ok(EncryptionSettings {
            user_key,
            private_key,
            org_keys: HashMap::new(),
        })
    }

    pub(crate) fn new_single_key(key: SymmetricCryptoKey) -> Self {
        EncryptionSettings {
            user_key: key,
            private_key: None,
            org_keys: HashMap::new(),
        }
    }

    #[cfg(feature = "internal")]
    pub(crate) fn set_org_keys(
        &mut self,
        org_enc_keys: Vec<(Uuid, CipherString)>,
    ) -> Result<&mut Self> {
        let private_key = self.private_key.as_ref().ok_or(Error::VaultLocked)?;

        // Decrypt the org keys with the private key
        for (org_id, org_enc_key) in org_enc_keys {
            let data = match org_enc_key {
                CipherString::Rsa2048_OaepSha1_B64 { data } => data,
                _ => return Err(CryptoError::InvalidKey.into()),
            };

            let dec = private_key
                .decrypt(Oaep::new::<sha1::Sha1>(), &data)
                .map_err(|_| CryptoError::KeyDecrypt)?;

            let org_key = SymmetricCryptoKey::try_from(dec.as_slice())?;

            self.org_keys.insert(org_id, org_key);
        }

        Ok(self)
    }

    fn get_key(&self, org_id: &Option<Uuid>) -> Option<&SymmetricCryptoKey> {
        // If we don't have a private key set (to decode multiple org keys), we just use the main user key
        if self.private_key.is_none() {
            return Some(&self.user_key);
        }

        match org_id {
            Some(org_id) => self.org_keys.get(org_id),
            None => Some(&self.user_key),
        }
    }

    pub(crate) fn decrypt(&self, cipher: &CipherString, org_id: &Option<Uuid>) -> Result<String> {
        let key = self.get_key(org_id).ok_or(CryptoError::NoKeyForOrg)?;
        let dec = decrypt(cipher, key)?;
        String::from_utf8(dec).map_err(|_| CryptoError::InvalidUtf8String.into())
    }

    pub(crate) fn encrypt(&self, data: &[u8], org_id: &Option<Uuid>) -> Result<CipherString> {
        let key = self.get_key(org_id).ok_or(CryptoError::NoKeyForOrg)?;

        let dec = encrypt_aes256(data, key.mac_key, key.key)?;
        Ok(dec)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::{EncryptionSettings, SymmetricCryptoKey};
    use crate::crypto::{Decryptable, Encryptable};

    #[test]
    fn test_symmetric_crypto_key() {
        let key = SymmetricCryptoKey::generate("test");
        let key2 = SymmetricCryptoKey::from_str(&key.to_base64()).unwrap();
        assert_eq!(key.key, key2.key);
        assert_eq!(key.mac_key, key2.mac_key);

        let key = "UY4B5N4DA4UisCNClgZtRr6VLy9ZF5BXXC7cDZRqourKi4ghEMgISbCsubvgCkHf5DZctQjVot11/vVvN9NNHQ==";
        let key2 = SymmetricCryptoKey::from_str(key).unwrap();
        assert_eq!(key, key2.to_base64());
    }

    #[test]
    fn test_encryption_settings() {
        let key = SymmetricCryptoKey::generate("test");
        let settings = EncryptionSettings::new_single_key(key);

        let test_string = "encrypted_test_string".to_string();
        let cipher = test_string.clone().encrypt(&settings, &None).unwrap();

        let decrypted_str = cipher.decrypt(&settings, &None).unwrap();
        assert_eq!(decrypted_str, test_string);
    }
}
