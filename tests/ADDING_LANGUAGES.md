# Adding New Language Support

This guide explains how to add support for a new language to the SDK Test Framework.

## Overview

Adding a new language requires:
1. Creating a test script in the language's directory
2. Implementing a runner class in C#
3. Updating configuration
4. Testing the implementation

## Step 1: Create Test Script

Create a test script at `languages/<language>/test/tests.<ext>` that implements these 6 operations:

1. **auth** - Authenticate with access token
2. **create_secret** - Create a new secret
3. **list_secrets** - List all secrets
4. **get_secret** - Get a specific secret
5. **delete_secret** - Delete a secret
6. **sync** - Sync secrets with/without date

### Required Output Format

The test script MUST output JSON in this exact format:

```json
{
  "language": "your-language",
  "sdk_version": "1.0.0",
  "operations": [
    {
      "operation": "auth",
      "success": true,
      "duration_ms": 150,
      "error": null,
      "details": {
        "method": "access_token"
      }
    },
    // ... other operations
  ],
  "total_duration_ms": 1500,
  "os": "macos",
  "architecture": "arm64",
  "timestamp": "2024-01-01T12:00:00Z"
}
```

### Environment Variables

Your test script will receive these environment variables:
- `ACCESS_TOKEN` - Authentication token
- `ORGANIZATION_ID` - Organization ID for operations
- `API_URL` - API endpoint URL
- `IDENTITY_URL` - Identity service URL
- `STATE_FILE` - Optional path for state persistence
- `TEST_MODE` - Either "fake-server" or "real-server"

### Exit Codes

- Exit with code `0` if all tests pass
- Exit with code `1` if any test fails

### Example Structure (Python)

```python
class LanguageSdkTester:
    def test_auth(self):
        # Authenticate with ACCESS_TOKEN
        return TestOperation("auth", success=True, duration_ms=100)

    def test_create_secret(self):
        # Create secret in ORGANIZATION_ID
        return TestOperation("create_secret", success=True, duration_ms=200)

    # ... other operations

    def run_all_tests(self):
        operations = [
            self.test_auth(),
            self.test_create_secret(),
            # ... etc
        ]
        return {
            "language": "python",
            "operations": operations,
            # ... other fields
        }

if __name__ == "__main__":
    tester = LanguageSdkTester()
    results = tester.run_all_tests()
    print(json.dumps(results))
    sys.exit(0 if all_passed else 1)
```

## Step 2: Create Runner Class

Create a runner class at `tests/SdkTestFramework/Runners/<Language>Runner.cs`:

```csharp
using SdkTestFramework.Config;

namespace SdkTestFramework.Runners;

/// <summary>
/// Test runner for [Language] SDK
/// </summary>
public class [Language]Runner : BaseRunner
{
    private const string COMMAND = "language-executable";
    private const string TEST_SCRIPT_PATH = "languages/[language]/test/tests.ext";

    public [Language]Runner(TestConfig config, ProcessRunner processRunner)
        : base(config, processRunner)
    {
    }

    public override string Language => "[language]";

    public override bool IsSupportedOnCurrentPlatform()
    {
        // Return true if supported on current OS
        // Use OsDetector.IsWindows(), OsDetector.IsMacOS(), OsDetector.IsLinux()
        return true;
    }

    protected override string GetTestScriptPath()
    {
        var basePath = GetSdkBasePath();
        return Path.Combine(basePath, TEST_SCRIPT_PATH);
    }

    protected override string GetExecuteCommand()
    {
        return COMMAND;
    }

    public override async Task<bool> VerifyPrerequisites()
    {
        try
        {
            // 1. Check if language runtime is installed
            var runtimeCheck = await CheckRuntimeInstalled();
            if (!runtimeCheck)
            {
                Console.WriteLine($"[Language] runtime not found. Please install [Language].");
                return false;
            }

            // 2. Check if test script exists
            var scriptPath = GetTestScriptPath();
            if (!File.Exists(scriptPath))
            {
                Console.WriteLine($"Test script not found: {scriptPath}");
                return false;
            }

            // 3. Check if SDK is built (for local-build mode)
            if (config.Configuration.IsLocalBuild())
            {
                var sdkCheck = await CheckSdkBuilt();
                if (!sdkCheck)
                {
                    Console.WriteLine("[Language] SDK not built.");
                    return false;
                }
            }

            return true;
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Prerequisites check failed: {ex.Message}");
            return false;
        }
    }

    private async Task<bool> CheckRuntimeInstalled()
    {
        var result = await processRunner.RunAsync(
            COMMAND,
            new[] { "--version" },
            new Dictionary<string, string>(),
            5000
        );

        if (result.Success)
        {
            Console.WriteLine($"Found [Language]: {result.Output.Trim()}");
        }

        return result.Success;
    }

    private async Task<bool> CheckSdkBuilt()
    {
        // Implement SDK build verification
        // This varies by language - check for built artifacts, libraries, etc.
        return true;
    }

    private string GetSdkBasePath()
    {
        var currentDir = Directory.GetCurrentDirectory();
        while (!string.IsNullOrEmpty(currentDir))
        {
            if (File.Exists(Path.Combine(currentDir, "Cargo.toml")) &&
                Directory.Exists(Path.Combine(currentDir, "languages")))
            {
                return currentDir;
            }
            var parent = Directory.GetParent(currentDir);
            if (parent == null) break;
            currentDir = parent.FullName;
        }
        return Directory.GetCurrentDirectory();
    }
}
```

## Step 3: Register Runner in Orchestrator

Update `TestOrchestrator.cs` to include your language:

```csharp
private Dictionary<string, BaseRunner> InitializeRunners()
{
    var runnerMap = new Dictionary<string, BaseRunner>();

    if (IsLanguageEnabled("python"))
    {
        runnerMap["python"] = new PythonRunner(config, processRunner);
    }

    if (IsLanguageEnabled("go"))
    {
        runnerMap["go"] = new GoRunner(config, processRunner);
    }

    // Add your language here
    if (IsLanguageEnabled("[language]"))
    {
        runnerMap["[language]"] = new [Language]Runner(config, processRunner);
    }

    return runnerMap;
}
```

## Step 4: Update Configuration

Add your language to `tests/test-config.json`:

```json
{
  "configuration": {
    "ENABLED_LANGUAGES": ["python", "go", "[language]"]
  }
}
```

## Step 5: Update GitHub Actions Workflow

Add language setup to `.github/workflows/sdk-tests.yml`:

```yaml
# Setup [Language]
- name: Setup [Language]
  uses: actions/setup-[language]@v[version]
  with:
    [language]-version: ${{ env.[LANGUAGE]_VERSION }}

# Build [Language] SDK (if needed)
- name: Build [Language] SDK
  if: ${{ github.event.inputs.sdk_source != 'published-package' }}
  run: |
    # Add build commands for your language
    cd languages/[language]
    # Build commands here
```

## Testing Your Implementation

### Local Testing

1. **Test the script directly**:
   ```bash
   cd languages/[language]/test
   export ACCESS_TOKEN=test-token
   export ORGANIZATION_ID=test-org
   export API_URL=http://localhost:4000
   export IDENTITY_URL=http://localhost:33656
   export TEST_MODE=fake-server
   [language] tests.[ext]
   ```

2. **Test through the framework**:
   ```bash
   cd tests/SdkTests
   dotnet run
   ```

3. **Run NUnit tests**:
   ```bash
   cd tests/SdkTests
   dotnet test
   ```

### Debugging Tips

- Check that JSON output is valid: `[language] tests.[ext] | jq .`
- Verify exit codes: `echo $?` after running
- Test with fake-server first before real server
- Add debug logging to your test script
- Use `TEST_MODE=fake-server` for initial development

## Common Patterns

### Handling Platform Differences

```csharp
protected override string GetExecuteCommand()
{
    if (OsDetector.IsWindows())
        return "language.exe";
    return "language";
}
```

### Supporting Multiple Versions

```csharp
private string GetCommandByVersion()
{
    // Try versioned commands first
    var commands = new[] { "language3", "language" };
    foreach (var cmd in commands)
    {
        if (CommandExists(cmd))
            return cmd;
    }
    throw new InvalidOperationException("Language runtime not found");
}
```

### Handling Virtual Environments

For languages with virtual environments (Python, Ruby, etc.):

```csharp
protected override async Task<TestResult> ExecuteTests()
{
    // Activate virtual environment if needed
    var activateScript = GetActivationScript();
    if (File.Exists(activateScript))
    {
        // Run with activation
    }
    return await base.ExecuteTests();
}
```

## Checklist

- [ ] Test script created at `languages/[language]/test/tests.[ext]`
- [ ] Test script outputs valid JSON
- [ ] Test script exits with appropriate code
- [ ] Runner class created at `tests/SdkTestFramework/Runners/[Language]Runner.cs`
- [ ] Runner registered in `TestOrchestrator.cs`
- [ ] Language added to `test-config.json`
- [ ] GitHub Actions workflow updated
- [ ] Local testing successful
- [ ] Documentation updated

## Need Help?

- Review existing implementations: `PythonRunner.cs`, `GoRunner.cs`
- Check test script examples: `languages/python/test/tests.py`, `languages/go/test/tests.go`
- Ensure JSON output matches the schema exactly
- Test with fake-server first for easier debugging