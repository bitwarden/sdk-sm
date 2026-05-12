# Bitwarden Secrets Manager SDK

C++ bindings for interacting with the [Bitwarden Secrets Manager]. This is a beta release and might
be missing some functionality.

## Create access token

Review the help documentation on [Access Tokens]

## Usage code snippets

### Client settings

```c++
// Optional - if not stressed, then default values are used
BitwardenSettings bitwardenSettings;
bitwardenSettings.set_api_url("<bitwarden-url>");
bitwardenSettings.set_identity_url("<bitwarden-identity>");
```

### Create new Bitwarden client

```c++
std::string accessToken = "<access-token>";
std::string stateFile = "<state-file>";
// Optional - argument in BitwardenClient
BitwardenClient bitwardenClient = BitwardenClient(bitwardenSettings);
bitwardenClient.loginAccessToken(accessToken, stateFile);
```

### Create new project

```c++
boost::uuids::uuid organizationUuid = boost::uuids::string_generator()("<organization-id>");
ProjectResponse projectResponseCreate = bitwardenClient.createProject(organizationUuid, "TestProject");
boost::uuids::uuid projectId = boost::uuids::string_generator()(projectResponseCreate.get_id());
```

### List all projects

```c++
ProjectsResponse projectResponseList = bitwardenClient.listProjects(organizationUuid);
```

### Get project details

```c++
ProjectResponse projectResponseGet = bitwardenClient.getProject(projectId);
```

### Update project

```c++
ProjectResponse projectResponseUpdate = bitwardenClient.updateProject(organizationUuid, projectId, "TestProjectUpdated");
```

### Delete projects

```c++
ProjectsDeleteResponse projectsDeleteResponse = bitwardenClient.deleteProjects({projectId});
```

### Add new secret

```c++
std::string key = "key";
std::string value = "value";
std::string note = "note";
SecretResponse secretResponseCreate = bitwardenClient.createSecret(organizationUuid, key, value, note, {projectId});
boost::uuids::uuid secretId = boost::uuids::string_generator()(secretResponseCreate.get_id());
```

### List secrets

```c++
SecretIdentifiersResponse secretIdentifiersResponse = bitwardenClient.listSecrets(organizationUuid);
```

### Get secret details

```c++
SecretResponse secretResponseGet = bitwardenClient.getSecret(secretId);
```

### Get multiple secrets by ids

```c++
std::vector<boost::uuids::uuid> secretIds = {secretId, secretId2};
SecretsResponse secretsResponseGet = bitwardenClient.getSecrets(secretIds);
```

### Update secret

```c++
SecretResponse secretResponseUpdate = bitwardenClient.updateSecret(organizationUuid, secretId, "key2", "value2", "note2", {projectId});
```

### Sync secrets

```c++
std::chrono::system_clock::time_point lastSyncedDate = std::chrono::system_clock::now();
SecretsSyncResponse secretsSyncResponse = bitwardenClient.sync(orgnizationUuid, lastSyncedDate);
```

# Delete secrets

```c++
SecretsDeleteResponse secretsDeleteResponse = bitwardenClient.deleteSecrets({secretId});
```

[Access Tokens]: https://bitwarden.com/help/access-tokens/
[Bitwarden Secrets Manager]: https://bitwarden.com/products/secrets-manager/
