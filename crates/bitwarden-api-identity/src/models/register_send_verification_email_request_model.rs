/*
 * Bitwarden Identity
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: v1
 *
 * Generated by: https://openapi-generator.tech
 */

use serde::{Deserialize, Serialize};

use crate::models;

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegisterSendVerificationEmailRequestModel {
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "email")]
    pub email: Option<String>,
    #[serde(
        rename = "receiveMarketingEmails",
        skip_serializing_if = "Option::is_none"
    )]
    pub receive_marketing_emails: Option<bool>,
}

impl RegisterSendVerificationEmailRequestModel {
    pub fn new(email: Option<String>) -> RegisterSendVerificationEmailRequestModel {
        RegisterSendVerificationEmailRequestModel {
            name: None,
            email,
            receive_marketing_emails: None,
        }
    }
}
