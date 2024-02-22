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
pub struct UserLicense {
    #[serde(rename = "licenseKey", skip_serializing_if = "Option::is_none")]
    pub license_key: Option<String>,
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<uuid::Uuid>,
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "email", skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(rename = "premium", skip_serializing_if = "Option::is_none")]
    pub premium: Option<bool>,
    #[serde(rename = "maxStorageGb", skip_serializing_if = "Option::is_none")]
    pub max_storage_gb: Option<i32>,
    #[serde(rename = "version", skip_serializing_if = "Option::is_none")]
    pub version: Option<i32>,
    #[serde(rename = "issued", skip_serializing_if = "Option::is_none")]
    pub issued: Option<String>,
    #[serde(rename = "refresh", skip_serializing_if = "Option::is_none")]
    pub refresh: Option<String>,
    #[serde(rename = "expires", skip_serializing_if = "Option::is_none")]
    pub expires: Option<String>,
    #[serde(rename = "trial", skip_serializing_if = "Option::is_none")]
    pub trial: Option<bool>,
    #[serde(rename = "licenseType", skip_serializing_if = "Option::is_none")]
    pub license_type: Option<crate::models::LicenseType>,
    #[serde(rename = "hash", skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(rename = "signature", skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl UserLicense {
    pub fn new() -> UserLicense {
        UserLicense {
            license_key: None,
            id: None,
            name: None,
            email: None,
            premium: None,
            max_storage_gb: None,
            version: None,
            issued: None,
            refresh: None,
            expires: None,
            trial: None,
            license_type: None,
            hash: None,
            signature: None,
        }
    }
}
