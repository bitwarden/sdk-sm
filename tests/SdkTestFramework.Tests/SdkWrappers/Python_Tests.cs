using NUnit.Framework;
using SdkTestFramework.TestRunners;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.DependencyInjection;

namespace SdkTestFramework.Tests.SdkWrappers;

/// <summary>
/// Python SDK integration tests
/// </summary>
[TestFixture]
[Category("SDK")]
[Category("Python")]
[Property("Language", "Python")]
[Property("TestType", "Integration")]
public class PythonTests : SdkTestBase
{
    protected override string SdkLanguage => "Python";

    protected override BaseTestRunner CreateTestRunner()
    {
        var logger = Global.GetService<IServiceProvider>()
            .GetRequiredService<ILogger<PythonTestRunner>>();

        return new PythonTestRunner(logger, ProcessExecutor, PlatformService);
    }

    [Test]
    [Description("Execute Python SDK test suite")]
    [Property("TestId", "SDK-PY-001")]
    public async Task Python_SDK_Should_Pass_All_Tests()
    {
        // Execute SDK tests
        var result = await ExecuteSdkTests();

        // Validate results
        ValidateTestResult(result, SdkLanguage);
    }
}
