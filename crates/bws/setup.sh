#!/usr/bin/env bash
# shellcheck disable=SC3040,SC3044
set -euo pipefail

if [ "${BWS_FORCE_BUILD_FROM_SOURCE:-0}" -eq 1 ]; then
  echo "BWS_FORCE_BUILD_FROM_SOURCE set. Building from source..."
  BUILD_FROM_SOURCE=1
elif ! command -v bws >/dev/null; then
  echo "bws is not installed. Building from source..."
  BUILD_FROM_SOURCE=1
else
  echo "bws is installed. Using the installed version."
  BUILD_FROM_SOURCE=0
fi

export BUILD_FROM_SOURCE
