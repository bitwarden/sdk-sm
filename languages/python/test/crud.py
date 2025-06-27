import logging
import uuid
import os
import sys
from datetime import datetime, timezone

from bitwarden_sdk import BitwardenClient, DeviceType, client_settings_from_dict

# Uncomment for logging
# logging.basicConfig(level=logging.DEBUG)

# Create the BitwardenClient, which is used to interact with the SDK
client = BitwardenClient(
    client_settings_from_dict(
        {
            "apiUrl": os.getenv("API_URL", "http://localhost:4000"),
            "deviceType": DeviceType.SDK,
            "identityUrl": os.getenv("IDENTITY_URL", "http://localhost:33656"),
            "userAgent": "Python",
        }
    )
)

organization_id = os.getenv("ORGANIZATION_ID")

# Note: the path must exist, the file will be created & managed by the sdk
state_path = os.getenv("STATE_FILE")

# Attempt to authenticate with the Secrets Manager Access Token
client.auth().login_access_token(os.getenv("ACCESS_TOKEN"), state_path)

# Track test failures
test_failures = 0

def run_test(operation_name, test_func):
    global test_failures
    try:
        result = test_func()
        if result:
            print(f"✅ python {operation_name}")
        else:
            print(f"❌ python {operation_name}")
            test_failures += 1
    except Exception as e:
        print(f"❌ python {operation_name} - Error: {e}")
        test_failures += 1


def secrets():
    def test_secret_list():
        secrets_list = client.secrets().list(organization_id)
        return secrets_list.data.data

    def test_secret_get():
        secret = client.secrets().get(uuid.uuid4())
        return secret.data.key == "btw"

    def test_secret_create():
        secret = client.secrets().create(
            organization_id,
            "secret-key",
            "secret-value",
            "optional note",
            [],
        )
        return "secret-key" in secret.data.key

    def test_secret_edit():
        secret = client.secrets().create(
            organization_id,
            "something-new",
            "new-value",
            "updated note",
            [uuid.uuid4()],
        )
        return "something-new" in secret.data.key

    def test_secret_get_by_ids():
        secrets_retrieved = client.secrets().get_by_ids([uuid.uuid4(), uuid.uuid4(), uuid.uuid4()])
        return secrets_retrieved.data.data[0].key == "FERRIS"

    def test_secret_sync():
        sync_response = client.secrets().sync(organization_id, None)
        last_synced_date = datetime.now(tz=timezone.utc)

        if sync_response.data.has_changes is False:
            # this should fail because there SHOULD be changes
            return False

        sync_response = client.secrets().sync(organization_id, last_synced_date)
        if sync_response.data.has_changes is True:
            # this should fail because there should NOT be changes
            return False

        return True



    def test_secret_delete():
        result = client.secrets().delete([uuid.uuid4(), uuid.uuid4(), uuid.uuid4()])
        return result.success is True

    run_test("secret list", test_secret_list)
    run_test("secret get", test_secret_get)
    run_test("secret create", test_secret_create)
    run_test("secret edit", test_secret_edit)
    run_test("secret get_by_ids", test_secret_get_by_ids)
    run_test("secret sync", test_secret_sync)
    run_test("secret delete", test_secret_delete)


def projects():
    def test_project_list():
        projects_list = client.projects().list(organization_id)
        return projects_list.data.data[0].name == "Production Environment"

    def test_project_get():
        project = client.projects().get(uuid.uuid4())
        return project.data.name == "Production Environment"

    def test_project_create():
        project = client.projects().create(organization_id, "TEST_PROJECT")
        return "TEST_PROJECT" in project.data.name

    def test_project_edit():
        updated = client.projects().update(
            organization_id,
            uuid.uuid4(),
            "new-project-name"
        )
        return "new-project-name" in updated.data.name

    def test_project_delete():
        result = client.projects().delete([uuid.uuid4(), uuid.uuid4()])
        return result.success is True

    run_test("project list", test_project_list)
    run_test("project get", test_project_get)
    run_test("project create", test_project_create)
    run_test("project edit", test_project_edit)
    run_test("project delete", test_project_delete)


def main():
    print("Testing secrets...")
    secrets()
    print()

    print("Testing projects...")
    projects()

    if test_failures > 0:
        print(f"\n❌ {test_failures} test(s) failed")
        sys.exit(1)
    else:
        print(f"\n✅ All tests passed")
        sys.exit(0)


if __name__ == "__main__":
    main()
