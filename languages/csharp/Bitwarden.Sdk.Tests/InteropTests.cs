#if DEBUG
using Bitwarden.Sdk;

namespace Bitwarden.Sdk.Tests;

public class InteropTests
{
    [Fact]
    public async Task CancelingTest_ThrowsTaskCanceledException()
    {
        using var client = new BitwardenClient();
        using var cts = new CancellationTokenSource(TimeSpan.FromMilliseconds(250));

        await Assert.ThrowsAsync<TaskCanceledException>(() => client.CancellationTestAsync(cts.Token));
    }

    [Fact]
    public async Task NoCancel_TaskCompletesSuccessfully()
    {
        using var client = new BitwardenClient();

        var result = await client.CancellationTestAsync(CancellationToken.None);
        Assert.Equal(42, result);
    }

    [Fact]
    public async Task Error_ThrowsException()
    {
        using var client = new BitwardenClient();

        var bitwardenException = await Assert.ThrowsAsync<BitwardenException>(client.ErrorTestAsync);
        Assert.Equal("Internal error: This is an error.", bitwardenException.Message);
    }
}
#endif
