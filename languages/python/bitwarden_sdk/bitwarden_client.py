import json
from typing import Any, List, Optional
from uuid import UUID

import bitwarden_py

from .schemas import (
    AccessTokenLoginRequest,
    ClientSettings,
    Command,
    GeneratorsCommand,
    PasswordGeneratorRequest,
    ProjectCreateRequest,
    ProjectGetRequest,
    ProjectPutRequest,
    ProjectsCommand,
    ProjectsDeleteRequest,
    ProjectsListRequest,
    ResponseForAccessTokenLoginResponse,
    ResponseForProjectResponse,
    ResponseForProjectsDeleteResponse,
    ResponseForProjectsResponse,
    ResponseForSecretIdentifiersResponse,
    ResponseForSecretResponse,
    ResponseForSecretsDeleteResponse,
    ResponseForSecretsResponse,
    ResponseForSecretsSyncResponse,
    ResponseForString,
    SecretCreateRequest,
    SecretGetRequest,
    SecretIdentifiersRequest,
    SecretPutRequest,
    SecretsCommand,
    SecretsDeleteRequest,
    SecretsGetRequest,
    SecretsSyncRequest,
)


class BitwardenClient:
    def __init__(self, settings: ClientSettings = None):
        if settings is None:
            self.inner = bitwarden_py.BitwardenClient(None)
        else:
            settings_json = json.dumps(settings.to_dict())
            self.inner = bitwarden_py.BitwardenClient(settings_json)

    def auth(self):
        return AuthClient(self)

    def secrets(self):
        return SecretsClient(self)

    def projects(self):
        return ProjectsClient(self)

    def generators(self):
        return GeneratorsClient(self)

    def _run_command(self, command: Command) -> Any:
        response_json = self.inner.run_command(json.dumps(command.to_dict()))
        response = json.loads(response_json)

        if response["success"] is False:
            raise Exception(response["errorMessage"])

        return response


class AuthClient:
    def __init__(self, client: BitwardenClient):
        self.client = client

    def login_access_token(
        self, access_token: str, state_file: str = None
    ) -> ResponseForAccessTokenLoginResponse:
        result = self.client._run_command(
            Command(
                login_access_token=AccessTokenLoginRequest(access_token, state_file)
            )
        )
        return ResponseForAccessTokenLoginResponse.from_dict(result)


class SecretsClient:
    """
    A client for managing secrets in Bitwarden Secrets Manager.

    This client provides methods to create, read, update, delete, and synchronize secrets.
    All operations require authentication with an access token.
    """

    def __init__(self, client: BitwardenClient):
        self.client = client

    def get(self, id: str) -> ResponseForSecretResponse:
        """
        Retrieve a single secret by its UUID. If you need to retrieve multiple secrets,
        consider using the get_by_ids() method to minimize network requests.

        Args:
            id (str): The UUID of the secret to retrieve

        Returns:
            ResponseForSecretResponse: A response containing the secret data if successful,
                                     or error information if the operation failed

        Raises:
            Exception: If the request fails due to network issues, authentication problems,
                      if the secret doesn't exist, or read access is denied

        Note:
            Requires authentication with an access token that has read access to the
            project containing the secret.
        """
        result = self.client._run_command(
            Command(secrets=SecretsCommand(get=SecretGetRequest(id)))
        )
        return ResponseForSecretResponse.from_dict(result)

    def get_by_ids(self, ids: List[UUID]) -> ResponseForSecretsResponse:
        """
        Retrieve multiple secrets by their UUIDs.

        Args:
            ids (List[UUID]): A list of UUIDs of the secrets to retrieve

        Returns:
            ResponseForSecretsResponse: A response containing a list of secret data if successful,
                                      or error information if the operation failed

        Raises:
            Exception: If the request fails due to network issues, authentication problems,
                      if the secrets don't exist, or read access is denied

        Note:
            Requires authentication with an access token that has read access to the
            project containing the secrets.
        """
        result = self.client._run_command(
            Command(secrets=SecretsCommand(get_by_ids=SecretsGetRequest(ids)))
        )
        return ResponseForSecretsResponse.from_dict(result)

    def create(
        self,
        organization_id: UUID,
        key: str,
        value: str,
        note: Optional[str],
        project_ids: Optional[List[UUID]] = None,
    ) -> ResponseForSecretResponse:
        """
        Create a new secret in the specified organization.

        Args:
            organization_id (UUID): The UUID of the organization where the secret will be created
            key (str): The name of the secret
            value (str): The secret value to store (e.g., password, API key, certificate)
            note (Optional[str]): Optional note or description of the secret. If None, an empty string is used
            project_ids (Optional[List[UUID]]): Optional list of project IDs that this secret should be associated with

        Returns:
            ResponseForSecretResponse: A response containing the newly created secret data if successful,
                                     or error information if the operation failed

        Raises:
            Exception: If the request fails due to network issues, authentication problems,
                      invalid input data, or write access is denied

        Note:
            Requires authentication with an access token that has write permissions
            for the specified organization.
        """
        if note is None:
            # secrets api does not accept empty notes
            note = ""
        result = self.client._run_command(
            Command(
                secrets=SecretsCommand(
                    create=SecretCreateRequest(
                        key, note, organization_id, value, project_ids
                    )
                )
            )
        )
        return ResponseForSecretResponse.from_dict(result)

    def list(self, organization_id: str) -> ResponseForSecretIdentifiersResponse:
        """
        List all secret identifiers for the specified organization.

        This method returns basic information (ID, key, organization ID) for all secrets
        that the authenticated user has access to within the organization. It does not include
        secret values. To retrieve the actual secret values, use the get() or get_by_ids() methods
        with the IDs returned by this method.

        Args:
            organization_id (str): The UUID of the organization to list secrets from

        Returns:
            ResponseForSecretIdentifiersResponse: A response containing a list of secret identifiers
                                                if successful, or error information if the operation failed

        Raises:
            Exception: If the request fails due to network issues, authentication problems,
                      if the organization doesn't exist, or access is denied

        Note:
            Requires authentication with an access token that has read permissions
            for the specified organization.
        """
        result = self.client._run_command(
            Command(
                secrets=SecretsCommand(list=SecretIdentifiersRequest(organization_id))
            )
        )
        return ResponseForSecretIdentifiersResponse.from_dict(result)

    def update(
        self,
        organization_id: str,
        id: str,
        key: str,
        value: str,
        note: Optional[str],
        project_ids: Optional[List[UUID]] = None,
    ) -> ResponseForSecretResponse:
        """
        Update an existing secret with new data.

        Args:
            organization_id (str): The UUID of the organization containing the secret
            id (str): The UUID of the secret to update
            key (str): The updated name of the secret
            value (str): The updated secret value
            note (Optional[str]): Updated note or description for the secret. If None, an empty string is used
            project_ids (Optional[List[UUID]]): Updated list of project IDs that this secret should be associated with

        Returns:
            ResponseForSecretResponse: A response containing the updated secret data if successful,
                                     or error information if the operation failed

        Raises:
            Exception: If the request fails due to network issues, authentication problems,
                      insufficient permissions, or if the secret doesn't exist

        Note:
            Requires authentication with an access token that has write permissions
            for the secret. All fields are updated with the provided values, so ensure
            all parameters contain the desired final state of the secret.
        """
        if note is None:
            # secrets api does not accept empty notes
            note = ""
        result = self.client._run_command(
            Command(
                secrets=SecretsCommand(
                    update=SecretPutRequest(
                        id, key, note, organization_id, value, project_ids
                    )
                )
            )
        )
        return ResponseForSecretResponse.from_dict(result)

    def delete(self, ids: List[str]) -> ResponseForSecretsDeleteResponse:
        """
        Delete one or more secrets by their UUID(s).

        Args:
            ids (List[str]): A list of UUIDs for the secrets to delete

        Returns:
            ResponseForSecretsDeleteResponse: A response containing the results of the deletion
                                            operation, including any errors for individual secrets

        Raises:
            Exception: If the request fails due to network issues or authentication problems

        Note:
            Requires authentication with an access token that has write permissions
            for the secret(s). The response will contain individual success/failure status
            for each secret ID provided. Some secrets may be successfully deleted while
            others fail due to permissions or other issues.
        """
        result = self.client._run_command(
            Command(secrets=SecretsCommand(delete=SecretsDeleteRequest(ids)))
        )
        return ResponseForSecretsDeleteResponse.from_dict(result)

    def sync(
        self, organization_id: str, last_synced_date: Optional[str]
    ) -> ResponseForSecretsSyncResponse:
        """
        Synchronize secrets for the specified organization since a given date. If no
        last_synced_date is provided, all secrets will be returned.

        This method retrieves all secrets accessible by the authenticated machine account.
        If a last_synced_date is provided, it will only return secrets if there have been
        changes since that date. This is useful for efficient incremental synchronization.

        Args:
            organization_id (str): The UUID of the organization to sync secrets from
            last_synced_date (Optional[str]): Optional datetime string representing
                                             when secrets were last synchronized. If provided,
                                             only changes since this date will be included

        Returns:
            ResponseForSecretsSyncResponse: A response containing sync results with a flag
                                          indicating if changes occurred, and the secret data
                                          if changes were detected

        Raises:
            Exception: If the request fails due to network issues, authentication problems,
                      or if the organization doesn't exist or access is denied

        Note:
            Requires authentication with an access token that has read permissions
            for the specified organization. Use this method for efficient bulk operations
            and synchronization workflows.
        """
        result = self.client._run_command(
            Command(
                secrets=SecretsCommand(
                    sync=SecretsSyncRequest(organization_id, last_synced_date)
                )
            )
        )
        return ResponseForSecretsSyncResponse.from_dict(result)


class ProjectsClient:
    """
    A client for managing projects in Bitwarden Secrets Manager.

    This client provides methods to create, read, update, delete, and list projects
    within Bitwarden organizations. Projects are used to organize and control access
    to secrets. All operations require authentication with an access token.
    """

    def __init__(self, client: BitwardenClient):
        self.client = client

    def get(self, id: str) -> ResponseForProjectResponse:
        """
        Retrieve a project by its UUID.

        Args:
            id (str): The UUID of the project to retrieve

        Returns:
            ResponseForProjectResponse: A response containing the project data if successful,
                                      or error information if the operation failed

        Raises:
            Exception: If the request fails due to network issues, authentication problems,
                      or if the project doesn't exist or access is denied

        Note:
            Requires authentication with an access token that has read permissions
            for the project's organization.
        """
        result = self.client._run_command(
            Command(projects=ProjectsCommand(get=ProjectGetRequest(id)))
        )
        return ResponseForProjectResponse.from_dict(result)

    def create(
        self,
        organization_id: str,
        name: str,
    ) -> ResponseForProjectResponse:
        """
        Create a new project in the specified organization.

        Args:
            organization_id (str): The UUID of the organization where the project will be created
            name (str): The name of the project

        Returns:
            ResponseForProjectResponse: A response containing the newly created project data if successful,
                                      or error information if the operation failed

        Raises:
            Exception: If the request fails due to network issues, authentication problems,
                      insufficient permissions, or invalid input data

        Note:
            Requires authentication with an access token that has create permissions
            for the specified organization.
        """
        result = self.client._run_command(
            Command(
                projects=ProjectsCommand(
                    create=ProjectCreateRequest(name, organization_id)
                )
            )
        )
        return ResponseForProjectResponse.from_dict(result)

    def list(self, organization_id: str) -> ResponseForProjectsResponse:
        """
        List all projects for the specified organization.

        This method returns information about all projects that the authenticated account
        has access to within the organization.

        Args:
            organization_id (str): The UUID of the organization to list projects from

        Returns:
            ResponseForProjectsResponse: A response containing a list of project data if successful,
                                       or error information if the operation failed

        Raises:
            Exception: If the request fails due to network issues, authentication problems,
                      if the organization doesn't exist, or access is denied

        Note:
            Requires authentication with an access token that has read permissions
            for the specified organization.
        """
        result = self.client._run_command(
            Command(projects=ProjectsCommand(list=ProjectsListRequest(organization_id)))
        )
        return ResponseForProjectsResponse.from_dict(result)

    def update(
        self,
        organization_id: str,
        id: str,
        name: str,
    ) -> ResponseForProjectResponse:
        """
        Update an existing project with new data.

        Args:
            organization_id (str): The UUID of the organization containing the project
            id (str): The UUID of the project to update
            name (str): The updated name of the project

        Returns:
            ResponseForProjectResponse: A response containing the updated project data if successful,
                                      or error information if the operation failed

        Raises:
            Exception: If the request fails due to network issues, authentication problems,
                      insufficient permissions, invalid input data, or if the project doesn't exist

        Note:
            Requires authentication with an access token that has write permissions
            for the project. The project name will be updated to the provided value.
        """
        result = self.client._run_command(
            Command(
                projects=ProjectsCommand(
                    update=ProjectPutRequest(id, name, organization_id)
                )
            )
        )
        return ResponseForProjectResponse.from_dict(result)

    def delete(self, ids: List[str]) -> ResponseForProjectsDeleteResponse:
        """
        Delete one or more projects by their UUIDs.

        Args:
            ids (List[str]): A list of UUIDs of the projects to delete

        Returns:
            ResponseForProjectsDeleteResponse: A response containing the results of the deletion
                                             operation, including any errors for individual projects

        Raises:
            Exception: If the request fails due to network issues or authentication problems

        Note:
            Requires authentication with an access token that has delete permissions
            for the projects. The response will contain individual success/failure status
            for each project ID provided. Some projects may be successfully deleted while
            others fail due to permissions or other issues.
        """
        result = self.client._run_command(
            Command(projects=ProjectsCommand(delete=ProjectsDeleteRequest(ids)))
        )
        return ResponseForProjectsDeleteResponse.from_dict(result)


class GeneratorsClient:
    """
    A client to generate secrets. Does not require authentication.
    """

    def __init__(self, client: BitwardenClient):
        self.client = client

    def generate(
        self,
        length: int = 24,
        avoid_ambiguous: bool = True,
        lowercase: bool = True,
        uppercase: bool = True,
        numbers: bool = True,
        special: bool = True,
        min_lowercase: Optional[int] = None,
        min_number: Optional[int] = None,
        min_special: Optional[int] = None,
        min_uppercase: Optional[int] = None,
    ) -> str:
        """
        Generate a secret.

        Args:
            length (int): Length of the password (default: 24)
            avoid_ambiguous (bool): Exclude ambiguous characters like 0/O, 1/l/I (default: True)
            lowercase (bool): Include the lowercase character set (default: True)
            uppercase (bool): Include the uppercase character set (default: True)
            numbers (bool): Include the numeric character set (default: True)
            special (bool): Include the special character set (default: True)
            min_lowercase (Optional[int]): Minimum lowercase characters to include (default: None)
            min_uppercase (Optional[int]): Minimum uppercase characters to include (default: None)
            min_number (Optional[int]): Minimum numeric characters to include (default: None)
            min_special (Optional[int]): Minimum special characters to include (default: None)

        Returns:
            str:
                Generated secret as a string

        Raises:
            ValueError:
                If the requested secret length is not between 4 and 255 (inclusive)

            ValueError:
                If at least one of lowercase, uppercase, numbers, or special characters are
                not greater than 0

            ValueError:
                If one of min_lowercase, min_uppercase, min_number, or min_special is a negative
                number

            ValueError:
                If one of min_lowercase, min_uppercase, min_number, or min_special is provided,
                but that character set is disabled

            ValueError:
                If the sum of minimum character set requirements exceeds requested secret length

            Exception:
                If secret generation fails for any other reason. This would generally indicate a problem
                with the FFI layer or system configuration.
        """

        def _is_valid_length(length):
            return isinstance(length, int) and 4 <= length <= 255

        # the SDK uses u8 for the generator values, so ensure we're under 255 characters and
        # above the minimum of 4 characters. if not, return a friendly error.
        if not _is_valid_length(length):
            raise ValueError("length must be between 4 and 255 (inclusive)")

        if not any([lowercase, uppercase, numbers, special]):
            raise ValueError(
                "At least one of lowercase, uppercase, numbers, or special must be enabled"
            )

        def _validate_min(name: str, value: Optional[int], enabled: bool) -> int:
            if value is None:
                return 0
            if value < 0:
                raise ValueError(f"{name} cannot be negative")
            if not enabled and value > 0:
                raise ValueError(f"{name} > 0 but its character set is disabled")
            return int(value)

        min_lc = _validate_min("min_lowercase", min_lowercase, lowercase)
        min_uc = _validate_min("min_uppercase", min_uppercase, uppercase)
        min_num = _validate_min("min_number", min_number, numbers)
        min_sp = _validate_min("min_special", min_special, special)

        if (min_lc + min_uc + min_num + min_sp) > length:
            raise ValueError("Sum of minimum requirements exceeds requested length")

        # create the password generator request
        password_request = PasswordGeneratorRequest(
            avoid_ambiguous=bool(avoid_ambiguous),
            length=int(length),
            lowercase=bool(lowercase),
            uppercase=bool(uppercase),
            numbers=bool(numbers),
            special=bool(special),
            min_lowercase=min_lc if min_lowercase is not None else None,
            min_uppercase=min_uc if min_uppercase is not None else None,
            min_number=min_num if min_number is not None else None,
            min_special=min_sp if min_special is not None else None,
        )

        result = self.client._run_command(
            command=Command(
                generators=GeneratorsCommand(generate_password=password_request)
            )
        )
        response = ResponseForString.from_dict(result)

        if not response.success:
            raise Exception(response.error_message or "Secret generation failed")

        return response.data
