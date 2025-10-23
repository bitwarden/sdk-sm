<?php

namespace Bitwarden\Sdk;

use Bitwarden\Sdk\Schemas\Command;
use Bitwarden\Sdk\Schemas\ProjectCreateRequest;
use Bitwarden\Sdk\Schemas\ProjectGetRequest;
use Bitwarden\Sdk\Schemas\ProjectPutRequest;
use Bitwarden\Sdk\Schemas\ProjectsCommand;
use Bitwarden\Sdk\Schemas\ProjectsDeleteRequest;
use Bitwarden\Sdk\Schemas\ProjectsListRequest;
use Exception;
use stdClass;

class ProjectsClient
{
    private CommandRunner $commandRunner;

    public function __construct(CommandRunner $commandRunner)
    {
        $this->commandRunner = $commandRunner;
    }

    /**
     * @throws Exception
     */
    public function get(string $project_id): stdClass
    {
        $project_get_request = new ProjectGetRequest($project_id);
        $project_get_request->validate();
        $project_command = new ProjectsCommand(get: $project_get_request, create: null, list: null, update: null,
            delete: null);
        return $this->run_project_command($project_command);
    }

    /**
     * @throws Exception
     */
    public function list(string $organization_id): stdClass
    {
        $project_list_request = new ProjectsListRequest($organization_id);
        $project_list_request->validate();
        $project_command = new ProjectsCommand(get: null, create: null, list: $project_list_request, update: null,
            delete: null);
        return $this->run_project_command($project_command);
    }

    /**
     * @throws Exception
     */
    public function create(string $organization_id, string $project_name): stdClass
    {
        $project_create_request = new ProjectCreateRequest(name: $project_name, organizationId: $organization_id);
        $project_create_request->validate();
        $project_command = new ProjectsCommand(get: null, create: $project_create_request, list: null, update: null,
            delete: null);
        return $this->run_project_command($project_command);
    }

    /**
     * @throws Exception
     */
    public function update(string $organization_id, string $project_id, string $project_name): stdClass
    {
        $project_put_request = new ProjectPutRequest(id: $project_id, name: $project_name,
            organizationId: $organization_id);
        $project_put_request->validate();
        $project_command = new ProjectsCommand(get: null, create: null, list: null, update: $project_put_request,
            delete: null);
        return $this->run_project_command($project_command);
    }

    /**
     * @throws Exception
     */
    public function delete(array $ids): stdClass
    {
        $projects_delete_request = new ProjectsDeleteRequest($ids);
        $projects_delete_request->validate();
        $project_command = new ProjectsCommand(get: null, create: null, list: null, update: null,
            delete: $projects_delete_request);
        return $this->run_project_command($project_command);
    }

    /**
     * @throws Exception
     */
    public function run_project_command($projectCommand): stdClass
    {
        $command = new Command(loginAccessToken: null, secrets: null, projects: $projectCommand, generators: null, debug: null);
        return $this->commandRunner->run($command);
    }
}
