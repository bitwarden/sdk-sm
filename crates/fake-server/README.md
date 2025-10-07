# Fake Server

A minimal API server that emulates the /secrets and /projects endpoints of the official Bitwarden
server. It does just enough to enable simple CRUD testing with our CLI, language bindings, and
integrations.

## Usage

### Start the fake server

```sh
export SM_FAKE_SERVER_PORT=3000 # default value
RUST_LOG=info cargo run --quiet --bin fake-server
```

### Example usage with the `bws` CLI

```sh
# this is a fake access token that only works with the fake server
export BWS_ACCESS_TOKEN="0.ec2c1d46-6a4b-4751-a310-af9601317f2d.C2IgxjjLF7qSshsbwe8JGcbM075YXw:X8vbvA0bduihIDe/qrzIQQ=="
export BWS_SERVER_URL="http://localhost:${SM_FAKE_SERVER_PORT:-3000}"

# secrets
bws secret list
bws secret get "$(uuidgen)" # the api server does not validate the uuid, so this will return a fake secret
bws secret create 'secret-key' 'secret-value' --note 'optional note' "$(uuidgen)"
bws secret edit --key 'something-new' --value 'new-value' --note 'updated note' "$(uuidgen)"
bws secret delete "$(uuidgen)" "$(uuidgen)" # delete as many fake secrets as you want

# projects
bws project list
bws project get "$(uuidgen)"
bws project create 'project-name'
bws project edit --name 'new-project-name' "$(uuidgen)"
bws project delete "$(uuidgen)" "$(uuidgen)"
```

## Creating new secrets and projects

Because we still need valid cryptography keys (from the fake access token), we need to leverage a
Secrets Manager client to generate valid ciphers. We can use the server logging to see the generated
ciphers:

```sh
RUST_LOG=info cargo run --quiet --bin fake-server

# in another shell session, create a secret
bws secret create 'my new key' 'my new value' --note 'optional note' "$(uuidgen)"
```

Observe that the server logs will show valid cipher keys that can be embedded in the fake server:

```log
2025-06-26T14:27:24.046014Z  INFO fake_server: Creating secret for organization f4e44a7f-1190-432a-9d4a-af96013127cb: CreateSecretRequest { key: "2.yvALRqGimwssILfP6fAPGw==|J1xFKmNZ9TiXaLPvvJt45Q==|/gT0Vg7ggBz6ek/SikW9MhnYCfUlt2HkAsPXaS3YVSs=", value: "2.sM76QhcSUWdLm9m11T2n2g==|8QSmcqlaRROxqr1BbejIyA==|owRtF/hYVrI/FggmQc7Jjo6xQSM4XudKAcDuBrfca28=", note: "2.y22rQSkkcb9c1MvceSmNXw==|0wFcpNq1XbfVSvBM8Gw/OA==|Syl8A3TgPbJwElNLHhsb+DHRdGfgQE7qwuX1ha2ilQ8=", project_ids: None }
```
