using System.Runtime.InteropServices;

namespace SdkTestFramework.Platform;

/// <summary>
/// Base class for platform-specific services
/// </summary>
public abstract class PlatformServiceBase : IPlatformService
{

    public abstract OSPlatform OperatingSystem { get; }
    public abstract string PlatformName { get; }
    public abstract (string fileName, string arguments) FormatCommand(string command, string? additionalArguments = null);

    public Architecture Architecture => RuntimeInformation.OSArchitecture;

    public string ArchitectureName => Architecture switch
    {
        Architecture.X64 => "x64",
        Architecture.X86 => "x86",
        Architecture.Arm => "arm",
        Architecture.Arm64 => "arm64",
        _ => Architecture.ToString().ToLowerInvariant()
    };
}