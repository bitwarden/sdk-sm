# Bitwarden Secrets Manager SDK

A Rust client SDK to interact with the
[Bitwarden Secrets Manager](https://bitwarden.com/products/secrets-manager/). This is a beta release
and might be missing some functionality.

## Usage

```toml
[dependencies]
bitwarden = { "*", features = ["secrets"] }
```

## Minimum Supported Rust Version

Rust **1.82** or higher.

## Example

```rust
use bitwarden::{
    auth::login::AccessTokenLoginRequest,
    error::Result,
    secrets_manager::{secrets::SecretIdentifiersRequest, ClientSecretsExt},
    Client, ClientSettings, DeviceType,
};
use uuid::Uuid;

async fn test() -> Result<()> {
    // Use the default values
    let mut client = Client::new(None);

    // Or set your own values
    let settings = ClientSettings {
        identity_url: "https://identity.bitwarden.com".to_string(),
        api_url: "https://api.bitwarden.com".to_string(),
        user_agent: "Bitwarden Rust-SDK".to_string(),
        device_type: DeviceType::SDK,
    };
    let mut client = Client::new(Some(settings));

    // Before we operate, we need to authenticate with a token
    let token = AccessTokenLoginRequest {
        access_token: String::from(""),
        state_file: None,
    };
    client.auth().login_access_token(&token).await.unwrap();

    let org_id = SecretIdentifiersRequest {
        organization_id: Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
    };
    println!(
        "Stored secrets: {:#?}",
        client.secrets().list(&org_id).await.unwrap()
    );
    Ok(())
}
```
