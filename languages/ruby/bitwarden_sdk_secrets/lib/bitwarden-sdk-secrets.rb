# frozen_string_literal: true

require 'json'
require 'dry-types'

require_relative 'schemas'
require_relative 'extended_schemas/schemas'
require_relative 'command_runner'
require_relative 'bitwarden_lib'
require_relative 'bitwarden_error'
require_relative 'projects'
require_relative 'secrets'
require_relative 'auth'

module BitwardenSDKSecrets
  class BitwardenSettings
    attr_accessor :api_url, :identity_url

    def initialize(api_url, identity_url)
      # if api_url.nil? || identity_url.nil?
      #   raise ArgumentError, "api_url and identity_url cannot be nil"
      # end

      @api_url = api_url
      @identity_url = identity_url
    end
  end

  class BitwardenClient
    attr_reader :bitwarden, :projects, :secrets, :auth

    def initialize(bitwarden_settings)
      client_settings = ClientSettings.new(
        api_url: bitwarden_settings.api_url,
        identity_url: bitwarden_settings.identity_url,
        user_agent: 'Bitwarden RUBY-SDK',
        device_type: nil,
        bitwarden_client_version: nil
      )

      @bitwarden = BitwardenLib
      @handle = @bitwarden.init(client_settings.to_dynamic.compact.to_json)
      @command_runner = CommandRunner.new(@bitwarden, @handle)
      @projects = ProjectsClient.new(@command_runner, self)
      @secrets = SecretsClient.new(@command_runner, self)
      @auth = AuthClient.new(@command_runner)
    end

    def get_access_token_organization
      @bitwarden.get_access_token_organization(@handle)
    end

    def free_mem
      @bitwarden.free_mem(@handle)
    end
  end
end
