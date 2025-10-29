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
    def __init__(self, client: BitwardenClient):
        self.client = client

    def get(self, id: str) -> ResponseForSecretResponse:
        result = self.client._run_command(
            Command(secrets=SecretsCommand(get=SecretGetRequest(id)))
        )
        return ResponseForSecretResponse.from_dict(result)

    def get_by_ids(self, ids: List[UUID]) -> ResponseForSecretsResponse:
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
        result = self.client._run_command(
            Command(secrets=SecretsCommand(delete=SecretsDeleteRequest(ids)))
        )
        return ResponseForSecretsDeleteResponse.from_dict(result)

    def sync(
        self, organization_id: str, last_synced_date: Optional[str]
    ) -> ResponseForSecretsSyncResponse:
        result = self.client._run_command(
            Command(
                secrets=SecretsCommand(
                    sync=SecretsSyncRequest(organization_id, last_synced_date)
                )
            )
        )
        return ResponseForSecretsSyncResponse.from_dict(result)


class ProjectsClient:
    def __init__(self, client: BitwardenClient):
        self.client = client

    def get(self, id: str) -> ResponseForProjectResponse:
        result = self.client._run_command(
            Command(projects=ProjectsCommand(get=ProjectGetRequest(id)))
        )
        return ResponseForProjectResponse.from_dict(result)

    def create(
        self,
        organization_id: str,
        name: str,
    ) -> ResponseForProjectResponse:
        result = self.client._run_command(
            Command(
                projects=ProjectsCommand(
                    create=ProjectCreateRequest(name, organization_id)
                )
            )
        )
        return ResponseForProjectResponse.from_dict(result)

    def list(self, organization_id: str) -> ResponseForProjectsResponse:
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
        result = self.client._run_command(
            Command(
                projects=ProjectsCommand(
                    update=ProjectPutRequest(id, name, organization_id)
                )
            )
        )
        return ResponseForProjectResponse.from_dict(result)

    def delete(self, ids: List[str]) -> ResponseForProjectsDeleteResponse:
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
