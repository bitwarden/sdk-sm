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
    private IPlatformService? _platformService;
    private IProcessExecutor? _processExecutor;
    private ITestResultFormatter? _resultFormatter;
    private TestConfiguration? _sdkTestConfiguration;

    protected IPlatformService PlatformService => _platformService ?? throw new InvalidOperationException("PlatformService not initialized. Ensure OneTimeSetUp has run.");
    protected IProcessExecutor ProcessExecutor => _processExecutor ?? throw new InvalidOperationException("ProcessExecutor not initialized. Ensure OneTimeSetUp has run.");
    private ITestResultFormatter ResultFormatter => _resultFormatter ?? throw new InvalidOperationException("ResultFormatter not initialized. Ensure OneTimeSetUp has run.");
    private TestConfiguration SdkTestConfiguration => _sdkTestConfiguration ?? throw new InvalidOperationException("SdkTestConfiguration not initialized. Ensure OneTimeSetUp has run.");

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

        // Get services from TestHelper
        _platformService = TestHelper.GetService<IPlatformService>();
        _processExecutor = TestHelper.GetService<IProcessExecutor>();
        _resultFormatter = TestHelper.GetService<ITestResultFormatter>();

        // Create test runner
        _testRunner = CreateTestRunner();

        // Build SDK test configuration from environment and config
        _sdkTestConfiguration = BuildTestConfiguration();

        await TestContext.Progress.WriteLineAsync($"Initialized {SdkLanguage} SDK test environment");
        await TestContext.Progress.WriteLineAsync($"Platform: {PlatformService.PlatformName} ({PlatformService.ArchitectureName})");
        await TestContext.Progress.WriteLineAsync($"Test Mode: {SdkTestConfiguration.TestMode}");
    }

    /// <summary>
    /// Builds test configuration from config file only
    /// </summary>
    private TestConfiguration BuildTestConfiguration()
    {
        return new TestConfiguration
        {
            Language = SdkLanguage.ToLower(),
            TestMode = TestConfig.Configuration.TestMode,
            JsonOutput = true, // Always use JSON for better parsing
            Verbose = IsVerboseMode(),
            TimeoutMs = TestConfig.Timeouts.DefaultTimeoutMs,
            PythonVersion = SdkLanguage == "Python"
                ? TestConfig.Configuration.PythonVersion
                : null,
            NoBuild = !TestConfig.Configuration.BuildSdk  // Invert BuildSdk to get NoBuild
        };
    }

    private static bool IsVerboseMode()
    {
        // Can still allow verbose override from test parameters for debugging
        return TestContext.Parameters.Get("verbose", "false") == "true";
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
