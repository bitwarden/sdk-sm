

namespace SdkTestFramework.Services;

/// <summary>
/// Result from process execution
/// </summary>
public class ProcessResult
{
    /// <summary>
    /// Exit code of the process
    /// </summary>
    public required int ExitCode { get; init; }

    /// <summary>
    /// Standard output from the process
    /// </summary>
    public required string StandardOutput { get; init; }

    /// <summary>
    /// Standard error from the process
    /// </summary>
    public required string StandardError { get; init; }

    /// <summary>
    /// Whether the process succeeded (exit code 0)
    /// </summary>
    public bool Success => ExitCode == 0;

    /// <summary>
    /// Execution duration
    /// </summary>
    public TimeSpan Duration { get; init; }

    /// <summary>
    /// The command that was executed
    /// </summary>
    public required string Command { get; init; }
}
