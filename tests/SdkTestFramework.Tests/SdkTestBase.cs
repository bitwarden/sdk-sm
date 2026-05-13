using NUnit.Framework;
using SdkTestFramework.Models;
using SdkTestFramework.Platform;
using SdkTestFramework.Services;
using SdkTestFramework.TestRunners;

namespace SdkTestFramework.Tests;

/// <summary>
/// Base class for SDK language tests. Provides common functionality for all SDK test implementations.
/// Follows Bitwarden test patterns with proper lifecycle management and service initialization.
/// </summary>
public abstract class SdkTestBase : TestBase
{
    // Services - initialized from Global service provider
    protected IPlatformService PlatformService { get; private set; } = null!;
    protected IProcessExecutor ProcessExecutor { get; private set; } = null!;
    private ITestResultFormatter ResultFormatter { get; set; } = null!;

    // Test configuration
    private TestConfiguration SdkTestConfiguration { get; set; } = null!;

    // Abstract properties for language-specific implementations
    protected abstract string SdkLanguage { get; }
    protected abstract BaseTestRunner CreateTestRunner();

    // Test runner instance
    private BaseTestRunner? _testRunner;
    private BaseTestRunner TestRunner => _testRunner ?? throw new InvalidOperationException("Test runner not initialized");

    [OneTimeSetUp]
    public virtual async Task SdkTestBase_OneTimeSetUp()
    {
        await base.TestBase_OneTimeSetUp();

        // Get services from Global
        PlatformService = Global.GetService<IPlatformService>();
        ProcessExecutor = Global.GetService<IProcessExecutor>();
        ResultFormatter = Global.GetService<ITestResultFormatter>();

        // Create test runner
        _testRunner = CreateTestRunner();

        // Build SDK test configuration from environment and config
        SdkTestConfiguration = BuildTestConfiguration();

        await TestContext.Progress.WriteLineAsync($"Initialized {SdkLanguage} SDK test environment");
        await TestContext.Progress.WriteLineAsync($"Platform: {PlatformService.PlatformName} ({PlatformService.ArchitectureName})");
        await TestContext.Progress.WriteLineAsync($"Test Mode: {SdkTestConfiguration.TestMode}");
        await TestContext.Progress.WriteLineAsync($"SDK Source: {SdkTestConfiguration.SdkSource}");
    }

    /// <summary>
    /// Builds test configuration from environment variables and config file
    /// </summary>
    private TestConfiguration BuildTestConfiguration()
    {
        var envTestMode = Environment.GetEnvironmentVariable("TEST_MODE");
        var envTimeout = Environment.GetEnvironmentVariable("TEST_TIMEOUT_MS");
        var envSdkSource = Environment.GetEnvironmentVariable("SDK_SOURCE");
        var envPythonVersion = Environment.GetEnvironmentVariable("PYTHON_VERSION");

        return new TestConfiguration
        {
            Language = SdkLanguage.ToLower(),
            TestMode = envTestMode ?? TestConfig.Configuration.TestMode,
            JsonOutput = true, // Always use JSON for better parsing
            Verbose = IsVerboseMode(),
            TimeoutMs = ParseTimeout(envTimeout),
            SdkSource = envSdkSource ?? TestConfig.Configuration.SdkSource,
            PythonVersion = SdkLanguage == "Python"
                ? (envPythonVersion ?? TestConfig.Configuration.PythonVersion)
                : null,
            NoBuild = Environment.GetEnvironmentVariable("TEST_NO_BUILD") == "true"
        };
    }

    private static bool IsVerboseMode()
    {
        return Environment.GetEnvironmentVariable("TEST_VERBOSE") == "true"
            || TestContext.Parameters.Get("verbose", "false") == "true";
    }

    private static int? ParseTimeout(string? timeoutValue)
    {
        return int.TryParse(timeoutValue, out var timeout) ? timeout : 300000;
    }

    /// <summary>
    /// Executes SDK tests and validates results
    /// </summary>
    protected async Task<TestResult> ExecuteSdkTests()
    {
        TestContext.WriteLine($"\nExecuting {SdkLanguage} SDK tests...");
        TestContext.WriteLine("=" + new string('=', 50));

        var result = await TestRunner.RunTestsAsync(SdkTestConfiguration);

        // Save results if output file specified
        await SaveTestResultToJsonAsync(result);

        // Display formatted results
        var formattedOutput = ResultFormatter.FormatTestResult(result);
        TestContext.WriteLine(formattedOutput);

        // Log individual operations
        LogTestOperations(result);

        return result;
    }

    [OneTimeTearDown]
    public virtual async Task SdkTestBase_OneTimeTearDown()
    {
        _testRunner = null;
        await TestContext.Progress.WriteLineAsync($"Completed {SdkLanguage} SDK tests");
        await base.TestBase_OneTimeTearDown();
    }
}
