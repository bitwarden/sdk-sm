# frozen_string_literal: true

require 'json'

module BitwardenSDKSecrets
  class SecretsClient
    def initialize(command_runner, bitwarden_client)
      @command_runner = command_runner
      @bitwarden_client = bitwarden_client
    end

    def get(id)
      command = create_command(get: SecretGetRequest.new(id: id))
      response = run_command(command)

      secrets_response = ResponseForSecretResponse.from_json!(response).to_dynamic

      if secrets_response.key?('success') && secrets_response['success'] == true &&
         secrets_response.key?('data')
        return secrets_response['data']
      end

      error_response(secrets_response)
    end

    def get_by_ids(ids)
      command = create_command(get_by_ids: SecretsGetRequest.new(ids: ids))
      response = run_command(command)

      secrets_response = ResponseForSecretIdentifiersResponse.from_json!(response).to_dynamic

      if secrets_response.key?('success') && secrets_response['success'] == true &&
        secrets_response.key?('data') && secrets_response['data'].key?('data')
        return secrets_response['data']['data']
      end

      error_response(secrets_response)
    end

    def sync(last_synced_date)
      org_id = @bitwarden_client.get_access_token_organization
      raise BitwardenError, 'Could not get organization id with access token' if org_id.nil? || org_id.empty?

      command = create_command(
        sync: SecretsSyncRequest.new(organization_id: org_id, last_synced_date: last_synced_date)
      )
      response = run_command(command)

      secrets_response = ResponseForSecretsSyncResponse.from_json!(response).to_dynamic

      if secrets_response.key?('success') && secrets_response['success'] == true &&
         secrets_response.key?('data')
        return secrets_response['data']
      end

      error_response(secrets_response)
    end

    def create(key, value, note, project_ids)
      org_id = @bitwarden_client.get_access_token_organization
      raise BitwardenError, 'Could not get organization id with access token' if org_id.nil? || org_id.empty?

      command = create_command(
        create: SecretCreateRequest.new(
          key: key, note: note, organization_id: org_id, project_ids: project_ids, value: value
        )
      )
      response = run_command(command)

      secrets_response = ResponseForSecretResponse.from_json!(response).to_dynamic

      if secrets_response.key?('success') && secrets_response['success'] == true &&
         secrets_response.key?('data')
        return secrets_response['data']
      end

      error_response(secrets_response)
    end

    def list
      org_id = @bitwarden_client.get_access_token_organization
      raise BitwardenError, 'Could not get organization id with access token' if org_id.nil? || org_id.empty?

      command = create_command(list: SecretIdentifiersRequest.new(organization_id: org_id))
      response = run_command(command)

      secrets_response = ResponseForSecretIdentifiersResponse.from_json!(response).to_dynamic

      if secrets_response.key?('success') && secrets_response['success'] == true &&
        secrets_response.key?('data') && secrets_response['data'].key?('data')
        return secrets_response['data']['data']
      end

      error_response(secrets_response)
    end

    def update(id, key, value, note, project_ids)
      org_id = @bitwarden_client.get_access_token_organization
      raise BitwardenError, 'Could not get organization id with access token' if org_id.nil? || org_id.empty?

      command = create_command(
        update: SecretPutRequest.new(
          id: id, key: key, note: note, organization_id: org_id, project_ids: project_ids, value: value
        )
      )
      response = run_command(command)

      secrets_response = ResponseForSecretResponse.from_json!(response).to_dynamic

      if secrets_response.key?('success') && secrets_response['success'] == true &&
         secrets_response.key?('data')
        return secrets_response['data']
      end

      error_response(secrets_response)
    end

    def delete(ids)
      command = create_command(delete: SecretsDeleteRequest.new(ids: ids))
      response = run_command(command)

      secrets_response = ResponseForSecretsDeleteResponse.from_json!(response).to_dynamic

      if secrets_response.key?('success') && secrets_response['success'] == true &&
         secrets_response.key?('data') && secrets_response['data'].key?('data')
        return secrets_response['data']['data']
      end

      error_response(secrets_response)
    end

    private

    def error_response(response)
      if response['errorMessage']
        raise BitwardenError, response['errorMessage'] if response.key?('errorMessage')
      else
        raise BitwardenError, 'Error while getting response'
      end
    end

    def create_command(commands)
      SelectiveCommand.new(secrets: SelectiveSecretsCommand.new(commands))
    end

    def run_command(command)
      response = @command_runner.run(command)
      raise BitwardenError, 'Error getting response' if response.nil?

      response
    end
  end
end
