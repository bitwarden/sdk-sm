use std::io::{self, Write};

use bitwarden::{
    OrganizationId,
    secrets_manager::{
        SecretsManagerClient,
        secrets::{
            SecretCreateRequest, SecretGetRequest, SecretIdentifiersByProjectRequest,
            SecretIdentifiersRequest, SecretPutRequest, SecretResponse, SecretsDeleteRequest,
            SecretsGetRequest,
        },
    },
};
use color_eyre::eyre::Result;
use uuid::Uuid;

use crate::{
    SecretCommand,
    error::{self, Op, Target},
    render::{OutputSettings, serialize_response, write_stdout},
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
    client: SecretsManagerClient,
    organization_id: OrganizationId,
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
    client: SecretsManagerClient,
    organization_id: OrganizationId,
    project_id: Option<Uuid>,
    output_settings: OutputSettings,
) -> Result<()> {
    let res = if let Some(project_id) = project_id {
        client
            .secrets()
            .list_by_project(&SecretIdentifiersByProjectRequest { project_id })
            .await
            .map_err(|e| error::sm_error(e, Target::Project(project_id), Op::Read))?
    } else {
        client
            .secrets()
            .list(&SecretIdentifiersRequest {
                organization_id: organization_id.into(),
            })
            .await
            .map_err(|e| error::sm_error(e, Target::None, Op::Read))?
    };

    if res.data.is_empty() {
        serialize_response(Vec::<SecretResponse>::new(), output_settings);
        return Ok(());
    }

    let secret_ids = res.data.into_iter().map(|e| e.id).collect();
    let secrets = client
        .secrets()
        .get_by_ids(SecretsGetRequest { ids: secret_ids })
        .await
        .map_err(|e| error::sm_error(e, Target::None, Op::Read))?
        .data;
    serialize_response(secrets, output_settings);

    Ok(())
}

pub(crate) async fn get(
    client: SecretsManagerClient,
    secret_id: Uuid,
    output_settings: OutputSettings,
) -> Result<()> {
    let secret = client
        .secrets()
        .get(&SecretGetRequest { id: secret_id })
        .await
        .map_err(|e| error::sm_error(e, Target::Secret(secret_id), Op::Read))?;
    serialize_response(secret, output_settings);

    Ok(())
}

pub(crate) async fn create(
    client: SecretsManagerClient,
    organization_id: OrganizationId,
    secret: SecretCreateCommandModel,
    output_settings: OutputSettings,
) -> Result<()> {
    let project_id = secret.project_id;
    let secret = client
        .secrets()
        .create(&SecretCreateRequest {
            organization_id: organization_id.into(),
            key: secret.key,
            value: secret.value,
            note: secret.note.unwrap_or_default(),
            project_ids: Some(vec![project_id]),
        })
        .await
        .map_err(|e| error::sm_error(e, Target::Project(project_id), Op::Write))?;
    serialize_response(secret, output_settings);

    Ok(())
}

pub(crate) async fn edit(
    client: SecretsManagerClient,
    organization_id: OrganizationId,
    secret: SecretEditCommandModel,
    output_settings: OutputSettings,
) -> Result<()> {
    let old_secret = client
        .secrets()
        .get(&SecretGetRequest { id: secret.id })
        .await
        .map_err(|e| error::sm_error(e, Target::Secret(secret.id), Op::Read))?;

    let value_changed = secret
        .value
        .as_ref()
        .is_some_and(|v| v != &old_secret.value);

    let target = match secret.project_id {
        Some(project_id) => Target::SecretOrProject(secret.id, project_id),
        None => Target::Secret(secret.id),
    };
    let new_secret = client
        .secrets()
        .update(&SecretPutRequest {
            id: secret.id,
            organization_id: organization_id.into(),
            key: secret.key.unwrap_or(old_secret.key),
            value: secret.value.unwrap_or(old_secret.value.clone()),
            note: secret.note.unwrap_or(old_secret.note),
            project_ids: secret
                .project_id
                .or(old_secret.project_id)
                .map(|id| vec![id]),
            value_changed,
        })
        .await
        .map_err(|e| error::sm_error(e, target, Op::Write))?;
    serialize_response(new_secret, output_settings);

    Ok(())
}

pub(crate) async fn delete(client: SecretsManagerClient, secret_ids: Vec<Uuid>) -> Result<()> {
    let count = secret_ids.len();
    let target = match secret_ids.as_slice() {
        [id] => Target::Secret(*id),
        _ => Target::Secrets,
    };

    let result = client
        .secrets()
        .delete(SecretsDeleteRequest { ids: secret_ids })
        .await
        .map_err(|e| error::sm_error(e, target, Op::Write))?;

    let secrets_failed: Vec<(Uuid, String)> = result
        .data
        .into_iter()
        .filter_map(|r| r.error.map(|e| (r.id, error::strip_controls(&e))))
        .collect();

    match secrets_failed.len() {
        2.. => eprintln!("{} secrets had errors:", secrets_failed.len()),
        1 => eprintln!("{} secret had an error:", secrets_failed.len()),
        _ => (),
    }

    for secret in &secrets_failed {
        eprintln!("{}: {}", secret.0, secret.1);
    }

    // The server may omit successful IDs from `data`, so count them from the request.
    let deleted_secrets = count.saturating_sub(secrets_failed.len());
    let summary = match deleted_secrets {
        2.. => format!("{deleted_secrets} secrets deleted successfully.\n"),
        1 => "1 secret deleted successfully.\n".to_string(),
        _ => String::new(),
    };

    if secrets_failed.is_empty() {
        write_stdout(summary);
        return Ok(());
    }

    // `write_stdout` exits on a closed or failing stdout, which would hide the failed deletes.
    _ = io::stdout().write_all(summary.as_bytes());
    Err(error::partial_delete(secrets_failed.len(), count, "secret").into())
}
