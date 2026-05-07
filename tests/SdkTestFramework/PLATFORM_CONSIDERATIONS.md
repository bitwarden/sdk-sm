# Platform-Specific Considerations for SDK Test Framework

## Overview
The SDK Test Framework is designed to run on Windows, macOS, and Linux. While the framework aims for cross-platform compatibility, there are important platform-specific differences that must be considered.

## Platform Detection
The framework uses `System.Runtime.InteropServices.RuntimeInformation` to detect:
- Operating System (Windows, macOS, Linux)
- Architecture (x64, ARM64)
- Version information

## Key Platform Differences

### 1. Bash Shell Availability

#### Windows
- **Requirement**: Git Bash, WSL, or Cygwin must be installed
- **Why**: Windows doesn't include bash by default
- **Detection**: Framework checks for bash and provides platform-specific error messages
- **Common Issues**:
  - Git Bash not in PATH
  - WSL not enabled
  - Running from PowerShell/CMD without bash

#### macOS & Linux
- **Requirement**: Bash is included by default
- **Version**: Usually bash 3.x on macOS, bash 4.x+ on Linux
- **Issues**: Rarely any, unless in minimal container environments

### 2. Python Virtual Environment

#### Windows
- **Activation Script**: `.venv/Scripts/activate`
- **Python Executable**: `python.exe` or `python3.exe`
- **Path Separator**: Backslash `\` (but forward slash `/` works in bash)

#### macOS & Linux
- **Activation Script**: `.venv/bin/activate`
- **Python Executable**: `python3` (sometimes `python`)
- **Path Separator**: Forward slash `/`

### 3. Go SDK Library Paths

#### Windows
- **Library Extension**: `.dll`
- **Path Variable**: `PATH`
- **Library Location**: `lib/windows-{arch}/`

#### macOS
- **Library Extension**: `.dylib`
- **Path Variable**: `DYLD_LIBRARY_PATH`
- **Library Location**: `lib/darwin-{arch}/`
- **Special Consideration**: System Integrity Protection (SIP) may affect DYLD_LIBRARY_PATH

#### Linux
- **Library Extension**: `.so`
- **Path Variable**: `LD_LIBRARY_PATH`
- **Library Location**: `lib/linux-{arch}/`
- **Special Requirement**: `patchelf` needed for Python builds

### 4. Process Execution

#### Windows
- **Executable Extension**: `.exe` required (e.g., `fake-server.exe`)
- **Process Creation**: Different process group handling
- **Shell Execute**: Must be false for captured output

#### macOS & Linux
- **Executable Extension**: No extension needed
- **Process Creation**: Standard Unix process model
- **Permissions**: Execute bit must be set (`chmod +x`)

### 5. File System Differences

#### Windows
- **Case Sensitivity**: Case-insensitive by default
- **Path Length**: Limited to 260 characters (unless long path support enabled)
- **Temp Directory**: `%TEMP%` or `%TMP%`

#### macOS
- **Case Sensitivity**: Case-insensitive by default (but case-preserving)
- **Path Length**: 1024 characters
- **Temp Directory**: `/var/folders/...` (via `mktemp -d`)

#### Linux
- **Case Sensitivity**: Case-sensitive
- **Path Length**: 4096 characters typically
- **Temp Directory**: `/tmp` or `/var/tmp`

## Platform-Specific Code in Test Framework

### C# Framework (SdkTestFramework)

1. **OsDetector.cs**
   - Detects platform using RuntimeInformation
   - Maps to enum: Windows, MacOS, Linux, Unknown
   - Provides convenience booleans: IsWindows, IsMacOs, IsLinux

2. **FakeServerManager.cs**
   ```csharp
   var executableName = osContext.IsWindows ? "fake-server.exe" : "fake-server";
   ```

3. **BaseRunner.cs**
   - Windows-specific bash error messages
   - Platform info included in test results

### Shell Scripts

1. **Python test.sh**
   ```bash
   # Linux-specific package installation
   if [ "$(uname -s)" = "Linux" ]; then
       uv pip install .[dev-linux]  # Includes patchelf
   else
       uv pip install .[dev]
   fi
   ```

2. **Go test.sh**
   ```bash
   # Platform detection
   case "$(uname -s)" in
       Darwin)  platform="darwin" ;;
       Linux)   platform="linux" ;;
       MINGW*|CYGWIN*|MSYS*)  platform="windows" ;;
   esac

   # Platform-specific library paths
   case "$platform" in
       darwin)  export DYLD_LIBRARY_PATH="$lib_dir:${DYLD_LIBRARY_PATH:-}" ;;
       linux)   export LD_LIBRARY_PATH="$lib_dir:${LD_LIBRARY_PATH:-}" ;;
       windows) export PATH="$lib_dir:${PATH:-}" ;;
   esac
   ```

## Architecture Considerations

### ARM64 Support
- **macOS**: Full support on Apple Silicon (M1/M2)
- **Linux**: Full support on ARM64 systems
- **Windows**: Limited support, marked as unsupported in OsDetector

### x64 Support
- Full support on all platforms

## Environment Variables by Platform

### Common (All Platforms)
- `ACCESS_TOKEN`
- `ORGANIZATION_ID`
- `API_URL`
- `IDENTITY_URL`
- `STATE_FILE`
- `TEST_MODE`
- `SDK_SOURCE`

### Platform-Specific
- **macOS**: `DYLD_LIBRARY_PATH` (Go SDK)
- **Linux**: `LD_LIBRARY_PATH` (Go SDK)
- **Windows**: Modified `PATH` for DLL loading

## Testing Recommendations by Platform

### Windows Testing
1. Install Git Bash or enable WSL2
2. Run tests from Git Bash terminal, not PowerShell/CMD
3. Ensure long path support is enabled for deep directory structures
4. Use forward slashes in paths even on Windows (bash handles conversion)

### macOS Testing
1. No special setup required for bash
2. May need to allow unsigned binaries in Security settings
3. Be aware of SIP restrictions on DYLD_LIBRARY_PATH
4. Xcode Command Line Tools should be installed

### Linux Testing
1. Ensure patchelf is installed for Python SDK builds
2. May need to install build-essential for compilation
3. In containers, ensure bash is installed (not just sh)
4. Check ulimits for file descriptors if running many tests

## CI/CD Considerations

### GitHub Actions
```yaml
strategy:
  matrix:
    os: [ubuntu-latest, windows-latest, macos-latest]
    include:
      - os: windows-latest
        shell: bash
      - os: ubuntu-latest
        shell: bash
      - os: macos-latest
        shell: bash
```

### Docker
- Base images should include bash
- Linux containers need patchelf for Python
- Set working directory permissions correctly

## Troubleshooting by Platform

### Windows Issues
1. **"bash: command not found"**
   - Install Git Bash: https://git-scm.com/
   - Or enable WSL2: `wsl --install`

2. **"cannot find fake-server.exe"**
   - Ensure cargo build completes
   - Check .exe extension is added

3. **Path too long errors**
   - Enable long path support in Windows
   - Use shorter base directories

### macOS Issues
1. **"dyld: Library not loaded"**
   - Check DYLD_LIBRARY_PATH is set
   - Verify library architecture matches (arm64 vs x86_64)

2. **"xcrun: error"**
   - Install Xcode Command Line Tools
   - `xcode-select --install`

### Linux Issues
1. **"patchelf: not found"**
   - Ubuntu/Debian: `apt-get install patchelf`
   - RHEL/Fedora: `yum install patchelf`

2. **Permission denied**
   - Ensure scripts are executable: `chmod +x *.sh`
   - Check directory permissions

## Future Improvements

1. **Add FreeBSD support** - Detect and handle BSD variants
2. **Improve Windows native support** - Consider PowerShell scripts alongside bash
3. **Container-specific detection** - Detect when running in Docker/Podman
4. **Architecture-specific optimizations** - Optimize for ARM64 where applicable
5. **Platform-specific test skipping** - Skip tests that can't run on certain platforms