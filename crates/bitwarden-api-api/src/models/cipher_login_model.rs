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
pub struct CipherLoginModel {
    #[serde(rename = "uri", skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(rename = "uris", skip_serializing_if = "Option::is_none")]
    pub uris: Option<Vec<models::CipherLoginUriModel>>,
    #[serde(rename = "username", skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(rename = "password", skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(
        rename = "passwordRevisionDate",
        skip_serializing_if = "Option::is_none"
    )]
    pub password_revision_date: Option<String>,
    #[serde(rename = "totp", skip_serializing_if = "Option::is_none")]
    pub totp: Option<String>,
    #[serde(rename = "autofillOnPageLoad", skip_serializing_if = "Option::is_none")]
    pub autofill_on_page_load: Option<bool>,
    #[serde(rename = "fido2Credentials", skip_serializing_if = "Option::is_none")]
    pub fido2_credentials: Option<Vec<models::CipherFido2CredentialModel>>,
}

impl CipherLoginModel {
    pub fn new() -> CipherLoginModel {
        CipherLoginModel {
            uri: None,
            uris: None,
            username: None,
            password: None,
            password_revision_date: None,
            totp: None,
            autofill_on_page_load: None,
            fido2_credentials: None,
        }
    }
}
