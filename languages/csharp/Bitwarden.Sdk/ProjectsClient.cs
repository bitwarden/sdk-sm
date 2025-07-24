namespace Bitwarden.Sdk;

public class ProjectsClient
{
    private readonly CommandRunner _commandRunner;

    internal ProjectsClient(CommandRunner commandRunner)
    {
        _commandRunner = commandRunner;
    }

    public async Task<ProjectResponse> GetAsync(Guid id, CancellationToken cancellationToken = default)
    {
        var command = new Command { Projects = new ProjectsCommand { Get = new ProjectGetRequest { Id = id } } };
        var result = await _commandRunner.RunCommandAsync<ResponseForProjectResponse>(command, cancellationToken);

        if (result is { Success: true })
        {
            return result.Data;
        }

        throw new BitwardenException(result != null ? result.ErrorMessage : "Project not found");
    }

    public async Task<ProjectResponse> CreateAsync(Guid organizationId, string name, CancellationToken cancellationToken = default)
    {
        var command = new Command
        {
            Projects = new ProjectsCommand
            {
                Create = new ProjectCreateRequest { OrganizationId = organizationId, Name = name }
            }
        };
        var result = await _commandRunner.RunCommandAsync<ResponseForProjectResponse>(command, cancellationToken);

        if (result is { Success: true })
        {
            return result.Data;
        }

        throw new BitwardenException(result != null ? result.ErrorMessage : "Project create failed");
    }

    public async Task<ProjectResponse> UpdateAsync(Guid organizationId, Guid id, string name, CancellationToken cancellationToken = default)
    {
        var command = new Command
        {
            Projects = new ProjectsCommand
            {
                Update = new ProjectPutRequest { Id = id, OrganizationId = organizationId, Name = name }
            }
        };
        var result = await _commandRunner.RunCommandAsync<ResponseForProjectResponse>(command, cancellationToken);

        if (result is { Success: true })
        {
            return result.Data;
        }

        throw new BitwardenException(result != null ? result.ErrorMessage : "Project update failed");
    }

    public async Task<ProjectsDeleteResponse> DeleteAsync(Guid[] ids, CancellationToken cancellationToken = default)
    {
        var command = new Command
        {
            Projects = new ProjectsCommand { Delete = new ProjectsDeleteRequest { Ids = ids } }
        };
        var result = await _commandRunner.RunCommandAsync<ResponseForProjectsDeleteResponse>(command, cancellationToken);

        if (result is { Success: true })
        {
            return result.Data;
        }

        throw new BitwardenException(result != null ? result.ErrorMessage : "Project delete failed");
    }

    public async Task<ProjectsResponse> ListAsync(Guid organizationId, CancellationToken cancellationToken = default)
    {
        var command = new Command
        {
            Projects = new ProjectsCommand { List = new ProjectsListRequest { OrganizationId = organizationId } }
        };
        var result = await _commandRunner.RunCommandAsync<ResponseForProjectsResponse>(command, cancellationToken);

        if (result is { Success: true })
        {
            return result.Data;
        }

        throw new BitwardenException(result != null ? result.ErrorMessage : "No projects for given organization");
    }
}
