using System.Text.Json.Serialization;

namespace SdkTestFramework.Models;

/// <summary>
/// Results from a single language SDK test run
/// </summary>
public record TestResult
{
    [JsonPropertyName("language")]
    public required string Language { get; init; }

    [JsonPropertyName("sdk_version")]
    public string? SdkVersion { get; init; }

    [JsonPropertyName("operations")]
    public required List<TestOperation> Operations { get; init; }

    [JsonPropertyName("total_duration_ms")]
    public required long TotalDurationMs { get; init; }

    [JsonPropertyName("os")]
    public required string OperatingSystem { get; init; }

    [JsonPropertyName("architecture")]
    public required string Architecture { get; init; }

    [JsonPropertyName("os_version")]
    public string? OperatingSystemVersion { get; init; }

    [JsonPropertyName("timestamp")]
    public required DateTime Timestamp { get; init; }

    public int PassedCount() => Operations.Count(op => op.IsSuccessful());
    public int FailedCount() => Operations.Count(op => !op.IsSuccessful());
    public bool AllPassed() => Operations.All(op => op.IsSuccessful());
    public double SuccessRate() => Operations.Count == 0 ? 0 : (double)PassedCount() / Operations.Count * 100;
}