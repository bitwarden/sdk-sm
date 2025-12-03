import {
  Convert,
  ProjectResponse,
  ProjectsDeleteResponse,
  ProjectsResponse,
  SecretIdentifiersResponse,
  SecretResponse,
  SecretsDeleteResponse,
} from "./schemas";

interface BitwardenSDKClient {
  run_command(js_input: string): Promise<any>;
}

function handleResponse<T>(response: { success: boolean; errorMessage?: string; data?: T }): T {
  if (!response.success) {
    throw new Error(response.errorMessage);
  }
  return response.data as T;
}

export class BitwardenClient {
  client: BitwardenSDKClient;

  constructor(client: BitwardenSDKClient) {
    this.client = client;
  }

  async accessTokenLogin(accessToken: string): Promise<void> {
    const response = await this.client.run_command(
      Convert.commandToJson({
        loginAccessToken: {
          accessToken,
        },
      }),
    );

    handleResponse(Convert.toResponseForAccessTokenLoginResponse(response));
  }

  secrets(): SecretsClient {
    return new SecretsClient(this.client);
  }

  projects(): ProjectsClient {
    return new ProjectsClient(this.client);
  }
}

export class SecretsClient {
  client: BitwardenSDKClient;

  constructor(client: BitwardenSDKClient) {
    this.client = client;
  }

  async get(id: string): Promise<SecretResponse> {
    const response = await this.client.run_command(
      Convert.commandToJson({
        secrets: {
          get: { id },
        },
      }),
    );

    return handleResponse(Convert.toResponseForSecretResponse(response));
  }

  async create(
    key: string,
    value: string,
    note: string,
    projectIds: string[],
    organizationId: string,
  ): Promise<SecretResponse> {
    const response = await this.client.run_command(
      Convert.commandToJson({
        secrets: {
          create: { key, value, note, projectIds, organizationId },
        },
      }),
    );

    return handleResponse(Convert.toResponseForSecretResponse(response));
  }

  async list(organizationId: string): Promise<SecretIdentifiersResponse> {
    const response = await this.client.run_command(
      Convert.commandToJson({
        secrets: {
          list: { organizationId },
        },
      }),
    );

    return handleResponse(Convert.toResponseForSecretIdentifiersResponse(response));
  }

  async update(
    id: string,
    key: string,
    value: string,
    note: string,
    projectIds: string[],
    organizationId: string,
  ): Promise<SecretResponse> {
    const response = await this.client.run_command(
      Convert.commandToJson({
        secrets: {
          update: { id, key, value, note, projectIds, organizationId },
        },
      }),
    );

    return handleResponse(Convert.toResponseForSecretResponse(response));
  }

  async delete(ids: string[]): Promise<SecretsDeleteResponse> {
    const response = await this.client.run_command(
      Convert.commandToJson({
        secrets: {
          delete: { ids },
        },
      }),
    );

    return handleResponse(Convert.toResponseForSecretsDeleteResponse(response));
  }
}

export class ProjectsClient {
  client: BitwardenSDKClient;

  constructor(client: BitwardenSDKClient) {
    this.client = client;
  }

  async get(id: string): Promise<ProjectResponse> {
    const response = await this.client.run_command(
      Convert.commandToJson({
        projects: {
          get: { id },
        },
      }),
    );

    return handleResponse(Convert.toResponseForProjectResponse(response));
  }

  async create(name: string, organizationId: string): Promise<ProjectResponse> {
    const response = await this.client.run_command(
      Convert.commandToJson({
        projects: {
          create: { name, organizationId },
        },
      }),
    );

    return handleResponse(Convert.toResponseForProjectResponse(response));
  }

  async list(organizationId: string): Promise<ProjectsResponse> {
    const response = await this.client.run_command(
      Convert.commandToJson({
        projects: {
          list: { organizationId },
        },
      }),
    );

    return handleResponse(Convert.toResponseForProjectsResponse(response));
  }

  async update(id: string, name: string, organizationId: string): Promise<ProjectResponse> {
    const response = await this.client.run_command(
      Convert.commandToJson({
        projects: {
          update: { id, name, organizationId },
        },
      }),
    );

    return handleResponse(Convert.toResponseForProjectResponse(response));
  }

  async delete(ids: string[]): Promise<ProjectsDeleteResponse> {
    const response = await this.client.run_command(
      Convert.commandToJson({
        projects: {
          delete: { ids },
        },
      }),
    );

    return handleResponse(Convert.toResponseForProjectsDeleteResponse(response));
  }
}
