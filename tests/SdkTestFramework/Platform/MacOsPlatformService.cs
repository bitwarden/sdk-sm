using System.Runtime.InteropServices;

namespace SdkTestFramework.Platform;

/// <summary>
/// macOS-specific platform service implementation
/// </summary>
public class MacOsPlatformService : PlatformServiceBase
{
    public override OSPlatform OperatingSystem => OSPlatform.OSX;

    public override string PlatformName => "darwin";

    public override (string fileName, string arguments) FormatCommand(string command, string? additionalArguments = null)
    {
        var fullCommand = string.IsNullOrEmpty(additionalArguments)
            ? command
            : $"{command} {additionalArguments}";

        // Prefer zsh (default on modern macOS), fall back to bash, then sh
        var shell = GetShell();

        // Use -l -c flags for login shell with proper PATH setup
        return (shell, $"-l -c \"{fullCommand.Replace("\"", "\\\"")}\"");
    }

    private static string GetShell()
    {
        if (File.Exists("/bin/zsh"))
            return "/bin/zsh";

        if (File.Exists("/bin/bash"))
            return "/bin/bash";

        return "/bin/sh";
    }
}