# frozen_string_literal: true

require 'json'

module BitwardenSDKSecrets
  class SecretsClient
    def initialize(command_runner)
      @command_runner = command_runner
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

    def sync(organization_id, last_synced_date)
      command = create_command(
        sync: SecretsSyncRequest.new(organization_id: organization_id, last_synced_date: last_synced_date)
      )
      response = run_command(command)

      secrets_response = ResponseForSecretsSyncResponse.from_json!(response).to_dynamic

      if secrets_response.key?('success') && secrets_response['success'] == true &&
         secrets_response.key?('data')
        return secrets_response['data']
      end

      error_response(secrets_response)
    end

    def create(organization_id, key, value, note, project_ids)
      command = create_command(
        create: SecretCreateRequest.new(
          key: key, note: note, organization_id: organization_id, project_ids: project_ids, value: value
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

    def list(organization_id)
      command = create_command(list: SecretIdentifiersRequest.new(organization_id: organization_id))
      response = run_command(command)

      secrets_response = ResponseForSecretIdentifiersResponse.from_json!(response).to_dynamic

      if secrets_response.key?('success') && secrets_response['success'] == true &&
        secrets_response.key?('data') && secrets_response['data'].key?('data')
        return secrets_response['data']['data']
      end

      error_response(secrets_response)
    end

    def update(organization_id, id, key, value, note, project_ids)
      begin
        old_secret = get(id)
      rescue => e
        raise BitwardenError, "Cannot update secret: failed to fetch current value for version history: #{e.message}"
      end

      value_changed = value != old_secret['value']

      command = create_command(
        update: SecretPutRequest.new(
          id: id, key: key, note: note, organization_id: organization_id, project_ids: project_ids, value: value, value_changed: value_changed
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
