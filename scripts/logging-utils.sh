#!/usr/bin/env bash
# Logging utilities for SDK test scripts
# Provides colored output and JSON-aware logging functions

# Colors for output (disabled if JSON output)
export RED='\033[0;31m'
export GREEN='\033[0;32m'
export YELLOW='\033[1;33m'
export BLUE='\033[0;34m'
export NC='\033[0m' # No Color

# Logging functions
log_info() {
	if [ "${JSON_OUTPUT:-false}" != true ]; then
		echo -e "${BLUE}[INFO]${NC} $1"
	fi
}

log_success() {
	if [ "${JSON_OUTPUT:-false}" != true ]; then
		echo -e "${GREEN}[✓]${NC} $1"
	fi
}

log_error() {
	if [ "${JSON_OUTPUT:-false}" != true ]; then
		echo -e "${RED}[✗]${NC} $1" >&2
	fi
}

log_warning() {
	if [ "${JSON_OUTPUT:-false}" != true ]; then
		echo -e "${YELLOW}[!]${NC} $1"
	fi
}

# Disable colors if JSON output is enabled
disable_colors_if_json() {
	if [ "${JSON_OUTPUT:-false}" = true ]; then
		RED=""
		GREEN=""
		YELLOW=""
		BLUE=""
		NC=""
	fi
}