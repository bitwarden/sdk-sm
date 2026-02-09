use std::error::Error;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "Response_for_{T}")]
pub struct Response<T: Serialize + JsonSchema> {
    /// Whether or not the SDK request succeeded.
    pub success: bool,
    /// A message for any error that may occur. Populated if `success` is false.
    pub error_message: Option<String>,
    /// The response data. Populated if `success` is true.
    pub data: Option<T>,
}

impl<T: Serialize + JsonSchema> Response<T> {
    pub fn new<TErr: Error>(response: Result<T, TErr>) -> Self {
        match response {
            Ok(data) => Self {
                success: true,
                error_message: None,
                data: Some(data),
            },
            Err(err) => Self {
                success: false,
                error_message: Some(err.to_string()),
                data: None,
            },
        }
    }
}

impl Response<()> {
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            error_message: Some(message),
            data: None,
        }
    }
}

pub(crate) trait ResponseIntoString {
    fn into_string(self) -> String;
}

impl<T: Serialize + JsonSchema, E: Error> ResponseIntoString for Result<T, E> {
    fn into_string(self) -> String {
        Response::new(self).into_string()
    }
}

impl<T: Serialize + JsonSchema> ResponseIntoString for Response<T> {
    fn into_string(self) -> String {
        match serde_json::to_string(&self) {
            Ok(ser) => ser,
            Err(e) => {
                let error = Response::error(format!("Failed to serialize Response: {}", e));
                serde_json::to_string(&error).expect("Serialize should be infallible")
            }
        }
    }
}
