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
pub enum CipherRepromptType {
    None = 0,
    Password = 1,

    #[serde(other)]
    UnknownValue = -1337,
}

impl ToString for CipherRepromptType {
    fn to_string(&self) -> String {
        match self {
            Self::None => String::from("0"),
            Self::Password => String::from("1"),
            Self::UnknownValue => String::from("UnknownValue"),
        }
    }
}

impl Default for CipherRepromptType {
    fn default() -> CipherRepromptType {
        Self::None
    }
}
