# Bitwarden Secrets Manager SDK

Java bindings for interacting with the [Bitwarden Secrets Manager]. This is a beta release and might
be missing some functionality.

## Create access token

Review the help documentation on [Access Tokens]

## Usage code snippets

### Create new Bitwarden client

```java
import com.bitwarden.sdk.*;
import com.bitwarden.sdk.schema.*;

import java.lang.System;
import java.util.UUID;
import java.time.OffsetDateTime;

String stateFile = System.getenv("STATE_FILE");

BitwardenSettings bitwardenSettings = new BitwardenSettings();
bitwardenSettings.setApiUrl("https://api.bitwarden.com");
bitwardenSettings.setIdentityUrl("https://identity.bitwarden.com");
BitwardenClient bitwardenClient = new BitwardenClient(bitwardenSettings);
bitwardenClient.auth().loginAccessToken("<access-token>", stateFile);
```

### Create new project

```java
UUID organizationId = UUID.fromString("<organization-id>");
var projectResponse = bitwardenClient.projects().create(organizationId, "TestProject");
UUID projectId = projectResponse.getID();
```

### Get project

```java
var projectResponse = bitwardenClient.projects().get(projectId);
```

### List all projects

```java
var projectsResponse = bitwardenClient.projects().list(organizationId);
```

### Update project

```java
var projectResponse = bitwardenClient.projects().update(organizationId, projectId, "TestProjectUpdated");
```

### Add new secret

```java
String key = "key";
String value = "value";
String note = "note";
var secretResponse = bitwardenClient.secrets().create(organizationId, key, value, note, new UUID[]{projectId});
UUID secretId = secretResponse.getID();
```

### Get secret

```java
var secretResponse = bitwardenClient.secrets().get(secretId);
```

### Get secrets by ids

```java
SecretsResponse secretsByIds = bitwardenClient.secrets().getByIds(new UUID[]{secretId});
for (SecretResponse sr : secretsByIds.getData()) {
    System.out.println(sr.getKey());
}
```

### Update secret

```java
var secretResponse = bitwardenClient.secrets().update(organizationId, secretId, key2, value2, note2, new UUID[]{projectId});
```

### List secrets

```java
var secretIdentifiersResponse = bitwardenClient.secrets().list(organizationId);
```

### Secrets sync

```java
SecretsSyncResponse syncResponse = bitwardenClient.secrets().sync(organizationId, OffsetDateTime.now());
System.out.println("Has changes: " + syncResponse.getHasChanges());
```

### Delete secret or project

```java
bitwardenClient.secrets().delete(new UUID[]{secretId});
bitwardenClient.projects().delete(new UUID[]{projectId});
```

[Access Tokens]: https://bitwarden.com/help/access-tokens/
[Bitwarden Secrets Manager]: https://bitwarden.com/products/secrets-manager/
