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

    /// <summary>
    /// Helper method to combine command with additional arguments
    /// </summary>
    protected static string CombineCommandArguments(string command, string? additionalArguments)
    {
        return string.IsNullOrEmpty(additionalArguments)
            ? command
            : $"{command} {additionalArguments}";
    }

    /// <summary>
    /// Detects the appropriate shell for Unix-like systems
    /// </summary>
    protected static string GetUnixShell(params string[]? preferredShells)
    {
        // Use provided shells or default to bash and sh
        var shells = preferredShells?.Length > 0
            ? preferredShells
            : ["/bin/bash", "/bin/sh"];

        // Find first existing shell or fallback to sh
        return shells.FirstOrDefault(File.Exists) ?? "/bin/sh";
    }

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