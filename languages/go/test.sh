#!/usr/bin/env bash
set -euo pipefail

pushd "$REPO_ROOT"/languages/go > /dev/null || exit 1

go test || exit 1

popd > /dev/null || exit 1
