using System.Diagnostics;
using SdkTestFramework.Common;
using SdkTestFramework.Config;

namespace SdkTestFramework.Runners;

/// <summary>
/// Manages the lifecycle of the fake-server process for testing
/// </summary>
public sealed class FakeServerManager(TestConfig config) : IDisposable
{
    private readonly TestConfig _config = config ?? throw new ArgumentNullException(nameof(config));
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

        // Check if fake-server is already running on the configured port
        if (await IsServerRunning(_config.Configuration.FakeServerPort))
        {
            Console.WriteLine($"Fake server already running on port {_config.Configuration.FakeServerPort}");
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
            if (!File.Exists(fakeServerPath))
            {
                Console.WriteLine($"Fake server executable not found at: {fakeServerPath}");
                Console.WriteLine("Attempting to build fake-server (this may take 2-5 minutes on first run)...");
                Console.WriteLine();
                Console.WriteLine(ConsoleFormatting.LineSeparator);
                Console.WriteLine("NOTE: First-time builds download dependencies and may take several minutes.");
                Console.WriteLine("To avoid timeouts, you can pre-build the fake-server:");
                Console.WriteLine("  cargo build -p fake-server");
                Console.WriteLine(ConsoleFormatting.LineSeparator);
                Console.WriteLine();

                // Try to build the fake-server
                if (!await BuildFakeServerAsync())
                {
                    Console.WriteLine();
                    ConsoleFormatting.PrintBoxedHeader("FAKE-SERVER BUILD FAILED");
                    Console.WriteLine("The automatic build timed out or failed. This often happens on first run");
                    Console.WriteLine("when downloading Rust dependencies.");
                    Console.WriteLine();
                    Console.WriteLine(ConsoleFormatting.DashedLine);
                    Console.WriteLine("SOLUTION 1 - Pre-build the fake-server manually:");
                    Console.WriteLine("  1. Open a terminal in the SDK root directory");
                    Console.WriteLine("  2. Run: cargo build -p fake-server");
                    Console.WriteLine("  3. Wait for the build to complete (2-5 minutes)");
                    Console.WriteLine("  4. Re-run your tests");
                    Console.WriteLine();
                    Console.WriteLine(ConsoleFormatting.DashedLine);
                    Console.WriteLine("SOLUTION 2 - Run fake-server separately:");
                    Console.WriteLine("  1. Set AUTO_START_FAKE_SERVER to false in test-config.json");
                    Console.WriteLine("  2. Start fake-server manually: cargo run -p fake-server");
                    Console.WriteLine("  3. Run your tests in another terminal");
                    Console.WriteLine();
                    Console.WriteLine(ConsoleFormatting.DashedLine);
                    Console.WriteLine("SOLUTION 3 - Use the test.sh script:");
                    Console.WriteLine("  Run: ./test.sh (handles building automatically)");
                    Console.WriteLine(ConsoleFormatting.LineSeparator);
                    return false;
                }

                // Check again after building
                if (!File.Exists(fakeServerPath))
                {
                    Console.WriteLine("Fake server still not found after build attempt");
                    Console.WriteLine("Please ensure cargo is installed and run: cargo build -p fake-server");
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
                        ["SM_FAKE_SERVER_PORT"] = _config.Configuration.FakeServerPort.ToString(),
                        ["RUST_LOG"] = "info"
                    }
                }
            };

            _fakeServerProcess.Start();

            // Give the server time to start
            await Task.Delay(2000);

            // Verify it started successfully
            if (_fakeServerProcess.HasExited)
            {
                Console.WriteLine("Fake server failed to start");
                return false;
            }

            Console.WriteLine($"Fake server started on port {_config.Configuration.FakeServerPort}");
            return true;
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Failed to start fake server: {ex.Message}");
            Console.WriteLine($"Exception type: {ex.GetType().Name}");
            if (ex.InnerException != null)
            {
                Console.WriteLine($"Inner exception: {ex.InnerException.Message}");
            }
            return false;
        }
    }

    /// <summary>
    /// Checks if a server is running on the specified port
    /// </summary>
    private static async Task<bool> IsServerRunning(int port)
    {
        try
        {
            using var client = new HttpClient();
            client.Timeout = TimeSpan.FromSeconds(2);
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
    private async Task<bool> BuildFakeServerAsync()
    {
        try
        {
            var sdkRoot = GetSdkRootPath();
            if (string.IsNullOrEmpty(sdkRoot))
            {
                Console.WriteLine("Could not find SDK root directory");
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
                Console.WriteLine("Failed to start cargo build process");
                return false;
            }

            var output = await process.StandardOutput.ReadToEndAsync();
            var error = await process.StandardError.ReadToEndAsync();

            await process.WaitForExitAsync();

            if (process.ExitCode != 0)
            {
                Console.WriteLine($"Cargo build failed with exit code {process.ExitCode}");
                if (!string.IsNullOrEmpty(error))
                {
                    Console.WriteLine($"Error: {error}");
                }
                return false;
            }

            Console.WriteLine("Successfully built fake-server");
            return true;
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Error building fake-server: {ex.Message}");
            if (ex.Message.Contains("cannot find the file specified") || ex.Message.Contains("No such file") ||
                ex.Message.Contains("file not found"))
            {
                Console.WriteLine();
                ConsoleFormatting.PrintBoxedHeader("RUST/CARGO NOT FOUND");
                Console.WriteLine("Rust toolchain is required for building the fake-server.");
                Console.WriteLine();
                Console.WriteLine("Please install Rust by following one of these methods:");
                Console.WriteLine();
                Console.WriteLine(ConsoleFormatting.DashedLine);
                Console.WriteLine("1. Official installer (recommended):");
                Console.WriteLine("   Visit https://rust-lang.org/tools/install and follow the instructions");
                Console.WriteLine();
                Console.WriteLine("2. Platform-specific:");
                Console.WriteLine("   - macOS: brew install rust");
                Console.WriteLine("   - Ubuntu/Debian: sudo apt install cargo");
                Console.WriteLine("   - Windows: Download from https://rust-lang.org/tools/install");
                Console.WriteLine();
                Console.WriteLine("After installation, restart your terminal and run 'cargo --version' to verify.");
                Console.WriteLine();
                Console.WriteLine(ConsoleFormatting.DashedLine);
                Console.WriteLine("Alternatively, you can:");
                Console.WriteLine("- Set AUTO_START_FAKE_SERVER to false in test-config.json and start fake-server manually");
                Console.WriteLine("- Run the test.sh scripts which handle building if cargo is available");
                Console.WriteLine(ConsoleFormatting.LineSeparator);
            }
            return false;
        }
    }

    /// <summary>
    /// Gets the SDK root path
    /// </summary>
    private static string? GetSdkRootPath()
    {
        var basePath = Directory.GetCurrentDirectory();

        // Go up to find the sdk root
        while (!string.IsNullOrEmpty(basePath) && !File.Exists(Path.Combine(basePath, "Cargo.toml")))
        {
            basePath = Directory.GetParent(basePath)?.FullName;
        }

        return basePath;
    }

    /// <summary>
    /// Gets the path to the fake-server executable
    /// </summary>
    private static string GetFakeServerExecutablePath()
    {
        var basePath = GetSdkRootPath();

        if (string.IsNullOrEmpty(basePath))
        {
            throw new InvalidOperationException("Could not find SDK root directory");
        }

        // Path to fake-server executable
        var osContext = OsDetector.GetCurrentOsContext();
        var executableName = osContext.IsWindows ? "fake-server.exe" : "fake-server";

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
            _fakeServerProcess.WaitForExit(5000);
            Console.WriteLine("Fake server stopped");
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Failed to stop fake server: {ex.Message}");
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
