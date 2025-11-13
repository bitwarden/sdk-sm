# Bitwarden Secrets Manager SDK

## Crates

The project is structured as a monorepo using cargo workspaces. Some of the more noteworthy crates
are:

- [`bitwarden`](./crates/bitwarden/): Rust-friendly API for interacting with the secrets manager.
- [`bitwarden-c`](./crates/bitwarden-c/): C bindings for FFI interop.
- [`bitwarden-json`](./crates/bitwarden-json/): JSON wrapper around the `bitwarden` crate. Powers
  the other language bindings.
- [`bitwarden-napi`](./crates/bitwarden-napi/): Node-API bindings for Node.js/TypeScript.
- [`bitwarden-wasm`](./crates/bitwarden-wasm/): WebAssembly bindings.
- [`bws`](./crates/bws/): CLI for interacting with the [Bitwarden Secrets Manager][secrets-manager].
  Review the [CLI documentation][bws-help].
- [`sdk-schemas`](./crates/sdk-schemas/): Generator for the _json schemas_.
- [`fake-server`](./crates/fake-server/): Development/testing server that emulates Bitwarden API
  endpoints for local testing without a real server instance.

### Language Bindings

The SDK provides bindings in the `/languages/` directory: C++, C#, Go, Java, JavaScript/TypeScript,
PHP, Python, and Ruby. All bindings share a consistent API for projects and secrets management.

## Schemas

To minimize the amount of work required to support additional bindings the project is structured
around a `json` based API, beginning with every binding only needing to implement one method, namely
a `run_command`. Additional work may be required to implement other functions, like a `free` command
for languages that require manual memory management. Additional language-specific implementation
details will apply based on the language.

To ensure type safety in the API, _json schemas_ are generated from the rust structs in `bitwarden`
using [schemars](https://crates.io/crates/schemars). The _json schemas_ are later used to generate
the API bindings for each language using [QuickType](https://github.com/quicktype/quicktype).

```bash
npm run schemas
```

## Key Concepts

### Architecture

The SDK uses a **layered architecture** to support multiple languages efficiently:

1. **Core Layer**: External Rust dependencies from
   [sdk-internal](https://github.com/bitwarden/sdk-internal) repository contain the core business
   logic and cryptography. The `Cargo.toml` file **must** be inspected for these dependencies.
2. **Public API Layer**: The public `bitwarden` crate provides the Rust API.
3. **JSON API Layer**: `bitwarden-json` wraps the Rust API with a single `run_command` method.
4. **Language Bindings**: All language bindings call `run_command` with JSON requests/responses.

This architecture means adding a new language or updating the API requires minimal per-language
work, as the heavy lifting is done in Rust and exposed via JSON schemas.

### Authentication

The SDK uses **access tokens** for authentication. All language bindings provide an authentication
method (named according to each language's conventions, such as `login_access_token`,
`accessTokenLogin`, or `LoginAccessToken`) to authenticate with the Secrets Manager API.

### State Files

The SDK supports optional **state files** for caching, improving performance by avoiding
re-authentication. State files can be specified when calling the authentication method or omitted by
passing `None`/`null`.

### Feature Flags

Key feature flags include:

- `secrets` (default): Enables Secrets Manager API
- `no-memory-hardening`: Disables memory security hardening features
- `wasm`: Enables WebAssembly support

## Development Workflows

### Generating Schemas

When you modify the Rust API structs, regenerate the JSON schemas and language bindings:

```bash
npm run schemas
```

This generates JSON schemas from Rust structs and updates all language binding code using QuickType.

### Local Testing with Fake Server

Use the `fake-server` crate to test locally without a real Bitwarden instance:

```bash
cargo run -p fake-server
# Then use bws or other clients against http://localhost:3000 (default port)
```

The fake server provides minimal CRUD operations for secrets and projects.

### Building Language Bindings

- **Python**: Requires Python 3 and `maturin` (`pip install maturin`)
- **Node.js**: Uses `napi-rs` for native addons
- **WebAssembly**: Requires `wasm32-unknown-unknown` target and `wasm-bindgen-cli`

### Code Quality

- **Formatting**: Requires nightly toolchain: `cargo +nightly fmt`
- **Linting**: `cargo clippy --all-features --tests` (workspace lints deny unwrap_used and
  unused_async)
- **MSRV**: Minimum Supported Rust Version is 1.85

## References

- [SDK Architecture](https://contributing.bitwarden.com/architecture/sdk/secrets-manager/)
- [Secrets Manager SDK Documentation](https://bitwarden.com/help/secrets-manager-sdk/)
- [Secrets Manager CLI Documentation](https://bitwarden.com/help/secrets-manager-cli/)
- [Secrets Manager Overview](https://bitwarden.com/help/secrets-manager-overview/)
- [Architectural Decision Records (ADRs)](https://contributing.bitwarden.com/architecture/adr/)
- [Contributing Guidelines](https://contributing.bitwarden.com/contributing/)
- [Setup Guide](https://contributing.bitwarden.com/getting-started/sdk/secrets-manager/)
- [Code Style](https://contributing.bitwarden.com/contributing/code-style/)
- [Security Whitepaper](https://bitwarden.com/help/bitwarden-security-white-paper/)
- [Security Definitions](https://contributing.bitwarden.com/architecture/security/definitions)
