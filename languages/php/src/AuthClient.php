<?php

namespace Bitwarden\Sdk;

use Bitwarden\Sdk\Schemas\AccessTokenLoginRequest;
use Bitwarden\Sdk\Schemas\Command;
use Exception;

class AuthClient
{
    private CommandRunner $commandRunner;

    public function __construct(CommandRunner $commandRunner)
    {
        $this->commandRunner = $commandRunner;
    }

    /**
     * @throws Exception
     */
    public function login_access_token(string $access_token, ?string $state_file): void
    {
        $access_token_request = new AccessTokenLoginRequest($access_token, $state_file);
        $command = new Command(loginAccessToken: $access_token_request,
            secrets: null, projects: null, generators: null, debug: null);
        try {
            $result = $this->commandRunner->run($command);
            if (!isset($result->authenticated) || !$result->authenticated) {
                throw new Exception("Unauthorized");
            }
        } catch (Exception $exception) {
            throw new Exception("Authorization error: " . $exception->getMessage());
        }
    }
}
