using System.Text.Json;
using SdkTestFramework.Config;
using SdkTestFramework.Models;

namespace SdkTestFramework.Runners;

/// <summary>
/// Base class for language-specific test runners
/// </summary>
public abstract class BaseRunner
{
    private const int MaxOutputSize = 1024 * 100; // 100KB max output
    private const string UnknownSdkVersion = "unknown";

    protected readonly TestConfig _config;
    protected readonly ProcessRunner ProcessRunner;
    protected readonly Dictionary<string, string> _environmentVariables;

    protected BaseRunner(TestConfig config, ProcessRunner processRunner)
    {
        _config = config ?? throw new ArgumentNullException(nameof(config));
        ProcessRunner = processRunner ?? throw new ArgumentNullException(nameof(processRunner));
        _environmentVariables = LoadEnvironmentVariables();
    }

    /// <summary>
    /// Language name for reporting
    /// </summary>
    protected abstract string Language { get; }

    /// <summary>
    /// Check if this runner is supported on the current platform
    /// </summary>
    protected abstract bool IsSupportedOnCurrentPlatform();

    /// <summary>
    /// Get the test script path for this language
    /// </summary>
    protected abstract string GetTestScriptPath();

    /// <summary>
    /// Get the command to execute the test script
    /// </summary>
    protected abstract string GetExecuteCommand();

    /// <summary>
    /// Get command arguments for executing the test
    /// </summary>
    protected virtual string[] GetExecuteArguments()
    {
        // Always pass --json flag to get JSON output for parsing
        return [GetTestScriptPath(), "--json"];
    }

    /// <summary>
    /// Verify prerequisites for running tests
    /// </summary>
    public virtual async Task<bool> VerifyPrerequisites()
    {
        try
        {
            // Check if bash is available
            var bashCheck = await CheckBashInstalled();
            if (!bashCheck)
            {
                var osContext = OsDetector.GetCurrentOsContext();
                if (osContext.IsWindows)
                {
                    Console.WriteLine("Bash not found. Please install Git Bash, WSL, or another bash shell for Windows.");
                }
                else
                {
                    Console.WriteLine("Bash not found. This is unusual for Linux/macOS. Please install bash.");
                }
                return false;
            }

            // Check if test.sh exists
            var scriptPath = GetTestScriptPath();
            if (!File.Exists(scriptPath))
            {
                Console.WriteLine($"Test script not found: {scriptPath}");
                return false;
            }

            // The test.sh script will handle:
            // - Checking if the language runtime is installed
            // - Building the SDK based on SDK_SOURCE configuration
            // - Running the appropriate tests

            return true;
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Prerequisites check failed: {ex.Message}");
            return false;
        }
    }

    /// <summary>
    /// Run the tests for this language
    /// </summary>
    public async Task<TestResult> RunTests()
    {
        if (!IsSupportedOnCurrentPlatform())
        {
            return CreateUnsupportedResult();
        }

        var prereqCheck = await VerifyPrerequisites();
        if (!prereqCheck)
        {
            return CreatePrerequisiteFailureResult();
        }

        return await ExecuteTests();
    }

    /// <summary>
    /// Execute the actual test script
    /// </summary>
    /// <summary>
    /// Check if bash is installed and available
    /// </summary>
    protected async Task<bool> CheckBashInstalled()
    {
        var result = await ProcessRunner.RunAsync(
            "bash",
            "--version",
            workingDirectory: null,
            [],
            5000
        );

        if (result.Success)
        {
            Console.WriteLine($"Found bash: {result.Output.Split('\n')[0]}");
        }

        return result.Success;
    }

    /// <summary>
    /// Get the SDK base path by looking for markers like Cargo.toml
    /// </summary>
    protected static string GetSdkBasePath()
    {
        // Navigate to SDK root directory
        var currentDir = Directory.GetCurrentDirectory();

        // Look for markers that indicate we're in the SDK root
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

        // Fallback to current directory
        return Directory.GetCurrentDirectory();
    }

    /// <summary>
    /// Get the working directory for test execution.
    /// Override this in derived classes if a specific working directory is needed.
    /// </summary>
    protected virtual string? GetWorkingDirectory() => null;

    protected virtual async Task<TestResult> ExecuteTests()
    {
        var command = GetExecuteCommand();
        var arguments = GetExecuteArguments();
        // Use longer timeout when building SDK, as builds can take much longer than tests
        var timeout = _config.Configuration.BuildSdk
            ? _config.Timeouts.BuildTimeoutMs
            : _config.Timeouts.DefaultTimeoutMs;
        var workingDir = GetWorkingDirectory();

        Console.WriteLine($"📋 Executing {Language} tests...");
        Console.WriteLine($"   Command: {command}");
        Console.WriteLine($"   Script: {string.Join(" ", arguments)}");
        Console.WriteLine($"   Timeout: {timeout}ms");
        Console.WriteLine($"   Working Directory: {workingDir ?? Directory.GetCurrentDirectory()}");

        try
        {
            Console.WriteLine($"🚀 Starting test process...");
            var result = await ProcessRunner.RunAsync(
                command,
                string.Join(" ", arguments),
                workingDir,
                _environmentVariables,
                timeout
            );

            Console.WriteLine($"⏱️ Process completed in {result.DurationMs}ms");

            if (!result.Success)
            {
                Console.WriteLine($"❌ Process failed with exit code: {result.ExitCode}");
                // Include both stdout and stderr for better debugging
                var errorDetails = $"{result.Error}";
                if (!string.IsNullOrWhiteSpace(result.Output))
                {
                    errorDetails += $"\nOutput: {result.Output}";
                }
                return CreateExecutionFailureResult(errorDetails);
            }

            Console.WriteLine($"✅ Process completed successfully, parsing output...");
            return ParseTestOutput(result.Output);
        }
        catch (Exception ex)
        {
            return CreateExceptionResult(ex);
        }
    }

    /// <summary>
    /// Parse JSON output from test script
    /// </summary>
    private TestResult ParseTestOutput(string output)
    {
        try
        {
            // Truncate output if too large
            if (output.Length > MaxOutputSize)
            {
                output = output.Substring(0, MaxOutputSize);
            }

            var options = new JsonSerializerOptions
            {
                PropertyNameCaseInsensitive = true
            };

            var result = JsonSerializer.Deserialize<TestResult>(output, options);
            return result ?? throw new InvalidOperationException("Failed to deserialize test result");
        }
        catch (JsonException ex)
        {
            return CreateParseFailureResult(ex, output);
        }
    }

    /// <summary>
    /// Load environment variables for test execution
    /// </summary>
    private Dictionary<string, string> LoadEnvironmentVariables()
    {
        var vars = new Dictionary<string, string>(); // Will be populated below

        // Add required environment variables
        AddEnvironmentVariable(vars, "ACCESS_TOKEN");
        AddEnvironmentVariable(vars, "ORGANIZATION_ID");
        AddEnvironmentVariable(vars, "API_URL");
        AddEnvironmentVariable(vars, "IDENTITY_URL");
        AddEnvironmentVariable(vars, "STATE_FILE");
        AddEnvironmentVariable(vars, "TEST_MODE", _config.Configuration.TestMode);
        AddEnvironmentVariable(vars, "SDK_SOURCE", _config.Configuration.SdkSource);
        AddEnvironmentVariable(vars, "PYTHON_VERSIONS", _config.Configuration.PythonVersion);

        // Add optional environment variables
        AddEnvironmentVariable(vars, "DYLD_LIBRARY_PATH");
        AddEnvironmentVariable(vars, "OBJC_DISABLE_INITIALIZE_FORK_SAFETY");

        // Add PATH with cargo bin directory if it exists
        var currentPath = Environment.GetEnvironmentVariable("PATH") ?? "";
        var cargoPath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cargo", "bin");
        if (Directory.Exists(cargoPath) && !currentPath.Contains(cargoPath))
        {
            vars["PATH"] = $"{cargoPath}:{currentPath}";
        }
        else
        {
            AddEnvironmentVariable(vars, "PATH");
        }

        return vars;
    }

    private static void AddEnvironmentVariable(
        Dictionary<string, string> vars,
        string name,
        string? defaultValue = null)
    {
        var value = Environment.GetEnvironmentVariable(name) ?? defaultValue;
        if (!string.IsNullOrWhiteSpace(value))
        {
            vars[name] = value;
        }
    }

    // Result creation methods for various failure scenarios
    private TestResult CreateBaseResult(List<TestOperation>? operations = null)
    {
        var osContext = OsDetector.GetCurrentOsContext();
        return new TestResult
        {
            Language = Language,
            SdkVersion = UnknownSdkVersion,
            Operations = operations ?? [],
            TotalDurationMs = 0,
            OperatingSystem = osContext.OsDisplayName,
            Architecture = osContext.Architecture,
            OperatingSystemVersion = osContext.Version,
            Timestamp = DateTime.UtcNow
        };
    }

    private TestResult CreateUnsupportedResult()
    {
        return CreateBaseResult();
    }

    private TestResult CreatePrerequisiteFailureResult()
    {
        // Create a failed operation to indicate prerequisite failure
        var failedOp = new TestOperation
        {
            Operation = "prerequisite_check",
            Success = false,
            Error = "Prerequisites not met",
            DurationMs = 0,
            Details = []
        };
        return CreateBaseResult([failedOp]);
    }

    private TestResult CreateExecutionFailureResult(string error)
    {
        // Create a failed operation to show the error
        var failedOp = new TestOperation
        {
            Operation = "test_execution",
            Success = false,
            Error = $"Execution failed: {error}",
            DurationMs = 0,
            Details = []
        };

        return CreateBaseResult([failedOp]);
    }

    private TestResult CreateExceptionResult(Exception ex)
    {
        // Create a failed operation to show the exception with full details
        var errorMessage = $"Exception: {ex.Message}";
        if (ex.InnerException != null)
        {
            errorMessage += $"\nInner Exception: {ex.InnerException.Message}";
        }

        var failedOp = new TestOperation
        {
            Operation = "test_execution",
            Success = false,
            Error = errorMessage,
            DurationMs = 0,
            Details = []
        };

        return CreateBaseResult([failedOp]);
    }

    private TestResult CreateParseFailureResult(JsonException ex, string output)
    {
        // Create a failed operation to show the parse error
        var failedOp = new TestOperation
        {
            Operation = "test_execution",
            Success = false,
            Error = $"Failed to parse output: {ex.Message}. Output: {output.Substring(0, Math.Min(500, output.Length))}",
            DurationMs = 0,
            Details = []
        };

        return CreateBaseResult([failedOp]);
    }
}
