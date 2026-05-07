namespace SdkTestFramework.Models;

/// <summary>
/// Operating system context information
/// </summary>
public record OsContext
{
    public required OperatingSystemType OsType { get; init; }
    public required string OsDisplayName { get; init; }
    public required string Architecture { get; init; }
    public required string Version { get; init; }
    public required bool IsWindows { get; init; }
    public required bool IsMacOs { get; init; }
    public required bool IsLinux { get; init; }
    public required bool IsArm { get; init; }

    public bool IsWindowsArm() => IsWindows && IsArm; // Not supported
}

public enum OperatingSystemType
{
    Windows,
    MacOS,
    Linux,
    Unknown
}
