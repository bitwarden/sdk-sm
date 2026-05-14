# SDK Test Framework

A .NET-based test framework for validating Bitwarden SDK implementations across Python and Go.

## 🚀 Quick Start

### Step 1: Install Prerequisites

You'll need these tools installed on your system:

| Tool | Required For | Installation |
|------|-------------|--------------|
| **.NET 10** | Running tests | [Download](https://dotnet.microsoft.com/download/dotnet/10.0) |
| **Rust** | Building SDKs & fake-server | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **Node.js 20+** | Generating schemas | [Download](https://nodejs.org/) or `brew install node` |
| **Python 3.12+** | Python SDK tests | Usually pre-installed, or `brew install python@3.12` |
| **Go 1.23+** | Go SDK tests | `brew install go` or [Download](https://go.dev/dl/) |
| **uv** | Python package manager | `curl -LsSf https://astral.sh/uv/install.sh \| sh` |

#### Verify Installation
```bash
dotnet --version  # Should show 10.x.x
cargo --version   # Should show 1.x.x
node --version    # Should show v20.x.x or higher
python3 --version # Should show 3.12.x or higher
go version        # Should show 1.23 or higher
uv --version      # Should show 0.x.x
```

### Step 2: One-Time Setup

Run these commands **once** after cloning the repository:

```bash
# 1. Generate SDK schemas (from repository root)
npm install       # Install schema generation tools
npm run schemas   # Generate language bindings

# 2. Build the fake-server (from repository root)
cargo build -p fake-server  # Takes 2-5 minutes first time

# 3. Set up test configuration
cd tests
cp SdkTestFramework.Tests/Configuration/.env.example \
   SdkTestFramework.Tests/Configuration/.env
```

### Step 3: Run Tests

```bash
cd tests
dotnet test

# Or run specific language tests
dotnet test --filter "Category=Python"
dotnet test --filter "Category=Go"
```

That's it! The tests should now run successfully. 🎉

## 📋 What Gets Tested

The framework tests these SDK operations:
- Authentication with access tokens
- Secret CRUD operations (Create, Read, Update, Delete)
- Project CRUD operations
- Password generation
- Secret synchronization

## 🔧 Configuration

### Test Modes

The framework supports two test modes, configured in `test-config.json`:

| Mode | Description | When to Use |
|------|-------------|-------------|
| `fake-server` | Uses local mock server | Development, CI/CD (default) |
| `real-server` | Uses actual Bitwarden server | Integration testing |

### Configuration File

Edit `tests/SdkTestFramework.Tests/Configuration/test-config.json`:

```json
{
  "configuration": {
    "TEST_MODE": "fake-server",    // or "real-server"
    "BUILD_SDK": true,              // Build SDKs before testing
    "AUTO_START_FAKE_SERVER": true, // Auto-start fake server
    "FAKE_SERVER_PORT": 4000,
    "ENABLED_LANGUAGES": ["python", "go"]
  }
}
```

### Environment Variables

For real server testing, update `.env` with your credentials:
```bash
# Real server credentials
ACCESS_TOKEN=your-access-token
ORGANIZATION_ID=your-organization-id
API_URL=https://api.bitwarden.com
IDENTITY_URL=https://identity.bitwarden.com
```

#### Getting an Access Token

To obtain an access token for real server testing, you'll need to create a machine account in Bitwarden. See the official documentation: [Managing Access Tokens](https://bitwarden.com/help/access-tokens/)

## 🐛 Troubleshooting

### Common Issues

<details>
<summary><b>"Failed to start fake server"</b></summary>

The fake-server needs to be built first:
```bash
cargo build -p fake-server
```
If cargo is not found, ensure Rust is installed and in your PATH:
```bash
source "$HOME/.cargo/env"
```
</details>

<details>
<summary><b>"schemas.py not found"</b></summary>

Generate the required schemas:
```bash
# From repository root
npm install
npm run schemas
```
</details>

<details>
<summary><b>"uv is required but not installed"</b></summary>

Install uv (Python package manager):
```bash
curl -LsSf https://astral.sh/uv/install.sh | sh
source $HOME/.local/bin/env  # Add to PATH
```
</details>

<details>
<summary><b>"maturin not found"</b></summary>

Maturin (Python build tool) will be installed automatically by the test framework if uv is available. If you need to install it manually:
```bash
pip install maturin
```
</details>

<details>
<summary><b>Tests timeout on first run</b></summary>

First-time setup can be slow due to dependency downloads. Pre-build everything:
```bash
# Build fake-server
cargo build -p fake-server

# Build Python SDK
cd languages/python
python3 -m pip install maturin
maturin develop

# Build Go dependencies
cd ../go
go mod download
```
</details>

### Debug Mode

For detailed output during test runs:
```bash
# Verbose test output
dotnet test --logger "console;verbosity=detailed"

# Check fake-server logs (manual start)
RUST_LOG=debug ./target/debug/fake-server

# Run with specific configuration
dotnet test -- TestRunParameters.Parameter(name="verbose", value="true")
```

## 📁 Project Structure

```
tests/
├── SdkTestFramework/           # Core framework library
│   ├── Config/                # Configuration management
│   ├── Models/                # Test result models
│   ├── Platform/              # Platform detection
│   ├── Services/              # Test services
│   └── TestRunners/           # Language-specific runners
│       ├── PythonTestRunner.cs
│       └── GoTestRunner.cs
│
├── SdkTestFramework.Tests/    # Test project
│   ├── Configuration/
│   │   ├── .env              # Environment variables (create from .env.example)
│   │   ├── .env.example      # Template for .env
│   │   └── test-config.json  # Test configuration
│   └── LanguageTests/
│       ├── PythonTests.cs    # Python SDK tests
│       └── GoTests.cs        # Go SDK tests
│
└── README.md                 # This file
```

## 🧪 How It Works

1. **Test Orchestration**: The .NET framework orchestrates language-specific test scripts
2. **Language Tests**: Each language has a test script in `languages/{lang}/test/`
3. **JSON Communication**: Test scripts output JSON results that the framework parses
4. **Fake Server**: A local mock server provides predictable API responses for testing

## 🚦 CI/CD Integration

### GitHub Actions

The repository includes a workflow for automated testing:

```yaml
name: SDK Tests
on: [push, pull_request]
```

### Manual Workflow Trigger

```bash
gh workflow run sdk-tests.yml \
  -f test_mode=fake-server \
  -f verbose=true \
  -f no_build=false
```

## 📊 Test Output

Successful test run output:
```
═══════════════════════════════════════════════════════════════════
  Python SDK Test Results
═══════════════════════════════════════════════════════════════════

  Status: ✅ PASSED
  Platform: Darwin (Arm64)
  Duration: 2.34s

  Test Summary:
  ├─ Total:    16 tests
  ├─ Passed:   16 ✅
  ├─ Failed:    0 ❌
  └─ Skipped:   0 ⏭️

  Test Operations:
  ──────────────────────────────────────────────────────────
  📦 Tests
     ├─ ✅ test_auth (45ms)
     ├─ ✅ test_secret_create (23ms)
     ├─ ✅ test_secret_list (18ms)
     └─ ... more tests
```

## ➕ Adding a New Language

To add support for a new SDK language:

1. Create test script at `languages/{language}/test/test_suite.{ext}`
2. Implement required operations (auth, CRUD, sync)
3. Output results as JSON
4. Create runner in `SdkTestFramework/TestRunners/{Language}TestRunner.cs`
5. Add test class in `SdkTestFramework.Tests/LanguageTests/{Language}Tests.cs`
6. Update `ENABLED_LANGUAGES` in `test-config.json`

## 📝 License

See the main repository LICENSE file.
