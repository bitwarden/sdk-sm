#!/usr/bin/env python3
"""
SDK Test Suite - Python
Simple, clean test suite focusing on test cases
"""

import argparse
import json
import os
import sys
import time
from datetime import datetime, timezone

# Add parent directory to path for imports
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from bitwarden_sdk import BitwardenClient, DeviceType, client_settings_from_dict


class PythonSdkTestSuite:
    """Python SDK test suite - simplified for maintainability"""

    def __init__(self, json_output: bool = False, verbose: bool = False):
        self.json_output = json_output
        self.verbose = verbose
        self.client = None
        self.organization_id = os.getenv("ORGANIZATION_ID")

        # Test tracking
        self.operations = []
        self.start_time = None

        # Check if we're testing against real server
        self.is_real_server = os.getenv("TEST_MODE") == "real-server"

    def setup_client(self) -> bool:
        """Initialize the Bitwarden client"""
        try:
            api_url = os.getenv("API_URL")
            identity_url = os.getenv("IDENTITY_URL")

            if not api_url or not identity_url:
                raise ValueError("API_URL and IDENTITY_URL environment variables must be set")

            if self.verbose:
                print(f"Setting up client with API: {api_url}, Identity: {identity_url}", file=sys.stderr)

            self.client = BitwardenClient(
                client_settings_from_dict({
                    "apiUrl": api_url,
                    "deviceType": DeviceType.SDK,
                    "identityUrl": identity_url,
                    "userAgent": "PythonSDKTests"
                })
            )
            return True
        except Exception as e:
            print(f"Failed to setup client: {e}", file=sys.stderr)
            return False

    def create_test_project(self, purpose: str) -> str:
        """Helper method to create a project for testing
        Returns the project ID"""
        project_name = f"test-project-{purpose}"
        project = self.client.projects().create(self.organization_id, project_name)
        return project.data.id

    def cleanup_project(self, project_id: str) -> None:
        """Helper method to delete a project and verify cleanup on real server"""
        self.client.projects().delete([project_id])

        if self.is_real_server:
            projects = self.client.projects().list(self.organization_id)
            project_ids = [p.id for p in projects.data.data] if projects.data.data else []
            if project_id in project_ids:
                raise Exception(f"Project {project_id} still exists after deletion")

    def verify_secret_deleted(self, secret_id: str) -> None:
        """Helper to verify a secret is deleted on real server"""
        if self.is_real_server:
            try:
                self.client.secrets().get(secret_id)
                raise Exception("Secret still exists after deletion")
            except Exception as e:
                if "not found" not in str(e).lower() and "Secret still exists" in str(e):
                    raise e

    def run_operation(self, operation_name: str, test_func, display_name: str = None):
        """Run a single test operation and track results"""
        if display_name is None:
            display_name = operation_name

        start = time.time()
        operation = {
            "operation": operation_name,
            "success": False,
            "duration_ms": 0,
            "error": None,
            "details": {}
        }

        try:
            if self.verbose:
                print(f"Running: {display_name}", file=sys.stderr)

            result = test_func()

            # Handle return types
            if isinstance(result, bool):
                operation["success"] = result
            elif isinstance(result, tuple) and len(result) == 2:
                operation["success"], operation["details"] = result
            else:
                operation["success"] = True
                operation["details"] = {"result": str(result)}

        except Exception as e:
            operation["error"] = str(e)
            if not self.json_output:
                print(f"❌ {display_name}: {e}", file=sys.stderr)

        operation["duration_ms"] = int((time.time() - start) * 1000)

        # Print progress for text mode
        if not self.json_output:
            if operation["error"]:
                print(f"❌ {display_name} ({operation['duration_ms']}ms): {operation['error']}")
            elif operation["success"]:
                print(f"✅ {display_name} ({operation['duration_ms']}ms)")
            else:
                print(f"❌ {display_name} ({operation['duration_ms']}ms)")

        self.operations.append(operation)
        return operation["success"]

    # ========== Test Operations ==========

    def test_auth(self):
        """Test authentication"""
        access_token = os.getenv("ACCESS_TOKEN")
        if not access_token:
            raise ValueError("ACCESS_TOKEN not set")

        state_file = os.getenv("STATE_FILE")
        self.client.auth().login_access_token(access_token, state_file)

        # Verify authentication worked by trying to sync
        try:
            sync_result = self.client.secrets().sync(self.organization_id, None)
            if not sync_result.success:
                raise Exception("Sync returned unsuccessful status")
        except Exception as e:
            raise Exception(f"Authentication verification failed: {e}")

        return True, {"has_state": bool(state_file)}

    def test_secret_create(self):
        """Create a secret"""
        project_id = self.create_test_project("secret-create")

        try:
            secret_name = "test-secret-create"
            secret = self.client.secrets().create(
                organization_id=self.organization_id,
                key=secret_name,
                value="test-value",
                note="Created by test suite",
                project_ids=[project_id]
            )

            # Clean up the secret
            self.client.secrets().delete([secret.data.id])
            self.verify_secret_deleted(secret.data.id)

            return True, {"id": secret.data.id, "key": secret.data.key}
        finally:
            self.cleanup_project(project_id)


    def test_secret_get(self):
        """Get a secret"""
        project_id = self.create_test_project("secret-get")

        try:
            # Create a secret to get
            secret_name = "test-secret-get"
            created_secret = self.client.secrets().create(
                organization_id=self.organization_id,
                key=secret_name,
                value="test-value",
                note="Created for get test",
                project_ids=[project_id]
            )
            secret_id = created_secret.data.id

            # Get the secret
            secret = self.client.secrets().get(secret_id)

            # On real server, verify we got the correct secret
            if self.is_real_server:
                if secret.data.id != secret_id:
                    raise Exception(f"Got wrong secret: expected {secret_id}, got {secret.data.id}")

            # Clean up the secret
            self.client.secrets().delete([secret_id])

            return True, {"id": secret.data.id, "key": secret.data.key, "verified": self.is_real_server}
        finally:
            self.cleanup_project(project_id)

    def test_secret_update(self):
        """Update a secret"""
        project_id = self.create_test_project("secret-update")

        try:
            # Create a secret to update
            secret_name = "test-secret-update"
            created_secret = self.client.secrets().create(
                organization_id=self.organization_id,
                key=secret_name,
                value="original-value",
                note="Created for update test",
                project_ids=[project_id]
            )
            secret_id = created_secret.data.id

            # Update the secret
            updated = self.client.secrets().update(
                self.organization_id,
                secret_id,
                "updated-key",
                "updated-value",
                "Updated by test",
                [project_id]
            )

            # Clean up the secret
            self.client.secrets().delete([secret_id])

            return True, {"id": secret_id, "key": updated.data.key}
        finally:
            self.cleanup_project(project_id)


    def test_secret_sync(self):
        """Test sync functionality"""
        # Initial sync with None date - should return all secrets
        sync1 = self.client.secrets().sync(self.organization_id, None)

        # Verify initial sync returns data (has_changes should be true for first sync)
        if not sync1.data.has_changes:
            raise Exception("Initial sync should return has_changes=True")

        # Sync with current date - should return no changes (nothing changed since now)
        sync2 = self.client.secrets().sync(
            self.organization_id,
            datetime.now(tz=timezone.utc)
        )

        # For fake-server, the behavior is currently inverted due to implementation
        # For real-server, this should properly return false for no changes
        if self.is_real_server:
            # Real server should return false when no changes since the given date
            expected_no_changes = not sync2.data.has_changes
        else:
            # Fake server incorrectly returns false for any past date
            expected_no_changes = not sync2.data.has_changes

        if not expected_no_changes:
            raise Exception("Sync with current date should return has_changes=False")

        return True, {
            "sync_succeeded": True,
            "initial_secrets": len(sync1.data.secrets) if sync1.data.secrets else 0
        }

    def test_secret_delete(self):
        """Delete secrets"""
        project_id = self.create_test_project("secret-delete")

        try:
            # Create a secret to delete
            secret_name = "test-secret-delete"
            created_secret = self.client.secrets().create(
                organization_id=self.organization_id,
                key=secret_name,
                value="test-value",
                note="Created for delete test",
                project_ids=[project_id]
            )
            secret_id = created_secret.data.id

            # Delete the secret
            result = self.client.secrets().delete([secret_id])

            if not result.success:
                raise Exception("Delete operation failed")

            # Verify the secret is actually deleted
            self.verify_secret_deleted(secret_id)

            # If we got here, deletion succeeded and was verified (for real-server)
            return True, {"deletion_succeeded": True}
        finally:
            self.cleanup_project(project_id)


    def test_project_list(self):
        """List projects"""
        project_id = self.create_test_project("list")

        try:
            # List projects
            projects = self.client.projects().list(self.organization_id)
            count = len(projects.data.data) if projects.data.data else 0

            # On real server, verify our created project is in the list
            if self.is_real_server:
                project_ids = [p.id for p in projects.data.data] if projects.data.data else []
                if project_id not in project_ids:
                    raise Exception(f"Created project {project_id} not found in list")

            return True, {"count": count, "verified": self.is_real_server}
        finally:
            self.cleanup_project(project_id)


    def test_project_update(self):
        """Update a project"""
        project_id = self.create_test_project("update")

        try:
            # Update the project
            new_name = "updated-project-name"
            updated = self.client.projects().update(self.organization_id, project_id, new_name)

            return new_name in updated.data.name, {"name": updated.data.name}
        finally:
            self.cleanup_project(project_id)

    def test_generator_default(self):
        """Test password generator with default parameters"""
        # Define character sets
        LOWERCASE_CHARACTERS = "abcdefghijklmnopqrstuvwxyz"
        UPPERCASE_CHARACTERS = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
        NUMERIC_CHARACTERS = "0123456789"
        SPECIAL_CHARACTERS = "!@#$%^&*()-_=+[]{};:'\",.<>/?\\|`~"

        generated_secret = self.client.generators().generate()

        # Should be exactly 24 chars
        if len(generated_secret) != 24:
            raise Exception(f"Expected length 24, got {len(generated_secret)}")

        # Should contain lowercase chars
        if not any(c in LOWERCASE_CHARACTERS for c in generated_secret):
            raise Exception("Generated secret missing lowercase characters")

        # Should contain uppercase chars
        if not any(c in UPPERCASE_CHARACTERS for c in generated_secret):
            raise Exception("Generated secret missing uppercase characters")

        # Should contain numeric chars
        if not any(c in NUMERIC_CHARACTERS for c in generated_secret):
            raise Exception("Generated secret missing numeric characters")

        # Should contain special chars
        if not any(c in SPECIAL_CHARACTERS for c in generated_secret):
            raise Exception("Generated secret missing special characters")

        return True, {"length": len(generated_secret), "has_all_types": True}

    def test_generator_custom(self):
        """Test password generator with custom parameters"""
        # Define character sets
        LOWERCASE_CHARACTERS = "abcdefghijklmnopqrstuvwxyz"
        UPPERCASE_CHARACTERS = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
        NUMERIC_CHARACTERS = "0123456789"
        SPECIAL_CHARACTERS = "!@#$%^&*()-_=+[]{};:'\",.<>/?\\|`~"
        AMBIGUOUS_CHARACTERS = "0O1lI"

        very_strong_secret = self.client.generators().generate(
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

        # Should be exactly 128 chars
        if len(very_strong_secret) != 128:
            raise Exception(f"Expected length 128, got {len(very_strong_secret)}")

        # Should contain ambiguous chars
        if not any(c in AMBIGUOUS_CHARACTERS for c in very_strong_secret):
            raise Exception("Generated secret missing ambiguous characters")

        # Should contain lowercase chars
        if not any(c in LOWERCASE_CHARACTERS for c in very_strong_secret):
            raise Exception("Generated secret missing lowercase characters")

        # Should contain uppercase chars
        if not any(c in UPPERCASE_CHARACTERS for c in very_strong_secret):
            raise Exception("Generated secret missing uppercase characters")

        # Should contain special chars
        if not any(c in SPECIAL_CHARACTERS for c in very_strong_secret):
            raise Exception("Generated secret missing special characters")

        # Should contain at least 2 lowercase chars
        lowercase_count = sum(1 for c in very_strong_secret if c in LOWERCASE_CHARACTERS)
        if lowercase_count < 2:
            raise Exception(f"Expected at least 2 lowercase, got {lowercase_count}")

        # Should contain at least 2 uppercase chars
        uppercase_count = sum(1 for c in very_strong_secret if c in UPPERCASE_CHARACTERS)
        if uppercase_count < 2:
            raise Exception(f"Expected at least 2 uppercase, got {uppercase_count}")

        # Should contain at least 4 numeric chars
        numeric_count = sum(1 for c in very_strong_secret if c in NUMERIC_CHARACTERS)
        if numeric_count < 4:
            raise Exception(f"Expected at least 4 numeric, got {numeric_count}")

        # Should contain at least 4 special chars
        special_count = sum(1 for c in very_strong_secret if c in SPECIAL_CHARACTERS)
        if special_count < 4:
            raise Exception(f"Expected at least 4 special, got {special_count}")

        return True, {
            "length": len(very_strong_secret),
            "lowercase_count": lowercase_count,
            "uppercase_count": uppercase_count,
            "numeric_count": numeric_count,
            "special_count": special_count
        }



    def discover_tests(self):
        """Discover all test methods using introspection"""
        tests = []
        # Define the order and display names for tests
        test_definitions = [
            ("test_auth", "Authentication"),
            ("test_secret_create", "Create Secret"),
            ("test_secret_get", "Get Secret"),
            ("test_secret_update", "Update Secret"),
            ("test_secret_sync", "Sync Secrets"),
            ("test_secret_delete", "Delete Secrets"),
            ("test_project_list", "List Projects"),
            ("test_project_update", "Update Project"),
            ("test_generator_default", "Generator Default"),
            ("test_generator_custom", "Generator Custom"),
        ]

        for name, display in test_definitions:
            if hasattr(self, name):
                tests.append((name, getattr(self, name), display))
        return tests

    def generate_report(self, total_duration):
        """Generate and output the test report"""
        if self.json_output:
            report = {
                "language": "python",
                "sdk_version": self._get_sdk_version(),
                "operations": self.operations,
                "total_duration_ms": total_duration,
                "os": sys.platform,
                "architecture": os.uname().machine if hasattr(os, 'uname') else "unknown",
                "timestamp": datetime.utcnow().isoformat() + "Z"
            }
            print(json.dumps(report, indent=2, default=str))
        else:
            # Print summary
            passed = sum(1 for op in self.operations if op["success"])
            failed = len(self.operations) - passed

            print()
            print("=" * 60)
            print(f"Results: {passed}/{len(self.operations)} passed ({total_duration}ms)")
            if failed > 0:
                print("Failed operations:")
                for op in self.operations:
                    if not op["success"]:
                        print(f"  - {op['operation']}: {op.get('error', 'Failed')}")
            print("=" * 60)

    def run_all_tests(self):
        """Run all test operations"""
        self.start_time = time.time()

        # Validate required environment variables
        if not self.organization_id:
            raise ValueError("ORGANIZATION_ID environment variable must be set")

        if not self.setup_client():
            raise ConnectionError("Failed to setup client")

        # Print header for text mode
        if not self.json_output:
            print("=" * 60)
            print("Python SDK Test Suite")
            print("=" * 60)
            print()

        # Discover and run tests
        tests = self.discover_tests()
        for name, func, display in tests:
            self.run_operation(name, func, display)

        total_duration = int((time.time() - self.start_time) * 1000)

        # Generate report
        self.generate_report(total_duration)

        # Return appropriate exit code
        all_passed = all(op["success"] for op in self.operations)
        return 0 if all_passed else 1

    def _get_sdk_version(self) -> str:
        """Get SDK version"""
        try:
            from bitwarden_sdk import __version__
            return __version__
        except Exception:
            return "unknown"


def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(description="Python SDK Test Suite")
    parser.add_argument("--json", action="store_true", help="Output JSON format")
    parser.add_argument("--verbose", action="store_true", help="Verbose output")

    args = parser.parse_args()

    try:
        suite = PythonSdkTestSuite(json_output=args.json, verbose=args.verbose)
        exit_code = suite.run_all_tests()
        sys.exit(exit_code)
    except Exception as e:
        if args.json:
            # Output error in JSON format
            error_report = {
                "language": "python",
                "sdk_version": "unknown",
                "operations": [],
                "total_duration_ms": 0,
                "os": sys.platform,
                "architecture": os.uname().machine if hasattr(os, 'uname') else "unknown",
                "timestamp": datetime.utcnow().isoformat() + "Z",
                "error": str(e)
            }
            print(json.dumps(error_report, indent=2, default=str))
        else:
            print(f"Fatal error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()