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
pub struct SetKeyConnectorKeyRequestModel {
    #[serde(rename = "key")]
    pub key: String,
    #[serde(rename = "keys")]
    pub keys: Box<crate::models::KeysRequestModel>,
    #[serde(rename = "kdf")]
    pub kdf: crate::models::KdfType,
    #[serde(rename = "kdfIterations")]
    pub kdf_iterations: i32,
    #[serde(rename = "kdfMemory", skip_serializing_if = "Option::is_none")]
    pub kdf_memory: Option<i32>,
    #[serde(rename = "kdfParallelism", skip_serializing_if = "Option::is_none")]
    pub kdf_parallelism: Option<i32>,
    #[serde(rename = "orgIdentifier")]
    pub org_identifier: String,
}

impl SetKeyConnectorKeyRequestModel {
    pub fn new(
        key: String,
        keys: crate::models::KeysRequestModel,
        kdf: crate::models::KdfType,
        kdf_iterations: i32,
        org_identifier: String,
    ) -> SetKeyConnectorKeyRequestModel {
        SetKeyConnectorKeyRequestModel {
            key,
            keys: Box::new(keys),
            kdf,
            kdf_iterations,
            kdf_memory: None,
            kdf_parallelism: None,
            org_identifier,
        }
    }
}
