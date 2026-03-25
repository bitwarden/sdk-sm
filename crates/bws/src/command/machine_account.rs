use bitwarden::{
    OrganizationId,
    secrets_manager::{
        SecretsManagerClient,
        access_policies::{GetGrantedPoliciesRequest, PutGrantedPoliciesRequest},
    },
};
use color_eyre::eyre::Result;
use uuid::Uuid;

use crate::{
    MachineAccountCommand,
    command::access_policy_helpers::parse_granted_project_flags,
    render::{OutputSettings, serialize_response},
};

pub(crate) async fn process_command(
    command: MachineAccountCommand,
    client: SecretsManagerClient,
    _organization_id: OrganizationId,
    output_settings: OutputSettings,
) -> Result<()> {
    match command {
        MachineAccountCommand::Access { machine_account_id } => {
            access(client, machine_account_id, output_settings).await
        }
        MachineAccountCommand::SetAccess {
            machine_account_id,
            project,
        } => set_access(client, machine_account_id, project, output_settings).await,
    }
}

async fn access(
    client: SecretsManagerClient,
    machine_account_id: Uuid,
    output_settings: OutputSettings,
) -> Result<()> {
    let response = client
        .access_policies()
        .get_granted_policies(&GetGrantedPoliciesRequest {
            service_account_id: machine_account_id,
        })
        .await?;
    serialize_response(response, output_settings);
    Ok(())
}

async fn set_access(
    client: SecretsManagerClient,
    machine_account_id: Uuid,
    project: Vec<String>,
    output_settings: OutputSettings,
) -> Result<()> {
    let projects = parse_granted_project_flags(&project)?;
    let response = client
        .access_policies()
        .put_granted_policies(&PutGrantedPoliciesRequest {
            service_account_id: machine_account_id,
            projects,
        })
        .await?;
    serialize_response(response, output_settings);
    Ok(())
}
