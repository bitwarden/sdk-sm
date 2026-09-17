use std::io::{self, Write};

use bitwarden::{
    OrganizationId,
    secrets_manager::{
        SecretsManagerClient,
        projects::{
            ProjectCreateRequest, ProjectGetRequest, ProjectPutRequest, ProjectsDeleteRequest,
            ProjectsListRequest,
        },
    },
};
use color_eyre::eyre::Result;
use uuid::Uuid;

use crate::{
    ProjectCommand,
    error::{self, Op, Target},
    render::{OutputSettings, serialize_response, write_stdout},
};

pub(crate) async fn process_command(
    command: ProjectCommand,
    client: SecretsManagerClient,
    organization_id: OrganizationId,
    output_settings: OutputSettings,
) -> Result<()> {
    match command {
        ProjectCommand::List => list(client, organization_id, output_settings).await,
        ProjectCommand::Get { project_id } => get(client, project_id, output_settings).await,
        ProjectCommand::Create { name } => {
            create(client, organization_id, name, output_settings).await
        }
        ProjectCommand::Edit { project_id, name } => {
            edit(client, organization_id, project_id, name, output_settings).await
        }
        ProjectCommand::Delete { project_ids } => delete(client, project_ids).await,
    }
}

pub(crate) async fn list(
    client: SecretsManagerClient,
    organization_id: OrganizationId,
    output_settings: OutputSettings,
) -> Result<()> {
    let projects = client
        .projects()
        .list(&ProjectsListRequest {
            organization_id: organization_id.into(),
        })
        .await
        .map_err(|e| error::sm_error(e, Target::None, Op::Read))?
        .data;
    serialize_response(projects, output_settings);

    Ok(())
}

pub(crate) async fn get(
    client: SecretsManagerClient,
    project_id: Uuid,
    output_settings: OutputSettings,
) -> Result<()> {
    let project = client
        .projects()
        .get(&ProjectGetRequest { id: project_id })
        .await
        .map_err(|e| error::sm_error(e, Target::Project(project_id), Op::Read))?;
    serialize_response(project, output_settings);

    Ok(())
}

pub(crate) async fn create(
    client: SecretsManagerClient,
    organization_id: OrganizationId,
    name: String,
    output_settings: OutputSettings,
) -> Result<()> {
    let project = client
        .projects()
        .create(&ProjectCreateRequest {
            organization_id: organization_id.into(),
            name,
        })
        .await
        .map_err(|e| error::sm_error(e, Target::None, Op::Write))?;
    serialize_response(project, output_settings);

    Ok(())
}

pub(crate) async fn edit(
    client: SecretsManagerClient,
    organization_id: OrganizationId,
    project_id: Uuid,
    name: String,
    output_settings: OutputSettings,
) -> Result<()> {
    let project = client
        .projects()
        .update(&ProjectPutRequest {
            id: project_id,
            organization_id: organization_id.into(),
            name,
        })
        .await
        .map_err(|e| error::sm_error(e, Target::Project(project_id), Op::Write))?;
    serialize_response(project, output_settings);

    Ok(())
}

pub(crate) async fn delete(client: SecretsManagerClient, project_ids: Vec<Uuid>) -> Result<()> {
    let count = project_ids.len();
    let target = match project_ids.as_slice() {
        [id] => Target::Project(*id),
        _ => Target::Projects,
    };

    let result = client
        .projects()
        .delete(ProjectsDeleteRequest { ids: project_ids })
        .await
        .map_err(|e| error::sm_error(e, target, Op::Write))?;

    let projects_failed: Vec<(Uuid, String)> = result
        .data
        .into_iter()
        .filter_map(|r| r.error.map(|e| (r.id, e)))
        .collect();

    match projects_failed.len() {
        2.. => eprintln!("{} projects had errors:", projects_failed.len()),
        1 => eprintln!("{} project had an error:", projects_failed.len()),
        _ => (),
    }

    for project in &projects_failed {
        eprintln!("{}: {}", project.0, project.1);
    }

    // The server may omit successful IDs from `data`, so count them from the request.
    let deleted_projects = count.saturating_sub(projects_failed.len());
    let summary = match deleted_projects {
        2.. => format!("{deleted_projects} projects deleted successfully.\n"),
        1 => "1 project deleted successfully.\n".to_string(),
        _ => String::new(),
    };

    if projects_failed.is_empty() {
        write_stdout(summary);
        return Ok(());
    }

    // `write_stdout` exits on a closed or failing stdout, which would hide the failed deletes.
    _ = io::stdout().write_all(summary.as_bytes());
    Err(error::partial_delete(projects_failed.len(), count, "project").into())
}
