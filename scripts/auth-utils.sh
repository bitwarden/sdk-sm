#!/usr/bin/env bash
# Authentication utilities for SDK test scripts
# Handles authentication with real and fake Bitwarden servers

# Source logging utilities if not already loaded
if [ -z "${TEST_UTILS_LOADED:-}" ]; then
	_UTIL_SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
	source "$_UTIL_SCRIPT_DIR/logging-utils.sh"
fi

# Authenticate with real Bitwarden server
authenticate_real_server() {
	# This function gets called only when TEST_MODE != "fake-server"

	local client_id="${BWS_CLIENT_ID:-}"
	local client_secret="${BWS_CLIENT_SECRET:-}"

	if [ -z "$client_id" ] || [ -z "$client_secret" ]; then
		log_error "BWS_CLIENT_ID and BWS_CLIENT_SECRET must be set for real server authentication"
		log_info "Add these to your .env file for real server testing"
		return 1
	fi

	log_info "Authenticating with real Bitwarden server..."

	# Create service account access token in Bitwarden format
	# Format: 0.{client_id}.{client_secret}:{optional_encryption_key}
	export ACCESS_TOKEN="0.${client_id}.${client_secret}"

	# Optionally validate the token if bws CLI is available
	if command -v bws >/dev/null 2>&1; then
		if BWS_ACCESS_TOKEN="$ACCESS_TOKEN" BWS_SERVER_URL="${API_URL%/api}" bws secret list --output json >/dev/null 2>&1; then
			log_success "Successfully authenticated with real server"
		else
			log_warning "Could not validate token with bws CLI - proceeding anyway"
		fi
	fi

	return 0
}

# Handle authentication based on TEST_MODE
handle_authentication() {
	if [ "$TEST_MODE" = "real-server" ]; then
		# Real server: authenticate dynamically using BWS credentials
		if ! authenticate_real_server; then
			log_error "Failed to authenticate with real server"
			return 1
		fi
	elif [ "$TEST_MODE" = "fake-server" ]; then
		# Fake server: ensure ACCESS_TOKEN is set from .env
		if [ -z "$ACCESS_TOKEN" ]; then
			log_error "ACCESS_TOKEN must be set in .env for fake server testing"
			return 1
		fi
	else
		log_error "Unknown TEST_MODE: $TEST_MODE (expected 'fake-server' or 'real-server')"
		return 1
	fi
	return 0
}