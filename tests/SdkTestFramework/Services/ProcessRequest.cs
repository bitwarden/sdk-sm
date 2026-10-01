namespace SdkTestFramework.Services;

/// <summary>
/// Request for process execution
/// </summary>
public class ProcessRequest
{
    /// <summary>
    /// The command to execute
    /// </summary>
    public required string Command { get; init; }

    /// <summary>
    /// Arguments for the command (optional, can be included in Command)
    /// </summary>
    public string? Arguments { get; init; }

    /// <summary>
    /// Working directory for the process
    /// </summary>
    public string? WorkingDirectory { get; init; }

    /// <summary>
    /// Environment variables to set
    /// </summary>
    public Dictionary<string, string>? EnvironmentVariables { get; init; }

    /// <summary>
    /// Timeout for the process execution
    /// </summary>
    public TimeSpan? Timeout { get; init; }

    /// <summary>
    /// Whether to capture output
    /// </summary>
    public bool CaptureOutput { get; init; } = true;

    /// <summary>
    /// Whether to redirect stderr to stdout
    /// </summary>
    public bool RedirectStandardError { get; init; } = true;

    /// <summary>
    /// Whether to use shell execute
    /// </summary>
    public bool UseShellExecute { get; init; }

    /// <summary>
    /// Whether to throw on non-zero exit code
    /// </summary>
    public bool ThrowOnError { get; init; } = true;
}
