//! Errors that can occur when using this SDK

use std::{borrow::Cow, fmt::Debug};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Internal error: {0}")]
    Internal(Cow<'static, str>),
}

// Ensure that the error messages implement Send and Sync
#[cfg(test)]
const _: () = {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    fn assert_all() {
        assert_send::<Error>();
        assert_sync::<Error>();
    }
};

pub type Result<T, E = Error> = std::result::Result<T, E>;
