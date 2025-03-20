using System.ComponentModel;
using System.Diagnostics.CodeAnalysis;
using System.Text.Json;

namespace Bitwarden.Sdk;

[Obsolete("DebugCommand is intended for tests only, using any of these commands will throw errors in production code.")]
[EditorBrowsable(EditorBrowsableState.Never)]
partial class DebugCommand
{

}

#if DEBUG
public sealed partial class BitwardenClient
{
    public async Task<int> CancellationTestAsync(CancellationToken token)
    {
        var result = await _commandRunner.RunCommandAsync<JsonElement>(
            new Command
            {
                Debug = new DebugCommand
                {
                    CancellationTest = new CancellationTest
                    {
                        DurationMillis = 200,
                    },
                },
            }, token);

        return ParseResult(result).GetInt32();
    }

    public async Task<int> ErrorTestAsync()
    {
        var result = await _commandRunner.RunCommandAsync<JsonElement>(
            new Command
            {
                Debug = new DebugCommand
                {
                    ErrorTest = new ErrorTest(),
                },
            }, CancellationToken.None);

        return ParseResult(result).GetInt32();
    }

    private JsonElement ParseResult(JsonElement result)
    {
        if (result.GetProperty("success").GetBoolean())
        {
            return result.GetProperty("data");
        }

        throw new BitwardenException(result.GetProperty("errorMessage").GetString());
    }
}
#endif
