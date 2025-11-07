use std::error::Error;

use schemars::{JsonSchema, json_schema};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Response<T: Serialize + JsonSchema> {
    /// Whether or not the SDK request succeeded.
    pub success: bool,
    /// A message for any error that may occur. Populated if `success` is false.
    pub error_message: Option<String>,
    /// The response data. Populated if `success` is true.
    pub data: Option<T>,
}

impl<T: Serialize + JsonSchema> JsonSchema for Response<T> {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        format!("Response_for_{}", T::schema_name()).into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        json_schema!({
          "type": "object",
          "required": [
            "success"
          ],
          "properties": {
            "success": {
              "description": "Whether or not the SDK request succeeded.",
              "type": "boolean"
            },
            "errorMessage": {
              "description": "A message for any error that may occur. Populated if `success` is false.",
              "type": [
                "string",
                "null"
              ]
            },
            "data": {
              "description": "The response data. Populated if `success` is true.",
              "anyOf": [
                generator.subschema_for::<T>(),
                {
                  "type": "null"
                }
              ]
            }
          },
          "additionalProperties": false
        })
    }
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
