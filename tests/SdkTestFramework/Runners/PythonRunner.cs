using SdkTestFramework.Config;

namespace SdkTestFramework.Runners;

/// <summary>
/// Test runner for Python SDK
/// </summary>
public class PythonRunner(TestConfig config, ProcessRunner processRunner) : BaseRunner(config, processRunner)
{
    private const string TestShellScript = "languages/python/test.sh";

    protected override string Language => "python";

    protected override bool IsSupportedOnCurrentPlatform()
    {
        // Python is supported on all platforms
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
        // Always use bash for all platforms
        // On Windows, this requires Git Bash, WSL, or similar
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

    protected override string GetWorkingDirectory() => GetSdkBasePath();
}
