# Bitwarden Secrets Manager SDK

.NET bindings for interacting with the [Bitwarden Secrets Manager]. This is a beta release and might
be missing some functionality.

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

bitwardenClient.LoginAccessToken(accessToken, stateFile);
```

### Create new project

```csharp
var organizationId = Guid.Parse("<organization-id>");
var projectResponse = bitwardenClient.Projects().Create(organizationId, "TestProject");
```

### List all projects

```csharp
var response = bitwardenClient.Projects.List(organizationId);
```

### Update project

```csharp
var projectId = projectResponse.Id;
projectResponse = bitwardenClient.Projects.Get(projectId);
projectResponse = bitwardenClient.Projects.Update(organizationId, projectId, "TestProjectUpdated");
```

### Add new secret

```csharp
var key = "key";
var value = "value";
var note = "note";
var secretResponse = bitwardenClient.Secrets.Create(organizationId, key, value, note, new[] { projectId });
```

### Update secret

```csharp
var secretId = secretResponse.Id;
secretResponse = bitwardenClient.Secrets.Get(secretId);
secretResponse = bitwardenClient.Secrets.Update(organizationId, secretId, "key2", "value2", "note2", new[] { projectId });
```

### Secret GetByIds

```csharp
var secretsResponse = bitwardenClient.Secrets.GetByIds(new[] { secretResponse.Id });
```

### List secrets

```csharp
var secretIdentifiersResponse = bitwardenClient.Secrets.List(organizationId);
```

### Sync secrets

```csharp
var syncResponse = bitwardenClient.Secrets.Sync(organizationId, null);
```

# Delete secret or project

```csharp
bitwardenClient.Secrets.Delete(new [] { secretId });
bitwardenClient.Projects.Delete(new [] { projectId });
```

[Access Tokens]: https://bitwarden.com/help/access-tokens/
[Bitwarden Secrets Manager]: https://bitwarden.com/products/secrets-manager/
