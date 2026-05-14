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
import uuid
from datetime import datetime, timezone
from typing import Dict, Any, Optional, List

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
        self.test_mode = os.getenv("TEST_MODE")

        # Test tracking
        self.operations = []
        self.start_time = None

        # Test data cleanup tracking
        self.created_secret_ids = []
        self.created_project_ids = []

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

    def run_operation(self, operation_name: str, test_func, display_name: str = None):
        """Run a single test operation and track results"""
        if display_name is None:
            display_name = operation_name.replace("_", " ").title()

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
        return True, {"method": "access_token", "has_state": bool(state_file)}

    def test_secret_create(self):
        """Create a secret"""
        secret_name = f"test-secret-{uuid.uuid4().hex[:8]}"
        secret = self.client.secrets().create(
            organization_id=self.organization_id,
            key=secret_name,
            value="test-value",
            note="Created by test suite",
            project_ids=[]
        )
        self.created_secret_ids.append(secret.data.id)
        return True, {"id": secret.data.id, "key": secret.data.key}

    def test_secret_list(self):
        """List secrets"""
        secrets = self.client.secrets().list(self.organization_id)
        count = len(secrets.data.data) if secrets.data.data else 0
        return True, {"count": count}

    def test_secret_get(self):
        """Get a secret"""
        if self.test_mode == "fake-server":
            # Fake server returns specific test data
            secret = self.client.secrets().get(str(uuid.uuid4()))
            return secret.data.key == "btw", {"key": secret.data.key}
        else:
            if not self.created_secret_ids:
                self.test_secret_create()
            secret = self.client.secrets().get(self.created_secret_ids[0])
            return True, {"id": secret.data.id, "key": secret.data.key}

    def test_secret_update(self):
        """Update a secret"""
        if not self.created_secret_ids:
            self.test_secret_create()

        secret_id = self.created_secret_ids[0]
        updated = self.client.secrets().update(
            self.organization_id,
            secret_id,
            "updated-key",
            "updated-value",
            "Updated by test",
            []
        )
        return True, {"id": secret_id, "key": updated.data.key}

    def test_secret_get_by_ids(self):
        """Get multiple secrets by IDs"""
        ids = [str(uuid.uuid4()) for _ in range(3)]
        secrets = self.client.secrets().get_by_ids(ids)

        if self.test_mode == "fake-server":
            # Fake server returns specific test data
            return secrets.data.data[0].key == "FERRIS", {"first_key": secrets.data.data[0].key}
        return len(secrets.data.data) > 0, {"count": len(secrets.data.data)}

    def test_secret_sync(self):
        """Test sync functionality"""
        # Initial sync
        sync1 = self.client.secrets().sync(self.organization_id, None)

        # Sync with current date
        sync2 = self.client.secrets().sync(
            self.organization_id,
            datetime.now(tz=timezone.utc)
        )

        return True, {
            "initial_has_changes": sync1.data.has_changes,
            "after_has_changes": sync2.data.has_changes
        }

    def test_secret_delete(self):
        """Delete secrets"""
        if self.created_secret_ids:
            ids = self.created_secret_ids[:2] if len(self.created_secret_ids) > 1 else self.created_secret_ids
        else:
            ids = [str(uuid.uuid4()) for _ in range(2)]

        result = self.client.secrets().delete(ids)

        # Clean tracking
        for sid in ids:
            if sid in self.created_secret_ids:
                self.created_secret_ids.remove(sid)

        return result.success is True, {"deleted": len(ids)}

    def test_project_create(self):
        """Create a project"""
        project_name = f"test-project-{uuid.uuid4().hex[:8]}"
        project = self.client.projects().create(self.organization_id, project_name)
        self.created_project_ids.append(project.data.id)
        return True, {"id": project.data.id, "name": project.data.name}

    def test_project_list(self):
        """List projects"""
        projects = self.client.projects().list(self.organization_id)
        count = len(projects.data.data) if projects.data.data else 0
        return True, {"count": count}

    def test_project_get(self):
        """Get a project"""
        if self.test_mode == "fake-server":
            project = self.client.projects().get(str(uuid.uuid4()))
            return project.data.name == "Production Environment", {"name": project.data.name}
        else:
            if not self.created_project_ids:
                self.test_project_create()
            project = self.client.projects().get(self.created_project_ids[0])
            return True, {"id": project.data.id, "name": project.data.name}

    def test_project_update(self):
        """Update a project"""
        if not self.created_project_ids:
            self.test_project_create()

        project_id = self.created_project_ids[0] if self.test_mode != "fake-server" else str(uuid.uuid4())
        new_name = f"updated-project-{uuid.uuid4().hex[:8]}"
        updated = self.client.projects().update(self.organization_id, project_id, new_name)

        return new_name in updated.data.name, {"name": updated.data.name}

    def test_project_delete(self):
        """Delete projects"""
        if self.created_project_ids:
            ids = self.created_project_ids[:2] if len(self.created_project_ids) > 1 else self.created_project_ids
        else:
            ids = [str(uuid.uuid4()) for _ in range(2)]

        result = self.client.projects().delete(ids)

        # Clean tracking
        for pid in ids:
            if pid in self.created_project_ids:
                self.created_project_ids.remove(pid)

        return result.success is True, {"deleted": len(ids)}

    def test_generator_default(self):
        """Test password generation with defaults"""
        generated = self.client.generators().generate()

        # Basic validation
        checks = {
            "length_ok": len(generated) == 24,
            "has_lowercase": any(c.islower() for c in generated),
            "has_uppercase": any(c.isupper() for c in generated),
            "has_numbers": any(c.isdigit() for c in generated),
            "has_special": any(not c.isalnum() for c in generated)
        }

        return all(checks.values()), checks

    def test_generator_custom(self):
        """Test password generation with custom params"""
        generated = self.client.generators().generate(
            length=32,
            lowercase=True,
            uppercase=True,
            numbers=True,
            special=True,
            min_lowercase=2,
            min_uppercase=2,
            min_number=2,
            min_special=2
        )

        return len(generated) == 32, {"length": len(generated)}

    def test_generator_validation(self):
        """Test generator input validation"""
        try:
            # Should fail - all character types disabled
            self.client.generators().generate(
                lowercase=False,
                uppercase=False,
                numbers=False,
                special=False
            )
            return False, {"error": "Should have raised ValueError"}
        except ValueError:
            return True, {"validation": "correctly rejected invalid params"}

    def discover_tests(self):
        """Discover all test methods using introspection"""
        tests = []
        # Define the order and display names for tests
        test_definitions = [
            ("test_auth", "Authentication"),
            ("test_secret_create", "Create Secret"),
            ("test_secret_list", "List Secrets"),
            ("test_secret_get", "Get Secret"),
            ("test_secret_update", "Update Secret"),
            ("test_secret_get_by_ids", "Get Secrets by IDs"),
            ("test_secret_sync", "Sync Secrets"),
            ("test_secret_delete", "Delete Secrets"),
            ("test_project_create", "Create Project"),
            ("test_project_list", "List Projects"),
            ("test_project_get", "Get Project"),
            ("test_project_update", "Update Project"),
            ("test_project_delete", "Delete Projects"),
            ("test_generator_default", "Generate Password (Default)"),
            ("test_generator_custom", "Generate Password (Custom)"),
            ("test_generator_validation", "Generator Validation"),
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
            print(f"Mode: {self.test_mode or 'default'}")
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
        return 0 if (all_passed or self.json_output) else 1

    def _get_sdk_version(self) -> str:
        """Get SDK version"""
        try:
            import bitwarden_sdk
            return getattr(bitwarden_sdk, "__version__", "unknown")
        except:
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