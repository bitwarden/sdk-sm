/*
 * Bitwarden Internal API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: latest
 *
 * Generated by: https://openapi-generator.tech
 */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GroupResponseModel {
    #[serde(rename = "object", skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<uuid::Uuid>,
    #[serde(rename = "organizationId", skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<uuid::Uuid>,
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "accessAll", skip_serializing_if = "Option::is_none")]
    pub access_all: Option<bool>,
    #[serde(rename = "externalId", skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
}

impl GroupResponseModel {
    pub fn new() -> GroupResponseModel {
        GroupResponseModel {
            object: None,
            id: None,
            organization_id: None,
            name: None,
            access_all: None,
            external_id: None,
        }
    }
}
