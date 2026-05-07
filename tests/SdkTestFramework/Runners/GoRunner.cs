using SdkTestFramework.Config;

namespace SdkTestFramework.Runners;

/// <summary>
/// Test runner for Go SDK
/// </summary>
public class GoRunner(TestConfig config, ProcessRunner processRunner) : BaseRunner(config, processRunner)
{
    private const string TestShellScript = "languages/go/test.sh";

    protected override string Language => "go";

    protected override bool IsSupportedOnCurrentPlatform()
    {
        // Go is supported on all platforms
        return true;
    }

    protected override string GetTestScriptPath()
    {
        // Get absolute path to test.sh script
        var basePath = GetSdkBasePath();
        return Path.Combine(basePath, TestShellScript);
    }

    protected override string GetExecuteCommand()
    {
        // Use bash to run the test.sh script
        return "bash";
    }

    protected override string[] GetExecuteArguments()
    {
        var args = new List<string> { GetTestScriptPath(), "--json" };

        // Add --no-build flag if BUILD_SDK is false in configuration
        if (!_config.Configuration.BuildSdk)
        {
            args.Add("--no-build");
        }

        return args.ToArray();
    }
}
