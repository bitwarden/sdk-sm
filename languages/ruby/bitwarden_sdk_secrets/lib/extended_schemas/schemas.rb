
module BitwardenSDKSecrets
class SelectiveCommand < Command
      attribute :login_access_token, AccessTokenLoginRequest.optional.default(nil)
      attribute :secrets,            SecretsCommand.optional.default(nil)
      attribute :projects,           ProjectsCommand.optional.default(nil)
      attribute :generators,         GeneratorsCommand.optional.default(nil)
      attribute :debug,              DebugCommand.optional.default(nil)

      def to_dynamic
        {
          "loginAccessToken" => login_access_token&.to_dynamic,
          "secrets"          => secrets&.to_dynamic,
          "projects"         => projects&.to_dynamic,
          "generators"       => generators&.to_dynamic,
          "debug"            => debug&.to_dynamic,
        }.compact
      end
    end

    class SelectiveProjectsCommand < ProjectsCommand
      attribute :get,    ProjectGetRequest.optional.default(nil)
      attribute :create, ProjectCreateRequest.optional.default(nil)
      attribute :list,   ProjectsListRequest.optional.default(nil)
      attribute :update, ProjectPutRequest.optional.default(nil)
      attribute :delete, ProjectsDeleteRequest.optional.default(nil)

      def to_dynamic
        {
          "get"    => get&.to_dynamic,
          "create" => create&.to_dynamic,
          "list"   => list&.to_dynamic,
          "update" => update&.to_dynamic,
          "delete" => delete&.to_dynamic,
        }.compact
      end
    end

    class SelectiveSecretsCommand < SecretsCommand
      attribute :get,        SecretGetRequest.optional.default(nil)
      attribute :get_by_ids, SecretsGetRequest.optional.default(nil)
      attribute :create,     SecretCreateRequest.optional.default(nil)
      attribute :list,       SecretIdentifiersRequest.optional.default(nil)
      attribute :update,     SecretPutRequest.optional.default(nil)
      attribute :delete,     SecretsDeleteRequest.optional.default(nil)
      attribute :sync,       SecretsSyncRequest.optional.default(nil)

      def to_dynamic
        {
          "get"      => get&.to_dynamic,
          "getByIds" => get_by_ids&.to_dynamic,
          "create"   => create&.to_dynamic,
          "list"     => list&.to_dynamic,
          "update"   => update&.to_dynamic,
          "delete"   => delete&.to_dynamic,
          "sync"     => sync&.to_dynamic,
        }.compact
      end
    end

    class SelectiveGeneratorsCommand < GeneratorsCommand
        attribute :generate_password,   PasswordGeneratorRequest.optional.default(nil)

        def to_dynamic
          {
            "generate_password"      => generate_password&.to_dynamic,
          }.compact
        end
      end
end
