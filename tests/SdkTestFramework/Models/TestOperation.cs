using System.Text.Json.Serialization;

namespace SdkTestFramework.Models;

/// <summary>
/// Represents a single test operation result
/// </summary>
public record TestOperation
{
    [JsonPropertyName("operation")]
    public required string Operation { get; init; }

    [JsonPropertyName("success")]
    public required bool Success { get; init; }

    [JsonPropertyName("duration_ms")]
    public required long DurationMs { get; init; }

    [JsonPropertyName("error")]
    public string? Error { get; init; }

    [JsonPropertyName("message")]
    public string? Message { get; init; }
}
