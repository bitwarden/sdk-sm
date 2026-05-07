#!/usr/bin/env bash
# Environment management utilities for SDK test scripts
# Handles loading and exporting environment variables from .env and test-config.json

# Source logging utilities if not already loaded
if [ -z "${TEST_UTILS_LOADED:-}" ]; then
	_UTIL_SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
	source "$_UTIL_SCRIPT_DIR/logging-utils.sh"
fi

# Load environment and configuration
load_test_environment() {
	local repo_root="$1"

	# Configuration file paths
	export SDK_TEST_ENV="$repo_root/tests/SdkTestFramework.Tests/Configuration/.env"
	export SDK_TEST_CONFIG="$repo_root/tests/SdkTestFramework.Tests/Configuration/test-config.json"

	# Load environment variables from .env first (always needed)
	if [ ! -f "$SDK_TEST_ENV" ]; then
		log_error "Error: .env file not found at $SDK_TEST_ENV"
		log_error "Please create the .env file in the SDKTestFramework Configuration directory"
		exit 1
	fi
	source "$SDK_TEST_ENV"

	# Export the environment variables for child processes
	export ACCESS_TOKEN
	export ORGANIZATION_ID
	export API_URL
	export IDENTITY_URL
	export STATE_FILE

	# Read TEST_MODE and SDK_SOURCE from test-config.json
	if [ ! -f "$SDK_TEST_CONFIG" ]; then
		log_error "Error: test-config.json not found at $SDK_TEST_CONFIG"
		exit 1
	fi

	# Extract configuration values from test-config.json
	if command -v jq >/dev/null 2>&1; then
		TEST_MODE=$(jq -r '.configuration.TEST_MODE' "$SDK_TEST_CONFIG")
		CONFIG_SDK_SOURCE=$(jq -r '.configuration.SDK_SOURCE // "local"' "$SDK_TEST_CONFIG")
	else
		# Fallback to grep/sed if jq is not available
		TEST_MODE=$(grep -o '"TEST_MODE"[[:space:]]*:[[:space:]]*"[^"]*"' "$SDK_TEST_CONFIG" | sed 's/.*: *"\([^"]*\)".*/\1/')
		CONFIG_SDK_SOURCE=$(grep -o '"SDK_SOURCE"[[:space:]]*:[[:space:]]*"[^"]*"' "$SDK_TEST_CONFIG" | sed 's/.*: *"\([^"]*\)".*/\1/')
		CONFIG_SDK_SOURCE="${CONFIG_SDK_SOURCE:-local}"
	fi
	export TEST_MODE

	# Use SDK_SOURCE from config (or override from command line if provided)
	SDK_SOURCE="${SDK_SOURCE:-$CONFIG_SDK_SOURCE}"
	export SDK_SOURCE
}