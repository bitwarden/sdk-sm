using NUnit.Framework;
using SdkTestFramework.TestRunners;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.DependencyInjection;
using System.Collections;

namespace SdkTestFramework.Tests.SdkWrappers;

/// <summary>
/// Python SDK integration tests with per-operation test case generation
/// </summary>
[TestFixture]
[Property("Language", "Python")]
public class PythonTests : SdkTestBase
{
    protected override string SdkLanguage => "Python";

    protected override BaseTestRunner CreateTestRunner()
    {
        var logger = TestHelper.GetService<IServiceProvider>()
            .GetRequiredService<ILogger<PythonTestRunner>>();

        return new PythonTestRunner(logger, ProcessExecutor, PlatformService, TestConfig);
    }

    /// <summary>
    /// Provides test cases for each operation in the Python test suite
    /// NUnit requires this to be static for TestCaseSource attribute
    /// </summary>
    public static IEnumerable GetPythonTestOperations() => CreateTestOperations("Python");

    /// <summary>
    /// Execute and validate individual test operations
    /// Each operation becomes a separate test in CI/test reports
    /// </summary>
    [Test]
    [TestCaseSource(nameof(GetPythonTestOperations))]
    public async Task Python_Operation_Should_Pass(string operationName)
    {
        // Get the full test result (cached after first run)
        var result = await GetCachedTestResultAsync();

        // Find the specific operation in the results
        var operation = result.Operations.FirstOrDefault(op => op.Operation == operationName);

        // Assert the operation exists
        Assert.That(operation, Is.Not.Null,
            $"Operation '{operationName}' not found in test results. Available operations: {string.Join(", ", result.Operations.Select(op => op.Operation))}");

        // Check if the operation succeeded
        if (!operation!.Success)
        {
            var errorMessage = operation.Error ?? "No error message provided";
            Assert.Fail($"Operation '{operationName}' failed: {errorMessage}");
        }

        // Log operation details for CI visibility
        TestContext.WriteLine($"✅ Operation: {operationName}");
        TestContext.WriteLine($"   Duration: {operation.DurationMs}ms");
    }
}
