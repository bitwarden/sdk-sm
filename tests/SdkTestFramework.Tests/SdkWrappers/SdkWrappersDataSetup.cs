using NUnit.Framework;

namespace SdkTestFramework.Tests.SdkWrappers
{
    /// <summary>
    /// Data setup for SDK wrapper tests
    /// </summary>
    [SetUpFixture]
    public class SdkWrappersDataSetup
    {
        [OneTimeSetUp]
        public async Task SetUp()
        {
            await TestContext.Progress.WriteLineAsync("Setting up SDK Wrappers test data...");

            // Any SDK-specific data initialization here
            // For example, ensuring fake server is running, creating test organizations, etc.

            await TestContext.Progress.WriteLineAsync("SDK Wrappers test data setup complete");
        }

        [OneTimeTearDown]
        public async Task TearDown()
        {
            await TestContext.Progress.WriteLineAsync("Cleaning up SDK Wrappers test data...");

            // Any cleanup specific to SDK wrapper tests

            await TestContext.Progress.WriteLineAsync("SDK Wrappers test data cleanup complete");
        }
    }
}
