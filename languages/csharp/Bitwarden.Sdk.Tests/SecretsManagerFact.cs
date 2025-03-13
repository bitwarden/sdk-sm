namespace Bitwarden.Sdk.Tests;

public class SecretsManagerFactAttribute : FactAttribute
{
    public SecretsManagerFactAttribute()
    {
        if (!TryGetEnvironment("IDENTITY_URL", out var identityUrl))
        {
            Skip = "Environment variable IDENTITY_URL was not provided.";
        }

        if (!Uri.TryCreate(identityUrl, UriKind.Absolute, out _))
        {
            Skip = $"The identity url {identityUrl} provided in IDENTITY_URL is not a valid URL.";
        }

        if (!TryGetEnvironment("API_URL", out var apiUrl))
        {
            Skip = "Environment variable API_URL was not provided.";
        }

        if (!Uri.TryCreate(apiUrl, UriKind.Absolute, out _))
        {
            Skip = $"The identity url {apiUrl} provided in API_URL is not a valid URL.";
        }

        if (!TryGetEnvironment("ORGANIZATION_ID", out var organizationId))
        {
            Skip = "Environment variable ORGANIZATION_ID was not provided.";
        }

        if (!Guid.TryParse(organizationId, out _))
        {
            Skip = $"The organization id {organizationId} provided in ORGANIZATION_ID is not a valid GUID.";
        }

        if (!TryGetEnvironment("ACCESS_TOKEN", out _))
        {
            Skip = "Environment variable ACCESS_TOKEN was not provided.";
        }
    }

    private static bool TryGetEnvironment(string variable, out string value)
    {
        value = Environment.GetEnvironmentVariable(variable);

        if (string.IsNullOrWhiteSpace(value))
        {
            return false;
        }

        return true;
    }
}
