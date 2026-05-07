using System.Reflection;
using Microsoft.Extensions.Configuration;
using DotNetEnv;

namespace SdkTestFramework.Tests
{
    /// <summary>
    /// Simple configuration service following Bitwarden's pattern
    /// </summary>
    public static class ConfigurationService
    {
        private static IConfigurationRoot? _configuration;

        public static IConfigurationRoot Configuration
        {
            get
            {
                if (_configuration == null)
                {
                    Initialize();
                }
                return _configuration!;
            }
        }

        public static void Initialize()
        {
            var builder = new ConfigurationBuilder();

            // Get the base directory
            var baseDir = Path.GetDirectoryName(Assembly.GetExecutingAssembly().Location)
                ?? Directory.GetCurrentDirectory();

            var configDir = Path.Combine(baseDir, "Configuration");

            // Load .env file if it exists
            var envFile = Path.Combine(configDir, ".env");
            if (File.Exists(envFile))
            {
                Env.Load(envFile);
            }

            // Add configuration sources
            builder.SetBasePath(configDir)
                .AddJsonFile("test-config.json", optional: false, reloadOnChange: true)
                .AddEnvironmentVariables();

            _configuration = builder.Build();
        }

        /// <summary>
        /// Get a configuration value by key
        /// </summary>
        public static string? GetValue(string key)
        {
            return Configuration[key];
        }

        /// <summary>
        /// Get a configuration section
        /// </summary>
        public static T GetSection<T>(string sectionName) where T : new()
        {
            return Configuration.GetSection(sectionName).Get<T>() ?? new T();
        }
    }
}
