#!/usr/bin/env bash
# shellcheck disable=SC1090
set -euo pipefail

# Default values
TMP_DIR="$(mktemp -d)"
PYTHON_VERSIONS="${PYTHON_VERSIONS:-3.13}"
JSON_OUTPUT=false
BUILD_SDK=true
OUTPUT_FILE=""
VERBOSE=false
TEST_SCRIPT="test_suite.py"  # The new unified test file

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Source shared utilities
source "$REPO_ROOT/scripts/test-utils.sh"

# Load environment and configuration
load_test_environment "$REPO_ROOT"

print_usage() {
	cat <<EOF
Usage: $0 [OPTIONS]

Run Python SDK tests with optional JSON output and build configuration.

OPTIONS:
    --json              Output results in simple JSON format for CI/CD integration
    --no-build          Skip SDK build (use existing build)
    --sdk-source TYPE   SDK source: local|main (default: local)
    --output-file FILE  Save test results to file (JSON format)
    --verbose           Enable verbose output
    --python VERSION    Python version to test (default: 3.13)
    --help              Show this help message

ENVIRONMENT VARIABLES:
    PYTHON_VERSIONS     Space-separated list of Python versions (default: 3.13)
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

    # Run with main branch SDK
    $0 --sdk-source main

    # Run with specific Python version
    $0 --python 3.11
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
		--python)
			PYTHON_VERSIONS="$2"
			shift 2
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

	if ! command -v python3 >/dev/null 2>&1; then
		log_error "python3 is required but not installed"
		missing_requirements=true
	fi

	# Only check for uv if we're going to build
	if [ "$BUILD_SDK" = true ]; then
		if ! command -v uv >/dev/null 2>&1; then
			log_error "uv is required for building but not installed"
			log_info "Install with: curl -LsSf https://astral.sh/uv/install.sh | sh"
			missing_requirements=true
		fi
	fi

	if [ ! -f "$SCRIPT_DIR/bitwarden_sdk/schemas.py" ]; then
		log_error "schemas.py not found. This file is required for Python SDK tests."
		log_info "To generate schemas, run from repository root:"
		log_info "  npm run schemas"
		missing_requirements=true
	fi

	if [ "$missing_requirements" = true ]; then
		exit 1
	fi

	log_success "All requirements met"
}

source_venv() {
	local python_version=$1

	# Try to activate virtual environment (cross-platform)
	if [ -f "$TMP_DIR/.venv-$python_version/bin/activate" ]; then
		source "$TMP_DIR/.venv-$python_version/bin/activate"
	elif [ -f "$TMP_DIR/.venv-$python_version/Scripts/activate" ]; then
		# Windows
		source "$TMP_DIR/.venv-$python_version/Scripts/activate"
	else
		log_error "Failed to activate virtual environment for $python_version"
		exit 1
	fi
}

setup_python_environment() {
	local python_version=$1

	# Skip virtual environment setup if not building
	if [ "$BUILD_SDK" = false ]; then
		log_info "Skipping virtual environment setup (--no-build flag set)"
		return 0
	fi

	log_info "Setting up Python $python_version virtual environment..."

	# Create virtual environment if it doesn't exist
	if [ ! -d "$TMP_DIR/.venv-$python_version" ]; then
		log_info "  - Creating new virtual environment in $TMP_DIR/.venv-$python_version"
		if [ "$JSON_OUTPUT" = true ]; then
			uv venv "$TMP_DIR/.venv-$python_version" --python "$python_version" >&2
		else
			uv venv "$TMP_DIR/.venv-$python_version" --python "$python_version"
		fi
		log_success "Created Python virtual environment for $python_version"
	else
		log_info "  - Using existing virtual environment"
	fi

	# Activate virtual environment
	log_info "  - Activating virtual environment"
	source_venv "$python_version"

	# Upgrade pip
	log_info "  - Upgrading pip to latest version"
	uv pip install --upgrade pip --quiet
	log_success "Environment ready"
}


build_package() {
	local python_version=$1

	if [ "$BUILD_SDK" = false ]; then
		log_info "Skipping SDK build (--no-build flag set)"
		return 0
	fi

	log_info "Building Python package for $python_version..."

	# Build from main if requested
	if [ "$SDK_SOURCE" = "main" ]; then
		build_sdk_from_main "python" "$REPO_ROOT" "$VERBOSE"
	fi

	# Activate virtual environment
	source_venv "$python_version"

	# Change to Python directory for package operations
	pushd "$SCRIPT_DIR" >/dev/null || {
		log_error "Failed to change to Python directory: $SCRIPT_DIR"
		return 1
	}

	# Check if maturin is already installed
	if ! command -v maturin >/dev/null 2>&1; then
		log_info "Installing dependencies..."
		log_info "  - Installing Python development packages (pytest, maturin, etc.)..."
		if [ "$(uname -s)" = "Linux" ]; then
			# Linux requires patchelf for binary compatibility
			log_info "  - Platform: Linux (including patchelf)"
			if [ "$JSON_OUTPUT" = true ]; then
				uv pip install .[dev-linux] >&2
			else
				uv pip install .[dev-linux]
			fi
		else
			log_info "  - Platform: $(uname -s)"
			if [ "$JSON_OUTPUT" = true ]; then
				uv pip install .[dev] >&2
			else
				uv pip install .[dev]
			fi
		fi
		log_success "Dependencies installed"
	else
		log_info "Dependencies already installed, skipping..."
	fi

	# Build with maturin
	log_info "Building Rust extension with maturin..."
	log_info "  - Compiling Rust code to Python extension..."
	if [ "$VERBOSE" = true ]; then
		maturin develop
	else
		# Redirect maturin output to stderr to avoid contaminating JSON output
		if [ "$JSON_OUTPUT" = true ]; then
			maturin develop >&2
		else
			maturin develop
		fi
	fi
	log_success "Rust extension built"

	popd >/dev/null

	log_success "Built Python package"
}


run_tests() {
	local python_version=$1
	log_info "Running tests for Python $python_version..."

	# Activate virtual environment only if we built
	if [ "$BUILD_SDK" = true ]; then
		source_venv "$python_version"
	fi

	# Start fake server if needed (only for fake-server mode)
	if [ "$TEST_MODE" = "fake-server" ]; then
		if ! start_fake_server_if_needed "$REPO_ROOT"; then
			log_error "Failed to ensure fake server is running"
			return 1
		fi
	fi

	# Handle authentication
	if ! handle_authentication; then
		return 1
	fi

	# Prepare test command
	local test_cmd="python3 $SCRIPT_DIR/test/$TEST_SCRIPT"

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

	# TEST_MODE is already exported from reading test-config.json

	# Run the test
	if [ "$VERBOSE" = true ]; then
		log_info "Running command: $test_cmd"
	fi

	eval "$test_cmd"
	local test_exit_code=$?

	return $test_exit_code
}

cleanup() {
	if [ "$VERBOSE" = true ]; then
		log_info "Cleaning up temporary directory: $TMP_DIR"
	fi
	rm -rf "$TMP_DIR"
}

main() {
	# Set up trap for cleanup
	trap cleanup EXIT

	# Only show header for non-JSON output
	if [ "$JSON_OUTPUT" != true ]; then
		echo "=========================================="
		echo "Python SDK Test Runner"
		echo "=========================================="
		echo ""
	fi

	check_requirements

	# Track overall success
	local overall_success=true

	for python_version in $PYTHON_VERSIONS; do
		if [ "$JSON_OUTPUT" != true ]; then
			echo ""
			echo "Testing Python $python_version"
			echo "------------------------------------------"
		fi

		setup_python_environment "$python_version"
		build_package "$python_version"

		if ! run_tests "$python_version"; then
			overall_success=false
			if [ "$JSON_OUTPUT" != true ]; then
				log_error "Tests failed for Python $python_version"
			fi
		else
			if [ "$JSON_OUTPUT" != true ]; then
				log_success "Tests passed for Python $python_version"
			fi
		fi
	done

	if [ "$JSON_OUTPUT" != true ]; then
		echo ""
		echo "=========================================="
		if [ "$overall_success" = true ]; then
			log_success "All tests completed successfully"
		else
			log_error "Some tests failed"
		fi
		echo "=========================================="
	fi

	# Exit with appropriate code
	if [ "$overall_success" = true ]; then
		exit 0
	else
		exit 1
	fi
}

# Run main function
main