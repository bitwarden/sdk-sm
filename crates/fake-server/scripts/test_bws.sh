#!/usr/bin/env bash
# shellcheck disable=SC3040
set -euo pipefail

# This access token is only used for testing purposes with the fake server
export BWS_ACCESS_TOKEN="0.ec2c1d46-6a4b-4751-a310-af9601317f2d.C2IgxjjLF7qSshsbwe8JGcbM075YXw:X8vbvA0bduihIDe/qrzIQQ=="
export BWS_SERVER_URL="http://localhost:${SM_FAKE_SERVER_PORT:-3000}"

secrets() {
  { bws secret list | grep -q 'FERRIS'; } \
    && echo "✅ bws secret list" || echo "❌ bws secret list"

  { bws secret get "$(uuidgen)" | grep -q 'btw'; } \
    && echo "✅ bws secret get" || echo "❌ bws secret get"

  { bws secret create 'secret-key' 'secret-value' --note 'optional note' "$(uuidgen)" | grep -q 'secret-key'; } \
    && echo "✅ bws secret create" || echo "❌ bws secret create"

  { bws secret edit --key 'something-new' --value 'new-value' --note 'updated note' "$(uuidgen)" | grep -q 'something-new'; } \
    && echo "✅ bws secret edit" || echo "❌ bws secret edit"

  { bws secret delete "$(uuidgen)" "$(uuidgen)" "$(uuidgen)" | grep -q '3 secrets deleted successfully.'; } \
    && echo "✅ bws secret delete" || echo "❌ bws secret delete"
}

projects() {
  { bws project list | grep -q 'Production Environment'; } \
    && echo "✅ bws project list"

  { bws project get "$(uuidgen)" | grep -q 'Production Environment'; } \
    && echo "✅ bws project get"

  { bws project create 'project-name' | grep -q 'project-name'; } \
    && echo "✅ bws project create"

  { bws project edit --name 'new-project-name' "$(uuidgen)" | grep -q 'new-project-name'; } \
    && echo "✅ bws project edit"

  { bws project delete "$(uuidgen)" "$(uuidgen)" | grep -q '2 projects deleted successfully.'; } \
    && echo "✅ bws project delete"
}

main() {
  echo "Testing secrets..."

  secrets
  echo

  echo "Testing projects..."
  projects
}

main "$@"
