using System.Runtime.InteropServices;

namespace SdkTestFramework.Platform;

/// <summary>
/// Platform-specific service interface for OS and architecture detection
/// </summary>
public interface IPlatformService
{
    /// <summary>
    /// Current operating system
    /// </summary>
    OSPlatform OperatingSystem { get; }

    /// <summary>
    /// Current architecture
    /// </summary>
    Architecture Architecture { get; }

    /// <summary>
    /// Platform name for library paths (e.g., "darwin", "linux", "windows")
    /// </summary>
    string PlatformName { get; }

    /// <summary>
    /// Architecture name for library paths (e.g., "arm64", "x64")
    /// </summary>
    string ArchitectureName { get; }

    /// <summary>
    /// Format a command for execution on this platform
    /// </summary>
    (string fileName, string arguments) FormatCommand(string command, string? additionalArguments = null);
}