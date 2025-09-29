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


def generators():
    LOWERCASE_CHARACTERS = "abcdefghijklmnopqrstuvwxyz"
    UPPERCASE_CHARACTERS = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
    NUMERIC_CHARACTERS = "0123456789"
    SPECIAL_CHARACTERS = "!@#$%^&*"  # https://github.com/bitwarden/sdk-internal/blob/d7ce769/crates/bitwarden-generators/src/password.rs#L80
    AMBIGUOUS_CHARACTERS = "IOl01"  # https://github.com/bitwarden/sdk-internal/blob/d7ce769/crates/bitwarden-generators/src/password.rs#L77-L79

    def test_generator_with_default_params():
        generated_secret = client.generators().generate()

        # should be exactly 24 chars
        len = generated_secret.__len__()
        if len != 24:
            return False

        # should contain lowercase chars
        if not any(c in LOWERCASE_CHARACTERS for c in generated_secret):
            print(f"lowercase characters were NOT found in '{generated_secret}'")
            return False

        # should contain uppercase chars
        if not any(c in UPPERCASE_CHARACTERS for c in generated_secret):
            print(f"uppercase characters were NOT found in '{generated_secret}'")
            return False

        # should contain numeric chars
        if not any(c in NUMERIC_CHARACTERS for c in generated_secret):
            print(f"numeric characters were NOT found in '{generated_secret}'")
            return False

        # should contain special chars:
        if not any(c in SPECIAL_CHARACTERS for c in generated_secret):
            print(f"special characters were NOT found in '{generated_secret}'")
            return False

        return True

    def test_generator_with_all_params():
        very_strong_secret = client.generators().generate(
            length=128,
            avoid_ambiguous=False,
            lowercase=True,
            uppercase=True,
            numbers=True,
            special=True,
            min_lowercase=2,
            min_uppercase=2,
            min_number=4,
            min_special=4,
        )

        # should be exactly 128 chars
        len = very_strong_secret.__len__()
        if len != 128:
            return False

        # should contain ambiguous chars:
        if not any(c in AMBIGUOUS_CHARACTERS for c in very_strong_secret):
            print(f"ambiguous characters were NOT found in '{very_strong_secret}'")
            return False

        # should contain lowercase chars:
        if not any(c in LOWERCASE_CHARACTERS for c in very_strong_secret):
            print(f"lowercase characters were NOT found in '{very_strong_secret}'")
            return False

        # should contain uppercase chars:
        if not any(c in UPPERCASE_CHARACTERS for c in very_strong_secret):
            print(f"uppercase characters were NOT found in '{very_strong_secret}'")
            return False

        # should contain special chars:
        if not any(c in SPECIAL_CHARACTERS for c in very_strong_secret):
            print(f"special characters were NOT found in '{very_strong_secret}'")
            return False

        # should contain at least 2 lowercase chars:
        lowercase_count = sum(1 for c in very_strong_secret if c in LOWERCASE_CHARACTERS)
        if lowercase_count < 2:
            print(f"found only {lowercase_count} lowercase characters in '{very_strong_secret}', expected at least 2")
            return False

        # should contain at least 2 uppercase chars:
        uppercase_count = sum(1 for c in very_strong_secret if c in UPPERCASE_CHARACTERS)
        if uppercase_count < 2:
            print(f"found only {uppercase_count} uppercase characters in '{very_strong_secret}', expected at least 2")
            return False

        # should contain at least 4 numeric chars:
        numeric_count = sum(1 for c in very_strong_secret if c in NUMERIC_CHARACTERS)
        if numeric_count < 4:
            print(f"found only {numeric_count} numeric characters in '{very_strong_secret}', expected at least 4")
            return False

        # should contain at least 4 special chars:
        special_count = sum(1 for c in very_strong_secret if c in SPECIAL_CHARACTERS)
        if special_count < 4:
            print(f"found only {special_count} special characters in '{very_strong_secret}', expected at least 4")
            return False

        return True

    def test_generator_all_char_sets_disabled():
        """Test that disabling all character sets raises ValueError"""
        try:
            client.generators().generate(
                lowercase=False,
                uppercase=False,
                numbers=False,
                special=False,
            )
            # if we get here, the test failed - no exception was raised
            return False
        except ValueError:
            # expected behavior
            return True
        except Exception:
            # unexpected exception type
            return False

    def test_generator_negative_min_values():
        """Test that negative minimum values raise ValueError"""
        test_cases = [
            {"min_lowercase": -1},
            {"min_uppercase": -1},
            {"min_number": -1},
            {"min_special": -1},
        ]

        for params in test_cases:
            try:
                client.generators().generate(**params)
                # if we get here, the test failed - no exception was raised
                return False
            except ValueError:
                # expected behavior
                continue
            except Exception:
                # unexpected exception type
                return False

        return True

    def test_generator_contradicting_minimum_char_sets():
        """Test that setting min values for disabled character sets raises ValueError"""
        test_cases = [
            {"lowercase": False, "min_lowercase": 1},
            {"uppercase": False, "min_uppercase": 1},
            {"numbers": False, "min_number": 1},
            {"special": False, "min_special": 1},
        ]

        for params in test_cases:
            try:
                client.generators().generate(**params)
                # if we get here, the test failed - no exception was raised
                return False
            except ValueError:
                # expected behavior
                continue
            except Exception:
                # unexpected exception type
                return False

        return True
    
    def test_generator_with_min_char_sets_greater_than_length():
        """Test that setting sum of min values greater than length raises ValueError"""
        try:
            client.generators().generate(
                length=5,
                min_lowercase=2,
                min_uppercase=2,
                min_number=2,
            )
            # if we get here, the test failed - no exception was raised
            return False
        except ValueError:
            # expected behavior
            return True
        except Exception:
            # unexpected exception type
            return False

    run_test("generate with default params", test_generator_with_default_params)
    run_test("generate with all params", test_generator_with_all_params)
    run_test("generate with all char sets disabled", test_generator_all_char_sets_disabled)
    run_test("generate with negative min values", test_generator_negative_min_values)
    run_test("generate with contradicting minimum char sets", test_generator_contradicting_minimum_char_sets)
    run_test("generate with min char sets greater than length", test_generator_with_min_char_sets_greater_than_length)

def main():
    print("Testing secrets...")
    secrets()
    print()

    print("Testing projects...")
    projects()
    print()

    print("Testing secrets generator...")
    generators()

    if test_failures > 0:
        print(f"\n❌ {test_failures} test(s) failed")
        sys.exit(1)
    else:
        print(f"\n✅ All tests passed")
        sys.exit(0)


if __name__ == "__main__":
    main()
