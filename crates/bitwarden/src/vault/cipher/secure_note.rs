use bitwarden_api_api::models::CipherSecureNoteModel;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{
    crypto::{purpose, KeyDecryptable, KeyEncryptable, SymmetricCryptoKey},
    error::{Error, Result},
};

#[derive(Clone, Copy, Serialize_repr, Deserialize_repr, Debug, JsonSchema)]
#[repr(u8)]
#[cfg_attr(feature = "mobile", derive(uniffi::Enum))]
pub enum SecureNoteType {
    Generic = 0,
}

#[derive(Clone, Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "mobile", derive(uniffi::Record))]
pub struct SecureNote {
    r#type: SecureNoteType,
}

#[derive(Clone, Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "mobile", derive(uniffi::Record))]
pub struct SecureNoteView {
    r#type: SecureNoteType,
}

impl
    KeyEncryptable<
        SymmetricCryptoKey<purpose::CipherEncryption>,
        purpose::CipherEncryption,
        SecureNote,
    > for SecureNoteView
{
    fn encrypt_with_key(
        self,
        _key: &SymmetricCryptoKey<purpose::CipherEncryption>,
    ) -> Result<SecureNote> {
        Ok(SecureNote {
            r#type: self.r#type,
        })
    }
}

impl
    KeyDecryptable<
        SymmetricCryptoKey<purpose::CipherEncryption>,
        purpose::CipherEncryption,
        SecureNoteView,
    > for SecureNote
{
    fn decrypt_with_key(
        &self,
        _key: &SymmetricCryptoKey<purpose::CipherEncryption>,
    ) -> Result<SecureNoteView> {
        Ok(SecureNoteView {
            r#type: self.r#type,
        })
    }
}

impl TryFrom<CipherSecureNoteModel> for SecureNote {
    type Error = Error;

    fn try_from(model: CipherSecureNoteModel) -> Result<Self> {
        Ok(Self {
            r#type: model.r#type.map(|t| t.into()).ok_or(Error::MissingFields)?,
        })
    }
}

impl From<bitwarden_api_api::models::SecureNoteType> for SecureNoteType {
    fn from(model: bitwarden_api_api::models::SecureNoteType) -> Self {
        match model {
            bitwarden_api_api::models::SecureNoteType::Variant0 => SecureNoteType::Generic,
        }
    }
}
