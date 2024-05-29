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
pub struct OrganizationUserUpdateRequestModel {
    #[serde(rename = "type")]
    pub r#type: crate::models::OrganizationUserType,
    #[serde(rename = "accessAll", skip_serializing_if = "Option::is_none")]
    pub access_all: Option<bool>,
    #[serde(
        rename = "accessSecretsManager",
        skip_serializing_if = "Option::is_none"
    )]
    pub access_secrets_manager: Option<bool>,
    #[serde(rename = "permissions", skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Box<crate::models::Permissions>>,
    #[serde(rename = "collections", skip_serializing_if = "Option::is_none")]
    pub collections: Option<Vec<crate::models::SelectionReadOnlyRequestModel>>,
    #[serde(rename = "groups", skip_serializing_if = "Option::is_none")]
    pub groups: Option<Vec<uuid::Uuid>>,
}

impl OrganizationUserUpdateRequestModel {
    pub fn new(r#type: crate::models::OrganizationUserType) -> OrganizationUserUpdateRequestModel {
        OrganizationUserUpdateRequestModel {
            r#type,
            access_all: None,
            access_secrets_manager: None,
            permissions: None,
            collections: None,
            groups: None,
        }
    }
}
