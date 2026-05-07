using NUnit.Framework;
using SdkTestFramework.Runners;

namespace SdkTestFramework.Tests.SdkWrappers
{
    /// <summary>
    /// Orchestrates Go SDK tests by running tests.go from languages/go/test/
    /// </summary>
    [TestFixture]
    [Category("SDK")]
    [Category("Go")]
    public class GoTests : SdkWrappersTestBase
    {
        private GoRunner? _goRunner;

        [OneTimeSetUp]
        public async Task SetUp()
        {
            _goRunner = await CreateAndInitializeGoRunner();

            // Skip tests if language is disabled
            if (_goRunner == null)
            {
                Assert.Ignore("Go tests are disabled in configuration");
            }
        }

        [Test, Description("GO-SDK-TEST")]
        public async Task Go_SDK_Tests()
        {
            // Act
            var result = await _goRunner!.RunTests();

            // Log results
            var failedOps = result.Operations.Where(op => !op.Success).ToList();
            if (failedOps.Any())
            {
                TestContext.WriteLine("❌ Go SDK tests failed:");
                foreach (var op in failedOps)
                {
                    TestContext.WriteLine($"  - {op.Operation}: {op.Error}");
                }
            }
            else
            {
                TestContext.WriteLine($"✅ Go SDK: All {result.Operations.Count} operations passed");
            }
            TestContext.WriteLine($"  Total execution time: {result.TotalDurationMs}ms");

            // Assert
            Assert.That(result.Operations.All(op => op.Success), Is.True,
                $"Go SDK tests failed - {result.Operations.Count(op => !op.Success)} operations failed");
        }
    }
}
