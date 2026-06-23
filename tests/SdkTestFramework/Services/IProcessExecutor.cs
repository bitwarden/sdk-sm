
namespace SdkTestFramework.Services;

/// <summary>
/// Interface for executing external processes in a cross-platform manner
/// </summary>
public interface IProcessExecutor
{
    /// <summary>
    /// Execute a process and wait for completion
    /// </summary>
    Task<ProcessResult> ExecuteAsync(ProcessRequest request, CancellationToken cancellationToken = default);
}
