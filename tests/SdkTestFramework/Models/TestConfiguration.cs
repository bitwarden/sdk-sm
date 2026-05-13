namespace SdkTestFramework.Models;

/// <summary>
/// Configuration for test execution
/// </summary>
public class TestConfiguration
{
    public required string Language { get; init; }
    public string TestMode { get; init; } = "fake-server";
    public bool JsonOutput { get; init; }
    public bool Verbose { get; init; }
    public int? TimeoutMs { get; init; }
    public string? SdkSource { get; init; }
    public string? PythonVersion { get; init; }
    public bool NoBuild { get; init; }
}
