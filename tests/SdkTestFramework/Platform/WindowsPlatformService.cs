using System.Runtime.InteropServices;

namespace SdkTestFramework.Platform;

/// <summary>
/// Windows-specific platform service implementation
/// </summary>
public class WindowsPlatformService : PlatformServiceBase
{
    public override OSPlatform OperatingSystem => OSPlatform.Windows;

    public override string PlatformName => "windows";

    public override (string fileName, string arguments) FormatCommand(string command, string? additionalArguments = null)
    {
        var fullCommand = CombineCommandArguments(command, additionalArguments);

        // Use cmd.exe for Windows command execution
        return ("cmd.exe", $"/c \"{fullCommand.Replace("\"", "\\\"")}\"");
    }
}