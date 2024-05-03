use reqwest::Url;

use super::{
    get_string_name_from_enum,
    types::{
        get_stub_selected_credential, AuthenticatorAssertionResponse,
        AuthenticatorAttestationResponse, ClientData, ClientExtensionResults, CredPropsResult,
    },
    Fido2Authenticator, PublicKeyCredentialAuthenticatorAssertionResponse,
    PublicKeyCredentialAuthenticatorAttestationResponse,
};
use crate::error::{Error, Result};

pub struct Fido2Client<'a> {
    pub(crate) authenticator: Fido2Authenticator<'a>,
}

impl<'a> Fido2Client<'a> {
    pub async fn register(
        &mut self,
        origin: String,
        request: String,
        client_data: ClientData,
    ) -> Result<PublicKeyCredentialAuthenticatorAttestationResponse> {
        let mut client =
            passkey::client::Client::new(self.authenticator.get_passkey_authenticator());

        // TODO(Fido2): Handle this error
        let origin = Url::parse(&origin).expect("Invalid URL");

        let request: passkey::types::webauthn::CredentialCreationOptions =
            serde_json::from_str(&request)?;

        let rp_id = request.public_key.rp.id.clone();

        let result = client.register(&origin, request, client_data).await?;

        let enc = self.authenticator.client.get_encryption_settings()?;
        let key = enc.get_key(&None).ok_or(Error::VaultLocked)?;

        Ok(PublicKeyCredentialAuthenticatorAttestationResponse {
            id: result.id,
            raw_id: result.raw_id.into(),
            ty: get_string_name_from_enum(result.ty)?,
            authenticator_attachment: result
                .authenticator_attachment
                .map(get_string_name_from_enum)
                .transpose()?,
            client_extension_results: ClientExtensionResults {
                cred_props: result.client_extension_results.cred_props.map(Into::into),
            },
            response: AuthenticatorAttestationResponse {
                client_data_json: result.response.client_data_json.into(),
                authenticator_data: result.response.authenticator_data.into(),
                public_key: result.response.public_key.map(|x| x.into()),
                public_key_algorithm: result.response.public_key_algorithm,
                attestation_object: result.response.attestation_object.into(),
                transports: if rp_id.unwrap_or_default() == "https://google.com" {
                    Some(vec!["internal".to_string(), "usb".to_string()])
                } else {
                    Some(vec!["internal".to_string()])
                },
            },
            selected_credential: get_stub_selected_credential(key)?,
        })
    }

    pub async fn authenticate(
        &mut self,
        origin: String,
        request: String,
        client_data: ClientData,
    ) -> Result<PublicKeyCredentialAuthenticatorAssertionResponse> {
        let mut client =
            passkey::client::Client::new(self.authenticator.get_passkey_authenticator());

        // TODO(Fido2): Handle this error
        let origin = Url::parse(&origin).expect("Invalid URL");

        let request: passkey::types::webauthn::CredentialRequestOptions =
            serde_json::from_str(&request)?;

        let result = client.authenticate(&origin, request, client_data).await?;

        let enc = self.authenticator.client.get_encryption_settings()?;
        let key = enc.get_key(&None).ok_or(Error::VaultLocked)?;

        Ok(PublicKeyCredentialAuthenticatorAssertionResponse {
            id: result.id,
            raw_id: result.raw_id.into(),
            ty: get_string_name_from_enum(result.ty)?,

            // TODO(Fido2): Had to change this type to Option, should we just use a default?
            authenticator_attachment: result
                .authenticator_attachment
                .map(get_string_name_from_enum)
                .transpose()?,
            client_extension_results: ClientExtensionResults {
                cred_props: result
                    .client_extension_results
                    .cred_props
                    .map(|c| CredPropsResult {
                        rk: c.discoverable,
                        authenticator_display_name: c.authenticator_display_name,
                    }),
            },
            response: AuthenticatorAssertionResponse {
                client_data_json: result.response.client_data_json.into(),
                authenticator_data: result.response.authenticator_data.into(),
                signature: result.response.signature.into(),
                user_handle: result.response.user_handle.unwrap_or_default().into(),
            },
            selected_credential: get_stub_selected_credential(key)?,
        })
    }
}
