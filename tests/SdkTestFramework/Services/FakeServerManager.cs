using System.Diagnostics;
using System.Runtime.InteropServices;
using Microsoft.Extensions.Logging;
using SdkTestFramework.Common;
using SdkTestFramework.Config;

namespace SdkTestFramework.Services;

/// <summary>
/// Manages the lifecycle of the fake-server process for testing
/// </summary>
public sealed class FakeServerManager(TestConfig config, ILogger<FakeServerManager> logger) : IDisposable
{
    private readonly TestConfig _config = config ?? throw new ArgumentNullException(nameof(config));
    private readonly ILogger<FakeServerManager> _logger = logger ?? throw new ArgumentNullException(nameof(logger));
    private Process? _fakeServerProcess;
    private bool _disposed;

    /// <summary>
    /// Starts the fake server if configured to auto-start and in fake-server mode
    /// </summary>
    public async Task<bool> StartIfNeeded()
    {
        // Only start if we're in fake-server mode and auto-start is enabled
        if (!_config.Configuration.IsFakeServerMode() || !_config.Configuration.AutoStartFakeServer)
        {
            return true;
        }

        var port = _config.Configuration.FakeServerPort;
        var isRunning = await IsServerRunning(port);

        if (isRunning)
        {
            _logger.LogInformation("Fake server is already running");
            return true;
        }

        return await StartFakeServer();
    }

    /// <summary>
    /// Starts the fake-server process
    /// </summary>
    private async Task<bool> StartFakeServer()
    {
        try
        {
            var fakeServerPath = GetFakeServerExecutablePath();
            var port = _config.Configuration.FakeServerPort;
            if (!File.Exists(fakeServerPath))
            {
                _logger.LogWarning("Fake server executable not found at: {Path}", fakeServerPath);
                _logger.LogInformation(
                    "Fake server executable not found. Will attempt to build it.\n" +
                    "NOTE: First-time builds download dependencies and may take several minutes.\n" +
                    "To avoid timeouts, you can pre-build the fake-server: cargo build -p fake-server");

                _logger.LogInformation("Attempting to build fake-server (this may take 2-5 minutes on first run)...");

                if (!await BuildFakeServerAsync(_logger))
                {
                    _logger.LogError(
                        "Fake-server build failed. The automatic build timed out or failed.\n" +
                        "This often happens on first run when downloading Rust dependencies.\n\n" +
                        "SOLUTIONS:\n" +
                        "1. Pre-build manually: cargo build -p fake-server\n" +
                        "2. Run separately: Set AUTO_START_FAKE_SERVER=false and run: cargo run -p fake-server\n" +
                        "3. Use test script: ./test.sh (handles building automatically)");
                    return false;
                }

                // Check again after building
                if (!File.Exists(fakeServerPath))
                {
                    _logger.LogError(
                        "Fake server still not found after build attempt. " +
                        "Please ensure cargo is installed and run: cargo build -p fake-server");
                    return false;
                }
            }

            _fakeServerProcess = new Process
            {
                StartInfo = new ProcessStartInfo
                {
                    FileName = fakeServerPath,
                    UseShellExecute = false,
                    RedirectStandardOutput = true,
                    RedirectStandardError = true,
                    CreateNoWindow = true,
                    Environment =
                    {
                        ["SM_FAKE_SERVER_PORT"] = port.ToString(),
                        ["RUST_LOG"] = "info"
                    }
                }
            };

            _fakeServerProcess.Start();

            // Give the server time to start
            await Task.Delay(_config.Timeouts.ServerStartupDelayMs);

            // Verify it started successfully
            if (_fakeServerProcess.HasExited)
            {
                _logger.LogError("Fake server failed to start");
                return false;
            }

            _logger.LogInformation("Fake server started successfully");
            return true;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Failed to start fake server");
            return false;
        }
    }

    /// <summary>
    /// Checks if a server is running on the specified port
    /// </summary>
    private async Task<bool> IsServerRunning(int port)
    {
        try
        {
            using var client = new HttpClient();
            client.Timeout = TimeSpan.FromMilliseconds(_config.Timeouts.HttpCheckTimeoutMs);
            var response = await client.GetAsync($"http://localhost:{port}/health");
            return response.IsSuccessStatusCode;
        }
        catch
        {
            return false;
        }
    }

    /// <summary>
    /// Builds the fake-server using cargo
    /// </summary>
    private static async Task<bool> BuildFakeServerAsync(ILogger logger)
    {
        try
        {
            var sdkRoot = PathUtilities.GetSdkRootPath();
            if (string.IsNullOrEmpty(sdkRoot))
            {
                logger.LogError("Could not find SDK root directory");
                return false;
            }

            var processStartInfo = new ProcessStartInfo
            {
                FileName = "cargo",
                Arguments = "build -p fake-server",
                WorkingDirectory = sdkRoot,
                UseShellExecute = false,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                CreateNoWindow = true
            };

            using var process = Process.Start(processStartInfo);
            if (process == null)
            {
                logger.LogError("Failed to start cargo build process");
                return false;
            }

            var error = await process.StandardError.ReadToEndAsync();

            await process.WaitForExitAsync();

            if (process.ExitCode != 0)
            {
                logger.LogError("Cargo build failed with exit code {ExitCode}", process.ExitCode);
                if (!string.IsNullOrEmpty(error))
                {
                    logger.LogError("Error output: {Error}", error);
                }
                return false;
            }

            logger.LogInformation("Successfully built fake-server");
            return true;
        }
        catch (Exception ex)
        {
            logger.LogError(ex, "Error building fake-server");
            if (ex.Message.Contains("cannot find the file specified") || ex.Message.Contains("No such file") ||
                ex.Message.Contains("file not found"))
            {
                logger.LogError(
                    "Rust/Cargo not found. Rust toolchain is required for building the fake-server.\n\n" +
                    "Install Rust:\n" +
                    "1. Official (recommended): Visit https://rust-lang.org/tools/install\n" +
                    "2. macOS: brew install rust\n" +
                    "3. Ubuntu/Debian: sudo apt install cargo\n" +
                    "4. Windows: Download from https://rust-lang.org/tools/install\n\n" +
                    "After installation, run 'cargo --version' to verify.\n\n" +
                    "Alternative: Set AUTO_START_FAKE_SERVER=false and start fake-server manually");
            }
            return false;
        }
    }


    /// <summary>
    /// Gets the path to the fake-server executable
    /// </summary>
    private static string GetFakeServerExecutablePath()
    {
        var basePath = PathUtilities.GetSdkRootPath();

        if (string.IsNullOrEmpty(basePath))
        {
            throw new InvalidOperationException("Could not find SDK root directory");
        }

        // Path to fake-server executable
        var executableName = RuntimeInformation.IsOSPlatform(OSPlatform.Windows)
            ? "fake-server.exe"
            : "fake-server";

        if (Path.IsPathRooted(executableName))
        {
            throw new InvalidOperationException("Executable name must be a relative file name.");
        }

        return Path.Combine(basePath, "target", "debug", executableName);
    }

    /// <summary>
    /// Stops the fake server if it was started by this manager
    /// </summary>
    private void Stop()
    {
        if (_fakeServerProcess == null || _fakeServerProcess.HasExited) return;

        try
        {
            _fakeServerProcess.Kill(entireProcessTree: true);
            _fakeServerProcess.WaitForExit(_config.Timeouts.ProcessExitWaitMs);
            _logger.LogInformation("Fake server stopped");
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Failed to stop fake server");
        }
    }

    public void Dispose()
    {
        if (_disposed) return;

        Stop();
        _fakeServerProcess?.Dispose();
        _disposed = true;
    }
}
