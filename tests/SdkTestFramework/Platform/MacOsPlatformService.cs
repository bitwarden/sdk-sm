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
        var fullCommand = CombineCommandArguments(command, additionalArguments);

        // Prefer zsh (default on modern macOS), fall back to bash, then sh
        var shell = GetUnixShell("/bin/zsh", "/bin/bash", "/bin/sh");

        // Use -l -c flags for login shell with proper PATH setup
        return (shell, $"-l -c \"{fullCommand.Replace("\"", "\\\"")}\"");
    }
}