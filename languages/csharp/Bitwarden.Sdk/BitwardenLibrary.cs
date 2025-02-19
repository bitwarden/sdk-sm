using System.Runtime.InteropServices;

namespace Bitwarden.Sdk;

internal static partial class BitwardenLibrary
{
    [LibraryImport("bitwarden_c", StringMarshalling = StringMarshalling.Utf8)]
    private static partial BitwardenSafeHandle init(string settings);

    [LibraryImport("bitwarden_c", StringMarshalling = StringMarshalling.Utf8)]
    private static partial void free_mem(IntPtr handle);

    [LibraryImport("bitwarden_c", StringMarshalling = StringMarshalling.Utf8)]
    private static partial string run_command(string json, BitwardenSafeHandle handle);

    internal delegate void OnCompleteCallback(IntPtr json);

    [LibraryImport("bitwarden_c", StringMarshalling = StringMarshalling.Utf8)]
    private static partial IntPtr run_command_async(string json,
        BitwardenSafeHandle handle,
        OnCompleteCallback onCompletedCallback,
        [MarshalAs(UnmanagedType.U1)] bool isCancellable);

    [LibraryImport("bitwarden_c", StringMarshalling = StringMarshalling.Utf8)]
    private static partial void abort_and_free_handle(IntPtr joinHandle);

    [LibraryImport("bitwarden_c", StringMarshalling = StringMarshalling.Utf8)]
    private static partial void free_handle(IntPtr joinHandle);

    internal static BitwardenSafeHandle Init(string settings) => init(settings);

    internal static void FreeMemory(IntPtr handle) => free_mem(handle);

    internal static string RunCommand(string json, BitwardenSafeHandle handle) => run_command(json, handle);

    internal static Task<string> RunCommandAsync(string json, BitwardenSafeHandle handle, CancellationToken cancellationToken)
    {
        cancellationToken.ThrowIfCancellationRequested();
        var tcs = new TaskCompletionSource<string>(TaskCreationOptions.RunContinuationsAsynchronously);

        IntPtr abortPointer = IntPtr.Zero;

        try
        {

            abortPointer = run_command_async(json, handle, (resultPointer) =>
            {
                var stringResult = Marshal.PtrToStringUTF8(resultPointer);
                tcs.SetResult(stringResult);

                if (abortPointer != IntPtr.Zero)
                {
                    free_handle(abortPointer);
                }
            }, cancellationToken.CanBeCanceled);
        }
        catch (Exception ex)
        {
            tcs.SetException(ex);
        }

        cancellationToken.Register((state) =>
        {
            // This register delegate will never be called unless the token is cancelable
            // therefore we know that the abortPointer is a valid pointer.
            abort_and_free_handle((IntPtr)state);
            tcs.SetCanceled(cancellationToken);
        }, abortPointer);

        return tcs.Task;
    }
}
