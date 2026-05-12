# Bitwarden SDK in Go

This SDK is designed to interact with Bitwarden services in Go. It includes implementations for
managing projects and secrets, as well as a client interface to facilitate operations like login.

## Prerequisites

- Go installed
- C environment to run CGO

## Installation

Follow the installation instructions [here](./INSTRUCTIONS.md).

## Table of Contents

- [Initialization](#initialization)
- [Login](#login)
- [Projects](#projects)
- [Secrets](#secrets)
- [Close Client](#close-client)

---

### Initialization

To initialize the client, you need to import the SDK and create a new `BitwardenClient` instance.

```go
import "github.com/bitwarden/sdk-go/v2"

bitwardenClient, _ := sdk.NewBitwardenClient(&apiURL, &identityURL)
```

---

### Login

To login using an access token. Define some `stateFile` and pass it to use state, or pass `nil`
instead to not use state.

```go
stateFile := os.Getenv("STATE_FILE")

err := bitwardenClient.AccessTokenLogin(accessToken, &stateFile)
```

---

### Projects

#### Create a Project

```go
project, err := bitwardenClient.Projects().Create("organization_id", "project_name")
```

#### List Projects

```go
projects, err := bitwardenClient.Projects().List("organization_id")
```

#### Get a Project

```go
project, err := bitwardenClient.Projects().Get("project_id")
```

#### Update a Project

```go
project, err := bitwardenClient.Projects().Update("project_id", "organization_id", "new_project_name")
```

#### Delete Projects

```go
project, err := bitwardenClient.Projects().Delete([]string{"project_id_1", "project_id_2"})
```

---

### Secrets

#### Create a Secret

```go
secret, err := bitwardenClient.Secrets().Create("key", "value", "note", "organization_id", []string{"project_id"})
```

#### List Secrets

```go
secrets, err := bitwardenClient.Secrets().List("organization_id")
```

#### Get a Secret

```go
secret, err := bitwardenClient.Secrets().Get("secret_id")
```

#### Get Multiple Secrets by IDs

```go
secrets, err := bitwardenClient.Secrets().GetByIDS([]string{"secret_ids"})
```

#### Update a Secret

```go
secret, err := bitwardenClient.Secrets().Update("secret_id", "new_key", "new_value", "new_note", "organization_id", []string{"project_id"})
```

#### Delete Secrets

```go
secret, err := bitwardenClient.Secrets().Delete([]string{"secret_id_1", "secret_id_2"})
```

#### Secrets Sync

```go
secretsSync, err := bitwardenClient.Secrets().Sync("organization_id", nil)

lastSyncedDate := time.Now()
secretsSync, err = bitwardenClient.Secrets().Sync("organization_id", lastSyncedDate)
```

---

### Close Client

To free up resources:

```go
defer bitwardenClient.Close()
```

---

For more detailed information, refer to the code comments and method signatures.
