use bitwarden_api_api::models::{
    BaseSecretResponseModel, BaseSecretResponseModelListResponseModel, SecretResponseModel,
};
use bitwarden_core::{client::encryption_settings::EncryptionSettings, error::Result, require};
use bitwarden_crypto::{CryptoError, EncString, KeyDecryptable};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SecretResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub project_id: Option<Uuid>,

    pub key: String,
    pub value: String,
    pub note: String,

    pub creation_date: DateTime<Utc>,
    pub revision_date: DateTime<Utc>,
}

impl SecretResponse {
    pub(crate) fn process_response(
        response: SecretResponseModel,
        enc: &EncryptionSettings,
    ) -> Result<SecretResponse> {
        let base = BaseSecretResponseModel {
            object: response.object,
            id: response.id,
            organization_id: response.organization_id,
            key: response.key,
            value: response.value,
            note: response.note,
            creation_date: response.creation_date,
            revision_date: response.revision_date,
            projects: response.projects,
        };
        Self::process_base_response(base, enc)
    }
    pub(crate) fn process_base_response(
        response: BaseSecretResponseModel,
        enc: &EncryptionSettings,
    ) -> Result<SecretResponse> {
        let org_id = response.organization_id;
        let enc_key = enc.get_key(&org_id).ok_or(CryptoError::MissingKey)?;

        let key = require!(response.key)
            .parse::<EncString>()?
            .decrypt_with_key(enc_key)?;
        let value = require!(response.value)
            .parse::<EncString>()?
            .decrypt_with_key(enc_key)?;
        let note = require!(response.note)
            .parse::<EncString>()?
            .decrypt_with_key(enc_key)?;

        let project = response
            .projects
            .and_then(|p| p.into_iter().next())
            .and_then(|p| p.id);

        Ok(SecretResponse {
            id: require!(response.id),
            organization_id: require!(org_id),
            project_id: project,
            key,
            value,
            note,

            creation_date: require!(response.creation_date).parse()?,
            revision_date: require!(response.revision_date).parse()?,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SecretsResponse {
    pub data: Vec<SecretResponse>,
}

impl SecretsResponse {
    pub(crate) fn process_response(
        response: BaseSecretResponseModelListResponseModel,
        enc: &EncryptionSettings,
    ) -> Result<SecretsResponse> {
        Ok(SecretsResponse {
            data: response
                .data
                .unwrap_or_default()
                .into_iter()
                .map(|r| SecretResponse::process_base_response(r, enc))
                .collect::<Result<_, _>>()?,
        })
    }
}
