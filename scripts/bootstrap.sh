#!/usr/bin/env bash
# shellcheck disable=SC1091,SC2155
set -euo pipefail

export REPO_ROOT="$(git rev-parse --show-toplevel)"
TMP_DIR="$(mktemp -d)"

# This access token is only used for testing purposes with the fake server
export ORGANIZATION_ID="f4e44a7f-1190-432a-9d4a-af96013127cb" # this must match the JWT returned by fake-server::routes.rs
export ACCESS_TOKEN="0.ec2c1d46-6a4b-4751-a310-af9601317f2d.C2IgxjjLF7qSshsbwe8JGcbM075YXw:X8vbvA0bduihIDe/qrzIQQ=="

export SERVER_URL="http://localhost:${SM_FAKE_SERVER_PORT:-3000}"
export API_URL="${SERVER_URL}/api"
export IDENTITY_URL="${SERVER_URL}/identity"
export STATE_FILE="${TMP_DIR}/state"

# input: bws, or any of the lanaguages in ./languages
# output: a build directory
build_directory() {
  local language="$1"

  if [ "$language" = "bws" ]; then
    echo "$REPO_ROOT/crates/bws"
  else
    echo "$REPO_ROOT/languages/$language"
  fi
}

common_setup() {
  npm install >/dev/null
  npm run schemas >/dev/null
  cargo build --quiet --release >/dev/null
}

# Start fake server in background
start_fake_server() {
  local port="${SM_FAKE_SERVER_PORT:-3000}"

  # Check if server is already running
  if curl -s "http://localhost:$port/health" >/dev/null 2>&1; then
    echo "✓ Fake server already running on port $port"
    return 0
  fi

  echo "Starting fake server on port $port..."
  cargo build -p fake-server >/dev/null
  SM_FAKE_SERVER_PORT="$port" cargo run -p fake-server >/dev/null 2>&1 &
  FAKE_SERVER_PID=$!

  # Wait for server to be ready
  local max_attempts=30
  local attempt=0
  while [ $attempt -lt $max_attempts ]; do
    if curl -s "http://localhost:$port/health" >/dev/null 2>&1; then
      echo "✓ Fake server is ready"
      return 0
    fi
    sleep 1
    attempt=$((attempt + 1))
  done

  echo "Error: Fake server failed to start within 30 seconds"
  kill $FAKE_SERVER_PID 2>/dev/null || true
  exit 1
}

# Stop fake server
stop_fake_server() {
  if [ -n "${FAKE_SERVER_PID:-}" ]; then
    echo "Stopping fake server..."
    kill "$FAKE_SERVER_PID" 2>/dev/null || true
    wait "$FAKE_SERVER_PID" 2>/dev/null || true
  fi
}

# Cleanup function
cleanup() {
  stop_fake_server
}

main() {
  local action="$1"
  local language="$2"
  local dir

  dir="$(build_directory "$language")"

  # Set up cleanup trap
  trap cleanup EXIT

  case "$action" in
  all)
    common_setup
    pushd "$dir" >/dev/null || {
      echo "Failed to change directory to $dir"
      exit 1
    }
    source ./setup.sh
    start_fake_server
    ./test.sh
    popd >/dev/null || {
      echo "Failed to return to previous directory"
      exit 1
    }
    ;;
  setup)
    common_setup

    # Check if setup.sh exists in $dir
    if [ ! -f "$dir/setup.sh" ]; then
      echo "Error: setup.sh not found in $dir"
      exit 1
    fi

    pushd "$dir" >/dev/null || {
      echo "Failed to change directory to $dir"
      exit 1
    }
    source ./setup.sh
    popd >/dev/null || {
      echo "Failed to return to previous directory"
      exit 1
    }
    ;;
  test)
    if [ ! -f "$dir/test.sh" ]; then
      echo "Error: test.sh not found in $dir"
      exit 1
    fi

    pushd "$dir" >/dev/null || {
      echo "Failed to change directory to $dir"
      exit 1
    }
    start_fake_server
    ./test.sh
    popd >/dev/null || {
      echo "Failed to return to previous directory"
      exit 1
    }
    ;;
  *)
    echo "Usage: $0 {all|setup|test} <language>"
    echo "Available languages: bws, python, csharp, java, js, etc."
    exit 1
    ;;
  esac
}

main "$@"
