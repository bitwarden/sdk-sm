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
pub struct FolderRequestModel {
    #[serde(rename = "name")]
    pub name: String,
}

impl FolderRequestModel {
    pub fn new(name: String) -> FolderRequestModel {
        FolderRequestModel { name }
    }
}
