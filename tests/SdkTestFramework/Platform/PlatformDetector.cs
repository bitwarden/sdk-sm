using System.Runtime.InteropServices;

namespace SdkTestFramework.Platform;

/// <summary>
/// Detects the current platform and provides the appropriate platform service
/// </summary>
public static class PlatformDetector
{
    /// <summary>
    /// Get the current platform
    /// </summary>
    private static OSPlatform GetCurrentPlatform()
    {
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            return OSPlatform.Windows;
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
            return OSPlatform.Linux;
        if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
            return OSPlatform.OSX;

        throw new PlatformNotSupportedException(
            $"Platform {RuntimeInformation.OSDescription} is not supported");
    }

    /// <summary>
    /// Create a platform service for the current platform
    /// </summary>
    public static IPlatformService CreatePlatformService()
    {
        var platform = GetCurrentPlatform();

        if (platform == OSPlatform.Windows)
        {
            return new WindowsPlatformService();
        }

        if (platform == OSPlatform.OSX)
        {
            return new MacOsPlatformService();
        }

        if (platform == OSPlatform.Linux)
        {
            return new LinuxPlatformService();
        }

        throw new PlatformNotSupportedException(
            $"No platform service available for {platform}");
    }
}
