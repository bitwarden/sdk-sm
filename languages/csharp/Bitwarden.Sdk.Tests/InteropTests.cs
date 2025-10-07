using Bitwarden.Sdk;
using System.Diagnostics;

namespace Bitwarden.Sdk.Tests;

public class InteropTests
{
    [Fact]
    public async void CancelingTest_ThrowsTaskCanceledException()
    {
        var client = new BitwardenClient();

        var cts = new CancellationTokenSource(TimeSpan.FromMilliseconds(250));

        await Assert.ThrowsAsync<TaskCanceledException>(async () => await client.CancellationTestAsync(cts.Token));
    }

    [Fact]
    public async void NoCancel_TaskCompletesSuccessfully()
    {
        var client = new BitwardenClient();

        var result = await client.CancellationTestAsync(CancellationToken.None);
        Assert.Equal(42, result);
    }

    [Fact]
    public async void Error_ThrowsException()
    {
        var client = new BitwardenClient();

        var bitwardenException = await Assert.ThrowsAsync<BitwardenException>(async () => await client.ErrorTestAsync());
        Assert.Equal("Internal error: This is an error.", bitwardenException.Message);
    }
}
