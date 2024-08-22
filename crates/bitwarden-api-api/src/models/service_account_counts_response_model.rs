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
pub struct ServiceAccountCountsResponseModel {
    #[serde(rename = "object", skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    #[serde(rename = "projects", skip_serializing_if = "Option::is_none")]
    pub projects: Option<i32>,
    #[serde(rename = "people", skip_serializing_if = "Option::is_none")]
    pub people: Option<i32>,
    #[serde(rename = "accessTokens", skip_serializing_if = "Option::is_none")]
    pub access_tokens: Option<i32>,
}

impl ServiceAccountCountsResponseModel {
    pub fn new() -> ServiceAccountCountsResponseModel {
        ServiceAccountCountsResponseModel {
            object: None,
            projects: None,
            people: None,
            access_tokens: None,
        }
    }
}
