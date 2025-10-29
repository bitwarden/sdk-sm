extern crate console_error_panic_hook;
use std::rc::Rc;

use argon2::{Algorithm, Argon2, Params, Version};
use bitwarden_json::client::Client as JsonClient;
use js_sys::Promise;
use log::{Level, set_max_level};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

#[wasm_bindgen]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

fn convert_level(level: LogLevel) -> Level {
    match level {
        LogLevel::Trace => Level::Trace,
        LogLevel::Debug => Level::Debug,
        LogLevel::Info => Level::Info,
        LogLevel::Warn => Level::Warn,
        LogLevel::Error => Level::Error,
    }
}

// Rc<...> is to avoid needing to take ownership of the Client during our async run_command
// function https://github.com/rustwasm/wasm-bindgen/issues/2195#issuecomment-799588401
#[wasm_bindgen]
pub struct BitwardenClient(Rc<JsonClient>);

#[wasm_bindgen]
impl BitwardenClient {
    #[wasm_bindgen(constructor)]
    pub fn new(settings_input: Option<String>, log_level: Option<LogLevel>) -> Self {
        console_error_panic_hook::set_once();
        let log_level = convert_level(log_level.unwrap_or(LogLevel::Info));
        if let Err(_e) = console_log::init_with_level(log_level) {
            set_max_level(log_level.to_level_filter())
        }

        Self(Rc::new(bitwarden_json::client::Client::new(settings_input)))
    }

    #[wasm_bindgen]
    pub fn run_command(&self, js_input: String) -> Promise {
        let rc = self.0.clone();
        future_to_promise(async move {
            let result = rc.run_command(&js_input).await;
            Ok(result.into())
        })
    }
}

#[wasm_bindgen]
pub fn argon2(
    password: &[u8],
    salt: &[u8],
    iterations: u32,
    memory: u32,
    parallelism: u32,
) -> Result<Vec<u8>, JsError> {
    let argon = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(
            memory * 1024, // Convert MiB to KiB
            iterations,
            parallelism,
            Some(32),
        )?,
    );

    let mut hash = [0u8; 32];
    argon.hash_password_into(password, salt, &mut hash)?;
    Ok(hash.to_vec())
}
