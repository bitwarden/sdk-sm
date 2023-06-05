/*
 * Bitwarden Internal API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: latest
 *
 * Generated by: https://openapi-generator.tech
 */

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct UpdateDomainsRequestModel {
    #[serde(rename = "equivalentDomains", skip_serializing_if = "Option::is_none")]
    pub equivalent_domains: Option<Vec<Vec<String>>>,
    #[serde(
        rename = "excludedGlobalEquivalentDomains",
        skip_serializing_if = "Option::is_none"
    )]
    pub excluded_global_equivalent_domains: Option<Vec<crate::models::GlobalEquivalentDomainsType>>,
}

impl UpdateDomainsRequestModel {
    pub fn new() -> UpdateDomainsRequestModel {
        UpdateDomainsRequestModel {
            equivalent_domains: None,
            excluded_global_equivalent_domains: None,
        }
    }
}
