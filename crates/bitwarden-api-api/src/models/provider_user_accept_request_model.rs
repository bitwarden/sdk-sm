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
pub struct ProviderUserAcceptRequestModel {
    #[serde(rename = "token")]
    pub token: String,
}

impl ProviderUserAcceptRequestModel {
    pub fn new(token: String) -> ProviderUserAcceptRequestModel {
        ProviderUserAcceptRequestModel { token }
    }
}
