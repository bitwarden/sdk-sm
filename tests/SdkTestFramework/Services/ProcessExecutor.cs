using System.Diagnostics;
using System.Text;
using Microsoft.Extensions.Logging;
using SdkTestFramework.Platform;

namespace SdkTestFramework.Services;

/// <summary>
/// Cross-platform process executor implementation
/// </summary>
public class ProcessExecutor(IPlatformService platformService, ILogger<ProcessExecutor> logger) : IProcessExecutor
{
    private readonly IPlatformService _platformService = platformService ?? throw new ArgumentNullException(nameof(platformService));
    private readonly ILogger<ProcessExecutor> _logger = logger ?? throw new ArgumentNullException(nameof(logger));

    /// <inheritdoc />
    public async Task<ProcessResult> ExecuteAsync(ProcessRequest request, CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(request);

        var stopwatch = Stopwatch.StartNew();
        var startInfo = CreateProcessStartInfo(request);

        _logger.LogDebug("Executing command: {Command} in {WorkingDirectory}",
            request.Command,
            request.WorkingDirectory ?? "current directory");

        using var process = new Process();
        process.StartInfo = startInfo;

        var (outputBuilder, errorBuilder) = ConfigureOutputCapture(process, request);

        try
        {
            await RunProcessAsync(process, request, cancellationToken);
            stopwatch.Stop();

            var result = CreateProcessResult(process, outputBuilder, errorBuilder, stopwatch.Elapsed, request.Command);

            _logger.LogDebug("Command completed with exit code {ExitCode} in {Duration}ms",
                result.ExitCode,
                result.Duration.TotalMilliseconds);

            ValidateResult(result, request);
            return result;
        }
        catch (OperationCanceledException)
        {
            HandleProcessCancellation(process);
            throw;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Failed to execute command: {Command}. Exception type: {ExceptionType}, Message: {ExceptionMessage}",
                request.Command, ex.GetType().Name, ex.Message);

            // Log inner exception if present
            if (ex.InnerException != null)
            {
                _logger.LogError("Inner exception: {InnerType}: {InnerMessage}",
                    ex.InnerException.GetType().Name, ex.InnerException.Message);
            }

            throw new InvalidOperationException($"Failed to execute command: {request.Command}", ex);
        }
    }

    private (StringBuilder outputBuilder, StringBuilder errorBuilder) ConfigureOutputCapture(Process process, ProcessRequest request)
    {
        var outputBuilder = new StringBuilder();
        var errorBuilder = new StringBuilder();

        if (!request.CaptureOutput)
            return (outputBuilder, errorBuilder);

        process.OutputDataReceived += (_, e) =>
        {
            if (e.Data == null) return;
            outputBuilder.AppendLine(e.Data);
            _logger.LogTrace("STDOUT: {Output}", e.Data);
        };

        process.ErrorDataReceived += (_, e) =>
        {
            if (e.Data == null) return;
            errorBuilder.AppendLine(e.Data);
            _logger.LogTrace("STDERR: {Error}", e.Data);
        };

        return (outputBuilder, errorBuilder);
    }

    private static async Task RunProcessAsync(Process process, ProcessRequest request, CancellationToken cancellationToken)
    {
        try
        {
            process.Start();
        }
        catch (Exception ex)
        {
            throw new InvalidOperationException($"Failed to start process: {process.StartInfo.FileName} {process.StartInfo.Arguments}. Working directory: {process.StartInfo.WorkingDirectory}", ex);
        }

        if (request.CaptureOutput)
        {
            process.BeginOutputReadLine();
            if (request.RedirectStandardError)
            {
                process.BeginErrorReadLine();
            }
        }

        await WaitForProcessExitAsync(process, request.Timeout, cancellationToken);
    }

    private static async Task WaitForProcessExitAsync(Process process, TimeSpan? timeout, CancellationToken cancellationToken)
    {
        if (!timeout.HasValue)
        {
            await process.WaitForExitAsync(cancellationToken);
            return;
        }

        using var cts = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
        cts.CancelAfter(timeout.Value);
        await process.WaitForExitAsync(cts.Token);
    }

    private static ProcessResult CreateProcessResult(Process process, StringBuilder outputBuilder, StringBuilder errorBuilder, TimeSpan duration, string command)
    {
        return new ProcessResult
        {
            ExitCode = process.ExitCode,
            StandardOutput = outputBuilder.ToString().TrimEnd(),
            StandardError = errorBuilder.ToString().TrimEnd(),
            Duration = duration,
            Command = command
        };
    }

    private static void ValidateResult(ProcessResult result, ProcessRequest request)
    {
        if (!request.ThrowOnError || result.ExitCode == 0)
            return;

        var errorDetails = string.IsNullOrEmpty(result.StandardError)
            ? string.Empty
            : $". Error: {result.StandardError}";

        throw new InvalidOperationException(
            $"Command '{request.Command}' failed with exit code {result.ExitCode}{errorDetails}",
            new Exception($"Process exited with code {result.ExitCode}"));
    }

    private void HandleProcessCancellation(Process process)
    {
        if (!process.HasExited)
        {
            _logger.LogWarning("Process cancelled, killing process {ProcessId}", process.Id);
            process.Kill();
        }
    }

    private ProcessStartInfo CreateProcessStartInfo(ProcessRequest request)
    {
        var (fileName, arguments) = _platformService.FormatCommand(request.Command, request.Arguments);

        // Debug logging
        _logger.LogDebug("Executing command: {Command} with arguments: {Arguments}", request.Command, request.Arguments);
        _logger.LogDebug("Formatted as: {FileName} {Arguments}", fileName, arguments);
        _logger.LogDebug("Working directory: {WorkingDirectory}", request.WorkingDirectory ?? Environment.CurrentDirectory);

        var notShellExecute = !request.UseShellExecute;

        var startInfo = new ProcessStartInfo
        {
            FileName = fileName,
            Arguments = arguments,
            UseShellExecute = request.UseShellExecute,
            CreateNoWindow = notShellExecute,
            RedirectStandardOutput = request.CaptureOutput && notShellExecute,
            RedirectStandardError = request.RedirectStandardError && notShellExecute,
            WorkingDirectory = request.WorkingDirectory ?? Environment.CurrentDirectory
        };

        // Inherit current environment variables FIRST if not using shell execute
        // This ensures PATH and other system variables are available
        if (!request.UseShellExecute)
        {
            foreach (var key in Environment.GetEnvironmentVariables().Keys)
            {
                var keyStr = key.ToString();
                if (keyStr == null) continue;

                var value = Environment.GetEnvironmentVariable(keyStr);
                if (value == null) continue;

                startInfo.Environment[keyStr] = value;
            }
        }

        // Then add/override with request-specific environment variables
        foreach (var (key, value) in request.EnvironmentVariables ?? [])
        {
            startInfo.Environment[key] = value;
        }

        return startInfo;
    }
}
