#!/usr/bin/env bash
# shellcheck disable=SC3043,SC3044
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
TMP_DIR="$(mktemp -d)"

# This access token is only used for testing purposes with the fake server
export ORGANIZATION_ID="ec2c1d46-6a4b-4751-a310-af9601317f2d"
export ACCESS_TOKEN="0.${ORGANIZATION_ID}.C2IgxjjLF7qSshsbwe8JGcbM075YXw:X8vbvA0bduihIDe/qrzIQQ=="

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
  cargo build --quiet >/dev/null
}

main() {
  local action="$1"
  local language="$2"

  local dir="$(build_directory "$language")"

  case "$action" in
    all)
      common_setup
      pushd "$dir" >/dev/null || {
        echo "Failed to change directory to $dir"
        exit 1
      }
      . "$dir/setup.sh"
      . "$dir/test.sh"
      popd >/dev/null || {
        echo "Failed to return to previous directory"
        exit 1
      }
      ;;
    setup)
      common_setup

      # Find setup.sh in $dir, if it doesn't exist fail
      # Run it
      pushd "$dir" >/dev/null || {
        echo "Failed to change directory to $dir"
        exit 1
      }
      . "$dir/setup.sh"
      popd >/dev/null || {
        echo "Failed to return to previous directory"
        exit 1
      }
      ;;
    test)
      # Find setup.sh in $dir, if it doesn't exist fail
      # Start running fake_server, set common environment for tests
      # Run it
      pushd "$dir" >/dev/null || {
        echo "Failed to change directory to $dir"
        exit 1
      }
      . "$dir/test.sh"
      popd >/dev/null || {
        echo "Failed to return to previous directory"
        exit 1
      }
      ;;
    *)
      echo "Usage: $0 {setup|test}"
      exit 1
      ;;
  esac
}

main "$@"
