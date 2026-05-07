using System.Diagnostics;
using System.Text;

namespace SdkTestFramework.Runners;

/// <summary>
/// Runs external processes and captures output
/// </summary>
public class ProcessRunner
{
    private const int DefaultTimeoutMs = 30000;

    public async Task<ProcessResult> RunAsync(
        string fileName,
        string arguments,
        string? workingDirectory = null,
        Dictionary<string, string>? environmentVariables = null,
        int timeoutMs = DefaultTimeoutMs)
    {
        try
        {
            using var process = CreateProcess(fileName, arguments, workingDirectory, environmentVariables);

            var outputBuilder = new StringBuilder();
            var errorBuilder = new StringBuilder();

            // Stream output to console in real-time for better visibility
            process.OutputDataReceived += (_, e) =>
            {
                if (!string.IsNullOrEmpty(e.Data))
                {
                    Console.WriteLine($"   📝 {e.Data}");
                    outputBuilder.AppendLine(e.Data);
                }
            };

            process.ErrorDataReceived += (_, e) =>
            {
                if (!string.IsNullOrEmpty(e.Data))
                {
                    Console.WriteLine($"   ⚠️  {e.Data}");
                    errorBuilder.AppendLine(e.Data);
                }
            };

            var stopwatch = Stopwatch.StartNew();

            if (!process.Start())
            {
                throw new InvalidOperationException("Failed to start the process");
            }

            process.BeginOutputReadLine();
            process.BeginErrorReadLine();

            var completed = await WaitForExitAsync(process, timeoutMs);

            stopwatch.Stop();

            if (!completed)
            {
                KillProcess(process);
                return ProcessResult.Timeout(timeoutMs);
            }

            return new ProcessResult
            {
                ExitCode = process.ExitCode,
                Output = outputBuilder.ToString().Trim(),
                Error = errorBuilder.ToString().Trim(),
                DurationMs = stopwatch.ElapsedMilliseconds,
                Success = process.ExitCode == 0
            };
        }
        catch (Exception ex)
        {
            // Return failure with exception details including stack trace
            var errorDetails = $"Failed to start process: {ex.Message}";
            errorDetails += $"\nException Type: {ex.GetType().FullName}";
            if (ex.InnerException != null)
            {
                errorDetails += $"\nInner Exception: {ex.InnerException.Message}";
            }
            errorDetails += $"\nCommand: {fileName} {arguments}";
            errorDetails += $"\nWorking Directory: {workingDirectory ?? "current"}";
            errorDetails += $"\nTimeout: {timeoutMs}ms";
            errorDetails += $"\nStack Trace: {ex.StackTrace}";

            return new ProcessResult
            {
                ExitCode = -1,
                Output = string.Empty,
                Error = errorDetails,
                DurationMs = 0,
                Success = false
            };
        }
    }

    private static Process CreateProcess(
        string fileName,
        string arguments,
        string? workingDirectory,
        Dictionary<string, string>? environmentVariables)
    {
        var processInfo = new ProcessStartInfo
        {
            FileName = fileName,
            Arguments = arguments,
            WorkingDirectory = workingDirectory ?? Environment.CurrentDirectory,
            UseShellExecute = false,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            CreateNoWindow = true
        };

        AddEnvironmentVariables(processInfo, environmentVariables);

        return new Process { StartInfo = processInfo };
    }

    private static void AddEnvironmentVariables(
        ProcessStartInfo processInfo,
        Dictionary<string, string>? environmentVariables)
    {
        if (environmentVariables == null) return;

        // IMPORTANT: When we set Environment on ProcessStartInfo, it completely replaces
        // the environment variables (doesn't inherit). We need to copy the current environment
        // first, otherwise PATH and other system variables won't be available.
        foreach (var envVar in Environment.GetEnvironmentVariables())
        {
            if (envVar is System.Collections.DictionaryEntry entry)
            {
                var key = entry.Key?.ToString();
                var value = entry.Value?.ToString();
                if (key != null && value != null)
                {
                    processInfo.Environment[key] = value;
                }
            }
        }

        // Then add/override with test-specific variables
        foreach (var (key, value) in environmentVariables)
        {
            processInfo.Environment[key] = value;
        }
    }

    private static async Task<bool> WaitForExitAsync(Process process, int timeoutMs)
    {
        // Ensure we have a reasonable timeout
        if (timeoutMs <= 0)
        {
            throw new ArgumentException($"Invalid timeout: {timeoutMs}ms. Must be greater than 0.");
        }

        using var cts = new CancellationTokenSource(timeoutMs);
        try
        {
            await process.WaitForExitAsync(cts.Token);
            return true;
        }
        catch (TaskCanceledException)
        {
            return false;
        }
        catch (OperationCanceledException)
        {
            // This means the process took too long
            return false;
        }
    }

    private static void KillProcess(Process process)
    {
        try
        {
            process.Kill(entireProcessTree: true);
        }
        catch
        {
            // Best effort - process might have already exited
        }
    }
}

public record ProcessResult
{
    public required int ExitCode { get; init; }
    public required string Output { get; init; }
    public required string Error { get; init; }
    public required long DurationMs { get; init; }
    public required bool Success { get; init; }

    public static ProcessResult Timeout(int timeoutMs) => new()
    {
        ExitCode = -1,
        Output = string.Empty,
        Error = $"Process timed out after {timeoutMs}ms",
        DurationMs = timeoutMs,
        Success = false
    };
}
