using System.Text.Json;

namespace Bitwarden.Sdk;

public sealed partial class BitwardenClient : IDisposable
{
    private readonly CommandRunner _commandRunner;
    private readonly BitwardenSafeHandle _handle;

    public BitwardenClient(BitwardenSettings? settings = null)
    {
        var clientSettings = new ClientSettings
        {
            ApiUrl = settings?.ApiUrl!,
            IdentityUrl = settings?.IdentityUrl!,
            UserAgent = "Bitwarden DOTNET-SDK"
        };

        _handle = BitwardenLibrary.Init(clientSettings.ToJson());
        _commandRunner = new CommandRunner(_handle);
        Projects = new ProjectsClient(_commandRunner);
        Secrets = new SecretsClient(_commandRunner);
        Auth = new AuthClient(_commandRunner);
    }

    public ProjectsClient Projects { get; }

    public SecretsClient Secrets { get; }

    public AuthClient Auth { get; set; }

    public void Dispose() => _handle.Dispose();
}
