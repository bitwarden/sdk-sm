#!/usr/bin/env bash
# Build utilities for SDK test scripts
# Handles building SDKs from source or main branch

# Source logging utilities if not already loaded
if [ -z "${TEST_UTILS_LOADED:-}" ]; then
	_UTIL_SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
	source "$_UTIL_SCRIPT_DIR/logging-utils.sh"
fi

# Build SDK from main branch using centralized script
build_sdk_from_main() {
	local language="$1"
	local repo_root="$2"
	local verbose="${3:-false}"

	log_info "Building SDK from latest main branch..."

	local build_script="$repo_root/scripts/build-from-main.sh"

	if [ ! -f "$build_script" ]; then
		log_error "build-from-main.sh script not found at $build_script"
		return 1
	fi

	# Make sure script is executable
	chmod +x "$build_script"

	# Run the build script
	local build_cmd="$build_script $language"
	if [ "$verbose" = true ]; then
		build_cmd="$build_cmd --verbose"
	fi

	if $build_cmd; then
		log_success "Built SDK from main branch"
		return 0
	else
		log_error "Failed to build SDK from main branch"
		return 1
	fi
}