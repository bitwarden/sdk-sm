# frozen_string_literal: true

require_relative 'bitwarden_error'

module BitwardenSDKSecrets
  class ProjectsClient
    def initialize(command_runner, bitwarden_client)
      @command_runner = command_runner
      @bitwarden_client = bitwarden_client
    end

    def create(project_name)
      org_id = @bitwarden_client.get_access_token_organization
      raise BitwardenError, 'Could not get organization id with access token' if org_id.nil? || org_id.empty?

      project_create_request = ProjectCreateRequest.new(
        create_name: project_name,
        organization_id: org_id
      )
      command = create_command(
        create: project_create_request
      )
      response = parse_response(command)

      projects_response = ResponseForProjectResponse.from_json!(response).to_dynamic

      if projects_response.key?('success') && projects_response['success'] == true &&
        projects_response.key?('data')
        return projects_response['data']
      end

      error_response(projects_response)
    end

    def get(project_id)
      project_get_request = ProjectGetRequest.new(id: project_id)
      command = create_command(get: project_get_request)
      response = parse_response(command)

      projects_response = ResponseForProjectResponse.from_json!(response).to_dynamic

      if projects_response.key?('success') && projects_response['success'] == true &&
        projects_response.key?('data')
        return projects_response['data']
      end

      error_response(projects_response)
    end

    def list
      org_id = @bitwarden_client.get_access_token_organization
      raise BitwardenError, 'Could not get organization id with access token' if org_id.nil? || org_id.empty?

      project_list_request = ProjectsListRequest.new(organization_id: org_id)
      command = create_command(list: project_list_request)
      response = parse_response(command)

      projects_response = ResponseForProjectsResponse.from_json!(response).to_dynamic

      if projects_response.key?('success') && projects_response['success'] == true &&
         projects_response.key?('data') && projects_response['data'].key?('data')
        return projects_response['data']['data']
      end

      error_response(projects_response)
    end

    def update(id, project_put_request_name)
      org_id = @bitwarden_client.get_access_token_organization
      raise BitwardenError, 'Could not get organization id with access token' if org_id.nil? || org_id.empty?

      project_put_request = ProjectPutRequest.new(
        id: id,
        update_name: project_put_request_name,
        organization_id: org_id
      )
      command = create_command(
        update: project_put_request
      )
      response = parse_response(command)

      projects_response = ResponseForProjectResponse.from_json!(response).to_dynamic

      if projects_response.key?('success') && projects_response['success'] == true &&
         projects_response.key?('data')
        return projects_response['data']
      end

      error_response(projects_response)
    end

    def delete(ids)
      project_delete_request = ProjectsDeleteRequest.new(ids: ids)
      command = create_command(delete: project_delete_request)
      response = parse_response(command)

      projects_response = ResponseForProjectsDeleteResponse.from_json!(response).to_dynamic

      if projects_response.key?('success') && projects_response['success'] == true &&
         projects_response.key?('data') && projects_response['data'].key?('data')
        return projects_response['data']['data']
      end

      error_response(projects_response)
    end

    private

    def error_response(response)
      raise BitwardenError, response['errorMessage'] if response.key?('errorMessage')

      raise BitwardenError, 'Error while getting response'
    end

    def create_command(commands)
      SelectiveCommand.new(projects: SelectiveProjectsCommand.new(commands))
    end

    def parse_response(command)
      response = @command_runner.run(command)
      raise BitwardenError, 'Error getting response' if response.nil?

      response
    end
  end
end
