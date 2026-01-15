use bitwarden_json::client::Client as JsonClient;
use pyo3::prelude::*;

#[pyclass]
pub struct BitwardenClient(tokio::runtime::Runtime, JsonClient);

#[pymethods]
impl BitwardenClient {
    #[new]
    #[pyo3(signature = (settings_string=None))]
    pub fn new(settings_string: Option<String>) -> Self {
        // This will only fail if another logger was already initialized, so we can ignore the
        // result
        let _ = pyo3_log::try_init();

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to build tokio runtime");

        Self(runtime, JsonClient::new(settings_string))
    }

    #[pyo3(text_signature = "($self, command_input)")]
    fn run_command(&self, command_input: String) -> String {
        self.0.block_on(self.1.run_command(&command_input))
    }

    #[pyo3(text_signature = "($self")]
    fn get_access_token_organization(&self) -> String {
        if let Some(organization_id) = self.1.get_access_token_organization() {
            organization_id.to_string()
        } else {
            "".to_string()
        }
    }
}
