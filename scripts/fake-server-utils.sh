#!/usr/bin/env bash
# Fake server management utilities for SDK test scripts
# Handles starting and managing the fake-server for testing

# Source logging utilities if not already loaded
if [ -z "${TEST_UTILS_LOADED:-}" ]; then
	_UTIL_SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
	source "$_UTIL_SCRIPT_DIR/logging-utils.sh"
fi

# Start fake server if needed
start_fake_server_if_needed() {
	local repo_root="$1"

	# First check if AUTO_START_FAKE_SERVER is enabled
	local auto_start="true"
	if command -v jq >/dev/null 2>&1; then
		auto_start=$(jq -r '.configuration.AUTO_START_FAKE_SERVER // true' "$SDK_TEST_CONFIG")
	fi

	if [ "$auto_start" != "true" ]; then
		log_info "AUTO_START_FAKE_SERVER is disabled, skipping fake server start"
		return 0
	fi

	local port="${FAKE_SERVER_PORT:-4000}"

	# Check if server is already running
	if curl -s "http://localhost:$port/health" >/dev/null 2>&1; then
		log_info "Fake server already running on port $port"
		return 0
	fi

	log_info "Starting fake server on port $port..."

	# Build fake-server if not built
	if [ ! -f "$repo_root/target/debug/fake-server" ]; then
		log_info "Building fake-server..."
		(cd "$repo_root" && cargo build -p fake-server --quiet)
	fi

	# Start fake server in background
	SM_FAKE_SERVER_PORT="$port" "$repo_root/target/debug/fake-server" >/dev/null 2>&1 &
	FAKE_SERVER_PID=$!

	# Wait for server to be ready
	local max_attempts=30
	local attempt=0
	while [ $attempt -lt $max_attempts ]; do
		if curl -s "http://localhost:$port/health" >/dev/null 2>&1; then
			log_success "Fake server is ready"
			export FAKE_SERVER_PID
			return 0
		fi
		sleep 1
		attempt=$((attempt + 1))
	done

	log_error "Fake server failed to start within 30 seconds"
	kill $FAKE_SERVER_PID 2>/dev/null || true
	return 1
}

# Stop fake server if running (can be called from cleanup)
stop_fake_server() {
	if [ -n "${FAKE_SERVER_PID:-}" ]; then
		if kill -0 "$FAKE_SERVER_PID" 2>/dev/null; then
			log_info "Stopping fake server (PID: $FAKE_SERVER_PID)"
			kill "$FAKE_SERVER_PID" 2>/dev/null || true
		fi
	fi
}