namespace Bitwarden.Sdk;

public class AuthClient
{
    private readonly CommandRunner _commandRunner;

    internal AuthClient(CommandRunner commandRunner)
    {
        _commandRunner = commandRunner;
    }

    public async Task LoginAccessTokenAsync(string accessToken, string stateFile = "", CancellationToken cancellationToken = default)
    {
        var command = new Command { LoginAccessToken = new AccessTokenLoginRequest { AccessToken = accessToken, StateFile = stateFile } };
        var response = await _commandRunner.RunCommandAsync<ResponseForApiKeyLoginResponse>(command, cancellationToken);
        if (response is not { Success: true })
        {
            throw new BitwardenAuthException(response != null ? response.ErrorMessage : "Login failed");
        }
    }
}
