using NUnit.Framework;
using SdkTestFramework.TestRunners;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.DependencyInjection;

namespace SdkTestFramework.Tests.SdkWrappers;

/// <summary>
/// Go SDK integration tests
/// </summary>
[TestFixture]
[Category("SDK")]
[Category("Go")]
[Property("Language", "Go")]
[Property("TestType", "Integration")]
public class GoTests : SdkTestBase
{
    protected override string SdkLanguage => "Go";

    protected override BaseTestRunner CreateTestRunner()
    {
        var logger = Global.GetService<IServiceProvider>()
            .GetRequiredService<ILogger<GoRunner>>();

        return new GoRunner(logger, ProcessExecutor, PlatformService);
    }

    [Test]
    [Description("Execute Go SDK test suite")]
    [Property("TestId", "SDK-GO-001")]
    public async Task Go_SDK_Should_Pass_All_Tests()
    {
        // Execute SDK tests
        var result = await ExecuteSdkTests();

        // Validate results
        ValidateTestResult(result, SdkLanguage);
    }
}
