namespace SdkTestFramework.Common;

/// <summary>
/// Centralized console formatting constants and utilities
/// </summary>
public static class ConsoleFormatting
{
    /// <summary>
    /// Box drawing characters for headers
    /// </summary>
    public const string BoxTop = "╔══════════════════════════════════════════════════════════════════╗";
    public const string BoxBottom = "╚══════════════════════════════════════════════════════════════════╝";
    private const string BoxSide = "║";
    private const int BoxWidth = 70; // Total width including borders

    /// <summary>
    /// Line separators
    /// </summary>
    public const string LineSeparator = "═══════════════════════════════════════════════════════════════════";
    public const string DashedLine = "-----------------------------------";

    /// <summary>
    /// Creates a centered header text within box borders
    /// </summary>
    public static string CreateBoxedHeader(string text)
    {
        var padding = BoxWidth - 2 - text.Length; // 2 for the side borders
        var leftPadding = padding / 2;
        var rightPadding = padding - leftPadding;

        return $"{BoxSide}{new string(' ', leftPadding)}{text}{new string(' ', rightPadding)}{BoxSide}";
    }

    /// <summary>
    /// Prints a complete boxed header with the given title
    /// </summary>
    public static void PrintBoxedHeader(string title)
    {
        Console.WriteLine(BoxTop);
        Console.WriteLine(CreateBoxedHeader(title));
        Console.WriteLine(BoxBottom);
        Console.WriteLine();
    }
}
