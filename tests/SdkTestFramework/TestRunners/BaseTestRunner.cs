using System.Text.Json;
using Microsoft.Extensions.Logging;
using SdkTestFramework.Common;
using SdkTestFramework.Config;
using SdkTestFramework.Models;
using SdkTestFramework.Platform;
using SdkTestFramework.Services;

namespace SdkTestFramework.TestRunners;

/// <summary>
/// Base class for all test runners with common utilities to avoid code duplication
/// </summary>
public abstract class BaseTestRunner(
    ILogger logger,
    IProcessExecutor processExecutor,
    IPlatformService platformService,
    TestConfig testConfig)
{
    protected readonly ILogger Logger = logger ?? throw new ArgumentNullException(nameof(logger));
    private readonly IProcessExecutor _processExecutor = processExecutor ?? throw new ArgumentNullException(nameof(processExecutor));
    protected readonly IPlatformService PlatformService = platformService ?? throw new ArgumentNullException(nameof(platformService));
    protected readonly TestConfig Config = testConfig ?? throw new ArgumentNullException(nameof(testConfig));
    protected readonly string RepoRoot = PathUtilities.FindRepositoryRoot(Directory.GetCurrentDirectory())
                                         ?? throw new InvalidOperationException("Could not find repository root");

    /// <summary>
    /// Language name for this runner
    /// </summary>
    protected abstract string Language { get; }

    /// <summary>
    /// Run tests for this language
    /// </summary>
    public abstract Task<TestResult> RunTestsAsync(TestConfiguration config, CancellationToken cancellationToken = default);


    /// <summary>
    /// Build standard environment variables for test execution
    /// </summary>
    protected static Dictionary<string, string> BuildEnvironmentVariables(TestConfiguration config)
    {
        var envVars = new Dictionary<string, string>
        {
            ["TEST_MODE"] = config.TestMode
        };

        // Copy existing environment variables if present
        var envVarNames = new[] {
            "API_URL",
            "IDENTITY_URL",
            "ORGANIZATION_ID",
            "ACCESS_TOKEN",
            "STATE_FILE"
        };
        foreach (var varName in envVarNames)
        {
            var value = Environment.GetEnvironmentVariable(varName);
            if (!string.IsNullOrEmpty(value))
                envVars[varName] = value;
        }

        return envVars;
    }

    /// <summary>
    /// Check if a required tool is installed
    /// </summary>
    protected async Task<bool> CheckToolAsync(string toolName, string versionArgument, CancellationToken cancellationToken)
    {
        Logger.LogDebug("Checking for {Tool} with argument: {Argument}", toolName, versionArgument);

        var result = await _processExecutor.ExecuteAsync(
            new ProcessRequest
            {
                Command = toolName,
                Arguments = versionArgument,
                Timeout = TimeSpan.FromMilliseconds(Config.Timeouts.ToolCheckTimeoutMs),
                ThrowOnError = false  // Don't throw if tool is not found
            },
            cancellationToken);

        if (result.Success)
        {
            var output = result.StandardOutput.Split('\n');
            Logger.LogDebug("{Tool} found: {Output}", toolName, output.Length > 0 ? output[0] : "(no output)");
        }
        else
        {
            Logger.LogWarning("{Tool} not found or failed to execute. Exit code: {ExitCode}, StdErr: {StdErr}",
                toolName, result.ExitCode, result.StandardError);
        }

        return result.Success;
    }

    /// <summary>
    /// Ensure a required tool is installed, throw if not
    /// </summary>
    protected async Task RequireToolAsync(string toolName, string versionArgument, CancellationToken cancellationToken)
    {
        if (!await CheckToolAsync(toolName, versionArgument, cancellationToken))
        {
            throw new InvalidOperationException($"{toolName} is required but not found");
        }
    }

    /// <summary>
    /// Create a basic test result (non-JSON output)
    /// </summary>
    protected TestResult CreateBasicTestResult(ProcessResult processResult)
    {
        return new TestResult
        {
            Language = Language,
            Success = processResult.Success,
            TotalTests = 0,
            PassedTests = 0,
            FailedTests = 0,
            SkippedTests = 0,
            Duration = processResult.Duration,
            TotalDurationMs = (long)processResult.Duration.TotalMilliseconds,
            Error = processResult.Success ? null : processResult.StandardError,
            Operations = new List<TestOperation>(),
            Timestamp = DateTime.UtcNow,
            OperatingSystem = PlatformService.OperatingSystem.ToString(),
            Architecture = PlatformService.Architecture.ToString()
        };
    }

    /// <summary>
    /// Create a test result from parsed JSON data
    /// </summary>
    private TestResult CreateJsonTestResult(
        List<TestOperation> operations,
        int passedCount,
        int failedCount,
        int skippedCount,
        TimeSpan duration)
    {
        return new TestResult
        {
            Language = Language,
            Success = failedCount == 0,
            TotalTests = passedCount + failedCount + skippedCount,
            PassedTests = passedCount,
            FailedTests = failedCount,
            SkippedTests = skippedCount,
            Duration = duration,
            TotalDurationMs = (long)duration.TotalMilliseconds,
            Operations = operations,
            Timestamp = DateTime.UtcNow,
            OperatingSystem = PlatformService.OperatingSystem.ToString(),
            Architecture = PlatformService.Architecture.ToString()
        };
    }

    /// <summary>
    /// Create an error result from an exception
    /// </summary>
    protected TestResult CreateErrorResult(Exception ex)
    {
        Logger.LogError(ex, "Test execution failed");

        return new TestResult
        {
            Language = Language,
            Success = false,
            Error = ex.Message,
            TotalTests = 0,
            PassedTests = 0,
            FailedTests = 0,
            SkippedTests = 0,
            Duration = TimeSpan.Zero,
            TotalDurationMs = 0,
            Operations = new List<TestOperation>(),
            Timestamp = DateTime.UtcNow,
            OperatingSystem = PlatformService.OperatingSystem.ToString(),
            Architecture = PlatformService.Architecture.ToString()
        };
    }

    /// <summary>
    /// Execute a process with standard configuration
    /// </summary>
    protected async Task<ProcessResult> ExecuteProcessAsync(
        string command,
        string arguments,
        string? workingDirectory,
        Dictionary<string, string>? environmentVariables,
        TimeSpan? timeout,
        CancellationToken cancellationToken,
        bool throwOnError = true)
    {
        Logger.LogDebug("Executing: {Command} {Arguments}", command, arguments);

        return await _processExecutor.ExecuteAsync(
            new ProcessRequest
            {
                Command = command,
                Arguments = arguments,
                WorkingDirectory = workingDirectory ?? RepoRoot,
                EnvironmentVariables = environmentVariables,
                Timeout = timeout ?? TimeSpan.FromMilliseconds(Config.Timeouts.DefaultTimeoutMs),
                ThrowOnError = throwOnError
            },
            cancellationToken);
    }

    /// <summary>
    /// Check if a file exists, with logging
    /// </summary>
    protected bool CheckFileExists(string path, string description)
    {
        if (File.Exists(path))
        {
            Logger.LogDebug("{Description} found at: {Path}", description, path);
            return true;
        }

        Logger.LogWarning("{Description} not found at: {Path}", description, path);
        return false;
    }

    /// <summary>
    /// Parse standard JSON test output format from language test suites
    /// </summary>
    protected TestResult ParseStandardJsonOutput(string jsonOutput)
    {
        try
        {
            var jsonDoc = JsonDocument.Parse(jsonOutput);
            var root = jsonDoc.RootElement;

            // Parse the operations array
            var operations = new List<TestOperation>();
            var passedCount = 0;
            var failedCount = 0;
            var skippedCount = 0;

            if (root.TryGetProperty("operations", out var operationsElement) &&
                operationsElement.ValueKind == JsonValueKind.Array)
            {
                foreach (var op in operationsElement.EnumerateArray())
                {
                    var testOp = ParseTestOperation(op);
                    if (testOp == null) continue;

                    operations.Add(testOp);
                    UpdateCounters(op, testOp, ref passedCount, ref failedCount, ref skippedCount);
                }
            }

            // Get total duration if available
            var totalDurationMs = root.TryGetProperty("total_duration_ms", out var durationElement)
                ? durationElement.GetInt64()
                : operations.Sum(op => op.DurationMs);

            return CreateJsonTestResult(
                operations,
                passedCount,
                failedCount,
                skippedCount,
                TimeSpan.FromMilliseconds(totalDurationMs));
        }
        catch (Exception ex)
        {
            Logger.LogError(ex, "Failed to parse JSON test output");
            throw new InvalidOperationException($"Failed to parse test output: {ex.Message}", ex);
        }
    }

    /// <summary>
    /// Update test counters based on operation status
    /// </summary>
    private static void UpdateCounters(JsonElement op, TestOperation testOp, ref int passedCount, ref int failedCount, ref int skippedCount)
    {
        var status = GetJsonString(op, "status");

        if (status == "skipped")
            skippedCount++;
        else if (testOp.Success)
            passedCount++;
        else
            failedCount++;
    }

    /// <summary>
    /// Parse a single test operation from JSON
    /// </summary>
    private TestOperation? ParseTestOperation(JsonElement operationElement)
    {
        try
        {
            var operationName = GetJsonString(operationElement, "operation") ?? "unknown";
            var success = operationElement.TryGetProperty("success", out var successElem) && successElem.GetBoolean();
            var durationMs = GetJsonInt64(operationElement, "duration_ms");
            var error = GetJsonString(operationElement, "error");

            // Get message from details object if available
            string? message = null;
            if (operationElement.TryGetProperty("details", out var details))
            {
                message = GetJsonString(details, "message");
            }

            // Use default message if none provided
            message ??= success ? "Test passed" : "Test failed";

            return new TestOperation
            {
                Operation = operationName,
                Success = success,
                DurationMs = durationMs,
                Message = message,
                Error = error
            };
        }
        catch (Exception ex)
        {
            Logger.LogWarning(ex, "Failed to parse test operation from JSON element");
            return null;
        }
    }

    /// <summary>
    /// Safely get a string value from a JSON element
    /// </summary>
    private static string? GetJsonString(JsonElement element, string propertyName, string? defaultValue = null)
    {
        return element.TryGetProperty(propertyName, out var prop) ? prop.GetString() ?? defaultValue : defaultValue;
    }

    /// <summary>
    /// Safely get an int64 value from a JSON element
    /// </summary>
    private static long GetJsonInt64(JsonElement element, string propertyName, long defaultValue = 0)
    {
        return element.TryGetProperty(propertyName, out var prop) && prop.TryGetInt64(out var value) ? value : defaultValue;
    }
}
