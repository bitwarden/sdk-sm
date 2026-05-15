using NUnit.Framework;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;
using SdkTestFramework.Common;
using SdkTestFramework.Config;
using SdkTestFramework.Platform;
using SdkTestFramework.Services;

namespace SdkTestFramework.Tests
{
    /// <summary>
    /// One time global set up fixture. OneTimeSetUp will run once before ANY test.
    /// OneTimeTearDown will run once after ALL tests have been completed.
    /// Manages global service registration and configuration.
    /// </summary>
    [SetUpFixture]
    public class Global
    {
        private static TestConfig? _testConfig;
        private static FakeServerManager? _fakeServerManager;
        private static IServiceProvider? _serviceProvider;

        [OneTimeSetUp]
        public async Task Global_SetUp()
        {
            await TestContext.Progress.WriteLineAsync(ConsoleFormatting.BoxTop);
            await TestContext.Progress.WriteLineAsync(ConsoleFormatting.CreateBoxedHeader("SDK Test Framework - Global Setup"));
            await TestContext.Progress.WriteLineAsync(ConsoleFormatting.BoxBottom);

            // Add command line parameters to environment
            await AddCommandLineParametersAsync();

            // Initialize configuration (loads .env and test-config.json)
            ConfigurationService.Initialize();

            // Load TestConfig from ConfigurationService for strongly-typed access
            _testConfig = TestConfig.LoadFromConfiguration(ConfigurationService.Configuration);

            // Debug: Check if configuration loaded properly
            if (_testConfig.Configuration == null)
            {
                throw new InvalidOperationException("TestConfig.Configuration is null after loading");
            }

            // Debug: Check if timeouts loaded properly
            if (_testConfig.Timeouts == null)
            {
                throw new InvalidOperationException("TestConfig.Timeouts is null after loading");
            }
            await TestContext.Progress.WriteLineAsync($"  Loaded timeout: DefaultTimeoutMs = {_testConfig.Timeouts.DefaultTimeoutMs}ms");

            // Initialize dependency injection container
            await InitializeServicesAsync();

            if (string.IsNullOrEmpty(_testConfig.Configuration.TestMode))
            {
                throw new InvalidOperationException($"TestMode is null or empty. Configuration section exists but TestMode property was not loaded.");
            }

            // Start fake server if needed
            if (_testConfig.Configuration.IsFakeServerMode())
            {
                var loggerFactory = _serviceProvider!.GetRequiredService<ILoggerFactory>();
                var logger = loggerFactory.CreateLogger<FakeServerManager>();
                _fakeServerManager = new FakeServerManager(_testConfig, logger);
                var serverStarted = await _fakeServerManager.StartIfNeeded();

                if (!serverStarted && _testConfig.Configuration.AutoStartFakeServer)
                {
                    throw new InvalidOperationException(
                        "Failed to start fake server. This typically happens on first run when the fake-server " +
                        "hasn't been built yet. Please run 'cargo build -p fake-server' first, then re-run the tests. " +
                        "See the console output above for detailed instructions.");
                }
            }

            // Validate required environment variables
            await ValidateRequiredVariablesAsync();

            await TestContext.Progress.WriteLineAsync("✅ Global setup complete");
        }

        private static async Task AddCommandLineParametersAsync()
        {
            await TestContext.Progress.WriteLineAsync("Adding test run parameters to environment variables...");

            foreach (var parameter in TestContext.Parameters.Names)
            {
                var value = TestContext.Parameters[parameter];
                Environment.SetEnvironmentVariable(parameter, value);
                await TestContext.Progress.WriteLineAsync($"  Set {parameter} = {value}");
            }
        }

        private static async Task ValidateRequiredVariablesAsync()
        {
            await TestContext.Progress.WriteLineAsync("Validating required environment variables...");

            // Always auto-generate STATE_FILE for test isolation
            var tempStateFile = Path.Combine(Path.GetTempPath(), $"sdk-state-{Guid.NewGuid()}.json");
            Environment.SetEnvironmentVariable("STATE_FILE", tempStateFile);
            await TestContext.Progress.WriteLineAsync($"  Using STATE_FILE: {tempStateFile}");

            var requiredVariables = new[]
            {
                "ACCESS_TOKEN",
                "ORGANIZATION_ID",
                "API_URL",
                "IDENTITY_URL"
            };

            var missingVariables = new List<string>();

            foreach (var variable in requiredVariables)
            {
                var value = ConfigurationService.GetValue(variable);
                if (string.IsNullOrWhiteSpace(value))
                {
                    missingVariables.Add(variable);
                }
                else
                {
                    // Set the environment variable so child processes can access it
                    Environment.SetEnvironmentVariable(variable, value);
                }
            }

            if (missingVariables.Count > 0)
            {
                await TestContext.Progress.WriteLineAsync("❌ Missing required environment variables:");
                foreach (var variable in missingVariables)
                {
                    await TestContext.Progress.WriteLineAsync($"  - {variable}");
                }
                throw new InvalidOperationException("Required environment variables not set. Please check your .env file.");
            }

            await TestContext.Progress.WriteLineAsync("  ✓ All required environment variables are set");
        }

        [OneTimeTearDown]
        public async Task Global_TearDown()
        {
            await TestContext.Progress.WriteLineAsync("");
            await TestContext.Progress.WriteLineAsync(ConsoleFormatting.LineSeparator);
            await TestContext.Progress.WriteLineAsync("                      Global Teardown");
            await TestContext.Progress.WriteLineAsync(ConsoleFormatting.LineSeparator);

            // Stop fake server if we started it
            _fakeServerManager?.Dispose();

            // Dispose service provider
            if (_serviceProvider is IDisposable disposable)
            {
                disposable.Dispose();
            }

            await TestContext.Progress.WriteLineAsync("✅ Global teardown complete");
        }

        /// <summary>
        /// Initialize dependency injection services
        /// </summary>
        private static async Task InitializeServicesAsync()
        {
            await TestContext.Progress.WriteLineAsync("Initializing services...");

            var services = new ServiceCollection();

            // Add logging
            services.AddLogging(builder =>
            {
                builder.AddConsole();
                builder.SetMinimumLevel(LogLevel.Information);
            });

            // Add platform services
            services.AddSingleton<IPlatformService>(_ => PlatformDetector.CreatePlatformService());

            // Add process executor
            services.AddSingleton<IProcessExecutor, ProcessExecutor>();

            // Add test result formatter
            services.AddSingleton<ITestResultFormatter, TestResultFormatter>();

            // Build service provider
            _serviceProvider = services.BuildServiceProvider();

            // Initialize the test helper with global configuration and services
            TestHelper.Initialize(_testConfig!, _serviceProvider);

            await TestContext.Progress.WriteLineAsync("  ✓ Services initialized");
        }
    }
}
