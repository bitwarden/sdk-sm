using NUnit.Framework;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;
using SdkTestFramework.Platform;
using SdkTestFramework.Services;

namespace SdkTestFramework.Tests;

[TestFixture]
public class ProcessExecutorTests
{
    private IProcessExecutor _processExecutor = null!;
    private ILogger<ProcessExecutorTests> _logger = null!;

    [OneTimeSetUp]
    public void Setup()
    {
        var services = new ServiceCollection();
        services.AddLogging(builder =>
        {
            builder.SetMinimumLevel(LogLevel.Debug);
            builder.AddConsole();
        });

        services.AddSingleton<IPlatformService>(_ => PlatformDetector.CreatePlatformService());
        services.AddSingleton<IProcessExecutor, ProcessExecutor>();

        var provider = services.BuildServiceProvider();
        _processExecutor = provider.GetRequiredService<IProcessExecutor>();
        _logger = provider.GetRequiredService<ILogger<ProcessExecutorTests>>();
    }

    [Test]
    public async Task ProcessExecutor_Should_Execute_Go_Version()
    {
        _logger.LogInformation("Testing ProcessExecutor with 'go version'...");
        _logger.LogInformation("Current PATH: {Path}", Environment.GetEnvironmentVariable("PATH"));

        var result = await _processExecutor.ExecuteAsync(new ProcessRequest
        {
            Command = "go",
            Arguments = "version",
            ThrowOnError = false,
            Timeout = TimeSpan.FromSeconds(10)
        });

        _logger.LogInformation("Result Success: {Success}", result.Success);
        _logger.LogInformation("Exit Code: {ExitCode}", result.ExitCode);
        _logger.LogInformation("StandardOutput: {Output}", result.StandardOutput);
        _logger.LogInformation("StandardError: {Error}", result.StandardError);

        Assert.That(result.Success, Is.True, $"Go execution should succeed. Output: {result.StandardOutput}, Error: {result.StandardError}");
        Assert.That(result.StandardOutput, Does.Contain("go version"));
    }

    [Test]
    public async Task ProcessExecutor_Should_Execute_Simple_Echo()
    {
        _logger.LogInformation("Testing ProcessExecutor with 'echo'...");

        var result = await _processExecutor.ExecuteAsync(new ProcessRequest
        {
            Command = "echo",
            Arguments = "hello",
            ThrowOnError = false,
            Timeout = TimeSpan.FromSeconds(5)
        });

        _logger.LogInformation("Echo Result Success: {Success}", result.Success);
        _logger.LogInformation("Echo Output: {Output}", result.StandardOutput);

        Assert.That(result.Success, Is.True);
        Assert.That(result.StandardOutput.Trim(), Is.EqualTo("hello"));
    }
}