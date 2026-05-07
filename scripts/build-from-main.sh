#!/usr/bin/env bash
set -euo pipefail

# Script to build SDK from latest main branch for testing
# This script is used by language-specific test.sh scripts when --sdk-source=latest-main is specified

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
LANGUAGE=""
BUILD_DIR="/tmp/sdk-main-$$"
VERBOSE=false
CLEAN_BUILD=false

print_usage() {
	cat <<EOF
Usage: $0 <language> [OPTIONS]

Build SDK from latest main branch for the specified language.

ARGUMENTS:
    language            Language to build for: python, go, java, csharp, js, ruby, php

OPTIONS:
    --build-dir DIR     Directory to clone/build in (default: /tmp/sdk-main-$$)
    --verbose           Enable verbose output
    --clean             Force clean build (remove existing build directory)
    --help              Show this help message

EXAMPLES:
    # Build Python SDK from main
    $0 python

    # Build Go SDK from main with verbose output
    $0 go --verbose

    # Build with custom directory
    $0 python --build-dir /workspace/sdk-main

    # Force clean build
    $0 python --clean
EOF
}

log_info() {
	echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
	echo -e "${GREEN}[✓]${NC} $1"
}

log_error() {
	echo -e "${RED}[✗]${NC} $1" >&2
}

log_warning() {
	echo -e "${YELLOW}[!]${NC} $1"
}

log_verbose() {
	if [ "$VERBOSE" = true ]; then
		echo -e "${BLUE}[DEBUG]${NC} $1"
	fi
}

# Parse arguments
if [ $# -eq 0 ]; then
	log_error "Missing required argument: language"
	print_usage
	exit 1
fi

LANGUAGE="$1"
shift

# Parse options
while [[ $# -gt 0 ]]; do
	case $1 in
		--build-dir)
			BUILD_DIR="$2"
			shift 2
			;;
		--verbose)
			VERBOSE=true
			shift
			;;
		--clean)
			CLEAN_BUILD=true
			shift
			;;
		--help)
			print_usage
			exit 0
			;;
		*)
			log_error "Unknown option: $1"
			print_usage
			exit 1
			;;
	esac
done

# Validate language
case "$LANGUAGE" in
	python|go|java|csharp|js|javascript|node|ruby|php|cpp|c)
		log_info "Building SDK for language: $LANGUAGE"
		;;
	*)
		log_error "Unsupported language: $LANGUAGE"
		log_error "Supported languages: python, go, java, csharp, js, ruby, php, cpp, c"
		exit 1
		;;
esac

check_requirements() {
	log_info "Checking requirements..."

	local missing=false

	if ! command -v git >/dev/null 2>&1; then
		log_error "git is required but not installed"
		missing=true
	fi

	if ! command -v cargo >/dev/null 2>&1; then
		log_error "cargo is required but not installed"
		log_error "Install from: https://rustup.rs/"
		missing=true
	fi

	if ! command -v npm >/dev/null 2>&1; then
		log_error "npm is required but not installed"
		log_error "Install Node.js from: https://nodejs.org/"
		missing=true
	fi

	if [ "$missing" = true ]; then
		exit 1
	fi

	log_success "All requirements met"
}

clone_or_update_main() {
	log_info "Setting up main branch in $BUILD_DIR..."

	# Clean if requested
	if [ "$CLEAN_BUILD" = true ] && [ -d "$BUILD_DIR" ]; then
		log_warning "Removing existing build directory..."
		rm -rf "$BUILD_DIR"
	fi

	# Clone or update
	if [ -d "$BUILD_DIR" ]; then
		log_info "Updating existing main branch clone..."
		(
			cd "$BUILD_DIR"
			git fetch origin main
			git reset --hard origin/main
			git clean -fdx
		)
	else
		log_info "Cloning main branch..."

		# Try to get origin URL, fallback to GitHub
		local repo_url
		if git remote get-url origin >/dev/null 2>&1; then
			repo_url=$(git remote get-url origin)
		else
			repo_url="https://github.com/bitwarden/sdk.git"
		fi

		git clone --depth 1 --branch main "$repo_url" "$BUILD_DIR"
	fi

	log_success "Main branch ready in $BUILD_DIR"
}

generate_schemas() {
	log_info "Generating schemas..."

	(
		cd "$BUILD_DIR"

		log_verbose "Installing npm dependencies..."
		npm install --loglevel=error

		log_verbose "Running schema generation..."
		npm run schemas
	)

	log_success "Schemas generated"
}

build_rust_core() {
	log_info "Building Rust core libraries..."

	(
		cd "$BUILD_DIR"

		# Build based on language requirements
		case "$LANGUAGE" in
			python)
				log_verbose "Building for Python (release mode)..."
				cargo build --release --quiet
				;;
			go|cpp|c)
				log_verbose "Building bitwarden-c library..."
				cargo build -p bitwarden-c --release --quiet
				;;
			java|csharp)
				log_verbose "Building bitwarden-c library..."
				cargo build -p bitwarden-c --release --quiet
				;;
			js|javascript|node)
				log_verbose "Building bitwarden-napi..."
				cargo build -p bitwarden-napi --release --quiet
				;;
			*)
				log_verbose "Building all packages..."
				cargo build --release --quiet
				;;
		esac
	)

	log_success "Rust core libraries built"
}

copy_artifacts_python() {
	local target_dir="$PROJECT_ROOT/languages/python"

	log_info "Copying Python artifacts..."

	# Copy built libraries
	if [ -d "$BUILD_DIR/target/release" ]; then
		log_verbose "Copying release build artifacts..."
		mkdir -p "$target_dir/target"
		cp -r "$BUILD_DIR/target/release" "$target_dir/target/"
	fi

	# Copy schemas
	if [ -f "$BUILD_DIR/languages/python/bitwarden_sdk/schemas.py" ]; then
		log_verbose "Copying generated schemas..."
		mkdir -p "$target_dir/bitwarden_sdk"
		cp "$BUILD_DIR/languages/python/bitwarden_sdk/schemas.py" "$target_dir/bitwarden_sdk/"
	fi

	log_success "Python artifacts copied"
}

copy_artifacts_go() {
	local target_dir="$PROJECT_ROOT/languages/go"

	log_info "Copying Go artifacts..."

	# Determine platform-specific library name
	local lib_name
	case "$(uname -s)" in
		Darwin)
			lib_name="libbitwarden_c.dylib"
			;;
		Linux)
			lib_name="libbitwarden_c.so"
			;;
		MINGW*|CYGWIN*|MSYS*)
			lib_name="bitwarden_c.dll"
			;;
		*)
			lib_name="libbitwarden_c.a"
			;;
	esac

	# Copy C library
	if [ -f "$BUILD_DIR/target/release/$lib_name" ]; then
		log_verbose "Copying $lib_name..."
		mkdir -p "$target_dir/internal/cinterface/lib"
		cp "$BUILD_DIR/target/release/$lib_name" "$target_dir/internal/cinterface/lib/"
	fi

	# Also copy static library if exists
	if [ -f "$BUILD_DIR/target/release/libbitwarden_c.a" ]; then
		log_verbose "Copying static library..."
		cp "$BUILD_DIR/target/release/libbitwarden_c.a" "$target_dir/internal/cinterface/lib/"
	fi

	# Copy Go bindings if updated
	if [ -f "$BUILD_DIR/languages/go/bitwarden_sdk.go" ]; then
		log_verbose "Copying Go bindings..."
		cp "$BUILD_DIR/languages/go/"*.go "$target_dir/" 2>/dev/null || true
	fi

	log_success "Go artifacts copied"
}

copy_artifacts_java() {
	local target_dir="$PROJECT_ROOT/languages/java"

	log_info "Copying Java artifacts..."

	# Copy JAR files if built
	if [ -d "$BUILD_DIR/languages/java/target" ]; then
		log_verbose "Copying Java build artifacts..."
		mkdir -p "$target_dir/target"
		cp -r "$BUILD_DIR/languages/java/target/"*.jar "$target_dir/target/" 2>/dev/null || true
	fi

	# Copy native libraries
	local lib_name
	case "$(uname -s)" in
		Darwin)
			lib_name="libbitwarden_c.dylib"
			;;
		Linux)
			lib_name="libbitwarden_c.so"
			;;
		MINGW*|CYGWIN*|MSYS*)
			lib_name="bitwarden_c.dll"
			;;
	esac

	if [ -f "$BUILD_DIR/target/release/$lib_name" ]; then
		log_verbose "Copying native library..."
		mkdir -p "$target_dir/src/main/resources"
		cp "$BUILD_DIR/target/release/$lib_name" "$target_dir/src/main/resources/"
	fi

	log_success "Java artifacts copied"
}

copy_artifacts_csharp() {
	local target_dir="$PROJECT_ROOT/languages/csharp"

	log_info "Copying C# artifacts..."

	# Copy built DLLs
	if [ -d "$BUILD_DIR/languages/csharp/Bitwarden.Sdk/bin" ]; then
		log_verbose "Copying C# build artifacts..."
		mkdir -p "$target_dir/Bitwarden.Sdk/bin"
		cp -r "$BUILD_DIR/languages/csharp/Bitwarden.Sdk/bin/"* "$target_dir/Bitwarden.Sdk/bin/" 2>/dev/null || true
	fi

	# Copy native libraries
	local lib_name
	case "$(uname -s)" in
		Darwin)
			lib_name="libbitwarden_c.dylib"
			;;
		Linux)
			lib_name="libbitwarden_c.so"
			;;
		MINGW*|CYGWIN*|MSYS*)
			lib_name="bitwarden_c.dll"
			;;
	esac

	if [ -f "$BUILD_DIR/target/release/$lib_name" ]; then
		log_verbose "Copying native library..."
		mkdir -p "$target_dir/src/Native"
		cp "$BUILD_DIR/target/release/$lib_name" "$target_dir/src/Native/"
	fi

	log_success "C# artifacts copied"
}

copy_artifacts_js() {
	local target_dir="$PROJECT_ROOT/languages/js"

	log_info "Copying JavaScript/Node.js artifacts..."

	# Copy NAPI bindings
	if [ -d "$BUILD_DIR/target/release" ]; then
		log_verbose "Copying NAPI bindings..."
		mkdir -p "$target_dir/native"

		# Find and copy .node files
		find "$BUILD_DIR/target/release" -name "*.node" -exec cp {} "$target_dir/native/" \; 2>/dev/null || true
	fi

	# Copy TypeScript definitions if generated
	if [ -d "$BUILD_DIR/languages/js/dist" ]; then
		log_verbose "Copying built JavaScript artifacts..."
		mkdir -p "$target_dir/dist"
		cp -r "$BUILD_DIR/languages/js/dist/"* "$target_dir/dist/" 2>/dev/null || true
	fi

	log_success "JavaScript/Node.js artifacts copied"
}

copy_artifacts_ruby() {
	local target_dir="$PROJECT_ROOT/languages/ruby"

	log_info "Copying Ruby artifacts..."

	# Copy native extension
	if [ -d "$BUILD_DIR/target/release" ]; then
		log_verbose "Copying Ruby native extension..."
		mkdir -p "$target_dir/lib/bitwarden_sdk"

		# Find and copy .so/.bundle files
		find "$BUILD_DIR/target/release" -name "*.so" -o -name "*.bundle" | while read -r file; do
			cp "$file" "$target_dir/lib/bitwarden_sdk/" 2>/dev/null || true
		done
	fi

	log_success "Ruby artifacts copied"
}

copy_artifacts_php() {
	local target_dir="$PROJECT_ROOT/languages/php"

	log_info "Copying PHP artifacts..."

	# Copy native extension
	if [ -d "$BUILD_DIR/target/release" ]; then
		log_verbose "Copying PHP native extension..."
		mkdir -p "$target_dir/ext"

		# Find and copy .so files
		find "$BUILD_DIR/target/release" -name "*.so" -exec cp {} "$target_dir/ext/" \; 2>/dev/null || true
	fi

	log_success "PHP artifacts copied"
}

copy_artifacts_cpp() {
	local target_dir="$PROJECT_ROOT/languages/cpp"

	log_info "Copying C++ artifacts..."

	# Copy headers and libraries
	if [ -f "$BUILD_DIR/target/release/libbitwarden_c.a" ]; then
		log_verbose "Copying C++ libraries..."
		mkdir -p "$target_dir/lib"
		cp "$BUILD_DIR/target/release/libbitwarden_c."* "$target_dir/lib/" 2>/dev/null || true
	fi

	# Copy headers if available
	if [ -d "$BUILD_DIR/crates/bitwarden-c/include" ]; then
		log_verbose "Copying C++ headers..."
		mkdir -p "$target_dir/include"
		cp -r "$BUILD_DIR/crates/bitwarden-c/include/"* "$target_dir/include/" 2>/dev/null || true
	fi

	log_success "C++ artifacts copied"
}

copy_artifacts() {
	log_info "Copying artifacts for $LANGUAGE..."

	case "$LANGUAGE" in
		python)
			copy_artifacts_python
			;;
		go)
			copy_artifacts_go
			;;
		java)
			copy_artifacts_java
			;;
		csharp)
			copy_artifacts_csharp
			;;
		js|javascript|node)
			copy_artifacts_js
			;;
		ruby)
			copy_artifacts_ruby
			;;
		php)
			copy_artifacts_php
			;;
		cpp|c)
			copy_artifacts_cpp
			;;
	esac

	log_success "All artifacts copied successfully"
}

run_language_specific_build() {
	log_info "Running language-specific build steps..."

	case "$LANGUAGE" in
		python)
			if command -v maturin >/dev/null 2>&1; then
				log_verbose "Running maturin build..."
				(
					cd "$BUILD_DIR/languages/python"
					maturin build --release
				)
			fi
			;;
		go)
			if [ -f "$BUILD_DIR/languages/go/setup.sh" ]; then
				log_verbose "Running Go setup..."
				(
					cd "$BUILD_DIR/languages/go"
					./setup.sh
				)
			fi
			;;
		java)
			if command -v mvn >/dev/null 2>&1; then
				log_verbose "Running Maven build..."
				(
					cd "$BUILD_DIR/languages/java"
					mvn package -DskipTests
				)
			fi
			;;
		csharp)
			if command -v dotnet >/dev/null 2>&1; then
				log_verbose "Running .NET build..."
				(
					cd "$BUILD_DIR/languages/csharp"
					dotnet build --configuration Release
				)
			fi
			;;
		js|javascript|node)
			log_verbose "Running npm build..."
			(
				cd "$BUILD_DIR/languages/js"
				npm install --loglevel=error
				npm run build
			)
			;;
	esac

	log_success "Language-specific build completed"
}

cleanup() {
	if [ "$VERBOSE" = true ]; then
		log_info "Build directory preserved at: $BUILD_DIR"
		log_info "To clean up manually, run: rm -rf $BUILD_DIR"
	else
		log_info "Cleaning up build directory..."
		rm -rf "$BUILD_DIR"
	fi
}

main() {
	echo "=========================================="
	echo "SDK Build from Main Branch"
	echo "Language: $LANGUAGE"
	echo "=========================================="
	echo ""

	# Set up trap for cleanup on exit
	if [ "$VERBOSE" != true ]; then
		trap cleanup EXIT
	fi

	check_requirements
	clone_or_update_main
	generate_schemas
	build_rust_core
	run_language_specific_build
	copy_artifacts

	echo ""
	echo "=========================================="
	log_success "Build from main completed successfully!"
	echo "=========================================="

	if [ "$VERBOSE" = true ]; then
		echo ""
		log_info "Build artifacts have been copied to your local SDK directory"
		log_info "Build directory preserved at: $BUILD_DIR"
	fi
}

# Run main function
main