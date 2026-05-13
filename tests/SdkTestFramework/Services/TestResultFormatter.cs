using System.Text;
using SdkTestFramework.Common;
using SdkTestFramework.Models;

namespace SdkTestFramework.Services;

/// <summary>
/// Formats test results for human-readable output
/// </summary>
public class TestResultFormatter : ITestResultFormatter
{
    public string FormatTestResult(TestResult result)
    {
        var sb = new StringBuilder();

        AppendHeader(sb, result.Language);
        AppendSummary(sb, result);
        AppendTestStatistics(sb, result);
        AppendTestOperations(sb, result);
        AppendErrorDetails(sb, result.Error);
        AppendFooter(sb);

        return sb.ToString();
    }

    private static void AppendHeader(StringBuilder sb, string language)
    {
        sb.AppendLine();
        sb.AppendLine(ConsoleFormatting.LineSeparator);
        sb.AppendLine($"  {language} SDK Test Results");
        sb.AppendLine(ConsoleFormatting.LineSeparator);
        sb.AppendLine();
    }

    private static void AppendSummary(StringBuilder sb, TestResult result)
    {
        var statusIcon = result.Success ? "✅" : "❌";
        var statusText = result.Success ? "PASSED" : "FAILED";

        sb.AppendLine($"  Status: {statusIcon} {statusText}");
        sb.AppendLine($"  Platform: {result.OperatingSystem} ({result.Architecture})");
        sb.AppendLine($"  Duration: {result.Duration.TotalSeconds:F2}s");
        sb.AppendLine();
    }

    private static void AppendTestStatistics(StringBuilder sb, TestResult result)
    {
        sb.AppendLine("  Test Summary:");
        sb.AppendLine($"  ├─ Total:   {result.TotalTests,3} tests");
        sb.AppendLine($"  ├─ Passed:  {result.PassedTests,3} ✅");
        sb.AppendLine($"  ├─ Failed:  {result.FailedTests,3} ❌");
        sb.AppendLine($"  └─ Skipped: {result.SkippedTests,3} ⏭️");
        sb.AppendLine();
    }

    private static void AppendTestOperations(StringBuilder sb, TestResult result)
    {
        if (!result.Operations.Any())
            return;

        sb.AppendLine("  Test Operations:");
        sb.AppendLine("  " + new string('─', 58));

        var groupedOperations = result.Operations
            .GroupBy(op => GetSuiteName(op.Operation))
            .OrderBy(g => g.Key);

        foreach (var group in groupedOperations)
        {
            AppendTestGroup(sb, group);
        }
    }

    private static void AppendTestGroup(StringBuilder sb, IGrouping<string, TestOperation> group)
    {
        sb.AppendLine($"  📦 {group.Key}");

        var operations = group.ToList();
        for (int i = 0; i < operations.Count; i++)
        {
            var op = operations[i];
            var isLast = i == operations.Count - 1;
            AppendTestOperation(sb, op, isLast);
        }
        sb.AppendLine();
    }

    private static void AppendTestOperation(StringBuilder sb, TestOperation op, bool isLast)
    {
        var prefix = isLast ? "└─" : "├─";
        var statusSymbol = op.Success ? "✅" : "❌";
        var testName = GetTestName(op.Operation);
        var duration = op.DurationMs > 0 ? $" ({op.DurationMs}ms)" : "";

        sb.AppendLine($"     {prefix} {statusSymbol} {testName}{duration}");

        AppendTestMessage(sb, op.Message, isLast);
        AppendTestError(sb, op.Error, isLast);
    }

    private static void AppendTestMessage(StringBuilder sb, string? message, bool isLast)
    {
        if (string.IsNullOrEmpty(message) || message == "Test passed")
            return;

        var messagePrefix = isLast ? "   " : "│  ";
        sb.AppendLine($"     {messagePrefix}    💬 {message}");
    }

    private static void AppendTestError(StringBuilder sb, string? error, bool isLast)
    {
        if (string.IsNullOrEmpty(error))
            return;

        var errorPrefix = isLast ? "   " : "│  ";
        var errorLines = error.Split('\n', StringSplitOptions.RemoveEmptyEntries);

        foreach (var line in errorLines.Take(3))
        {
            sb.AppendLine($"     {errorPrefix}    ⚠️  {line}");
        }

        if (errorLines.Length > 3)
        {
            sb.AppendLine($"     {errorPrefix}    ... ({errorLines.Length - 3} more lines)");
        }
    }

    private static void AppendErrorDetails(StringBuilder sb, string? error)
    {
        if (string.IsNullOrEmpty(error))
            return;

        sb.AppendLine("  ⚠️  Error Details:");
        sb.AppendLine("  " + new string('─', 58));

        var errorLines = error.Split('\n');
        foreach (var line in errorLines)
        {
            sb.AppendLine($"  {line}");
        }
        sb.AppendLine();
    }

    private static void AppendFooter(StringBuilder sb)
    {
        sb.AppendLine(ConsoleFormatting.LineSeparator);
    }

    private static string GetSuiteName(string operation)
    {
        var parts = operation.Split('.');
        return parts.Length > 1 ? string.Join(".", parts.Take(parts.Length - 1)) : "Tests";
    }

    private static string GetTestName(string operation)
    {
        var parts = operation.Split('.');
        return parts.Length > 0 ? parts[^1] : operation;
    }
}

/// <summary>
/// Interface for test result formatting
/// </summary>
public interface ITestResultFormatter
{
    /// <summary>
    /// Format a complete test result for display
    /// </summary>
    string FormatTestResult(TestResult result);
}
