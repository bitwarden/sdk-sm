/*
 * Bitwarden Internal API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: latest
 *
 * Generated by: https://openapi-generator.tech
 */

use serde::{Deserialize, Serialize};

use crate::models;

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct CipherWithIdRequestModel {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<models::CipherType>,
    #[serde(rename = "organizationId", skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
    #[serde(rename = "folderId", skip_serializing_if = "Option::is_none")]
    pub folder_id: Option<String>,
    #[serde(rename = "favorite", skip_serializing_if = "Option::is_none")]
    pub favorite: Option<bool>,
    #[serde(rename = "reprompt", skip_serializing_if = "Option::is_none")]
    pub reprompt: Option<models::CipherRepromptType>,
    #[serde(rename = "key", skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "notes", skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(rename = "fields", skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<models::CipherFieldModel>>,
    #[serde(rename = "passwordHistory", skip_serializing_if = "Option::is_none")]
    pub password_history: Option<Vec<models::CipherPasswordHistoryModel>>,
    #[serde(rename = "attachments", skip_serializing_if = "Option::is_none")]
    pub attachments: Option<std::collections::HashMap<String, String>>,
    #[serde(rename = "attachments2", skip_serializing_if = "Option::is_none")]
    pub attachments2: Option<std::collections::HashMap<String, models::CipherAttachmentModel>>,
    #[serde(rename = "login", skip_serializing_if = "Option::is_none")]
    pub login: Option<Box<models::CipherLoginModel>>,
    #[serde(rename = "card", skip_serializing_if = "Option::is_none")]
    pub card: Option<Box<models::CipherCardModel>>,
    #[serde(rename = "identity", skip_serializing_if = "Option::is_none")]
    pub identity: Option<Box<models::CipherIdentityModel>>,
    #[serde(rename = "secureNote", skip_serializing_if = "Option::is_none")]
    pub secure_note: Option<Box<models::CipherSecureNoteModel>>,
    #[serde(
        rename = "lastKnownRevisionDate",
        skip_serializing_if = "Option::is_none"
    )]
    pub last_known_revision_date: Option<String>,
    #[serde(rename = "id")]
    pub id: uuid::Uuid,
}

impl CipherWithIdRequestModel {
    pub fn new(name: String, id: uuid::Uuid) -> CipherWithIdRequestModel {
        CipherWithIdRequestModel {
            r#type: None,
            organization_id: None,
            folder_id: None,
            favorite: None,
            reprompt: None,
            key: None,
            name,
            notes: None,
            fields: None,
            password_history: None,
            attachments: None,
            attachments2: None,
            login: None,
            card: None,
            identity: None,
            secure_note: None,
            last_known_revision_date: None,
            id,
        }
    }
}
