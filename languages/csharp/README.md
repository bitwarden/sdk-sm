# Bitwarden Secrets Manager SDK

.NET bindings for interacting with the [Bitwarden Secrets Manager]. This is a beta release and might be missing some functionality.

## Create access token

Review the help documentation on [Access Tokens]

## Usage code snippets

### Create new Bitwarden client

```csharp
const string accessToken = "<access-token>";
const string stateFile = "<state-file>";

using var bitwardenClient = new BitwardenClient(new BitwardenSettings
{
    ApiUrl = apiUrl,
    IdentityUrl = identityUrl
});

await bitwardenClient.Auth.LoginAccessTokenAsync(accessToken, stateFile);
```

### Create new project

```csharp
var organizationId = Guid.Parse("<organization-id>");
var projectResponse = await bitwardenClient.Projects.CreateAsync(organizationId, "TestProject");
```

### List all projects

```csharp
var projectList = await bitwardenClient.Projects.ListAsync(organizationId);
```

### Update project

```csharp
var projectId = projectResponse.Id;
projectResponse = await bitwardenClient.Projects.UpdateAsync(organizationId, projectId, "TestProjectUpdated");
projectResponse = await bitwardenClient.Projects.GetAsync(projectId);
```

### Add new secret

```csharp
var key = "key";
var value = "value";
var note = "note";
var secretResponse = await bitwardenClient.Secrets.CreateAsync(organizationId, key, value, note, new[] { projectId });
```

### Update secret
```csharp
var secretId = secretResponse.Id;
secretResponse = await bitwardenClient.Secrets.UpdateAsync(organizationId, secretId, "key2", "value2", "note2", new[] { projectId });
secretResponse = await bitwardenClient.Secrets.GetAsync(secretId);
```

### Secret GetByIds

```csharp
var secretsResponse = await bitwardenClient.Secrets.GetByIdsAsync(new[] { secretResponse.Id });
```

### List secrets

```csharp
var secretsList = await bitwardenClient.Secrets.ListAsync(organizationId);
```

### Sync secrets

```csharp
var syncResponse = await bitwardenClient.Secrets.SyncAsync(organizationId, null);
```

# Delete secret or project

```csharp
await bitwardenClient.Secrets.DeleteAsync(new [] { secretId });
await bitwardenClient.Projects.DeleteAsync(new [] { projectId });
```
# All main SDK methods are asynchronous. Use `await` and ensure your calling code is in an `async Task` method.

[Access Tokens]: https://bitwarden.com/help/access-tokens/
[Bitwarden Secrets Manager]: https://bitwarden.com/products/secrets-manager/