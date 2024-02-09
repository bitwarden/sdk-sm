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
pub enum CipherType {
    Login = 1,
    SecureNote = 2,
    Card = 3,
    Identity = 4,

    #[serde(other)]
    UnknownValue = -1337,
}

impl ToString for CipherType {
    fn to_string(&self) -> String {
        match self {
            Self::Login => String::from("1"),
            Self::SecureNote => String::from("2"),
            Self::Card => String::from("3"),
            Self::Identity => String::from("4"),
            Self::UnknownValue => String::from("UnknownValue"),
        }
    }
}

impl Default for CipherType {
    fn default() -> CipherType {
        Self::Login
    }
}
