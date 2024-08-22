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
pub struct OrganizationAuthRequestUpdateManyRequestModel {
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<uuid::Uuid>,
    #[serde(rename = "key", skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(rename = "approved", skip_serializing_if = "Option::is_none")]
    pub approved: Option<bool>,
}

impl OrganizationAuthRequestUpdateManyRequestModel {
    pub fn new() -> OrganizationAuthRequestUpdateManyRequestModel {
        OrganizationAuthRequestUpdateManyRequestModel {
            id: None,
            key: None,
            approved: None,
        }
    }
}
