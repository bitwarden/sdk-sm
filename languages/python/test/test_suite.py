#!/usr/bin/env python3
"""
SDK Test Framework - Python Tests
Comprehensive test suite with JSON and human-readable output support
Merges functionality from crud.py and tests.py
"""

import argparse
import json
import logging
import os
import platform
import sys
import time
import traceback
import uuid
from datetime import datetime, timezone
from typing import Dict, Any, Optional, List, Callable


class UUIDEncoder(json.JSONEncoder):
    """JSON encoder that handles UUID objects"""
    def default(self, obj):
        if isinstance(obj, uuid.UUID):
            return str(obj)
        return super().default(obj)


# Add parent directory to path for imports
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from bitwarden_sdk import BitwardenClient, DeviceType, client_settings_from_dict


class TestResult:
    """Enhanced test result with detailed information"""

    def __init__(self, name: str, display_name: str, category: str):
        self.name = name
        self.display_name = display_name
        self.category = category
        self.start_time = None
        self.duration_ms = 0
        self.status = "pending"  # pending, passed, failed, skipped, error
        self.error_info = None
        self.stack_trace = []
        self.failed_assertions = []
        self.request_details = None
        self.response_details = None
        self.details = {}
        self.assertions = 0

    def start(self):
        """Mark the start of the test"""
        self.start_time = time.time()

    def complete(self, success: bool, error: Optional[Exception] = None,
                details: Optional[Dict] = None, assertions: int = 0):
        """Mark the completion of the test"""
        if self.start_time:
            self.duration_ms = int((time.time() - self.start_time) * 1000)

        self.status = "passed" if success else "failed"
        self.details = details or {}
        self.assertions = assertions

        if error:
            self._capture_error_info(error)

    def skip(self, reason: str):
        """Mark test as skipped"""
        self.status = "skipped"
        self.details = {"reason": reason}

    def error(self, exception: Exception):
        """Mark test as errored (different from failed)"""
        self.status = "error"
        self._capture_error_info(exception)

    def _capture_error_info(self, exception: Exception):
        """Capture detailed error information"""
        exc_type, exc_value, exc_traceback = sys.exc_info()

        # Get stack trace
        if exc_traceback:
            self.stack_trace = traceback.format_exception(exc_type, exc_value, exc_traceback)

            # Get file and line info
            frame = traceback.extract_tb(exc_traceback)[-1]
            file_name = frame.filename
            line_number = frame.lineno
            function_name = frame.name
        else:
            file_name = "unknown"
            line_number = 0
            function_name = self.name

        self.error_info = {
            "type": type(exception).__name__,
            "message": str(exception),
            "code": self._get_error_code(type(exception).__name__),
            "file": file_name,
            "line": line_number,
            "function": function_name
        }

    def _get_error_code(self, error_type: str) -> str:
        """Map error types to error codes"""
        error_codes = {
            "AssertionError": "ASSERTION_FAILED",
            "ConnectionError": "CONNECTION_FAILED",
            "TimeoutError": "TIMEOUT",
            "ValueError": "INVALID_VALUE",
            "KeyError": "KEY_NOT_FOUND",
            "AttributeError": "ATTRIBUTE_ERROR",
            "TypeError": "TYPE_ERROR"
        }
        return error_codes.get(error_type, "UNKNOWN_ERROR")

    def add_assertion(self, expected: Any, actual: Any, operator: str, message: str, passed: bool):
        """Add assertion details"""
        self.assertions += 1
        if not passed:
            self.failed_assertions.append({
                "expected": str(expected),
                "actual": str(actual),
                "operator": operator,
                "message": message
            })

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON output"""
        base = {
            "name": self.name,
            "display_name": self.display_name,
            "duration_ms": self.duration_ms,
            "category": self.category,
            "status": self.status
        }

        if self.status == "passed":
            base["assertions"] = self.assertions
            if self.details:
                base["details"] = self.details
        elif self.status == "failed" or self.status == "error":
            if self.error_info:
                base["error"] = self.error_info
            if self.stack_trace:
                base["stack_trace"] = self.stack_trace
            if self.failed_assertions:
                base["failed_assertions"] = self.failed_assertions
            if self.request_details:
                base["request_details"] = self._sanitize_request(self.request_details)
            if self.response_details:
                base["response_details"] = self.response_details
        elif self.status == "skipped":
            base["reason"] = self.details.get("reason", "Unknown reason")

        return base

    def _sanitize_request(self, request_details: Dict) -> Dict:
        """Redact sensitive information from request details"""
        if not request_details:
            return {}

        sanitized = request_details.copy()
        if "headers" in sanitized and "Authorization" in sanitized["headers"]:
            sanitized["headers"]["Authorization"] = "[REDACTED]"

        return sanitized


class PythonSdkTestSuite:
    """Comprehensive SDK test suite"""

    def __init__(self, output_format: str = "text", verbose: bool = False):
        self.output_format = output_format
        self.verbose = verbose
        self.client = None
        self.organization_id = os.getenv("ORGANIZATION_ID")
        self.state_file = os.getenv("STATE_FILE")
        self.test_mode = os.getenv("TEST_MODE", "fake-server")
        self.sdk_source = os.getenv("SDK_SOURCE", "local-build")

        # Test results tracking
        self.results = []
        self.logs = {"stdout": [], "stderr": [], "debug": []}
        self.start_time = None
        self.end_time = None

        # Test data
        self.created_secret_ids = []
        self.created_project_ids = []

        # Character sets for generator tests
        self.LOWERCASE_CHARACTERS = "abcdefghijklmnopqrstuvwxyz"
        self.UPPERCASE_CHARACTERS = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
        self.NUMERIC_CHARACTERS = "0123456789"
        self.SPECIAL_CHARACTERS = "!@#$%^&*"
        self.AMBIGUOUS_CHARACTERS = "IOl01"

    def log_stdout(self, message: str):
        """Log to stdout with timestamp"""
        timestamp = datetime.utcnow().isoformat() + "Z"
        log_entry = f"[{timestamp}] {message}"
        self.logs["stdout"].append(log_entry)
        if self.output_format == "text":
            print(message)

    def log_stderr(self, message: str):
        """Log to stderr with timestamp"""
        timestamp = datetime.utcnow().isoformat() + "Z"
        log_entry = f"[{timestamp}] {message}"
        self.logs["stderr"].append(log_entry)
        if self.output_format == "text":
            print(message, file=sys.stderr)

    def log_debug(self, message: str):
        """Log debug information"""
        if self.verbose:
            timestamp = datetime.utcnow().isoformat() + "Z"
            log_entry = f"[{timestamp}] DEBUG: {message}"
            self.logs["debug"].append(log_entry)
            if self.output_format == "text":
                print(f"DEBUG: {message}", file=sys.stderr)

    def setup_client(self) -> bool:
        """Initialize the Bitwarden client"""
        try:
            api_url = os.getenv("API_URL", "http://localhost:4000")
            identity_url = os.getenv("IDENTITY_URL", "http://localhost:33656")

            self.log_debug(f"Setting up client with API: {api_url}, Identity: {identity_url}")

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
            self.log_stderr(f"Failed to setup client: {e}")
            return False

    def run_test(self, test_func: Callable, name: str, display_name: str,
                 category: str) -> TestResult:
        """Run a single test and capture results"""
        result = TestResult(name, display_name, category)
        result.start()

        try:
            self.log_debug(f"Running test: {display_name}")
            test_output = test_func()

            # Handle different return types
            if isinstance(test_output, bool):
                success = test_output
                details = {}
            elif isinstance(test_output, tuple):
                success, details = test_output
            else:
                success = False
                details = {"error": "Invalid test return type"}

            result.complete(success, details=details)

            if self.output_format == "text":
                if success:
                    self.log_stdout(f"✅ {display_name} ({result.duration_ms}ms)")
                else:
                    self.log_stdout(f"❌ {display_name} ({result.duration_ms}ms)")

        except AssertionError as e:
            result.complete(False, error=e)
            if self.output_format == "text":
                self.log_stderr(f"❌ {display_name} ({result.duration_ms}ms): {e}")

        except Exception as e:
            result.error(e)
            if self.output_format == "text":
                self.log_stderr(f"💥 {display_name} ({result.duration_ms}ms): {e}")

        self.results.append(result)
        return result

    # ========== Authentication Tests ==========

    def test_auth(self) -> tuple:
        """Test authentication with access token"""
        access_token = os.getenv("ACCESS_TOKEN")
        if not access_token:
            raise ValueError("ACCESS_TOKEN not set")

        self.client.auth().login_access_token(access_token, self.state_file)
        return True, {"method": "access_token", "state_file": bool(self.state_file)}

    # ========== Secret Tests ==========

    def test_secret_create(self) -> tuple:
        """Test creating a new secret"""
        secret_name = f"test-secret-{uuid.uuid4().hex[:8]}"
        secret = self.client.secrets().create(
            organization_id=self.organization_id,
            key=secret_name,
            value="test-value",
            note="Created by Python SDK test",
            project_ids=[]
        )

        self.created_secret_ids.append(secret.data.id)
        return True, {
            "secret_id": secret.data.id,
            "secret_key": secret.data.key
        }

    def test_secret_list(self) -> tuple:
        """Test listing secrets"""
        secrets_list = self.client.secrets().list(self.organization_id)
        secret_count = len(secrets_list.data.data) if secrets_list.data.data else 0
        return True, {"count": secret_count}

    def test_secret_get(self) -> tuple:
        """Test getting a specific secret"""
        if self.test_mode == "fake-server":
            # Fake server returns "btw" for any random UUID
            secret = self.client.secrets().get(str(uuid.uuid4()))
            success = secret.data.key == "btw"
            return success, {"secret_key": secret.data.key}
        else:
            # Real server - get the created secret
            if not self.created_secret_ids:
                raise Exception("No secret created to retrieve")
            secret = self.client.secrets().get(self.created_secret_ids[0])
            success = secret.data.id == self.created_secret_ids[0]
            return success, {"secret_id": secret.data.id, "secret_key": secret.data.key}

    def test_secret_update(self) -> tuple:
        """Test updating a secret"""
        if self.created_secret_ids:
            secret_id = self.created_secret_ids[0]
        else:
            # Create a secret first
            secret = self.client.secrets().create(
                self.organization_id,
                "update-test",
                "original-value",
                "To be updated",
                []
            )
            secret_id = secret.data.id
            self.created_secret_ids.append(secret_id)

        # Update the secret
        updated = self.client.secrets().update(
            self.organization_id,
            secret_id,
            "updated-key",
            "updated-value",
            "Updated note",
            []
        )

        success = "updated-key" in updated.data.key
        return success, {"secret_id": secret_id, "new_key": updated.data.key}

    def test_secret_get_by_ids(self) -> tuple:
        """Test getting multiple secrets by IDs"""
        ids = [str(uuid.uuid4()) for _ in range(3)]
        secrets_retrieved = self.client.secrets().get_by_ids(ids)

        if self.test_mode == "fake-server":
            # Fake server returns "FERRIS" for the first secret
            success = secrets_retrieved.data.data[0].key == "FERRIS"
        else:
            success = len(secrets_retrieved.data.data) > 0

        return success, {"requested": len(ids), "retrieved": len(secrets_retrieved.data.data)}

    def test_secret_sync(self) -> tuple:
        """Test sync functionality"""
        # First sync without date (should have changes)
        sync_response = self.client.secrets().sync(self.organization_id, None)
        has_changes_initial = sync_response.data.has_changes

        # Second sync with current date (should have no changes)
        last_sync_date = datetime.now(tz=timezone.utc)
        sync_response_with_date = self.client.secrets().sync(
            self.organization_id,
            last_sync_date
        )
        has_changes_after = sync_response_with_date.data.has_changes

        # For fake-server, expect initial=True, after=False
        # For real server, behavior may vary
        if self.test_mode == "fake-server":
            success = has_changes_initial and not has_changes_after
        else:
            # Real server: just verify the calls succeed
            success = True

        return success, {
            "initial_has_changes": has_changes_initial,
            "after_has_changes": has_changes_after
        }

    def test_secret_delete(self) -> tuple:
        """Test deleting secrets"""
        # Use created secrets or generate random IDs
        if self.created_secret_ids:
            ids_to_delete = self.created_secret_ids[:2] if len(self.created_secret_ids) > 1 else self.created_secret_ids
        else:
            ids_to_delete = [str(uuid.uuid4()) for _ in range(2)]

        result = self.client.secrets().delete(ids_to_delete)

        # Clean up our tracking
        for secret_id in ids_to_delete:
            if secret_id in self.created_secret_ids:
                self.created_secret_ids.remove(secret_id)

        return result.success is True, {"deleted_count": len(ids_to_delete)}

    # ========== Project Tests ==========

    def test_project_create(self) -> tuple:
        """Test creating a project"""
        project_name = f"test-project-{uuid.uuid4().hex[:8]}"
        project = self.client.projects().create(self.organization_id, project_name)

        self.created_project_ids.append(project.data.id)
        return True, {
            "project_id": project.data.id,
            "project_name": project.data.name
        }

    def test_project_list(self) -> tuple:
        """Test listing projects"""
        projects_list = self.client.projects().list(self.organization_id)
        project_count = len(projects_list.data.data) if projects_list.data.data else 0

        if self.test_mode == "fake-server" and project_count > 0:
            # Fake server returns "Production Environment" as first project
            expected_name = projects_list.data.data[0].name == "Production Environment"
            return expected_name, {"count": project_count, "first_name": projects_list.data.data[0].name}

        return True, {"count": project_count}

    def test_project_get(self) -> tuple:
        """Test getting a specific project"""
        if self.test_mode == "fake-server":
            # Fake server returns "Production Environment" for any UUID
            project = self.client.projects().get(str(uuid.uuid4()))
            success = project.data.name == "Production Environment"
            return success, {"project_name": project.data.name}
        else:
            if not self.created_project_ids:
                raise Exception("No project created to retrieve")
            project = self.client.projects().get(self.created_project_ids[0])
            success = project.data.id == self.created_project_ids[0]
            return success, {"project_id": project.data.id, "project_name": project.data.name}

    def test_project_update(self) -> tuple:
        """Test updating a project"""
        if self.created_project_ids:
            project_id = self.created_project_ids[0]
        else:
            # Create a project first
            project = self.client.projects().create(self.organization_id, "update-test")
            project_id = project.data.id
            self.created_project_ids.append(project_id)

        new_name = f"updated-project-{uuid.uuid4().hex[:8]}"
        updated = self.client.projects().update(
            self.organization_id,
            project_id if not self.test_mode == "fake-server" else str(uuid.uuid4()),
            new_name
        )

        success = new_name in updated.data.name
        return success, {"project_id": project_id, "new_name": updated.data.name}

    def test_project_delete(self) -> tuple:
        """Test deleting projects"""
        # Use created projects or generate random IDs
        if self.created_project_ids:
            ids_to_delete = self.created_project_ids[:2] if len(self.created_project_ids) > 1 else self.created_project_ids
        else:
            ids_to_delete = [str(uuid.uuid4()) for _ in range(2)]

        result = self.client.projects().delete(ids_to_delete)

        # Clean up our tracking
        for project_id in ids_to_delete:
            if project_id in self.created_project_ids:
                self.created_project_ids.remove(project_id)

        return result.success is True, {"deleted_count": len(ids_to_delete)}

    # ========== Generator Tests ==========

    def test_generator_default(self) -> tuple:
        """Test generator with default parameters"""
        generated = self.client.generators().generate()

        # Validate default: 24 chars with all character types
        checks = {
            "length_24": len(generated) == 24,
            "has_lowercase": any(c in self.LOWERCASE_CHARACTERS for c in generated),
            "has_uppercase": any(c in self.UPPERCASE_CHARACTERS for c in generated),
            "has_numbers": any(c in self.NUMERIC_CHARACTERS for c in generated),
            "has_special": any(c in self.SPECIAL_CHARACTERS for c in generated)
        }

        success = all(checks.values())
        return success, {"checks": checks, "sample": generated[:10] + "..."}

    def test_generator_custom_params(self) -> tuple:
        """Test generator with custom parameters"""
        generated = self.client.generators().generate(
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

        checks = {
            "length_128": len(generated) == 128,
            "has_ambiguous": any(c in self.AMBIGUOUS_CHARACTERS for c in generated),
            "min_lowercase_2": sum(1 for c in generated if c in self.LOWERCASE_CHARACTERS) >= 2,
            "min_uppercase_2": sum(1 for c in generated if c in self.UPPERCASE_CHARACTERS) >= 2,
            "min_numbers_4": sum(1 for c in generated if c in self.NUMERIC_CHARACTERS) >= 4,
            "min_special_4": sum(1 for c in generated if c in self.SPECIAL_CHARACTERS) >= 4
        }

        success = all(checks.values())
        return success, {"checks": checks}

    def test_generator_validation_errors(self) -> tuple:
        """Test generator input validation"""
        error_cases = []

        # Test 1: All character sets disabled
        try:
            self.client.generators().generate(
                lowercase=False, uppercase=False, numbers=False, special=False
            )
            error_cases.append(("all_disabled", False, "Should have raised ValueError"))
        except ValueError:
            error_cases.append(("all_disabled", True, "Correctly raised ValueError"))
        except Exception as e:
            error_cases.append(("all_disabled", False, f"Wrong exception: {type(e).__name__}"))

        # Test 2: Negative min values
        try:
            self.client.generators().generate(min_lowercase=-1)
            error_cases.append(("negative_min", False, "Should have raised ValueError"))
        except ValueError:
            error_cases.append(("negative_min", True, "Correctly raised ValueError"))
        except Exception as e:
            error_cases.append(("negative_min", False, f"Wrong exception: {type(e).__name__}"))

        # Test 3: Min values for disabled character sets
        try:
            self.client.generators().generate(lowercase=False, min_lowercase=1)
            error_cases.append(("contradicting_min", False, "Should have raised ValueError"))
        except ValueError:
            error_cases.append(("contradicting_min", True, "Correctly raised ValueError"))
        except Exception as e:
            error_cases.append(("contradicting_min", False, f"Wrong exception: {type(e).__name__}"))

        # Test 4: Min values exceed length
        try:
            self.client.generators().generate(length=5, min_lowercase=2, min_uppercase=2, min_number=2)
            error_cases.append(("min_exceeds_length", False, "Should have raised ValueError"))
        except ValueError:
            error_cases.append(("min_exceeds_length", True, "Correctly raised ValueError"))
        except Exception as e:
            error_cases.append(("min_exceeds_length", False, f"Wrong exception: {type(e).__name__}"))

        # Test 5: Length > 255
        try:
            self.client.generators().generate(length=256)
            error_cases.append(("length_too_large", False, "Should have raised ValueError"))
        except ValueError:
            error_cases.append(("length_too_large", True, "Correctly raised ValueError"))
        except Exception as e:
            error_cases.append(("length_too_large", False, f"Wrong exception: {type(e).__name__}"))

        all_passed = all(case[1] for case in error_cases)
        return all_passed, {"error_cases": [{"test": c[0], "passed": c[1], "message": c[2]} for c in error_cases]}

    def run_all_tests(self):
        """Run all test operations"""
        self.start_time = time.time()

        # Setup client
        if not self.setup_client():
            raise ConnectionError("Failed to setup client")

        # Run all tests
        all_tests = [
            # Authentication
            (self.test_auth, "test_auth", "Authentication with Access Token", "auth"),

            # Secret operations
            (self.test_secret_create, "test_secret_create", "Create Secret", "secrets"),
            (self.test_secret_list, "test_secret_list", "List Secrets", "secrets"),
            (self.test_secret_get, "test_secret_get", "Get Secret", "secrets"),
            (self.test_secret_update, "test_secret_update", "Update Secret", "secrets"),
            (self.test_secret_get_by_ids, "test_secret_get_by_ids", "Get Secrets by IDs", "secrets"),
            (self.test_secret_sync, "test_secret_sync", "Sync Secrets", "secrets"),
            (self.test_secret_delete, "test_secret_delete", "Delete Secrets", "secrets"),

            # Project operations
            (self.test_project_create, "test_project_create", "Create Project", "projects"),
            (self.test_project_list, "test_project_list", "List Projects", "projects"),
            (self.test_project_get, "test_project_get", "Get Project", "projects"),
            (self.test_project_update, "test_project_update", "Update Project", "projects"),
            (self.test_project_delete, "test_project_delete", "Delete Projects", "projects"),

            # Generator operations
            (self.test_generator_default, "test_generator_default", "Generate with Default Params", "generators"),
            (self.test_generator_custom_params, "test_generator_custom_params", "Generate with Custom Params", "generators"),
            (self.test_generator_validation_errors, "test_generator_validation_errors", "Generator Input Validation", "generators"),
        ]

        # Print header for text output
        if self.output_format == "text":
            self.log_stdout("=" * 60)
            self.log_stdout("Python SDK Test Suite")
            self.log_stdout(f"Test Mode: {self.test_mode}")
            self.log_stdout(f"SDK Source: {self.sdk_source}")
            self.log_stdout("=" * 60)
            self.log_stdout("")

        # Run tests by category
        current_category = None
        for test_func, name, display_name, category in all_tests:
            if self.output_format == "text" and category != current_category:
                if current_category is not None:
                    self.log_stdout("")
                self.log_stdout(f"Testing {category}...")
                current_category = category

            self.run_test(test_func, name, display_name, category)

        self.end_time = time.time()

        # Print summary for text output
        if self.output_format == "text":
            self._print_summary()

    def _print_summary(self):
        """Print test summary for text output"""
        passed = sum(1 for r in self.results if r.status == "passed")
        failed = sum(1 for r in self.results if r.status == "failed")
        errored = sum(1 for r in self.results if r.status == "error")
        skipped = sum(1 for r in self.results if r.status == "skipped")
        total = len(self.results)

        self.log_stdout("")
        self.log_stdout("=" * 60)
        self.log_stdout("Test Summary")
        self.log_stdout(f"Total: {total} | Passed: {passed} | Failed: {failed} | Errors: {errored} | Skipped: {skipped}")
        self.log_stdout(f"Duration: {int((self.end_time - self.start_time) * 1000)}ms")

        if failed > 0 or errored > 0:
            self.log_stdout("")
            self.log_stdout("Failed/Errored Tests:")
            for r in self.results:
                if r.status in ["failed", "error"]:
                    self.log_stdout(f"  ❌ {r.display_name}: {r.error_info['message'] if r.error_info else 'Unknown error'}")

        self.log_stdout("=" * 60)

        if failed > 0 or errored > 0:
            self.log_stdout("❌ Test suite failed")
        else:
            self.log_stdout("✅ All tests passed")

    def generate_ci_json_report(self) -> Dict[str, Any]:
        """Generate minimal JSON report for CI mode"""
        total_duration = int((self.end_time - self.start_time) * 1000) if self.end_time and self.start_time else 0

        # Convert results to operation format expected by orchestrator
        operations = []
        for result in self.results:
            op = {
                "operation": result.name,
                "success": result.status == "passed",
                "duration_ms": result.duration_ms,
                "error": result.error_info["message"] if result.error_info else None,
                "details": result.details or {}
            }
            operations.append(op)

        return {
            "language": "python",
            "sdk_version": self._get_sdk_version(),
            "operations": operations,
            "total_duration_ms": total_duration,
            "os": sys.platform,
            "architecture": os.uname().machine if hasattr(os, 'uname') else "unknown",
            "timestamp": datetime.utcnow().isoformat() + "Z"
        }

    def generate_json_report(self) -> Dict[str, Any]:
        """Generate comprehensive JSON report"""
        # Categorize results
        passed_tests = [r for r in self.results if r.status == "passed"]
        failed_tests = [r for r in self.results if r.status == "failed"]
        error_tests = [r for r in self.results if r.status == "error"]
        skipped_tests = [r for r in self.results if r.status == "skipped"]

        total_duration = int((self.end_time - self.start_time) * 1000) if self.end_time and self.start_time else 0

        return {
            "language": "python",
            "sdk_version": self._get_sdk_version(),
            "test_results": {
                "summary": {
                    "total": len(self.results),
                    "passed": len(passed_tests),
                    "failed": len(failed_tests),
                    "errored": len(error_tests),
                    "skipped": len(skipped_tests),
                    "duration_ms": total_duration,
                    "status": "passed" if len(failed_tests) == 0 and len(error_tests) == 0 else "failed"
                },
                "passed_tests": [r.to_dict() for r in passed_tests],
                "failed_tests": [r.to_dict() for r in failed_tests],
                "error_tests": [r.to_dict() for r in error_tests],
                "skipped_tests": [r.to_dict() for r in skipped_tests]
            },
            "test_execution": {
                "start_time": datetime.fromtimestamp(self.start_time).isoformat() + "Z" if self.start_time else None,
                "end_time": datetime.fromtimestamp(self.end_time).isoformat() + "Z" if self.end_time else None,
                "duration_ms": total_duration,
                "test_order": "sequential",
                "parallelism": 1,
                "retry_policy": {
                    "enabled": False,
                    "max_retries": 0
                }
            },
            "environment": self._get_environment_info(),
            "build_info": self._get_build_info(),
            "logs": self.logs,
            "timestamp": datetime.utcnow().isoformat() + "Z"
        }

    def _get_sdk_version(self) -> str:
        """Get SDK version"""
        try:
            import bitwarden_sdk
            return getattr(bitwarden_sdk, "__version__", "unknown")
        except:
            return "unknown"

    def _get_environment_info(self) -> Dict[str, Any]:
        """Gather environment information"""
        import subprocess

        # Get CPU info
        cpu_info = {
            "cores": os.cpu_count()
        }

        # Try to get CPU model on different platforms
        try:
            if platform.system() == "Darwin":
                cpu_model = subprocess.check_output(["sysctl", "-n", "machdep.cpu.brand_string"], text=True).strip()
                cpu_info["model"] = cpu_model
            elif platform.system() == "Linux":
                with open("/proc/cpuinfo") as f:
                    for line in f:
                        if line.startswith("model name"):
                            cpu_info["model"] = line.split(":")[1].strip()
                            break
        except:
            pass

        # Get memory info
        memory_info = {}
        try:
            if platform.system() == "Darwin":
                mem_bytes = int(subprocess.check_output(["sysctl", "-n", "hw.memsize"], text=True).strip())
                memory_info["total_gb"] = round(mem_bytes / (1024**3), 1)
            elif platform.system() == "Linux":
                with open("/proc/meminfo") as f:
                    for line in f:
                        if line.startswith("MemTotal"):
                            mem_kb = int(line.split()[1])
                            memory_info["total_gb"] = round(mem_kb / (1024**2), 1)
                            break
        except:
            pass

        return {
            "os": {
                "platform": platform.system().lower(),
                "version": platform.version(),
                "kernel": platform.release()
            },
            "arch": platform.machine(),
            "cpu": cpu_info,
            "memory": memory_info,
            "runtime": {
                "name": "Python",
                "version": platform.python_version(),
                "path": sys.executable
            },
            "sdk": {
                "source": self.sdk_source,
                "path": os.path.dirname(os.path.dirname(__file__))
            },
            "server": {
                "type": self.test_mode,
                "url": os.getenv("API_URL", "http://localhost:4000"),
                "healthy": True  # Would check this with actual health endpoint
            },
            "env_variables": {
                "ORGANIZATION_ID": self.organization_id,
                "API_URL": os.getenv("API_URL"),
                "IDENTITY_URL": os.getenv("IDENTITY_URL"),
                "STATE_FILE": self.state_file,
                "TEST_MODE": self.test_mode,
                "SDK_SOURCE": self.sdk_source
            }
        }

    def _get_build_info(self) -> Dict[str, Any]:
        """Get build information"""
        import subprocess

        build_info = {
            "built_at": datetime.utcnow().isoformat() + "Z"
        }

        try:
            build_info["commit"] = subprocess.check_output(
                ["git", "rev-parse", "HEAD"],
                text=True,
                stderr=subprocess.DEVNULL
            ).strip()

            build_info["branch"] = subprocess.check_output(
                ["git", "rev-parse", "--abbrev-ref", "HEAD"],
                text=True,
                stderr=subprocess.DEVNULL
            ).strip()

            # Check if working directory is dirty
            status = subprocess.check_output(
                ["git", "status", "--porcelain"],
                text=True,
                stderr=subprocess.DEVNULL
            )
            build_info["dirty"] = bool(status.strip())

            # Get latest tag if available
            try:
                build_info["tag"] = subprocess.check_output(
                    ["git", "describe", "--tags", "--abbrev=0"],
                    text=True,
                    stderr=subprocess.DEVNULL
                ).strip()
            except:
                pass

        except:
            build_info["commit"] = "unknown"
            build_info["branch"] = "unknown"
            build_info["dirty"] = False

        # Try to get cargo version
        try:
            cargo_version = subprocess.check_output(
                ["cargo", "--version"],
                text=True,
                stderr=subprocess.DEVNULL
            ).strip().split()[1]
            build_info["cargo_version"] = cargo_version
        except:
            pass

        return build_info


def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(description="Python SDK Test Suite")
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output results in simple JSON format for CI/CD integration"
    )
    parser.add_argument(
        "--output-file",
        type=str,
        help="Save results to file (JSON format)"
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Enable verbose output"
    )
    parser.add_argument(
        "--test-mode",
        type=str,
        choices=["fake-server", "real-server"],
        help="Override TEST_MODE environment variable"
    )
    parser.add_argument(
        "--sdk-source",
        type=str,
        choices=["local-build", "latest-main"],
        help="Override SDK_SOURCE environment variable"
    )

    args = parser.parse_args()

    # Override environment variables if specified
    if args.test_mode:
        os.environ["TEST_MODE"] = args.test_mode
    if args.sdk_source:
        os.environ["SDK_SOURCE"] = args.sdk_source

    # Determine output format
    output_format = "text"
    simple_json = False
    if args.json:
        output_format = "json"
        simple_json = True  # --json flag means simple format for CI/CD
    elif args.output_file:
        output_format = "json"
        # outputFile without --json flag means comprehensive format

    # Setup logging if verbose
    if args.verbose:
        logging.basicConfig(level=logging.DEBUG)

    # Run tests
    try:
        tester = PythonSdkTestSuite(output_format=output_format, verbose=args.verbose)
        tester.run_all_tests()

        # Generate report
        if output_format == "json" or args.output_file:
            # Use simple JSON format if --json flag was used
            if simple_json:
                report = tester.generate_ci_json_report()
            else:
                report = tester.generate_json_report()

            # Output to file if specified
            if args.output_file:
                with open(args.output_file, 'w') as f:
                    json.dump(report, f, indent=2, cls=UUIDEncoder)
                if output_format == "text":
                    print(f"Results saved to {args.output_file}")

            # Output to stdout if JSON format requested
            if output_format == "json":
                print(json.dumps(report, indent=2, cls=UUIDEncoder))

        # Exit with appropriate code
        all_passed = all(r.status == "passed" or r.status == "skipped" for r in tester.results)
        # In JSON mode, always exit 0 and let the JSON report the status
        # Otherwise the C# test framework will fail even though it got valid JSON
        if output_format == "json":
            sys.exit(0)
        else:
            sys.exit(0 if all_passed else 1)

    except Exception as e:
        # Handle fatal errors
        if output_format == "json":
            if simple_json:
                # Minimal error report for simple JSON format
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
            else:
                # Full error report
                error_report = {
                    "language": "python",
                    "sdk_version": "unknown",
                    "test_results": {
                        "summary": {
                            "total": 0,
                            "passed": 0,
                            "failed": 0,
                            "errored": 1,
                            "skipped": 0,
                            "status": "error"
                        }
                    },
                    "error": str(e),
                    "timestamp": datetime.utcnow().isoformat() + "Z"
                }
            print(json.dumps(error_report, indent=2, cls=UUIDEncoder))
        else:
            print(f"Fatal error: {e}", file=sys.stderr)
            if args.verbose:
                traceback.print_exc()

        sys.exit(1)


if __name__ == "__main__":
    main()