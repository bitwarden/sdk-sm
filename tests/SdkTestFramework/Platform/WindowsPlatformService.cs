using System.Runtime.InteropServices;
using Microsoft.Extensions.Logging;

namespace SdkTestFramework.Platform;

/// <summary>
/// Windows-specific platform service implementation
/// </summary>
public class WindowsPlatformService(ILogger<WindowsPlatformService> logger) : PlatformServiceBase(logger)
{
    public override OSPlatform OperatingSystem => OSPlatform.Windows;

    public override string PlatformName => "windows";

    public override (string fileName, string arguments) FormatCommand(string command, string? additionalArguments = null)
    {
        var fullCommand = string.IsNullOrEmpty(additionalArguments)
            ? command
            : $"{command} {additionalArguments}";

        // Use cmd.exe for Windows command execution
        return ("cmd.exe", $"/c \"{fullCommand.Replace("\"", "\\\"")}\"");
    }
}