using Microsoft.Extensions.Logging;
using SdkTestFramework.Models;
using SdkTestFramework.Platform;
using SdkTestFramework.Services;

namespace SdkTestFramework.TestRunners;

/// <summary>
/// Go test runner - executes Go SDK tests directly without shell scripts
/// </summary>
public class GoTestRunner : BaseTestRunner
{
    private readonly string _goDir;

    public GoTestRunner(
        ILogger<GoTestRunner> logger,
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
                    return ParseStandardJsonOutput(result.StandardOutput);
                }
                catch (Exception ex)
                {
                    Logger.LogWarning(ex, "Failed to parse JSON output, using basic result");
                    return CreateBasicTestResult(result);
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
}
