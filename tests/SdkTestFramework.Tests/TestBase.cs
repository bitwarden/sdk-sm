using NUnit.Framework;
using NUnit.Framework.Interfaces;
using SdkTestFramework.Config;
using SdkTestFramework.Runners;

namespace SdkTestFramework.Tests
{
    /// <summary>
    /// Base test class that provides configuration for all test classes.
    /// All test classes should either directly or indirectly inherit this class.
    /// </summary>
    [TestFixture]
    public abstract class TestBase
    {
        protected TestConfig TestConfig { get; private set; } = null!;
        protected ProcessRunner ProcessRunner { get; private set; } = null!;


        [OneTimeSetUp]
        public virtual async Task TestBase_OneTimeSetUp()
        {
            // Load configuration from Global
            TestConfig = Global.GetTestConfig();
            ProcessRunner = new ProcessRunner();

            await TestContext.Progress.WriteLineAsync($"Running tests inside test class: {GetType().Name}");
        }

        [SetUp]
        public virtual async Task TestBase_SetUp()
        {
            var testId = TestContext.CurrentContext.Test.Properties.Get("Description") ?? "NoTestId";
            var testName = TestContext.CurrentContext.Test.Name;

            await TestContext.Progress.WriteLineAsync($"Testing: {testId}-{testName}");
        }

        [TearDown]
        public virtual async Task TestBase_TearDown()
        {
            var testResult = TestContext.CurrentContext.Result;

            if (testResult.Outcome.Status == TestStatus.Failed)
            {
                await TestContext.Progress.WriteLineAsync($"❌ Test failed: {testResult.Message}");
                if (testResult.StackTrace != null)
                {
                    await TestContext.Progress.WriteLineAsync($"Stack trace: {testResult.StackTrace}");
                }
            }
            else if (testResult.Outcome.Status == TestStatus.Passed)
            {
                await TestContext.Progress.WriteLineAsync($"✅ Test passed");
            }
            else if (testResult.Outcome.Status == TestStatus.Skipped)
            {
                await TestContext.Progress.WriteLineAsync($"⏭️ Test skipped: {testResult.Message}");
            }
        }

        [OneTimeTearDown]
        public virtual async Task TestBase_OneTimeTearDown()
        {
            // Any class-level cleanup
            await TestContext.Progress.WriteLineAsync($"Completed tests in class: {GetType().Name}");
        }

        /// <summary>
        /// Helper method to check if a language is enabled
        /// </summary>
        protected bool IsLanguageEnabled(string language)
        {
            return TestConfig.Configuration.EnabledLanguages
                .Contains(language, StringComparer.OrdinalIgnoreCase);
        }
    }
}
