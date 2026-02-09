<?php

namespace Bitwarden\Sdk;

use Bitwarden\Sdk\Schemas\Command;
use Bitwarden\Sdk\Schemas\SecretCreateRequest;
use Bitwarden\Sdk\Schemas\SecretGetRequest;
use Bitwarden\Sdk\Schemas\SecretIdentifiersRequest;
use Bitwarden\Sdk\Schemas\SecretPutRequest;
use Bitwarden\Sdk\Schemas\SecretsCommand;
use Bitwarden\Sdk\Schemas\SecretsDeleteRequest;
use Bitwarden\Sdk\Schemas\SecretsGetRequest;
use Bitwarden\Sdk\Schemas\SecretsSyncRequest;
use Exception;
use stdClass;

class SecretsClient
{
    private CommandRunner $commandRunner;

    public function __construct(CommandRunner $commandRunner)
    {
        $this->commandRunner = $commandRunner;
    }

    /**
     * @throws Exception
     */
    public function get(string $secret_id): stdClass
    {
        $secret_get_request = new SecretGetRequest($secret_id);
        $secret_get_request->validate();
        $secrets_command = new SecretsCommand(get: $secret_get_request, getByIds: null, create: null, list: null,
            update: null, delete: null, sync: null);
        return $this->run_secret_command($secrets_command);
    }

    /**
     * @throws Exception
     */
    public function get_by_ids(array $secret_ids): stdClass
    {
        $project_get_by_ids_request = new SecretsGetRequest($secret_ids);
        $project_get_by_ids_request->validate();
        $secrets_command = new SecretsCommand(get: null, getByIds: $project_get_by_ids_request, create: null, list: null,
            update: null, delete: null, sync: null);
        return $this->run_secret_command($secrets_command);
    }

    /**
     * @throws Exception
     */
    public function list(string $organization_id): stdClass
    {
        $secrets_list_request = new SecretIdentifiersRequest($organization_id);
        $secrets_list_request->validate();
        $secrets_command = new SecretsCommand(get: null, getByIds: null, create: null, list: $secrets_list_request,
            update: null, delete: null, sync: null);
        return $this->run_secret_command($secrets_command);
    }

    /**
     * @throws Exception
     */
    public function create(string $organization_id, string $key, string $value, string $note, array $project_ids): stdClass
    {
        $secrets_create_request = new SecretCreateRequest(key: $key, note: $note, organizationId: $organization_id,
            projectIds: $project_ids, value: $value);
        $secrets_create_request->validate();
        $secrets_command = new SecretsCommand(get: null, getByIds: null, create: $secrets_create_request, list: null,
            update: null, delete: null, sync: null);
        return $this->run_secret_command($secrets_command);
    }

    /**
     * @throws Exception
     */
    public function update(string $organization_id, string $id, string $key, string $value, string $note, array $project_ids): stdClass
    {
        $secrets_put_request = new SecretPutRequest(id: $id, key: $key, note: $note, organizationId: $organization_id,
            projectIds: $project_ids, value: $value);
        $secrets_put_request->validate();
        $secrets_command = new SecretsCommand(get: null, getByIds: null, create: null, list: null,
            update: $secrets_put_request, delete: null, sync: null);
        return $this->run_secret_command($secrets_command);
    }

    /**
     * @throws Exception
     */
    public function delete(array $secrets_ids): stdClass
    {
        $secrets_delete_request = new SecretsDeleteRequest($secrets_ids);
        $secrets_delete_request->validate();
        $secrets_command = new SecretsCommand(get: null, getByIds: null, create: null, list: null,
            update: null, delete: $secrets_delete_request, sync: null);
        return $this->run_secret_command($secrets_command);
    }

    /**
     * @throws Exception
     */
    public function sync(string $organization_id, ?string $last_synced_date): stdClass
    {
        if (empty($last_synced_date)) {
            $last_synced_date = "1970-01-01T00:00:00.000Z";
        }

        $secrets_sync_request = new SecretsSyncRequest(lastSyncedDate: $last_synced_date, organizationId: $organization_id);
        $secrets_sync_request->validate();
        $secrets_command = new SecretsCommand(get: null, getByIds: null, create: null, list: null,
            update: null, delete: null, sync: $secrets_sync_request);
        return $this->run_secret_command($secrets_command);
    }

    /**
     * @throws Exception
     */
    public function run_secret_command($secretsCommand): stdClass
    {
        $command = new Command(loginAccessToken: null, secrets: $secretsCommand, projects: null, generators: null, debug: null);
        return $this->commandRunner->run($command);
    }
}
