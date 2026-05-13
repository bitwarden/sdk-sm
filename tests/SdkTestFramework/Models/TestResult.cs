using System.Text.Json.Serialization;

namespace SdkTestFramework.Models;

/// <summary>
/// Results from a single language SDK test run
/// </summary>
public record TestResult
{
    [JsonPropertyName("language")]
    public required string Language { get; init; }

    [JsonPropertyName("operations")]
    public required List<TestOperation> Operations { get; init; }

    [JsonPropertyName("total_duration_ms")]
    public required long TotalDurationMs { get; init; }

    [JsonPropertyName("os")]
    public required string OperatingSystem { get; init; }

    [JsonPropertyName("architecture")]
    public required string Architecture { get; init; }

    [JsonPropertyName("timestamp")]
    public required DateTime Timestamp { get; init; }

    // Additional properties for new framework
    [JsonPropertyName("success")]
    public bool Success { get; init; }

    [JsonPropertyName("error")]
    public string? Error { get; init; }

    [JsonPropertyName("total_tests")]
    public int TotalTests { get; init; }

    [JsonPropertyName("passed_tests")]
    public int PassedTests { get; init; }

    [JsonPropertyName("failed_tests")]
    public int FailedTests { get; init; }

    [JsonPropertyName("skipped_tests")]
    public int SkippedTests { get; init; }

    [JsonPropertyName("duration")]
    [JsonIgnore]
    public TimeSpan Duration { get; init; }

    [JsonIgnore]
    public bool Verbose { get; init; }
}
