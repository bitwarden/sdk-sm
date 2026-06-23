
namespace SdkTestFramework.Common;

/// <summary>
/// Utilities for working with file system paths in the SDK
/// </summary>
public static class PathUtilities
{
    /// <summary>
    /// Finds the repository root by looking for a marker file or directory
    /// </summary>
    /// <param name="startPath">The path to start searching from</param>
    /// <param name="marker">The marker to look for (e.g., ".git" directory or "Cargo.toml" file)</param>
    /// <param name="isDirectory">Whether the marker is a directory (true) or file (false)</param>
    /// <returns>The root path, or null if not found</returns>
    private static string? FindRootPath(string startPath, string marker, bool isDirectory = false)
    {
        if (Path.IsPathRooted(marker))
        {
            throw new ArgumentException("Marker must be a relative path.", nameof(marker));
        }

        var dir = new DirectoryInfo(startPath);

        while (dir != null)
        {
            var markerPath = Path.Combine(dir.FullName, marker);
            var exists = isDirectory ? Directory.Exists(markerPath) : File.Exists(markerPath);

            if (exists)
            {
                return dir.FullName;
            }

            dir = dir.Parent;
        }

        return null;
    }

    /// <summary>
    /// Finds the repository root by looking for .git directory
    /// </summary>
    public static string? FindRepositoryRoot(string startPath)
    {
        return FindRootPath(startPath, ".git", isDirectory: true);
    }

    /// <summary>
    /// Finds the SDK root by looking for Cargo.toml file with [workspace] marker
    /// </summary>
    private static string? FindSdkRoot(string startPath)
    {
        var dir = new DirectoryInfo(startPath);

        while (dir != null)
        {
            var cargoTomlPath = Path.Combine(dir.FullName, "Cargo.toml");

            if (File.Exists(cargoTomlPath))
            {
                // Read the file and check for [workspace] marker
                try
                {
                    var content = File.ReadAllText(cargoTomlPath);
                    if (content.Contains("[workspace]"))
                    {
                        // This is the workspace root
                        return dir.FullName;
                    }
                }
                catch
                {
                    // If we can't read the file, continue searching
                }
            }

            dir = dir.Parent;
        }

        return null;
    }

    /// <summary>
    /// Gets the current SDK root path
    /// </summary>
    public static string? GetSdkRootPath()
    {
        return FindSdkRoot(Directory.GetCurrentDirectory());
    }
}
