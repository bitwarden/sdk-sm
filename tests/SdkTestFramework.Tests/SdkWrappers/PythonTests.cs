using NUnit.Framework;
using SdkTestFramework.Runners;

namespace SdkTestFramework.Tests.SdkWrappers
{
    /// <summary>
    /// Orchestrates Python SDK tests by running tests.py from languages/python/test/
    /// </summary>
    [TestFixture]
    [Category("SDK")]
    [Category("Python")]
    public class PythonTests : SdkWrappersTestBase
    {
        private PythonRunner? _pythonRunner;

        [OneTimeSetUp]
        public async Task SetUp()
        {
            _pythonRunner = await CreateAndInitializePythonRunner();

            // Skip tests if language is disabled
            if (_pythonRunner == null)
            {
                Assert.Ignore("Python tests are disabled in configuration");
            }
        }

        [Test, Description("PYTHON-SDK-TEST")]
        public async Task Python_SDK_Tests()
        {
            // Act
            var result = await _pythonRunner!.RunTests();

            // Log results
            var failedOps = result.Operations.Where(op => !op.Success).ToList();
            if (failedOps.Any())
            {
                TestContext.WriteLine("❌ Python SDK tests failed:");
                foreach (var op in failedOps)
                {
                    TestContext.WriteLine($"  - {op.Operation}: {op.Error}");
                }
            }
            else
            {
                TestContext.WriteLine($"✅ Python SDK: All {result.Operations.Count} operations passed");
            }
            TestContext.WriteLine($"  Total execution time: {result.TotalDurationMs}ms");

            // Assert
            Assert.That(result.Operations.All(op => op.Success), Is.True,
                $"Python SDK tests failed - {result.Operations.Count(op => !op.Success)} operations failed");
        }
    }
}
