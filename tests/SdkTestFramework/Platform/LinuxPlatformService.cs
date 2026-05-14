using System.Runtime.InteropServices;

namespace SdkTestFramework.Platform;

/// <summary>
/// Linux-specific platform service implementation
/// </summary>
public class LinuxPlatformService : PlatformServiceBase
{
    public override OSPlatform OperatingSystem => OSPlatform.Linux;

    public override string PlatformName => "linux";

    public override (string fileName, string arguments) FormatCommand(string command, string? additionalArguments = null)
    {
        var fullCommand = string.IsNullOrEmpty(additionalArguments)
            ? command
            : $"{command} {additionalArguments}";

        // Use bash if available, otherwise sh
        var shell = GetShell();

        // Use -c flag for command execution
        return (shell, $"-c \"{fullCommand.Replace("\"", "\\\"")}\"");
    }

    private static string GetShell()
    {
        if (File.Exists("/bin/bash"))
            return "/bin/bash";

        return "/bin/sh";
    }
}