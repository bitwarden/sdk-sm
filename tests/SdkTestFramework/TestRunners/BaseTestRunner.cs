using Microsoft.Extensions.Logging;
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
    IPlatformService platformService)
{
    protected readonly ILogger Logger = logger ?? throw new ArgumentNullException(nameof(logger));
    private readonly IProcessExecutor _processExecutor = processExecutor ?? throw new ArgumentNullException(nameof(processExecutor));
    protected readonly IPlatformService PlatformService = platformService ?? throw new ArgumentNullException(nameof(platformService));
    protected readonly string RepoRoot = FindRepoRoot(Directory.GetCurrentDirectory())
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
    /// Find the repository root by looking for .git directory
    /// </summary>
    private static string? FindRepoRoot(string startPath)
    {
        var dir = new DirectoryInfo(startPath);
        while (dir != null)
        {
            if (Directory.Exists(Path.Combine(dir.FullName, ".git")))
            {
                return dir.FullName;
            }
            dir = dir.Parent;
        }
        return null;
    }

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
        var envVarNames = new[] { "API_URL", "IDENTITY_URL", "ORGANIZATION_ID", "ACCESS_TOKEN", "STATE_FILE" };
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
                Timeout = TimeSpan.FromSeconds(5),
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
    protected TestResult CreateJsonTestResult(
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
                Timeout = timeout ?? TimeSpan.FromMinutes(5),
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
}
