#!/usr/bin/env bash
set -euo pipefail

# Default values
JSON_OUTPUT=false
BUILD_SDK=true
OUTPUT_FILE=""
VERBOSE=false
TEST_SCRIPT="test_suite.go"

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Source shared utilities
source "$PROJECT_ROOT/scripts/test-utils.sh"

# Load environment and configuration
load_test_environment "$PROJECT_ROOT"

print_usage() {
	cat <<EOF
Usage: $0 [OPTIONS]

Run Go SDK tests with optional JSON output and build configuration.

OPTIONS:
    --json              Output results in simple JSON format for CI/CD integration
    --no-build          Skip SDK build (use existing build)
    --sdk-source TYPE   SDK source: local|main (default: local)
    --output-file FILE  Save test results to file (JSON format)
    --verbose           Enable verbose output
    --help              Show this help message

ENVIRONMENT VARIABLES:
    ORGANIZATION_ID     Organization ID for tests
    ACCESS_TOKEN        Access token for authentication
    API_URL            API server URL (default: http://localhost:4000)
    IDENTITY_URL       Identity server URL (default: http://localhost:33656)
    STATE_FILE         State file path for SDK
    TEST_MODE          Test mode: fake-server|real-server (default: fake-server)
    SDK_SOURCE         SDK source: local|main (default: local)

EXAMPLES:
    # Run with human-readable output (default)
    $0

    # Run with JSON output for CI
    $0 --json --output-file results.json

    # Run without building SDK (use existing build)
    $0 --no-build

    # Run with latest main branch SDK
    $0 --sdk-source latest-main

    # Run with verbose output
    $0 --verbose
EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
	case $1 in
		--json)
			JSON_OUTPUT=true
			shift
			;;
		--no-build)
			BUILD_SDK=false
			shift
			;;
		--sdk-source)
			SDK_SOURCE="$2"
			shift 2
			;;
		--output-file)
			OUTPUT_FILE="$2"
			shift 2
			;;
		--verbose)
			VERBOSE=true
			shift
			;;
		--help)
			print_usage
			exit 0
			;;
		*)
			echo "Error: Unknown option $1" >&2
			print_usage
			exit 1
			;;
	esac
done

# Disable colors if JSON output
disable_colors_if_json

check_requirements() {
	log_info "Checking requirements..."

	local missing_requirements=false

	if ! command -v go >/dev/null 2>&1; then
		log_error "go is required but not installed"
		log_info "Install from: https://golang.org/dl/"
		missing_requirements=true
	fi

	# Only check for cargo if we actually need to build
	if [ "$BUILD_SDK" = true ]; then
		if ! command -v cargo >/dev/null 2>&1; then
			log_error "cargo is required for building but not installed"
			log_info "Install from: https://rustup.rs/"
			missing_requirements=true
		fi
	fi

	if [ "$missing_requirements" = true ]; then
		exit 1
	fi

	log_success "All requirements met"
}


setup_library_paths() {
	log_info "Setting up library paths..."

	# Determine platform and architecture
	local platform
	local arch

	case "$(uname -s)" in
		Darwin)
			platform="darwin"
			;;
		Linux)
			platform="linux"
			;;
		MINGW*|CYGWIN*|MSYS*)
			platform="windows"
			;;
		*)
			platform="unknown"
			;;
	esac

	case "$(uname -m)" in
		x86_64)
			arch="x64"
			;;
		arm64|aarch64)
			arch="arm64"
			;;
		*)
			arch="unknown"
			;;
	esac

	local lib_dir="$SCRIPT_DIR/internal/cinterface/lib/${platform}-${arch}"

	# Create directory if it doesn't exist
	mkdir -p "$lib_dir"

	# Check if libraries are linked
	if [ ! -f "$lib_dir/libbitwarden_c.a" ] && [ ! -f "$lib_dir/libbitwarden_c.dylib" ] && [ ! -f "$lib_dir/libbitwarden_c.so" ]; then
		log_warning "Libraries not found in $lib_dir, running setup..."

		# Run setup.sh if it exists
		if [ -f "$SCRIPT_DIR/setup.sh" ]; then
			log_info "Running setup.sh..."
			(cd "$SCRIPT_DIR" && REPO_ROOT="$PROJECT_ROOT" ./setup.sh)
		else
			log_error "setup.sh not found, cannot set up libraries"
			exit 1
		fi
	fi

	# Set library paths for runtime
	case "$platform" in
		darwin)
			export DYLD_LIBRARY_PATH="$lib_dir:${DYLD_LIBRARY_PATH:-}"
			;;
		linux)
			export LD_LIBRARY_PATH="$lib_dir:${LD_LIBRARY_PATH:-}"
			;;
		windows)
			export PATH="$lib_dir:${PATH:-}"
			;;
	esac

	log_success "Library paths configured"
}

build_sdk() {
	if [ "$BUILD_SDK" = false ]; then
		log_info "Skipping SDK build (--no-build flag set)"
		return 0
	fi

	log_info "Building Go SDK..."

	# Build from main if requested
	if [ "$SDK_SOURCE" = "main" ]; then
		build_sdk_from_main "go" "$PROJECT_ROOT" "$VERBOSE"
	else
		# Build local SDK
		log_info "Building bitwarden-c library..."

		# Build the C library
		if [ "$JSON_OUTPUT" = true ]; then
			# Suppress cargo output in JSON mode
			(cd "$PROJECT_ROOT" && cargo build -p bitwarden-c --release) >/dev/null 2>&1
		else
			(cd "$PROJECT_ROOT" && cargo build -p bitwarden-c --release)
		fi

		# Run setup to link libraries
		if [ -f "$SCRIPT_DIR/setup.sh" ]; then
			log_info "Linking libraries..."
			if [ "$JSON_OUTPUT" = true ]; then
				# Suppress setup.sh output in JSON mode
				(cd "$SCRIPT_DIR" && REPO_ROOT="$PROJECT_ROOT" ./setup.sh) >/dev/null 2>&1
			else
				(cd "$SCRIPT_DIR" && REPO_ROOT="$PROJECT_ROOT" ./setup.sh)
			fi
		fi
	fi

	# Ensure Go dependencies are up to date
	log_info "Updating Go dependencies..."
	if [ "$JSON_OUTPUT" = true ]; then
		# Suppress go mod tidy output in JSON mode
		(cd "$SCRIPT_DIR" && go mod tidy) >/dev/null 2>&1
	else
		(cd "$SCRIPT_DIR" && go mod tidy)
	fi

	log_success "Built Go SDK"
}


run_tests() {
	log_info "Running Go SDK tests..."

	# Start fake server if needed (only for fake-server mode)
	if [ "$TEST_MODE" = "fake-server" ]; then
		if ! start_fake_server_if_needed "$PROJECT_ROOT"; then
			log_error "Failed to ensure fake server is running"
			return 1
		fi
	fi

	# Handle authentication
	if ! handle_authentication; then
		return 1
	fi

	# Change to test directory
	cd "$SCRIPT_DIR/test"

	# Prepare test command
	local test_cmd="go run $TEST_SCRIPT"

	# Add flags based on options
	if [ "$JSON_OUTPUT" = true ]; then
		test_cmd="$test_cmd --json"
	fi

	if [ "$VERBOSE" = true ]; then
		test_cmd="$test_cmd --verbose"
	fi

	if [ -n "$OUTPUT_FILE" ]; then
		test_cmd="$test_cmd --output-file $OUTPUT_FILE"
	fi

	# Add environment info
	if [ -n "$SDK_SOURCE" ]; then
		export SDK_SOURCE="$SDK_SOURCE"
	fi

	# Run the test
	if [ "$VERBOSE" = true ]; then
		log_info "Running command: $test_cmd"
	fi

	eval "$test_cmd"
	local test_exit_code=$?

	return $test_exit_code
}

main() {
	# Only show header for non-JSON output
	if [ "$JSON_OUTPUT" != true ]; then
		echo "=========================================="
		echo "Go SDK Test Runner"
		echo "=========================================="
		echo ""
	fi

	check_requirements
	setup_library_paths
	build_sdk

	# Run tests
	if run_tests; then
		if [ "$JSON_OUTPUT" != true ]; then
			log_success "All tests completed successfully"
		fi
		exit 0
	else
		if [ "$JSON_OUTPUT" != true ]; then
			log_error "Some tests failed"
		fi
		exit 1
	fi
}

# Run main function
main