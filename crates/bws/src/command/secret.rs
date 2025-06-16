use bitwarden::{
    secrets_manager::{
        secrets::{
            SecretCreateRequest, SecretGetRequest, SecretIdentifiersByProjectRequest,
            SecretIdentifiersRequest, SecretPutRequest, SecretResponse, SecretsDeleteRequest,
            SecretsGetRequest,
        },
        ClientSecretsExt,
    },
    Client,
};
use color_eyre::eyre::{bail, Result};
use uuid::Uuid;

use crate::{
    render::{serialize_response, OutputSettings},
    SecretCommand,
};

#[derive(Debug)]
pub(crate) struct SecretCreateCommandModel {
    pub(crate) key: String,
    pub(crate) value: String,
    pub(crate) note: Option<String>,
    pub(crate) project_id: Uuid,
}

#[derive(Debug)]
pub(crate) struct SecretEditCommandModel {
    pub(crate) id: Uuid,
    pub(crate) key: Option<String>,
    pub(crate) value: Option<String>,
    pub(crate) note: Option<String>,
    pub(crate) project_id: Option<Uuid>,
}

pub(crate) async fn process_command(
    command: SecretCommand,
    client: Client,
    organization_id: Uuid,
    output_settings: OutputSettings,
) -> Result<()> {
    match command {
        SecretCommand::List { project_id } => {
            list(client, organization_id, project_id, output_settings).await
        }
        SecretCommand::Get { secret_id } => get(client, secret_id, output_settings).await,
        SecretCommand::Create {
            key,
            value,
            note,
            project_id,
        } => {
            create(
                client,
                organization_id,
                SecretCreateCommandModel {
                    key,
                    value,
                    note,
                    project_id,
                },
                output_settings,
            )
            .await
        }
        SecretCommand::Edit {
            secret_id,
            key,
            value,
            note,
            project_id,
        } => {
            edit(
                client,
                organization_id,
                SecretEditCommandModel {
                    id: secret_id,
                    key,
                    value,
                    note,
                    project_id,
                },
                output_settings,
            )
            .await
        }
        SecretCommand::Delete { secret_ids } => delete(client, secret_ids).await,
    }
}

pub(crate) async fn list(
    client: Client,
    organization_id: Uuid,
    project_id: Option<Uuid>,
    output_settings: OutputSettings,
) -> Result<()> {
    let res = if let Some(project_id) = project_id {
        client
            .secrets()
            .list_by_project(&SecretIdentifiersByProjectRequest { project_id })
            .await?
    } else {
        client
            .secrets()
            .list(&SecretIdentifiersRequest { organization_id })
            .await?
    };

    if res.data.is_empty() {
        serialize_response(Vec::<SecretResponse>::new(), output_settings);
        return Ok(());
    }

    let secret_ids = res.data.into_iter().map(|e| e.id).collect();
    let secrets = client
        .secrets()
        .get_by_ids(SecretsGetRequest { ids: secret_ids })
        .await?
        .data;
    serialize_response(secrets, output_settings);

    Ok(())
}

pub(crate) async fn get(
    client: Client,
    secret_id: Uuid,
    output_settings: OutputSettings,
) -> Result<()> {
    let secret = client
        .secrets()
        .get(&SecretGetRequest { id: secret_id })
        .await?;
    serialize_response(secret, output_settings);

    Ok(())
}

pub(crate) async fn create(
    client: Client,
    organization_id: Uuid,
    secret: SecretCreateCommandModel,
    output_settings: OutputSettings,
) -> Result<()> {
    let secret = client
        .secrets()
        .create(&SecretCreateRequest {
            organization_id,
            key: secret.key,
            value: secret.value,
            note: secret.note.unwrap_or_default(),
            project_ids: Some(vec![secret.project_id]),
        })
        .await?;
    serialize_response(secret, output_settings);

    Ok(())
}

pub(crate) async fn edit(
    client: Client,
    organization_id: Uuid,
    secret: SecretEditCommandModel,
    output_settings: OutputSettings,
) -> Result<()> {
    let old_secret = client
        .secrets()
        .get(&SecretGetRequest { id: secret.id })
        .await?;

    let new_secret = client
        .secrets()
        .update(&SecretPutRequest {
            id: secret.id,
            organization_id,
            key: secret.key.unwrap_or(old_secret.key),
            value: secret.value.unwrap_or(old_secret.value),
            note: secret.note.unwrap_or(old_secret.note),
            project_ids: secret
                .project_id
                .or(old_secret.project_id)
                .map(|id| vec![id]),
        })
        .await?;
    serialize_response(new_secret, output_settings);

    Ok(())
}

pub(crate) async fn delete(client: Client, secret_ids: Vec<Uuid>) -> Result<()> {
    let count = secret_ids.len();

    let result = client
        .secrets()
        .delete(SecretsDeleteRequest { ids: secret_ids })
        .await?;

    let secrets_failed: Vec<(Uuid, String)> = result
        .data
        .into_iter()
        .filter_map(|r| r.error.map(|e| (r.id, e)))
        .collect();
    let deleted_secrets = count - secrets_failed.len();

    match deleted_secrets {
        2.. => println!("{} secrets deleted successfully.", deleted_secrets),
        1 => println!("{} secret deleted successfully.", deleted_secrets),
        _ => (),
    }

    match secrets_failed.len() {
        2.. => eprintln!("{} secrets had errors:", secrets_failed.len()),
        1 => eprintln!("{} secret had an error:", secrets_failed.len()),
        _ => (),
    }

    for secret in &secrets_failed {
        eprintln!("{}: {}", secret.0, secret.1);
    }

    if !secrets_failed.is_empty() {
        bail!("Errors when attempting to delete secrets.");
    }

    Ok(())
}
