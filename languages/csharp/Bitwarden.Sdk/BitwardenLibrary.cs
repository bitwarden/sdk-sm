using System.Runtime.InteropServices;

namespace Bitwarden.Sdk;

internal static partial class BitwardenLibrary
{
    [DllImport("bitwarden_c", CharSet = CharSet.Ansi, EntryPoint = "init")]
    private static extern BitwardenSafeHandle init(string settings);

    [DllImport("bitwarden_c", CharSet = CharSet.Ansi, EntryPoint = "free_mem")]
    private static extern void free_mem(IntPtr handle);

    [DllImport("bitwarden_c", CharSet = CharSet.Ansi, EntryPoint = "run_command")]
    private static extern IntPtr run_command(string json, BitwardenSafeHandle handle);

    internal delegate void OnCompleteCallback(IntPtr json);

    [DllImport("bitwarden_c", CharSet = CharSet.Ansi, EntryPoint = "run_command_async")]
    private static extern IntPtr run_command_async(string json,
        BitwardenSafeHandle handle,
        OnCompleteCallback onCompletedCallback,
        [MarshalAs(UnmanagedType.U1)] bool isCancellable);

    [DllImport("bitwarden_c", CharSet = CharSet.Ansi, EntryPoint = "abort_and_free_handle")]
    private static extern void abort_and_free_handle(IntPtr joinHandle);

    [DllImport("bitwarden_c", CharSet = CharSet.Ansi, EntryPoint = "free_handle")]
    private static extern void free_handle(IntPtr joinHandle);

    internal static BitwardenSafeHandle Init(string settings) => init(settings);

    internal static void FreeMemory(IntPtr handle) => free_mem(handle);

    internal static string RunCommand(string json, BitwardenSafeHandle handle)
    {
        IntPtr resultPtr = run_command(json, handle);
        return Marshal.PtrToStringAnsi(resultPtr);
    }

    internal static Task<string> RunCommandAsync(string json, BitwardenSafeHandle handle, CancellationToken cancellationToken)
    {
        cancellationToken.ThrowIfCancellationRequested();
        var tcs = new TaskCompletionSource<string>(TaskCreationOptions.RunContinuationsAsynchronously);

        IntPtr abortPointer = IntPtr.Zero;

        try
        {

            abortPointer = run_command_async(json, handle, (resultPointer) =>
            {
                var stringResult = Marshal.PtrToStringAnsi(resultPointer);
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
            tcs.SetCanceled();
        }, abortPointer);

        return tcs.Task;
    }
}
