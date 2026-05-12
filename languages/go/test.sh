#!/usr/bin/env bash
set -euo pipefail

if command -v go >/dev/null; then
  go test || exit 1
else
  echo "Go is not installed. Please install Go to run the tests." >&2
  exit 1
fi

