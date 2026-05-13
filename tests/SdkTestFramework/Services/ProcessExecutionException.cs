using System;

namespace SdkTestFramework.Services;

/// <summary>
/// Exception thrown when process execution fails
/// </summary>
public class ProcessExecutionException : Exception
{
    public ProcessExecutionException(string message, Exception innerException)
        : base(message, innerException)
    {
    }
}