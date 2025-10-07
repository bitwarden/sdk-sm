using Bitwarden.Sdk;

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

// Projects List
var projectsList = (await bitwardenClient.Projects.ListAsync(organizationId)).Data;
Console.WriteLine("A list of all projects:");
foreach (ProjectResponse pr in projectsList)
{
    Console.WriteLine("  Project: " + pr.Name);
}

Console.Write("Press enter to continue...");
Console.ReadLine();

// Projects Create, Update, & Get
Console.WriteLine("Creating and updating a project");
var projectResponse = await bitwardenClient.Projects.CreateAsync(organizationId, "NewTestProject");
projectResponse = await bitwardenClient.Projects.UpdateAsync(organizationId, projectResponse.Id, "NewTestProject Renamed");
projectResponse = await bitwardenClient.Projects.GetAsync(projectResponse.Id);
Console.WriteLine("Here is the project we created and updated:");
Console.WriteLine(projectResponse.Name);

Console.Write("Press enter to continue...");
Console.ReadLine();

// Secrets list
var secretsList = (await bitwardenClient.Secrets.ListAsync(organizationId)).Data;
Console.WriteLine("A list of all secrets:");
foreach (SecretIdentifierResponse sr in secretsList)
{
    Console.WriteLine("  Secret: " + sr.Key);
}

Console.Write("Press enter to continue...");
Console.ReadLine();

// Secrets Create, Update, Get
Console.WriteLine("Creating and updating a secret");
var secretResponse = await bitwardenClient.Secrets.CreateAsync(organizationId, "New Secret", "the secret value", "the secret note", new[] { projectResponse.Id });
secretResponse = await bitwardenClient.Secrets.UpdateAsync(organizationId, secretResponse.Id, "New Secret Name", "the secret value", "the secret note", new[] { projectResponse.Id });
secretResponse = await bitwardenClient.Secrets.GetAsync(secretResponse.Id);
Console.WriteLine("Here is the secret we created and updated:");
Console.WriteLine(secretResponse.Key);

Console.Write("Press enter to continue...");
Console.ReadLine();

// Secrets GetByIds
var secretsResponse = await bitwardenClient.Secrets.GetByIdsAsync(new[] { secretResponse.Id });

// Secrets Sync
var syncResponse = await bitwardenClient.Secrets.SyncAsync(organizationId, null);

// Secrets & Projects Delete
Console.WriteLine("Deleting our secret and project");
await bitwardenClient.Secrets.DeleteAsync(new[] { secretResponse.Id });
await bitwardenClient.Projects.DeleteAsync(new[] { projectResponse.Id });
