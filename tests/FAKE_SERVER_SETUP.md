# Fake Server Setup Guide

## Overview

The SDK test framework uses a local fake server to simulate the Bitwarden API for testing. This guide explains how to set up and troubleshoot the fake server.

## Quick Start

### First-Time Setup (Required)

**Pre-build the fake server before running tests:**

```bash
# From the SDK root directory
cargo build -p fake-server

# This creates: target/debug/fake-server
# Build time: 2-5 minutes on first run
```

> **Why is this needed?** The first build downloads all Rust dependencies and compiles the server. Without pre-building, the test framework may timeout waiting for the automatic build to complete.

### Running Tests

After the fake server is built, you have three options:

#### Option 1: Automatic Start (Default)
```bash
cd tests
dotnet test
# The framework will automatically start/stop the fake server
```

#### Option 2: Manual Start
```bash
# Terminal 1: Start fake server
cargo run -p fake-server

# Terminal 2: Run tests
cd tests
dotnet test
```

#### Option 3: Using Test Scripts
```bash
# The test scripts handle everything automatically
./languages/python/test.sh
# or
./languages/go/test.sh
```

## Configuration

The fake server behavior is controlled by `test-config.json`:

```json
{
  "configuration": {
    "TEST_MODE": "fake-server",
    "AUTO_START_FAKE_SERVER": true,    // Auto-start the server
    "FAKE_SERVER_PORT": 4000           // Port to use
  },
  "timeouts": {
    "BUILD_TIMEOUT_MS": 120000         // 2 minutes for build
  }
}
```

## How It Works

### Automatic Build Process

When `AUTO_START_FAKE_SERVER` is enabled:

1. **Check for Executable**: Looks for `target/debug/fake-server`
2. **Build if Missing**: Runs `cargo build -p fake-server`
3. **Start Server**: Launches on configured port
4. **Health Check**: Verifies server is responding
5. **Run Tests**: Proceeds with test execution
6. **Cleanup**: Stops server after tests complete

### Build Scripts

The SDK includes several scripts that handle the fake server:

- **`bootstrap.sh`**: Builds and starts fake server (line 77)
- **`test-utils.sh`**: Shared function `start_fake_server_if_needed` (lines 118-169)
- **Language test scripts**: Use test-utils.sh functions

## Troubleshooting

### Common Issues and Solutions

#### "Failed to start fake server" Error

**Cause**: The fake server hasn't been built yet, and the automatic build timed out.

**Solutions**:
1. Pre-build manually: `cargo build -p fake-server`
2. Increase timeout: Edit `BUILD_TIMEOUT_MS` in test-config.json
3. Disable auto-start: Set `AUTO_START_FAKE_SERVER: false` and start manually

#### Rust/Cargo Not Found

**Cause**: Rust toolchain is not installed or not in PATH.

**Solutions**:
1. Install Rust: https://rust-lang.org/tools/install
2. Source cargo environment:
   ```bash
   source "$HOME/.cargo/env"  # For bash/zsh
   ```
3. Verify installation: `cargo --version`

#### Port Already in Use

**Cause**: Another process is using the configured port.

**Solutions**:
1. Check what's using the port:
   ```bash
   lsof -i :4000  # macOS/Linux
   netstat -ano | findstr :4000  # Windows
   ```
2. Change port in test-config.json
3. Kill the existing process

#### Server Starts but Tests Fail to Connect

**Cause**: Firewall, wrong URLs, or server crash.

**Solutions**:
1. Check server health: `curl http://localhost:4000/health`
2. Review server logs: Run fake server manually to see output
3. Verify environment variables in .env file

## Performance Tips

1. **Pre-build Once**: Build the fake server once and reuse it
2. **Keep Server Running**: During development, start it once manually
3. **Parallel Tests**: Disable if experiencing connection issues
4. **Resource Limits**: Ensure sufficient memory for Rust compilation

## CI/CD Considerations

For CI/CD pipelines:

1. **Cache Dependencies**: Cache `~/.cargo` and `target/` directories
2. **Pre-build Step**: Add explicit build step before tests
3. **Timeout Settings**: Increase timeouts for first-time builds
4. **Error Recovery**: Implement retry logic for transient failures

## Manual Server Management

### Starting the Server
```bash
# Default port (3000)
cargo run -p fake-server

# Custom port
SM_FAKE_SERVER_PORT=4000 cargo run -p fake-server

# Background process
SM_FAKE_SERVER_PORT=4000 ./target/debug/fake-server &
```

### Checking Server Status
```bash
# Health check
curl http://localhost:4000/health

# List secrets (test endpoint)
curl -H "Authorization: Bearer $ACCESS_TOKEN" \
     http://localhost:4000/api/secrets
```

### Stopping the Server
```bash
# If running in foreground: Ctrl+C

# If running in background:
pkill fake-server
# or
kill $(pgrep fake-server)
```

## Environment Variables

The fake server respects these environment variables:

- `SM_FAKE_SERVER_PORT`: Port to listen on (default: 3000)
- `RUST_LOG`: Logging level (e.g., "info", "debug")

## Additional Resources

- [Main Test README](./README.md)
- [Fake Server Source](../crates/fake-server/)
- [Test Framework](./SdkTestFramework/)
- [Test Configuration](./SdkTestFramework.Tests/Configuration/)