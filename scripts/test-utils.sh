#!/usr/bin/env bash
# Central test utilities loader for SDK test scripts
# This file sources all the modular utility scripts

# Get the directory of this script
UTILS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source all utility modules in the correct order
# 1. Logging must be first as others depend on it
source "$UTILS_DIR/logging-utils.sh"

# 2. Environment utilities
source "$UTILS_DIR/env-utils.sh"

# 3. Authentication utilities
source "$UTILS_DIR/auth-utils.sh"

# 4. Fake server utilities
source "$UTILS_DIR/fake-server-utils.sh"

# 5. Build utilities
source "$UTILS_DIR/build-utils.sh"

# Export a flag to indicate all utilities are loaded
export TEST_UTILS_LOADED=true