using System.Linq;
using Microsoft.Extensions.Logging;
using SdkTestFramework.Config;
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
    private string _goCommand = "go";

    public GoTestRunner(
        ILogger<GoTestRunner> logger,
        IProcessExecutor processExecutor,
        IPlatformService platformService,
        TestConfig testConfig)
        : base(logger, processExecutor, platformService, testConfig)
    {
        _goDir = Path.Join(RepoRoot, "languages", "go");
        InitializeGoCommand();
    }

    private void InitializeGoCommand()
    {
        // Try common Go installation paths if go is not in PATH
        var commonPaths = new[]
        {
            "/opt/homebrew/bin/go",     // Homebrew on Apple Silicon
            "/usr/local/bin/go",         // Common installation path
            "/usr/local/go/bin/go",      // Default Go installation
            "/usr/bin/go"                // System installation
        };

        var goPath = commonPaths.FirstOrDefault(File.Exists);
        if (!string.IsNullOrEmpty(goPath))
        {
            Logger.LogInformation("Found go at {Path}, will use full path for execution", goPath);
            _goCommand = goPath;
            return;
        }

        // If not found in common paths, assume it might be in PATH
        // (will be verified in CheckGoRequirementsAsync)
        Logger.LogDebug("Go not found in common paths, will try PATH");
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

            Logger.LogInformation("Executing Go test suite with command: {Command} {Args}",
                _goCommand, string.Join(" ", args));
            Logger.LogDebug("Working directory: {Dir}", _goDir);
            Logger.LogDebug("Environment variables set: {Count}", envVars.Count);

            // Execute tests with environment variables
            var result = await ExecuteGoDirectWithEnvAsync(
                _goCommand,
                string.Join(" ", args),
                _goDir,
                envVars,
                cancellationToken);

            // Parse JSON output if available
            if (!config.JsonOutput || !result.Success)
            {
                return CreateBasicTestResult(result);
            }

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
        catch (Exception ex)
        {
            return CreateErrorResult(ex);
        }
    }

    private async Task CheckGoRequirementsAsync(CancellationToken cancellationToken)
    {
        Logger.LogInformation("Checking Go requirements...");
        Logger.LogDebug("Using go command: {Command}", _goCommand);

        // Verify the go command works by executing directly without shell wrapper
        var goWorks = await ExecuteGoDirectAsync(_goCommand, "version", null, cancellationToken);

        if (!goWorks.Success)
        {
            // If the pre-configured command doesn't work, it means:
            // 1. None of the common paths had go
            // 2. "go" is not in PATH
            Logger.LogError("go is required but not found. Tried command: {Command}", _goCommand);
            Logger.LogError("Checked common installation locations: /opt/homebrew/bin/go, /usr/local/bin/go, /usr/local/go/bin/go, /usr/bin/go");
            Logger.LogError("Error output: {Error}", goWorks.StandardError);
            throw new InvalidOperationException($"go is required but not found (tried command: {_goCommand})");
        }

        Logger.LogInformation("Go verified successfully using: {Command}", _goCommand);
        Logger.LogDebug("Go version output: {Output}", goWorks.StandardOutput.Trim());
    }

    /// <summary>
    /// Execute Go command directly without shell wrapper
    /// </summary>
    private async Task<ProcessResult> ExecuteGoDirectAsync(
        string goCommand,
        string arguments,
        string? workingDirectory,
        CancellationToken cancellationToken)
    {
        return await ExecuteGoDirectWithEnvAsync(goCommand, arguments, workingDirectory, null, cancellationToken);
    }

    /// <summary>
    /// Execute Go command directly without shell wrapper with environment variables
    /// </summary>
    private async Task<ProcessResult> ExecuteGoDirectWithEnvAsync(
        string goCommand,
        string arguments,
        string? workingDirectory,
        Dictionary<string, string>? environmentVariables,
        CancellationToken cancellationToken)
    {
        var startInfo = new System.Diagnostics.ProcessStartInfo
        {
            FileName = goCommand,
            Arguments = arguments,
            UseShellExecute = false,
            CreateNoWindow = true,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            WorkingDirectory = workingDirectory ?? _goDir
        };

        // Add environment variables if provided
        if (environmentVariables != null)
        {
            foreach (var (key, value) in environmentVariables)
            {
                startInfo.Environment[key] = value;
            }
        }

        using var process = new System.Diagnostics.Process();
        process.StartInfo = startInfo;
        var sw = System.Diagnostics.Stopwatch.StartNew();

        try
        {
            Logger.LogDebug("Executing Go directly: {Command} {Args} in {Dir}",
                goCommand, arguments, startInfo.WorkingDirectory);

            process.Start();

            // Wait for process with timeout
            using var cts = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
            cts.CancelAfter(TimeSpan.FromMilliseconds(Config.Timeouts.DefaultTimeoutMs));

            // Read output with cancellation support
            var outputTask = process.StandardOutput.ReadToEndAsync(cts.Token);
            var errorTask = process.StandardError.ReadToEndAsync(cts.Token);

            await process.WaitForExitAsync(cts.Token);

            var output = await outputTask;
            var error = await errorTask;

            sw.Stop();

            return new ProcessResult
            {
                ExitCode = process.ExitCode,
                StandardOutput = output,
                StandardError = error,
                Duration = sw.Elapsed,
                Command = $"{goCommand} {arguments}"
            };
        }
        catch (Exception ex)
        {
            sw.Stop();
            Logger.LogError(ex, "Failed to execute Go command: {Command} {Args}", goCommand, arguments);
            return new ProcessResult
            {
                ExitCode = -1,
                StandardOutput = string.Empty,
                StandardError = ex.Message,
                Duration = sw.Elapsed,
                Command = $"{goCommand} {arguments}"
            };
        }
    }

    private async Task BuildGoSdkAsync(CancellationToken cancellationToken)
    {
        Logger.LogInformation("Building Go SDK using command: {Command}", _goCommand);

        // Download dependencies
        Logger.LogInformation("Downloading Go dependencies with: {Command} get -v ./...", _goCommand);
        var getResult = await ExecuteGoDirectAsync(
            _goCommand,
            "get -v ./...",
            _goDir,
            cancellationToken);

        if (!getResult.Success)
        {
            Logger.LogWarning("Failed to download dependencies: {Error}", getResult.StandardError);
        }

        // Build the SDK
        Logger.LogInformation("Building Go SDK with: {Command} build -v ./...", _goCommand);
        var buildResult = await ExecuteGoDirectAsync(
            _goCommand,
            "build -v ./...",
            _goDir,
            cancellationToken);

        if (!buildResult.Success)
        {
            throw new InvalidOperationException($"Failed to build Go SDK: {buildResult.StandardError}");
        }

        Logger.LogInformation("Go SDK built successfully");
    }
}
