using Microsoft.Extensions.DependencyInjection;
using SdkTestFramework.Config;

namespace SdkTestFramework.Tests;

/// <summary>
/// Helper class for accessing global test services and configuration
/// </summary>
internal static class TestHelper
{
    private static TestConfig? _testConfig;
    private static IServiceProvider? _serviceProvider;

    /// <summary>
    /// Initialize the test helper with configuration and services
    /// </summary>
    internal static void Initialize(TestConfig testConfig, IServiceProvider serviceProvider)
    {
        _testConfig = testConfig;
        _serviceProvider = serviceProvider;
    }

    /// <summary>
    /// Get the global test configuration
    /// </summary>
    internal static TestConfig GetTestConfig()
    {
        if (_testConfig == null)
        {
            throw new InvalidOperationException("Test configuration not loaded. Ensure Global setup has run.");
        }
        return _testConfig;
    }

    /// <summary>
    /// Get a service from the global service provider
    /// </summary>
    internal static T GetService<T>() where T : class
    {
        if (_serviceProvider == null)
        {
            throw new InvalidOperationException("Service provider not initialized. Ensure Global setup has run.");
        }
        return _serviceProvider.GetRequiredService<T>();
    }
}