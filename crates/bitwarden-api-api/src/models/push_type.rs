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
pub enum PushType {
    SyncCipherUpdate = 0,
    SyncCipherCreate = 1,
    SyncLoginDelete = 2,
    SyncFolderDelete = 3,
    SyncCiphers = 4,
    SyncVault = 5,
    SyncOrgKeys = 6,
    SyncFolderCreate = 7,
    SyncFolderUpdate = 8,
    SyncCipherDelete = 9,
    SyncSettings = 10,
    LogOut = 11,
    SyncSendCreate = 12,
    SyncSendUpdate = 13,
    SyncSendDelete = 14,
    AuthRequest = 15,
    AuthRequestResponse = 16,

    #[serde(other)]
    UnknownValue = -1337,
}

impl ToString for PushType {
    fn to_string(&self) -> String {
        match self {
            Self::SyncCipherUpdate => String::from("0"),
            Self::SyncCipherCreate => String::from("1"),
            Self::SyncLoginDelete => String::from("2"),
            Self::SyncFolderDelete => String::from("3"),
            Self::SyncCiphers => String::from("4"),
            Self::SyncVault => String::from("5"),
            Self::SyncOrgKeys => String::from("6"),
            Self::SyncFolderCreate => String::from("7"),
            Self::SyncFolderUpdate => String::from("8"),
            Self::SyncCipherDelete => String::from("9"),
            Self::SyncSettings => String::from("10"),
            Self::LogOut => String::from("11"),
            Self::SyncSendCreate => String::from("12"),
            Self::SyncSendUpdate => String::from("13"),
            Self::SyncSendDelete => String::from("14"),
            Self::AuthRequest => String::from("15"),
            Self::AuthRequestResponse => String::from("16"),
            Self::UnknownValue => String::from("UnknownValue"),
        }
    }
}

impl Default for PushType {
    fn default() -> PushType {
        Self::SyncCipherUpdate
    }
}
