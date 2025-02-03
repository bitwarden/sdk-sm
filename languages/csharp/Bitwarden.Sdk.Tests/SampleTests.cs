namespace Bitwarden.Sdk.Tests;

public class SampleTests
{
    [SecretsManagerFact]
    public async Task RunSample_Works()
    {
        // Get environment variables
        var identityUrl = Environment.GetEnvironmentVariable("IDENTITY_URL")!;
        var apiUrl = Environment.GetEnvironmentVariable("API_URL")!;
        var organizationId = Guid.Parse(Environment.GetEnvironmentVariable("ORGANIZATION_ID")!);
        var accessToken = Environment.GetEnvironmentVariable("ACCESS_TOKEN")!;
        var stateFile = Environment.GetEnvironmentVariable("STATE_FILE")!;

        // Create the SDK Client
        using var bitwardenClient = new BitwardenClient(new BitwardenSettings
        {
            ApiUrl = apiUrl,
            IdentityUrl = identityUrl
        });

        // Authenticate
        await bitwardenClient.Auth.LoginAccessTokenAsync(accessToken, stateFile);

        // Projects Create, Update, & Get
        var projectResponse = await bitwardenClient.Projects.CreateAsync(organizationId, "NewTestProject");
        projectResponse = await bitwardenClient.Projects.UpdateAsync(organizationId, projectResponse.Id, "NewTestProject Renamed");
        projectResponse = await bitwardenClient.Projects.GetAsync(projectResponse.Id);

        Assert.Equal("NewTestProject Renamed", projectResponse.Name);

        var projectList = await bitwardenClient.Projects.ListAsync(organizationId);

        Assert.True(projectList.Data.Count() >= 1);

        // Secrets list
        var secretsList = await bitwardenClient.Secrets.ListAsync(organizationId);

        // Secrets Create, Update, Get
        var secretResponse = await bitwardenClient.Secrets.CreateAsync(organizationId, "New Secret", "the secret value", "the secret note", new[] { projectResponse.Id });
        secretResponse = await bitwardenClient.Secrets.UpdateAsync(organizationId, secretResponse.Id, "New Secret Name", "the secret value", "the secret note", new[] { projectResponse.Id });
        secretResponse = await bitwardenClient.Secrets.GetAsync(secretResponse.Id);

        Assert.Equal("New Secret Name", secretResponse.Key);

        // Secrets GetByIds
        var secretsResponse = await bitwardenClient.Secrets.GetByIdsAsync(new[] { secretResponse.Id });

        // Secrets Sync
        var syncResponse = await bitwardenClient.Secrets.SyncAsync(organizationId, null);

        // Secrets & Projects Delete
        await bitwardenClient.Secrets.DeleteAsync(new[] { secretResponse.Id });
        await bitwardenClient.Projects.DeleteAsync(new[] { projectResponse.Id });
    }
}
