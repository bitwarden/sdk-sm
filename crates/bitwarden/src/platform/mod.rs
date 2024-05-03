pub mod client_platform;
mod domain;
#[cfg(feature = "mobile")]
pub mod fido2;
mod generate_fingerprint;
mod get_user_api_key;
mod secret_verification_request;
mod sync;

#[cfg(feature = "mobile")]
pub use fido2::{ClientFido2, Fido2Authenticator, Fido2Client};
pub use generate_fingerprint::{FingerprintRequest, FingerprintResponse};
pub(crate) use get_user_api_key::get_user_api_key;
pub use get_user_api_key::UserApiKeyResponse;
pub use secret_verification_request::SecretVerificationRequest;
pub(crate) use sync::sync;
pub use sync::{SyncRequest, SyncResponse};
