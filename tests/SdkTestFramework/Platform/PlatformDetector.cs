using System.Runtime.InteropServices;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;

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
    public static IPlatformService CreatePlatformService(IServiceProvider serviceProvider)
    {
        var platform = GetCurrentPlatform();

        if (platform == OSPlatform.Windows)
        {
            var logger = serviceProvider.GetRequiredService<ILogger<WindowsPlatformService>>();
            return new WindowsPlatformService(logger);
        }
        else if (platform == OSPlatform.OSX)
        {
            var logger = serviceProvider.GetRequiredService<ILogger<MacOsPlatformService>>();
            return new MacOsPlatformService(logger);
        }
        else if (platform == OSPlatform.Linux)
        {
            var logger = serviceProvider.GetRequiredService<ILogger<LinuxPlatformService>>();
            return new LinuxPlatformService(logger);
        }

        throw new PlatformNotSupportedException(
            $"No platform service available for {platform}");
    }
}
