using System;
using System.ComponentModel;
using System.Diagnostics.CodeAnalysis;
using Newtonsoft.Json.Linq;

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
        var result = await _commandRunner.RunCommandAsync<JToken>(
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

        return ParseResult(result).Value<int>();
    }

    public async Task<int> ErrorTestAsync()
    {
        var result = await _commandRunner.RunCommandAsync<JToken>(
            new Command
            {
                Debug = new DebugCommand
                {
                    ErrorTest = new ErrorTest(),
                },
            }, CancellationToken.None);

        return ParseResult(result).Value<int>();
    }

    private JToken ParseResult(JToken result)
    {
        // Expecting: { "success": true|false, "data": ..., "errorMessage": "..." }
        if (result is JObject obj && obj.Value<bool?>("success") == true)
        {
            var data = obj["data"];
            if (data is null)
            {
                throw new BitwardenException("Missing 'data' in successful response.");
            }
            return data;
        }

        var message = (result as JObject)?.Value<string>("errorMessage") ?? "Unknown error.";
        throw new BitwardenException(message);
    }
}
#endif
