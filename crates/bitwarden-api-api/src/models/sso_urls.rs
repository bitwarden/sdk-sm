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
pub struct SsoUrls {
    #[serde(rename = "callbackPath", skip_serializing_if = "Option::is_none")]
    pub callback_path: Option<String>,
    #[serde(
        rename = "signedOutCallbackPath",
        skip_serializing_if = "Option::is_none"
    )]
    pub signed_out_callback_path: Option<String>,
    #[serde(rename = "spEntityId", skip_serializing_if = "Option::is_none")]
    pub sp_entity_id: Option<String>,
    #[serde(rename = "spEntityIdStatic", skip_serializing_if = "Option::is_none")]
    pub sp_entity_id_static: Option<String>,
    #[serde(rename = "spMetadataUrl", skip_serializing_if = "Option::is_none")]
    pub sp_metadata_url: Option<String>,
    #[serde(rename = "spAcsUrl", skip_serializing_if = "Option::is_none")]
    pub sp_acs_url: Option<String>,
}

impl SsoUrls {
    pub fn new() -> SsoUrls {
        SsoUrls {
            callback_path: None,
            signed_out_callback_path: None,
            sp_entity_id: None,
            sp_entity_id_static: None,
            sp_metadata_url: None,
            sp_acs_url: None,
        }
    }
}
