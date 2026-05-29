using System.Runtime.InteropServices;
using Microsoft.Extensions.Logging;
using SdkTestFramework.Config;
using SdkTestFramework.Models;
using SdkTestFramework.Platform;
using SdkTestFramework.Services;

namespace SdkTestFramework.TestRunners;

/// <summary>
/// Python test runner - executes Python SDK tests directly without shell scripts
/// </summary>
public class PythonTestRunner : BaseTestRunner
{
    private const string PythonCommand = "python3";

    private readonly string _pythonDir;
    private readonly string _venvDir;
    private readonly string _pythonExecutable;
    private readonly string _pipExecutable;

    public PythonTestRunner(
        ILogger<PythonTestRunner> logger,
        IProcessExecutor processExecutor,
        IPlatformService platformService,
        TestConfig testConfig)
        : base(logger, processExecutor, platformService, testConfig)
    {
        _pythonDir = Path.Combine(RepoRoot, "languages", "python");

        // Set up virtual environment paths
        var tempDir = Path.GetTempPath();
        _venvDir = Path.Combine(tempDir, $".venv-{PythonCommand}");

        // Platform-specific Python and pip paths in venv
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
        {
            _pythonExecutable = Path.Combine(_venvDir, "Scripts", "python.exe");
            _pipExecutable = Path.Combine(_venvDir, "Scripts", "pip.exe");
        }
        else
        {
            _pythonExecutable = Path.Combine(_venvDir, "bin", "python");
            _pipExecutable = Path.Combine(_venvDir, "bin", "pip");
        }
    }

    protected override string Language => "Python";

    public override async Task<TestResult> RunTestsAsync(TestConfiguration config, CancellationToken cancellationToken = default)
    {
        Logger.LogInformation("Running Python SDK tests directly");

        try
        {
            // Check prerequisites
            await CheckPythonRequirementsAsync(cancellationToken);

            // Set up virtual environment if building
            if (!config.NoBuild)
            {
                await SetupVirtualEnvironmentAsync(cancellationToken);
            }

            // Build SDK if needed
            if (!config.NoBuild)
            {
                await BuildPythonSdkAsync(cancellationToken);
            }

            // Run the Python test suite directly
            var testScriptPath = Path.Combine(_pythonDir, "test", "test_suite.py");
            var args = new List<string> { testScriptPath };

            if (config.JsonOutput) args.Add("--json");
            if (config.Verbose) args.Add("--verbose");

            // Set up environment variables using base class method
            var envVars = BuildEnvironmentVariables(config);

            // Use venv Python if it exists, otherwise fall back to system python
            var pythonCommand = File.Exists(_pythonExecutable) ? _pythonExecutable : PythonCommand;

            Logger.LogDebug("Executing Python tests: {Python} {Args}", pythonCommand, string.Join(" ", args));

            // Execute tests using base class method
            var processResult = await ExecuteProcessAsync(
                pythonCommand,
                string.Join(" ", args),
                _pythonDir,  // Run from Python directory
                envVars,
                TimeSpan.FromMilliseconds(config.TimeoutMs ?? Config.Timeouts.DefaultTimeoutMs),
                cancellationToken,
                throwOnError: false);  // Don't throw, handle errors in result

            // Parse the JSON output if available
            if (config.JsonOutput && processResult.Success)
            {
                return ParseStandardJsonOutput(processResult.StandardOutput);
            }

            // Use base class method for basic result
            return CreateBasicTestResult(processResult);
        }
        catch (Exception ex)
        {
            Logger.LogError(ex, "Python test execution failed with exception: {Message}", ex.Message);
            return CreateErrorResult(ex);
        }
    }

    private async Task SetupVirtualEnvironmentAsync(CancellationToken cancellationToken)
    {
        Logger.LogInformation("Setting up Python virtual environment...");

        // Check if venv already exists
        if (!Directory.Exists(_venvDir))
        {
            Logger.LogInformation("Creating virtual environment at {Path}", _venvDir);

            // First check if we have uv available
            var hasUv = await CheckToolAsync("uv", "--version", cancellationToken);

            if (hasUv)
            {
                // Use uv to create venv (like test.sh does)
                Logger.LogDebug("Creating virtual environment with uv");
                var venvResult = await ExecuteProcessAsync(
                    "uv",
                    $"venv \"{_venvDir}\" --python {PythonCommand}",
                    _pythonDir,
                    null,
                    TimeSpan.FromMilliseconds(Config.Timeouts.PipInstallTimeoutMs),
                    cancellationToken,
                    throwOnError: false);

                if (!venvResult.Success)
                {
                    Logger.LogWarning("Failed to create virtual environment with uv: {Error}", venvResult.StandardError);
                    // Fall back to standard venv
                    venvResult = await ExecuteProcessAsync(
                        PythonCommand,
                        $"-m venv \"{_venvDir}\"",
                        _pythonDir,
                        null,
                        TimeSpan.FromMilliseconds(Config.Timeouts.PipInstallTimeoutMs),
                        cancellationToken,
                        throwOnError: false);

                    if (!venvResult.Success)
                    {
                        Logger.LogWarning("Failed to create virtual environment: {Error}. Will continue with system Python", venvResult.StandardError);
                        return;
                    }
                }
            }
            else
            {
                // Fall back to standard Python venv
                var venvResult = await ExecuteProcessAsync(
                    PythonCommand,
                    $"-m venv \"{_venvDir}\"",
                    _pythonDir,
                    null,
                    TimeSpan.FromMilliseconds(Config.Timeouts.PipInstallTimeoutMs),
                    cancellationToken,
                    throwOnError: false);

                if (!venvResult.Success)
                {
                    Logger.LogWarning("Failed to create virtual environment: {Error}. Will continue with system Python", venvResult.StandardError);
                    return;
                }
            }

            Logger.LogInformation("Virtual environment created successfully");

            // Upgrade pip in the venv using uv if available
            if (File.Exists(_pythonExecutable))
            {
                if (hasUv)
                {
                    Logger.LogDebug("Upgrading pip in virtual environment with uv");
                    await ExecuteProcessAsync(
                        "uv",
                        $"pip install -p \"{_pythonExecutable}\" --upgrade pip",
                        _pythonDir,
                        null,
                        TimeSpan.FromMilliseconds(Config.Timeouts.PipInstallTimeoutMs),
                        cancellationToken,
                        throwOnError: false);
                }
                else
                {
                    Logger.LogDebug("Upgrading pip in virtual environment");
                    await ExecuteProcessAsync(
                        _pythonExecutable,
                        "-m pip install --upgrade pip",
                        _pythonDir,
                        null,
                        TimeSpan.FromMilliseconds(Config.Timeouts.PipInstallTimeoutMs),
                        cancellationToken,
                        throwOnError: false);
                }
            }
        }
        else
        {
            Logger.LogInformation("Using existing virtual environment at {Path}", _venvDir);
        }
    }

    private async Task CheckPythonRequirementsAsync(CancellationToken cancellationToken)
    {
        Logger.LogDebug("Checking Python requirements");

        // Try to find python with various approaches
        var pythonFound = await CheckToolAsync(PythonCommand, "--version", cancellationToken);

        if (!pythonFound)
        {
            // Try with full path
            Logger.LogDebug("{Python} not found in PATH, trying common locations", PythonCommand);
            var commonPaths = new[] { "/usr/bin/python3", "/usr/local/bin/python3", "/opt/homebrew/bin/python3" };
            var existingPythonPath = commonPaths.FirstOrDefault(File.Exists);

            if (existingPythonPath == null)
            {
                throw new InvalidOperationException($"{PythonCommand} is required but not found");
            }

            Logger.LogInformation("Found {Python} at {Path}, but PATH may not be configured correctly", PythonCommand, existingPythonPath);
            // For now, continue anyway as python3 exists on the system
        }

        // Schema files are now checked globally in Global.cs
        // The global setup will either auto-generate them or fail fast
        var schemasPath = Path.Combine(_pythonDir, "bitwarden_sdk", "schemas.py");
        if (!File.Exists(schemasPath))
        {
            // This should not happen as Global.cs checks schemas first
            // But if it does, fail immediately with clear message
            throw new InvalidOperationException(
                $"schemas.py not found at {schemasPath}\n" +
                "This should have been caught by global setup. Please check your configuration.");
        }
        Logger.LogDebug("schemas.py found at: {Path}", schemasPath);
    }

    private async Task BuildPythonSdkAsync(CancellationToken cancellationToken)
    {
        Logger.LogInformation("Building Python SDK");

        var pythonCommand = File.Exists(_pythonExecutable) ? _pythonExecutable : PythonCommand;
        var deps = PlatformService.OperatingSystem == OSPlatform.Linux ? ".[dev-linux]" : ".[dev]";

        // Check for uv and install dependencies
        var hasUv = await CheckForUvAsync(pythonCommand, cancellationToken);
        await InstallDependenciesAsync(hasUv, deps, cancellationToken);

        // Check for maturin
        var hasMaturin = await CheckForMaturinAsync(pythonCommand, cancellationToken);
        if (!hasMaturin)
        {
            // Log more details about what Python we're using
            Logger.LogError("Maturin not found. Python command: {PythonCommand}, Venv Python: {VenvPython}",
                pythonCommand, _pythonExecutable);

            // Try to see what's installed in the venv
            if (File.Exists(_pythonExecutable))
            {
                var listResult = await ExecuteProcessAsync(
                    _pythonExecutable,
                    "-m pip list",
                    _pythonDir,
                    null,
                    TimeSpan.FromMilliseconds(Config.Timeouts.ToolCheckTimeoutMs),
                    cancellationToken,
                    throwOnError: false);

                Logger.LogError("Installed packages in venv: {Packages}", listResult.StandardOutput);
            }

            const string errorMsg = "maturin is required to build the Python SDK but was not found.\n" +
                                   "Please install it using one of these methods:\n" +
                                   "  - System-wide: pip install maturin\n" +
                                   "  - With uv: uv pip install maturin\n" +
                                   "  - Or ensure .[dev] dependencies are installed: pip install -e .[dev]";
            Logger.LogError(errorMsg);
            throw new InvalidOperationException(errorMsg);
        }

        // Build with maturin
        await BuildWithMaturinAsync(pythonCommand, cancellationToken);

        Logger.LogInformation("Python SDK built successfully");
    }

    private async Task<bool> CheckForUvAsync(string pythonCommand, CancellationToken cancellationToken)
    {
        Logger.LogDebug("Checking for uv tool...");

        // Check in venv first
        if (File.Exists(_pythonExecutable))
        {
            var uvCheckResult = await ExecuteProcessAsync(
                pythonCommand,
                "-m uv --version",
                _pythonDir,
                null,
                TimeSpan.FromMilliseconds(Config.Timeouts.ToolCheckTimeoutMs),
                cancellationToken,
                throwOnError: false);

            if (uvCheckResult.Success)
            {
                Logger.LogDebug("Has uv: true (venv)");
                return true;
            }
        }

        // Try system uv as fallback
        var hasUv = await CheckToolAsync("uv", "--version", cancellationToken);
        Logger.LogDebug("Has uv: {HasUv} (system)", hasUv);
        return hasUv;
    }

    private async Task InstallDependenciesAsync(bool hasUv, string deps, CancellationToken cancellationToken)
    {
        if (hasUv)
        {
            Logger.LogDebug("Installing Python dependencies with uv into venv");

            // If we have a venv, we need to tell uv to install into it
            // Use -p shorthand and quote the path to handle spaces
            // Note: we pass the deps directly, uv will handle the .[dev] syntax
            var uvArgs = File.Exists(_pythonExecutable)
                ? $"pip install -p \"{_pythonExecutable}\" \"{deps}\""
                : $"pip install \"{deps}\"";

            Logger.LogDebug("Running uv with args: {Args}", uvArgs);

            var installResult = await ExecuteProcessAsync(
                "uv",
                uvArgs,
                _pythonDir,
                null,
                TimeSpan.FromMilliseconds(Config.Timeouts.BuildTimeoutMs),
                cancellationToken,
                throwOnError: false);

            if (installResult.Success)
            {
                Logger.LogDebug("Successfully installed dependencies with uv");
                return;
            }

            Logger.LogWarning("Failed to install dependencies with uv: {Error}", installResult.StandardError);
            Logger.LogDebug("Falling back to pip for dependency installation");
        }
        else
        {
            Logger.LogDebug("uv not found, using pip for dependency installation");
        }

        await InstallWithPipAsync(deps, cancellationToken);
    }

    private async Task<bool> CheckForMaturinAsync(string pythonCommand, CancellationToken cancellationToken)
    {
        // First try system maturin (most common case)
        var systemMaturin = await CheckToolAsync("maturin", "--version", cancellationToken);
        if (systemMaturin)
        {
            Logger.LogDebug("Found system maturin");
            return true;
        }

        // If venv exists, check for maturin as a Python module
        if (File.Exists(_pythonExecutable))
        {
            var maturinCheckResult = await ExecuteProcessAsync(
                pythonCommand,
                "-m maturin --version",
                _pythonDir,
                null,
                TimeSpan.FromMilliseconds(Config.Timeouts.ToolCheckTimeoutMs),
                cancellationToken,
                throwOnError: false);

            if (maturinCheckResult.Success)
            {
                Logger.LogDebug("Found maturin in virtual environment");
                return true;
            }
        }

        Logger.LogDebug("Maturin not found in system PATH or virtual environment");
        return false;
    }

    private async Task BuildWithMaturinAsync(string pythonCommand, CancellationToken cancellationToken)
    {
        // Set up environment variables for maturin
        Dictionary<string, string>? maturinEnv = null;
        if (Directory.Exists(_venvDir))
        {
            maturinEnv = new Dictionary<string, string> { ["VIRTUAL_ENV"] = _venvDir };
            Logger.LogDebug("Setting VIRTUAL_ENV to: {VenvDir}", _venvDir);
        }

        // Try running maturin as a Python module first (works with both venv and system installs)
        Logger.LogDebug("Attempting to build with maturin using Python: {Python}", pythonCommand);
        var maturinResult = await ExecuteProcessAsync(
            pythonCommand,
            "-m maturin develop",
            _pythonDir,
            maturinEnv,
            TimeSpan.FromMilliseconds(Config.Timeouts.BuildTimeoutMs),
            cancellationToken,
            throwOnError: false);

        // If module not found, try system maturin command
        if (!maturinResult.Success && maturinResult.StandardError.Contains("No module named maturin"))
        {
            Logger.LogDebug("Maturin not found as Python module, trying system maturin command");
            maturinResult = await ExecuteProcessAsync(
                "maturin",
                "develop",
                _pythonDir,
                maturinEnv,
                TimeSpan.FromMilliseconds(Config.Timeouts.BuildTimeoutMs),
                cancellationToken,
                throwOnError: false);
        }

        if (!maturinResult.Success)
        {
            Logger.LogError("Failed to build Python SDK with maturin: {Error}", maturinResult.StandardError);
            throw new InvalidOperationException($"Failed to build Python SDK with maturin: {maturinResult.StandardError}");
        }

        Logger.LogDebug("Successfully built Python SDK with maturin");
    }

    private async Task InstallWithPipAsync(string deps, CancellationToken cancellationToken)
    {
        Logger.LogDebug("Installing Python dependencies with pip");

        // Use venv pip if available, otherwise system pip3
        var pipCommand = File.Exists(_pipExecutable) ? _pipExecutable : "pip3";

        Logger.LogDebug("Using pip at: {PipCommand}", pipCommand);

        var pipResult = await ExecuteProcessAsync(
            pipCommand,
            $"install \"{deps}\"",
            _pythonDir,
            null,
            TimeSpan.FromMilliseconds(Config.Timeouts.BuildTimeoutMs),
            cancellationToken,
            throwOnError: false);

        if (!pipResult.Success)
        {
            Logger.LogWarning("Failed to install dependencies with pip: {Error}", pipResult.StandardError);
            // Dependencies might already be installed, continue anyway
        }
    }
}
