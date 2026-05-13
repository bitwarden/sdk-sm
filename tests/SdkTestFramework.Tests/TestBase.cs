using NUnit.Framework;
using NUnit.Framework.Interfaces;
using SdkTestFramework.Config;
using SdkTestFramework.Models;
using System.Text.Json;

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


        [OneTimeSetUp]
        public virtual async Task TestBase_OneTimeSetUp()
        {
            // Load configuration from Global
            TestConfig = Global.GetTestConfig();

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

        /// <summary>
        /// Saves test result to JSON file if output file is specified
        /// </summary>
        protected async Task SaveTestResultToJsonAsync(TestResult result)
        {
            var outputFile = Environment.GetEnvironmentVariable("TEST_OUTPUT_FILE");
            if (string.IsNullOrEmpty(outputFile))
                return;

            var json = JsonSerializer.Serialize(result, new JsonSerializerOptions
            {
                WriteIndented = true,
                PropertyNamingPolicy = JsonNamingPolicy.CamelCase
            });

            await File.WriteAllTextAsync(outputFile, json);
            TestContext.WriteLine($"Test results saved to: {outputFile}");
        }

        /// <summary>
        /// Logs individual test operation results to TestContext
        /// </summary>
        protected static void LogTestOperations(TestResult result)
        {
            foreach (var op in result.Operations)
            {
                if (op.Success)
                {
                    TestContext.WriteLine($"✅ {op.Operation}: {op.Message ?? "Passed"}");
                }
                else
                {
                    TestContext.WriteLine($"❌ {op.Operation}: {op.Error ?? op.Message ?? "Failed"}");
                    if (!string.IsNullOrEmpty(op.Error))
                    {
                        TestContext.WriteLine($"   Error: {op.Error}");
                    }
                }
            }
        }

        /// <summary>
        /// Validates test result and provides detailed assertion messages
        /// </summary>
        protected static void ValidateTestResult(TestResult result, string expectedLanguage)
        {
            Assert.That(result, Is.Not.Null, "Test result should not be null");
            Assert.That(result.Language, Is.EqualTo(expectedLanguage), $"Language should be {expectedLanguage}");

            if (!result.Success && !string.IsNullOrEmpty(result.Error))
            {
                TestContext.WriteLine($"Test execution error: {result.Error}");
            }

            // Check for failed operations
            if (result.Operations.Any())
            {
                var failedOps = result.Operations.Where(op => !op.Success).ToList();
                if (failedOps.Any())
                {
                    var failureDetails = string.Join("\n", failedOps.Select(op =>
                        $"  - {op.Operation}: {op.Error ?? op.Message ?? "Failed"}"));

                    Assert.That(result.Success, Is.True,
                        $"{expectedLanguage} SDK tests failed - {failedOps.Count} operations failed:\n{failureDetails}");
                }
                else
                {
                    Assert.That(result.Success, Is.True, $"All {expectedLanguage} SDK tests should pass");
                }
            }
            else
            {
                Assert.That(result.Success, Is.True,
                    result.Error ?? $"{expectedLanguage} SDK tests should complete successfully");
            }
        }
    }
}
