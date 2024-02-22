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
pub struct AttachmentUploadDataResponseModel {
    #[serde(rename = "object", skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    #[serde(rename = "attachmentId", skip_serializing_if = "Option::is_none")]
    pub attachment_id: Option<String>,
    #[serde(rename = "url", skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(rename = "fileUploadType", skip_serializing_if = "Option::is_none")]
    pub file_upload_type: Option<crate::models::FileUploadType>,
    #[serde(rename = "cipherResponse", skip_serializing_if = "Option::is_none")]
    pub cipher_response: Option<Box<crate::models::CipherResponseModel>>,
    #[serde(rename = "cipherMiniResponse", skip_serializing_if = "Option::is_none")]
    pub cipher_mini_response: Option<Box<crate::models::CipherMiniResponseModel>>,
}

impl AttachmentUploadDataResponseModel {
    pub fn new() -> AttachmentUploadDataResponseModel {
        AttachmentUploadDataResponseModel {
            object: None,
            attachment_id: None,
            url: None,
            file_upload_type: None,
            cipher_response: None,
            cipher_mini_response: None,
        }
    }
}
