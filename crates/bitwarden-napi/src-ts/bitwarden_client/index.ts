import * as rust from "../../binding";
import { LogLevel } from "../../binding";
import {
  Convert,
  ClientSettings,
  ProjectResponse,
  ProjectsDeleteResponse,
  ProjectsResponse,
  SecretIdentifiersResponse,
  SecretResponse,
  SecretsDeleteResponse,
  SecretsResponse,
  SecretsSyncResponse,
} from "./schemas";

function handleResponse<T>(response: {
  success: boolean;
  errorMessage?: string | null;
  data?: T | null;
}): T {
  if (!response.success) {
    throw new Error(response.errorMessage || "");
  }

  if (response.data === null) {
    throw new Error(response.errorMessage || "SDK response data is null");
  }

  return response.data as T;
}

export class BitwardenClient {
  client: rust.BitwardenClient;

  constructor(settings?: ClientSettings, loggingLevel?: LogLevel) {
    const settingsJson = settings == null ? null : Convert.clientSettingsToJson(settings);
    this.client = new rust.BitwardenClient(settingsJson, loggingLevel ?? LogLevel.Info);
  }

  secrets(): SecretsClient {
    return new SecretsClient(this.client);
  }

  projects(): ProjectsClient {
    return new ProjectsClient(this.client);
  }

  auth(): AuthClient {
    return new AuthClient(this.client);
  }
}

export class SecretsClient {
  client: rust.BitwardenClient;

  constructor(client: rust.BitwardenClient) {
    this.client = client;
  }

  async get(id: string): Promise<SecretResponse> {
    const response = await this.client.runCommand(
      Convert.commandToJson({
        secrets: {
          get: { id },
        },
      }),
    );

    return handleResponse(Convert.toResponseForSecretResponse(response));
  }

  async getByIds(ids: string[]): Promise<SecretsResponse> {
    const response = await this.client.runCommand(
      Convert.commandToJson({
        secrets: {
          getByIds: { ids },
        },
      }),
    );

    return handleResponse(Convert.toResponseForSecretsResponse(response));
  }

  async create(
    organizationId: string,
    key: string,
    value: string,
    note: string,
    projectIds: string[],
  ): Promise<SecretResponse> {
    const response = await this.client.runCommand(
      Convert.commandToJson({
        secrets: {
          create: { key, value, note, projectIds, organizationId },
        },
      }),
    );

    return handleResponse(Convert.toResponseForSecretResponse(response));
  }

  async list(organizationId: string): Promise<SecretIdentifiersResponse> {
    const response = await this.client.runCommand(
      Convert.commandToJson({
        secrets: {
          list: { organizationId },
        },
      }),
    );

    return handleResponse(Convert.toResponseForSecretIdentifiersResponse(response));
  }

  async update(
    organizationId: string,
    id: string,
    key: string,
    value: string,
    note: string,
    projectIds: string[],
  ): Promise<SecretResponse> {
    const oldSecret = await this.get(id);
    const valueChanged = value !== oldSecret.value;

    const response = await this.client.runCommand(
      Convert.commandToJson({
        secrets: {
          update: { id, key, value, note, projectIds, organizationId, valueChanged },
        },
      }),
    );

    return handleResponse(Convert.toResponseForSecretResponse(response));
  }

  async delete(ids: string[]): Promise<SecretsDeleteResponse> {
    const response = await this.client.runCommand(
      Convert.commandToJson({
        secrets: {
          delete: { ids },
        },
      }),
    );

    return handleResponse(Convert.toResponseForSecretsDeleteResponse(response));
  }

  async sync(organizationId: string, lastSyncedDate?: Date): Promise<SecretsSyncResponse> {
    const response = await this.client.runCommand(
      Convert.commandToJson({
        secrets: {
          sync: { organizationId, lastSyncedDate },
        },
      }),
    );

    return handleResponse(Convert.toResponseForSecretsSyncResponse(response));
  }
}

export class ProjectsClient {
  client: rust.BitwardenClient;

  constructor(client: rust.BitwardenClient) {
    this.client = client;
  }

  async get(id: string): Promise<ProjectResponse> {
    const response = await this.client.runCommand(
      Convert.commandToJson({
        projects: {
          get: { id },
        },
      }),
    );

    return handleResponse(Convert.toResponseForProjectResponse(response));
  }

  async create(organizationId: string, name: string): Promise<ProjectResponse> {
    const response = await this.client.runCommand(
      Convert.commandToJson({
        projects: {
          create: { name, organizationId },
        },
      }),
    );

    return handleResponse(Convert.toResponseForProjectResponse(response));
  }

  async list(organizationId: string): Promise<ProjectsResponse> {
    const response = await this.client.runCommand(
      Convert.commandToJson({
        projects: {
          list: { organizationId },
        },
      }),
    );

    return handleResponse(Convert.toResponseForProjectsResponse(response));
  }

  async update(organizationId: string, id: string, name: string): Promise<ProjectResponse> {
    const response = await this.client.runCommand(
      Convert.commandToJson({
        projects: {
          update: { id, name, organizationId },
        },
      }),
    );

    return handleResponse(Convert.toResponseForProjectResponse(response));
  }

  async delete(ids: string[]): Promise<ProjectsDeleteResponse> {
    const response = await this.client.runCommand(
      Convert.commandToJson({
        projects: {
          delete: { ids },
        },
      }),
    );

    return handleResponse(Convert.toResponseForProjectsDeleteResponse(response));
  }
}

export class AuthClient {
  client: rust.BitwardenClient;

  constructor(client: rust.BitwardenClient) {
    this.client = client;
  }

  async loginAccessToken(accessToken: string, stateFile?: string): Promise<void> {
    const response = await this.client.runCommand(
      Convert.commandToJson({
        loginAccessToken: {
          accessToken,
          stateFile,
        },
      }),
    );

    handleResponse(Convert.toResponseForAccessTokenLoginResponse(response));
  }
}
