#!/usr/bin/env bash
# shellcheck disable=SC3040,SC3044
set -euo pipefail

TEST_FAILURES=0

run_test() {
  test_name="$1"
  test_command="$2"

  if [ "$BUILD_FROM_SOURCE" -eq 1 ]; then
    ./target/release/bws --version >/dev/null || cargo build --bin bws --quiet --release
    modified_command=$(echo "$test_command" | sed 's/bws/.\/target\/release\/bws/')
  else
    modified_command="$test_command"
  fi

  if eval "$modified_command"; then
    echo "✅ bws $test_name"
  else
    echo "❌ bws $test_name"
    TEST_FAILURES=$((TEST_FAILURES + 1))
  fi
}

secrets() {
  run_test "secret list"   "bws secret list | grep -q 'FERRIS'"
  run_test "secret get"    "bws secret get $(uuidgen) | grep -q 'btw'"
  run_test "secret create" "bws secret create 'secret-key' 'secret-value' --note 'optional note' $(uuidgen) | grep -q 'secret-key'"
  run_test "secret edit"   "bws secret edit --key 'something-new' --value 'new-value' --note 'updated note' $(uuidgen) | grep -q 'something-new'"
  run_test "secret delete" "bws secret delete $(uuidgen) $(uuidgen) $(uuidgen) | grep -q '3 secrets deleted successfully.'"
}

projects() {
  run_test "project list"   "bws project list | grep -q 'Production Environment'"
  run_test "project get"    "bws project get $(uuidgen) | grep -q 'Production Environment'"
  run_test "project create" "bws project create 'project-name' | grep -q 'project-name'"
  run_test "project edit"   "bws project edit --name 'new-project-name' $(uuidgen) | grep -q 'new-project-name'"
  run_test "project delete" "bws project delete $(uuidgen) $(uuidgen) | grep -q '2 projects deleted successfully.'"
}

main() {
  pushd "${REPO_ROOT}" >/dev/null || exit 1
  echo "Testing secrets..."
  secrets
  echo

  echo "Testing projects..."
  projects

  if [ "$TEST_FAILURES" -gt 0 ]; then
    echo
    echo "❌ $TEST_FAILURES test(s) failed"
    exit 1
  else
    echo
    echo "✅ All tests passed"
    exit 0
  fi
}

cleanup() {
  # Only popd if we have something on the directory stack
  # shellcheck disable=SC2317
  if dirs -v | grep -q "1"; then
    popd >/dev/null 2>&1 || true
  fi
}

trap cleanup EXIT
main "$@"
