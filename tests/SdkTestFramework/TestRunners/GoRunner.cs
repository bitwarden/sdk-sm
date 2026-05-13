using Microsoft.Extensions.Logging;
using SdkTestFramework.Models;
using SdkTestFramework.Platform;
using SdkTestFramework.Services;

namespace SdkTestFramework.TestRunners;

/// <summary>
/// Go test runner - executes Go SDK tests directly without shell scripts
/// </summary>
public class GoRunner : BaseTestRunner
{
    private readonly string _goDir;

    public GoRunner(
        ILogger<GoRunner> logger,
        IProcessExecutor processExecutor,
        IPlatformService platformService)
        : base(logger, processExecutor, platformService)
    {
        _goDir = Path.Combine(RepoRoot, "languages", "go");
    }

    protected override string Language => "Go";

    public override async Task<TestResult> RunTestsAsync(TestConfiguration config, CancellationToken cancellationToken = default)
    {
        Logger.LogInformation("Running Go tests directly");

        try
        {
            // Check prerequisites
            await CheckGoRequirementsAsync(cancellationToken);

            // Build SDK if needed
            if (!config.NoBuild)
            {
                await BuildGoSdkAsync(cancellationToken);
            }

            // Build test command - run the test_suite.go executable directly
            var args = new List<string> { "run", "./test/test_suite.go" };
            if (config.JsonOutput)
            {
                args.Add("--json");
            }
            if (config.Verbose)
            {
                args.Add("--verbose");
            }

            // Set up environment variables using base class method
            var envVars = BuildEnvironmentVariables(config);

            Logger.LogDebug("Executing Go test suite: go {Args}", string.Join(" ", args));

            // Execute tests using base class method
            var result = await ExecuteProcessAsync(
                "go",
                string.Join(" ", args),
                _goDir,
                envVars,
                TimeSpan.FromMilliseconds(config.TimeoutMs ?? 300000),
                cancellationToken);

            // Parse JSON output if available
            if (config.JsonOutput && result.Success)
            {
                try
                {
                    return ParseGoJsonOutput(result.StandardOutput, result.Duration);
                }
                catch (Exception ex)
                {
                    Logger.LogWarning(ex, "Failed to parse JSON output, using basic result");
                }
            }

            // Use base class method for basic result
            return CreateBasicTestResult(result);
        }
        catch (Exception ex)
        {
            return CreateErrorResult(ex);
        }
    }

    private async Task CheckGoRequirementsAsync(CancellationToken cancellationToken)
    {
        Logger.LogDebug("Checking Go requirements");

        // Check for Go using base class method
        await RequireToolAsync("go", "version", cancellationToken);
    }

    private async Task BuildGoSdkAsync(CancellationToken cancellationToken)
    {
        Logger.LogInformation("Building Go SDK");

        // Download dependencies
        Logger.LogDebug("Downloading Go dependencies");
        var getResult = await ExecuteProcessAsync(
            "go",
            "get -v ./...",
            _goDir,
            null,
            TimeSpan.FromMinutes(5),
            cancellationToken);

        if (!getResult.Success)
        {
            Logger.LogWarning("Failed to download dependencies: {Error}", getResult.StandardError);
        }

        // Build the SDK
        Logger.LogDebug("Building Go SDK");
        var buildResult = await ExecuteProcessAsync(
            "go",
            "build -v ./...",
            _goDir,
            null,
            TimeSpan.FromMinutes(5),
            cancellationToken);

        if (!buildResult.Success)
        {
            throw new InvalidOperationException($"Failed to build Go SDK: {buildResult.StandardError}");
        }

        Logger.LogInformation("Go SDK built successfully");
    }

    private TestResult ParseGoJsonOutput(string jsonOutput, TimeSpan duration)
    {
        try
        {
            // Parse the test_suite.go JSON output format
            var jsonDoc = System.Text.Json.JsonDocument.Parse(jsonOutput);
            var root = jsonDoc.RootElement;

            // Get operations array
            if (!root.TryGetProperty("operations", out var operationsElement))
            {
                Logger.LogWarning("No operations found in JSON output");
                return CreateBasicTestResult(new ProcessResult
                {
                    ExitCode = 1,
                    StandardOutput = string.Empty,
                    StandardError = "No operations in JSON",
                    Duration = duration,
                    Command = "go run test_suite.go"
                });
            }

            var operations = new List<TestOperation>();
            var passedCount = 0;
            var failedCount = 0;
            var skippedCount = 0;

            foreach (var op in operationsElement.EnumerateArray())
            {
                var testOp = ParseTestOperation(op);
                if (testOp != null)
                {
                    operations.Add(testOp);
                    if (testOp.Success)
                        passedCount++;
                    else if (testOp.Message?.Contains("skipped", StringComparison.OrdinalIgnoreCase) == true)
                        skippedCount++;
                    else
                        failedCount++;
                }
            }

            // Use base class method to create result
            return CreateJsonTestResult(
                operations,
                passedCount,
                failedCount,
                skippedCount,
                duration);
        }
        catch (Exception ex)
        {
            Logger.LogError(ex, "Failed to parse Go test JSON output");
            return CreateBasicTestResult(new ProcessResult
            {
                ExitCode = 1,
                StandardOutput = string.Empty,
                StandardError = ex.Message,
                Duration = duration,
                Command = "go run test_suite.go"
            });
        }
    }

    private static TestOperation? ParseTestOperation(System.Text.Json.JsonElement operationElement)
    {
        try
        {
            // Extract values with defaults - ensure non-null
            string operationName = "unknown";
            if (operationElement.TryGetProperty("operation", out var opName))
            {
                operationName = opName.GetString() ?? "unknown";
            }

            var success = operationElement.TryGetProperty("success", out var successElem)
                && successElem.GetBoolean();

            var durationMs = operationElement.TryGetProperty("duration_ms", out var durationElem)
                ? durationElem.GetInt64()
                : 0L;

            var message = operationElement.TryGetProperty("message", out var messageElem)
                ? messageElem.GetString()
                : null;

            var error = operationElement.TryGetProperty("error", out var errorElem)
                ? errorElem.GetString()
                : null;

            // Use default message if none provided
            message ??= success ? "Test passed" : "Test failed";

            // Create with object initializer
            return new TestOperation
            {
                Operation = operationName,
                Success = success,
                DurationMs = durationMs,
                Message = message,
                Error = error
            };
        }
        catch
        {
            return null;
        }
    }
}
