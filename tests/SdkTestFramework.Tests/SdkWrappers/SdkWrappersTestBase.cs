using NUnit.Framework;
using SdkTestFramework.Runners;

namespace SdkTestFramework.Tests.SdkWrappers
{
    /// <summary>
    /// Base class for SDK wrapper tests
    /// </summary>
    public abstract class SdkWrappersTestBase : TestBase
    {
        [OneTimeSetUp]
        public override async Task TestBase_OneTimeSetUp()
        {
            await base.TestBase_OneTimeSetUp();

            // Validate required environment variables are set
            ValidateEnvironmentVariables();

            await TestContext.Progress.WriteLineAsync($"SDK test setup complete for: {GetType().Name}");
        }

        private static void ValidateEnvironmentVariables()
        {
            var required = new[] { "API_URL", "IDENTITY_URL", "ORGANIZATION_ID", "ACCESS_TOKEN" };
            var missing = required.Where(v => string.IsNullOrEmpty(Environment.GetEnvironmentVariable(v))).ToList();

            if (missing.Count > 0)
            {
                throw new InvalidOperationException(
                    $"Missing required environment variables: {string.Join(", ", missing)}");
            }
        }

        /// <summary>
        /// Create and initialize a Python runner with current configuration
        /// </summary>
        protected async Task<PythonRunner> CreateAndInitializePythonRunner()
        {
            if (!IsLanguageEnabled("python"))
            {
                await TestContext.Progress.WriteLineAsync("⚠️ Python tests are disabled in configuration");
                return null!;
            }

            var runner = new PythonRunner(TestConfig, ProcessRunner);

            // Verify prerequisites
            var prereqResult = await runner.VerifyPrerequisites();
            if (!prereqResult)
            {
                await TestContext.Progress.WriteLineAsync("❌ Python prerequisites check failed");
                await TestContext.Progress.WriteLineAsync("   Ensure Python 3 and bitwarden-sdk are installed");
                throw new InvalidOperationException("Python SDK test environment not available - prerequisites check failed");
            }

            return runner;
        }

        /// <summary>
        /// Create and initialize a Go runner with current configuration
        /// </summary>
        protected async Task<GoRunner> CreateAndInitializeGoRunner()
        {
            if (!IsLanguageEnabled("go"))
            {
                await TestContext.Progress.WriteLineAsync("⚠️ Go tests are disabled in configuration");
                return null!;
            }

            var runner = new GoRunner(TestConfig, ProcessRunner);

            // Verify prerequisites
            var prereqResult = await runner.VerifyPrerequisites();
            if (!prereqResult)
            {
                await TestContext.Progress.WriteLineAsync("❌ Go prerequisites check failed");
                await TestContext.Progress.WriteLineAsync("   Ensure Go is installed and SDK module is available");
                throw new InvalidOperationException("Go SDK test environment not available - prerequisites check failed");
            }

            return runner;
        }
    }
}
