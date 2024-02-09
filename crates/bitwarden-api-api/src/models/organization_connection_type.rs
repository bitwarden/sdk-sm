/*
 * Bitwarden Internal API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: latest
 *
 * Generated by: https://openapi-generator.tech
 */

///
#[repr(i64)]
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize_repr, Deserialize_repr,
)]
pub enum OrganizationConnectionType {
    CloudBillingSync = 1,
    Scim = 2,

    #[serde(other)]
    UnknownValue = -1337,
}

impl ToString for OrganizationConnectionType {
    fn to_string(&self) -> String {
        match self {
            Self::CloudBillingSync => String::from("1"),
            Self::Scim => String::from("2"),
            Self::UnknownValue => String::from("UnknownValue"),
        }
    }
}

impl Default for OrganizationConnectionType {
    fn default() -> OrganizationConnectionType {
        Self::CloudBillingSync
    }
}
