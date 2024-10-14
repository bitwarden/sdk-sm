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
pub struct ProviderOrganizationResponseModel {
    #[serde(rename = "object", skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<uuid::Uuid>,
    #[serde(rename = "providerId", skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<uuid::Uuid>,
    #[serde(rename = "organizationId", skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<uuid::Uuid>,
    #[serde(rename = "key", skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(rename = "settings", skip_serializing_if = "Option::is_none")]
    pub settings: Option<String>,
    #[serde(rename = "creationDate", skip_serializing_if = "Option::is_none")]
    pub creation_date: Option<String>,
    #[serde(rename = "revisionDate", skip_serializing_if = "Option::is_none")]
    pub revision_date: Option<String>,
    #[serde(rename = "userCount", skip_serializing_if = "Option::is_none")]
    pub user_count: Option<i32>,
    #[serde(rename = "seats", skip_serializing_if = "Option::is_none")]
    pub seats: Option<i32>,
    #[serde(rename = "occupiedSeats", skip_serializing_if = "Option::is_none")]
    pub occupied_seats: Option<i32>,
    #[serde(rename = "remainingSeats", skip_serializing_if = "Option::is_none")]
    pub remaining_seats: Option<i32>,
    #[serde(rename = "plan", skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
}

impl ProviderOrganizationResponseModel {
    pub fn new() -> ProviderOrganizationResponseModel {
        ProviderOrganizationResponseModel {
            object: None,
            id: None,
            provider_id: None,
            organization_id: None,
            key: None,
            settings: None,
            creation_date: None,
            revision_date: None,
            user_count: None,
            seats: None,
            occupied_seats: None,
            remaining_seats: None,
            plan: None,
        }
    }
}
