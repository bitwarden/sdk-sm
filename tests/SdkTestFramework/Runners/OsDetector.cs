using System.Runtime.InteropServices;
using SdkTestFramework.Models;

namespace SdkTestFramework.Runners;

/// <summary>
/// Detects operating system information and platform capabilities
/// </summary>
public static class OsDetector
{
    private const string WindowsName = "Windows";
    private const string MacOsName = "macOS";
    private const string LinuxName = "Linux";
    private const string UnknownName = "Unknown";

    public static OsContext GetCurrentOsContext()
    {
        var osType = GetOperatingSystemType();
        var architecture = GetArchitecture();
        var version = Environment.OSVersion.Version.ToString();

        return new OsContext
        {
            OsType = osType,
            OsDisplayName = GetOsDisplayName(osType),
            Architecture = architecture,
            Version = version,
            IsWindows = osType == OperatingSystemType.Windows,
            IsMacOs = osType == OperatingSystemType.MacOS,
            IsLinux = osType == OperatingSystemType.Linux,
            IsArm = IsArmArchitecture(architecture)
        };
    }

    private static OperatingSystemType GetOperatingSystemType()
    {
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            return OperatingSystemType.Windows;

        if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
            return OperatingSystemType.MacOS;

        if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
            return OperatingSystemType.Linux;

        return OperatingSystemType.Unknown;
    }

    private static string GetOsDisplayName(OperatingSystemType osType) => osType switch
    {
        OperatingSystemType.Windows => WindowsName,
        OperatingSystemType.MacOS => MacOsName,
        OperatingSystemType.Linux => LinuxName,
        _ => UnknownName
    };

    private static string GetArchitecture()
    {
        var arch = RuntimeInformation.ProcessArchitecture;
        return arch switch
        {
            Architecture.X64 => "x64",
            Architecture.X86 => "x86",
            Architecture.Arm64 => "arm64",
            Architecture.Arm => "arm",
            _ => arch.ToString().ToLowerInvariant()
        };
    }

    private static bool IsArmArchitecture(string architecture) =>
        architecture.Contains("arm", StringComparison.OrdinalIgnoreCase);

    public static bool IsPlatformSupported(OsContext context, string testCategory)
    {
        // Windows ARM is not supported for certain operations
        if (context.IsWindowsArm())
            return false;

        // All language SDK tests are supported on all platforms
        return true;
    }
}