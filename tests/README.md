# SDK Test Framework POC

A streamlined test framework for validating Bitwarden SDK implementations across multiple programming languages.

## Overview

This POC framework provides testing for:
- **Language SDKs**: Python, Go (more languages can be added)
- **Platforms**: Windows, macOS, Linux
- **Architectures**: x64, ARM64
- **Test Modes**: Fake-server (local) or Real-server testing

## Quick Start

### Prerequisites

- .NET 8.0 SDK
- **Node.js 20+ and npm** (REQUIRED for schema generation)
  - Install from: https://nodejs.org/ or via package manager
  - Verify: `node --version` and `npm --version`
- **Python 3.12+** (REQUIRED for Python SDK tests)
  - **uv** (REQUIRED for Python tests):
    ```bash
    curl -LsSf https://astral.sh/uv/install.sh | sh
    source $HOME/.local/bin/env  # Add to PATH
    ```
  - maturin will be installed automatically by uv
- **Go 1.23+** (REQUIRED for Go SDK tests)
  - Install from: https://go.dev/dl/ or via package manager
  - macOS: `brew install go`
  - Verify: `go version`
- **Rust/Cargo** (REQUIRED for building SDKs and fake-server)
  - Install from: https://rust-lang.org/tools/install
  - Verify installation: `cargo --version`
  - After installation, source the cargo environment:
    ```bash
    source "$HOME/.cargo/env"  # For bash/zsh
    # OR
    source "$HOME/.cargo/env.fish"  # For fish shell
    ```

### Environment Setup

**Ensure all tools are in your PATH:**
```bash
# Add to your shell profile (~/.bashrc, ~/.zshrc, etc.):
export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$PATH"

# Or source them temporarily for the current session:
source "$HOME/.cargo/env"      # For Rust/Cargo
source "$HOME/.local/bin/env"  # For uv

# Verify tools are available:
cargo --version  # Should show cargo version
uv --version     # Should show uv version
```

### Initial Setup (IMPORTANT - First Time Only)

**1. Generate SDK schemas (required for language bindings):**

```bash
# From the repository root, install npm dependencies and generate JSON schemas
npm install          # First time only - installs QuickType and other tools
npm run schemas      # Generate the schema files

# This creates schema files needed by language bindings, including:
# - languages/python/bitwarden_sdk/schemas.py
# - Other language binding schemas

# Note: Schema generation requires cargo in PATH:
# export PATH="$HOME/.cargo/bin:$PATH"
```

**2. Build the fake-server (required for local testing):**

```bash
# Build the fake-server (takes 2-5 minutes on first run)
cargo build -p fake-server

# The build creates: target/debug/fake-server
# Subsequent test runs will use this pre-built binary
```

> **Note**: Both steps are one-time setup tasks:
> - Schema generation creates type definitions for all language bindings
> - The fake-server build downloads Rust dependencies and compiles the server
> - Once completed, tests will run quickly using these pre-built artifacts

### Running Tests

```bash
# Ensure tools are in PATH (if not already in your shell profile)
export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$PATH"

# After fake-server is built, run all SDK tests
cd tests
dotnet test

# Run specific language tests
dotnet test --filter "Category=Python"
dotnet test --filter "Category=Go"

# Run with detailed output
dotnet test --logger "console;verbosity=detailed"
```

### Configuration

1. **Copy environment template**:
   ```bash
   cp tests/SdkTestFramework.Tests/Configuration/.env.example tests/SdkTestFramework.Tests/Configuration/.env
   ```

2. **The `.env.example` file comes pre-configured for fake-server testing**:
   - Uses test credentials that work with the fake server
   - Both API_URL and IDENTITY_URL point to `http://localhost:4000`
   - STATE_FILE is auto-generated (no need to set manually)

   For real server testing, update the `.env` file with your actual credentials and URLs.

### Test Configuration Matrix

The framework supports multiple test configurations. Here are the key options and their combinations:

| Configuration | Options | Description | Use Case |
|--------------|---------|-------------|----------|
| **TEST_MODE** | `fake-server` | Uses local mock server | Development, CI/CD, offline testing |
| | `real-server` | Uses actual Bitwarden server | Integration testing, validation |
| **SDK_SOURCE** | `local` | Uses locally built SDK | Development, testing local changes |
| | `main` | Builds SDK from main branch | Testing against latest stable |
| **AUTO_START_FAKE_SERVER** | `true` | Auto-start/stop fake server | Convenient for local development |
| | `false` | Manual server management | CI/CD, debugging, custom setup |
| **ENABLED_LANGUAGES** | `["python", "go"]` | Test specific languages | Focused testing |
| **PARALLEL_EXECUTION** | `true` | Run language tests in parallel | Faster execution |
| | `false` | Sequential execution | Debugging, resource constraints |

#### Common Configuration Scenarios

**1. Local Development (Default)**
```json
{
  "TEST_MODE": "fake-server",
  "SDK_SOURCE": "local",
  "AUTO_START_FAKE_SERVER": true,
  "ENABLED_LANGUAGES": ["python", "go"]
}
```
- Best for: Active development, quick iteration
- Requirements: Built fake-server, generated schemas

**2. CI/CD Pipeline**
```json
{
  "TEST_MODE": "fake-server",
  "SDK_SOURCE": "main",
  "AUTO_START_FAKE_SERVER": false,
  "ENABLED_LANGUAGES": ["python", "go"],
  "PARALLEL_EXECUTION": true
}
```
- Best for: Automated testing, validation
- Note: Server managed by CI pipeline

**3. Integration Testing**
```json
{
  "TEST_MODE": "real-server",
  "SDK_SOURCE": "local",
  "AUTO_START_FAKE_SERVER": false,
  "ENABLED_LANGUAGES": ["python"]
}
```
- Best for: Testing against production-like environment
- Requirements: Valid Bitwarden credentials

**4. Release Validation**
```json
{
  "TEST_MODE": "real-server",
  "SDK_SOURCE": "main",
  "AUTO_START_FAKE_SERVER": false,
  "ENABLED_LANGUAGES": ["python", "go"],
  "PARALLEL_EXECUTION": true
}
```
- Best for: Pre-release validation
- Tests: Latest main branch against real server

**5. Debugging Mode**
```json
{
  "TEST_MODE": "fake-server",
  "SDK_SOURCE": "local",
  "AUTO_START_FAKE_SERVER": false,
  "ENABLED_LANGUAGES": ["python"],
  "PARALLEL_EXECUTION": false
}
```
- Best for: Troubleshooting specific issues
- Benefits: Sequential execution, manual server control

#### Configuration Override Options

Settings can be overridden at multiple levels (in order of precedence):

1. **Command-line parameters** (highest priority):
   ```bash
   dotnet test -p:TEST_MODE=real-server -p:SDK_SOURCE=main
   ```

2. **Environment variables**:
   ```bash
   export TEST_MODE=real-server
   export SDK_SOURCE=main
   dotnet test
   ```

3. **test-config.json file** (lowest priority)

#### Output Format Options

Control test output with the `OUTPUT_FORMAT` setting:
- `json` - Machine-readable JSON output
- `text` - Human-readable console output
- `both` - Both JSON and text output (default)

3. **Configure test settings** in `test-config.json`:
   ```json
   {
     "configuration": {
       "TEST_MODE": "fake-server",
       "SDK_SOURCE": "local-build",
       "AUTO_START_FAKE_SERVER": false,
       "FAKE_SERVER_PORT": 4000,
       "ENABLED_LANGUAGES": ["python", "go"]
     }
   }
   ```

## Architecture

### Test Hierarchy
```
[SetUpFixture] Global.cs (One-time setup/teardown)
    └── [TestFixture] TestBase.cs (Base for all tests)
        └── SdkWrappersTestBase.cs (Base for SDK tests)
            ├── PythonTests.cs (Single test method: Python_SDK_Tests)
            └── GoTests.cs (Single test method: Go_SDK_Tests)
```

### Framework Flow
```
NUnit Test Framework (C#)
    ├── PythonTests.cs
    │   └── Executes: languages/python/test/tests.py
    │       └── Tests Python SDK operations
    └── GoTests.cs
        └── Executes: languages/go/test/tests.go
            └── Tests Go SDK operations
```

## Test Operations

Each language SDK must implement 6 standard operations:

1. **auth** - Authenticate with access token
2. **create_secret** - Create a new secret
3. **list_secrets** - List all secrets
4. **get_secret** - Retrieve a specific secret
5. **delete_secret** - Delete a secret
6. **sync** - Synchronize secrets

## Directory Structure

```
tests/
├── SdkTestFramework/           # Core test framework
│   ├── Common/                # Shared utilities (ConsoleFormatting)
│   ├── Config/                # Configuration management
│   ├── Models/                # Data models (TestResult, TestOperation)
│   └── Runners/               # Test runners
│       ├── BaseRunner.cs      # Base runner class
│       ├── FakeServerManager.cs # Manages fake-server lifecycle
│       ├── GoRunner.cs        # Go test runner
│       ├── OsDetector.cs      # OS detection
│       ├── ProcessRunner.cs   # Process execution
│       └── PythonRunner.cs    # Python test runner
├── SdkTestFramework.Tests/    # NUnit test project
│   ├── Configuration/
│   │   ├── .env              # Environment variables
│   │   └── test-config.json  # Test configuration
│   ├── SdkWrappers/          # SDK language tests
│   │   ├── GoTests.cs        # Go SDK test
│   │   ├── PythonTests.cs    # Python SDK test
│   │   └── SdkWrappersTestBase.cs # Base for SDK tests
│   ├── ConfigurationService.cs # Config service
│   ├── Global.cs             # Global setup/teardown
│   └── TestBase.cs           # Base test class
└── README.md                 # This file
```

## Troubleshooting

### Common Issues and Solutions

| Issue | Solution | Related Config |
|-------|----------|----------------|
| **"Failed to start fake server"** | Run `cargo build -p fake-server` first | `AUTO_START_FAKE_SERVER: true` |
| **"schemas.py not found"** | Run `npm install && npm run schemas` | Initial setup |
| **"Access token isn't associated with an organization"** | Ensure IDENTITY_URL points to same port as API_URL (4000 for fake-server) | `TEST_MODE: fake-server` |
| **Tests timeout during dependency installation** | Increase `DEFAULT_TIMEOUT_MS` in test-config.json (300000ms recommended) | Timeout settings |
| **"uv is required but not installed"** | Install uv: `curl -LsSf https://astral.sh/uv/install.sh \| sh` | Python prerequisites |
| **"cargo: command not found"** | Add cargo to PATH: `export PATH="$HOME/.cargo/bin:$PATH"` | Rust prerequisites |
| **Authentication fails with real server** | Verify BWS_CLIENT_ID and BWS_CLIENT_SECRET are set | `TEST_MODE: real-server` |
| **Parallel execution issues** | Set `PARALLEL_EXECUTION: false` for debugging | `PARALLEL_EXECUTION` |

### Debug Tips

1. **Enable verbose output**:
   ```bash
   dotnet test --logger "console;verbosity=detailed"
   ```

2. **Test single language**:
   ```bash
   dotnet test --filter "Category=Python"
   ```

3. **Check fake-server logs**:
   ```bash
   # The framework automatically sets RUST_LOG=info when starting fake-server
   # For manual debugging with different log levels:
   RUST_LOG=debug ./target/debug/fake-server
   ```

4. **Verify configuration**:
   - Check `test-config.json` matches your intended scenario from the Configuration Matrix
   - Ensure `.env` has correct values for your test mode
   - Use configuration override options for quick testing

5. **Manual fake-server control** (for debugging):
   - Set `AUTO_START_FAKE_SERVER: false`
   - Start server manually: `./target/debug/fake-server`
   - Run tests in separate terminal

## Test Modes

### Fake Server Mode
- Uses local mock server
- Predictable responses
- Fast execution
- Good for development

**Configuration Options:**
- `TEST_MODE`: Set to `"fake-server"` to use local fake server
- `AUTO_START_FAKE_SERVER`:
  - `true`: Framework automatically starts/stops fake-server
  - `false`: Assumes fake-server is already running (manual start)
- `FAKE_SERVER_PORT`: Port for fake-server (default: 4000)

**Manual Start (Development):**
```bash
# Start fake-server manually
cargo run -p fake-server

# Configure with AUTO_START_FAKE_SERVER: false
# Run tests
dotnet test
```

**Automatic Start (CI/CD):**
```json
{
  "AUTO_START_FAKE_SERVER": true,
  "FAKE_SERVER_PORT": 4000
}
```

### Real Server Mode
- Tests against actual Bitwarden server
- Requires valid credentials
- Slower but more realistic
- Good for validation
- Set `TEST_MODE` to `"real-server"`

### SDK Source Options
- `SDK_SOURCE`: Controls which SDK packages to test
  - `"local-build"`: Uses locally built SDK packages
  - `"published"`: Uses published SDK packages from package managers

## Adding a New Language

To add support for a new language SDK:

1. **Create test script** at `languages/<language>/test/tests.<ext>`
   - Implement the 6 standard operations
   - Output results as JSON
   - Exit with code 0 on success, 1 on failure

2. **Create runner class** in `SdkTestFramework/Runners/<Language>Runner.cs`
   - Inherit from `BaseRunner`
   - Override necessary methods
   - Handle language-specific setup

3. **Create test class** in `SdkTestFramework.Tests/SdkWrappers/<Language>Tests.cs`
   - Inherit from `SdkWrappersTestBase`
   - Add single test method that runs the language script

4. **Update configuration**
   - Add language to `ENABLED_LANGUAGES` in test-config.json

## CI/CD

### GitHub Actions

The framework includes a complete GitHub Actions workflow:

```yaml
name: SDK Tests
on:
  push:
  pull_request:
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM UTC
```

### Manual Execution

Trigger manually with parameters:

```bash
gh workflow run sdk-tests.yml \
  -f test_mode=fake-server \
  -f sdk_source=local-build
```

## Output Format

### Console Output
```
╔══════════════════════════════════════════════════════════════════╗
║                 SDK Test Framework - Global Setup                  ║
╚══════════════════════════════════════════════════════════════════╝
Adding test run parameters to environment variables...
Validating required environment variables...
  ✓ All required environment variables are set
✅ Global setup complete

Running tests inside test class: PythonTests
Python SDK: All 6 operations passed
Total execution time: 1234ms
✅ Test passed
```

### Test Results (JSON from language scripts)
```json
{
  "language": "python",
  "sdk_version": "1.0.0",
  "operations": [
    {
      "operation": "auth",
      "success": true,
      "duration_ms": 123,
      "error": null,
      "details": {}
    }
  ],
  "total_duration_ms": 1234,
  "os": "darwin",
  "architecture": "x86_64",
  "timestamp": "2024-04-30T12:00:00Z"
}
```

## Troubleshooting

### Common Issues

**"Failed to start fake server" error on first run**
- **Cause**: Fake-server needs to be built first (takes 2-5 minutes initially)
- **Solution**: Run `cargo build -p fake-server` before running tests
- **Alternative**: Set `AUTO_START_FAKE_SERVER: false` and start manually with `cargo run -p fake-server`
- **Note**: The auto-build feature may timeout on first run due to dependency downloads

**Environment validation fails**
- Check `.env` file exists and has required variables
- Verify ACCESS_TOKEN is valid
- Ensure ORGANIZATION_ID is correct

**Python tests fail with "uv is required but not installed"**
- Install uv: `curl -LsSf https://astral.sh/uv/install.sh | sh`
- Restart your terminal after installation
- Verify with: `uv --version`

**Language runtime not found**
- Install required language runtime
- Check PATH environment variable
- Verify version compatibility

**SDK build fails**
- Run schema generation: `npm run schemas`
- Build C library: `cargo build -p bitwarden-c`
- Check language-specific build scripts

**Tests timeout**
- Increase timeout in `test-config.json`
- Check network connectivity
- Verify server is accessible
- For first-time setup, ensure fake-server is pre-built

### Debug Mode

Enable verbose logging:
```bash
dotnet test --logger "console;verbosity=detailed"
```

View fake server logs:
```bash
cargo run -p fake-server
```

## Performance

- Total suite: < 2 minutes
- Per operation: < 10 seconds
- Language tests run sequentially (can be parallelized if needed)

## Contributing

1. Keep the architecture simple
2. Tests should be easy to understand and debug
3. Language-specific logic stays in language test scripts
4. C# framework only orchestrates, doesn't test SDK directly
5. Update documentation when adding languages

## Support

- GitHub Issues: Report bugs or request features
- Documentation: Check guides in `tests/` directory
- Examples: Review existing implementations

## License

See main repository LICENSE file.