# Bitwarden Secrets Manager SDK wrapper for PHP

PHP bindings for interacting with the [Bitwarden Secrets Manager]. This is a beta release and might
be missing some functionality.

## Installation

See the [installation instructions](./INSTALL.md)

## Usage

### Create access token

To interact with the client first you need to obtain the access token from Bitwarden. Review the
help documentation on [Access Tokens].

### Create new Bitwarden client

```php
require_once 'vendor/autoload.php';

$access_token = "<access-token>";
$state_file = "<state-file>";
$organization_id = "<organization-id>";
$api_url = "https://api.bitwarden.com";
$identity_url = "https://identity.bitwarden.com";

$bitwarden_settings = new \Bitwarden\Sdk\BitwardenSettings($api_url, $identity_url);

$bitwarden_client = new \Bitwarden\Sdk\BitwardenClient($bitwarden_settings);
$bitwarden_client->auth->login_access_token($access_token, $state_file);
```

Initialize `BitwardenSettings` by passing `$api_url` and `$identity_url` or set to null to use the
defaults. The default for `api_url` is `https://api.bitwarden.com` and for `identity_url` is
`https://identity.bitwarden.com`.

### Create new project

```php
$name = "PHP project";
$res = $bitwarden_client->projects->create($organization_id, $name);
$project_id = $res->id;
```

### Get project

```php
$res = $bitwarden_client->projects->get($project_id);
```

### List all projects

```php
$res = $bitwarden_client->projects->list($organization_id);
```

### Update project

```php
$name = "Updated PHP project";
$res = $bitwarden_client->projects->update($organization_id, $project_id, $name);
```

### Delete project

```php
$res = $bitwarden_client->projects->delete([$project_id]);
```

### Create new secret

```php
$key = "Secret key";
$note = "Secret note";
$value = "Secret value";
$res = $bitwarden_client->secrets->create($organization_id, $key, $value, $note, [$project_id]);
$secret_id = $res->id;
```

### Get secret

```php
$res = $bitwarden_client->secrets->get($secret_id);
```

### Get multiple secrets

```php
$res = $bitwarden_client->secrets->get_by_ids([$secret_id]);
```

### List all secrets

```php
$res = $bitwarden_client->secrets->list($organization_id);
```

### Update secret

```php
$key = "Updated key";
$note = "Updated note";
$value = "Updated value";
$res = $bitwarden_client->secrets->update($organization_id, $secret_id, $key, $value, $note, [$project_id]);
```

### Sync secrets

```php
$last_synced_date = "2024-09-01T00:00:00Z";
$res = $bitwarden_client->secrets->sync($organization_id, $last_synced_date);
```

### Delete secret

```php
$res = $bitwarden_client->secrets->delete([$secret_id]);
```

[Access Tokens]: https://bitwarden.com/help/access-tokens/
[Bitwarden Secrets Manager]: https://bitwarden.com/products/secrets-manager/
