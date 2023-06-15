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
pub enum SecureNoteType {
    Variant0 = 0,
}

impl ToString for SecureNoteType {
    fn to_string(&self) -> String {
        match self {
            Self::Variant0 => String::from("0"),
        }
    }
}

impl Default for SecureNoteType {
    fn default() -> SecureNoteType {
        Self::Variant0
    }
}
