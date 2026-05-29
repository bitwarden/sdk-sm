using System.Text.Json.Serialization;
using Microsoft.Extensions.Configuration;

namespace SdkTestFramework.Config;

/// <summary>
/// Test framework configuration - provides strongly-typed access to configuration
/// </summary>
public record TestConfig
{
    [JsonPropertyName("testFramework")]
    public required TestFrameworkInfo TestFramework { get; init; }

    [JsonPropertyName("configuration")]
    public required ConfigurationSettings Configuration { get; init; }

    [JsonPropertyName("timeouts")]
    public required TimeoutSettings Timeouts { get; init; }

    /// <summary>
    /// Load configuration from IConfiguration (matches Bitwarden pattern)
    /// </summary>
    public static TestConfig LoadFromConfiguration(IConfiguration configuration)
    {
        var config = new TestConfig
        {
            TestFramework = configuration.GetSection("testFramework").Get<TestFrameworkInfo>()
                ?? new TestFrameworkInfo { Name = "SDK Test Framework", Version = "1.0.0" },

            Configuration = new ConfigurationSettings
            {
                TestMode = configuration["configuration:TEST_MODE"] ?? throw new InvalidOperationException("TEST_MODE not found in configuration"),
                BuildSdk = configuration.GetValue<bool>("configuration:BUILD_SDK"),
                AutoStartFakeServer = configuration.GetValue<bool>("configuration:AUTO_START_FAKE_SERVER"),
                AutoGenerateSchemas = configuration.GetValue<bool>("configuration:AUTO_GENERATE_SCHEMAS"),
                FakeServerPort = configuration.GetValue<int>("configuration:FAKE_SERVER_PORT"),
                PythonVersion = configuration["configuration:PYTHON_VERSION"] ?? "3.13",
                EnabledLanguages = configuration.GetSection("configuration:ENABLED_LANGUAGES").Get<List<string>>()?.AsReadOnly() ?? new List<string>().AsReadOnly()
            },

            Timeouts = new TimeoutSettings
            {
                DefaultTimeoutMs = configuration.GetValue("timeouts:DEFAULT_TIMEOUT_MS", 300000),
                BuildTimeoutMs = configuration.GetValue("timeouts:BUILD_TIMEOUT_MS", 300000),
                ToolCheckTimeoutMs = configuration.GetValue("timeouts:TOOL_CHECK_TIMEOUT_MS", 5000),
                PipInstallTimeoutMs = configuration.GetValue("timeouts:PIP_INSTALL_TIMEOUT_MS", 120000),
                HttpCheckTimeoutMs = configuration.GetValue("timeouts:HTTP_CHECK_TIMEOUT_MS", 2000),
                ServerStartupDelayMs = configuration.GetValue("timeouts:SERVER_STARTUP_DELAY_MS", 2000),
                ProcessExitWaitMs = configuration.GetValue("timeouts:PROCESS_EXIT_WAIT_MS", 5000)
            }
        };

        return config;
    }
}

public record TestFrameworkInfo
{
    [JsonPropertyName("name")]
    public required string Name { get; init; }

    [JsonPropertyName("version")]
    public required string Version { get; init; }
}

public record ConfigurationSettings
{
    [JsonPropertyName("TEST_MODE")]
    public required string TestMode { get; init; }

    [JsonPropertyName("BUILD_SDK")]
    public required bool BuildSdk { get; init; }

    [JsonPropertyName("AUTO_START_FAKE_SERVER")]
    public required bool AutoStartFakeServer { get; init; }

    [JsonPropertyName("AUTO_GENERATE_SCHEMAS")]
    public bool AutoGenerateSchemas { get; init; }

    [JsonPropertyName("FAKE_SERVER_PORT")]
    public required int FakeServerPort { get; init; }

    [JsonPropertyName("PYTHON_VERSION")]
    public required string PythonVersion { get; init; }

    [JsonPropertyName("ENABLED_LANGUAGES")]
    public required IReadOnlyList<string> EnabledLanguages { get; init; }

    public bool IsFakeServerMode() => TestMode.Equals("fake-server", StringComparison.OrdinalIgnoreCase);
}

public record TimeoutSettings
{
    [JsonPropertyName("DEFAULT_TIMEOUT_MS")]
    public required int DefaultTimeoutMs { get; init; }

    [JsonPropertyName("BUILD_TIMEOUT_MS")]
    public required int BuildTimeoutMs { get; init; }

    [JsonPropertyName("TOOL_CHECK_TIMEOUT_MS")]
    public int ToolCheckTimeoutMs { get; init; } = 5000;

    [JsonPropertyName("PIP_INSTALL_TIMEOUT_MS")]
    public int PipInstallTimeoutMs { get; init; } = 120000;

    [JsonPropertyName("HTTP_CHECK_TIMEOUT_MS")]
    public int HttpCheckTimeoutMs { get; init; } = 2000;

    [JsonPropertyName("SERVER_STARTUP_DELAY_MS")]
    public int ServerStartupDelayMs { get; init; } = 2000;

    [JsonPropertyName("PROCESS_EXIT_WAIT_MS")]
    public int ProcessExitWaitMs { get; init; } = 5000;
}
