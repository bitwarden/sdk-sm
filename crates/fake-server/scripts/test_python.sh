#!/usr/bin/env bash
# shellcheck disable=SC3040,SC3044

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
PYTHON_VERSION="${PYTHON_VERSION:-3.9}" # default to oldest supported version
TMP_DIR="$(mktemp -d)"

# This access token is only used for testing purposes with the fake server
export ORGANIZATION_ID="ec2c1d46-6a4b-4751-a310-af9601317f2d"
export ACCESS_TOKEN="0.${ORGANIZATION_ID}.C2IgxjjLF7qSshsbwe8JGcbM075YXw:X8vbvA0bduihIDe/qrzIQQ=="

export SERVER_URL="http://localhost:${SM_FAKE_SERVER_PORT:-3000}"
export API_URL="${SERVER_URL}/api"
export IDENTITY_URL="${SERVER_URL}/identity"
export STATE_FILE="${TMP_DIR}/state"

_pip() {
  # After venv activation, always use python -m pip to ensure we're in the venv
  if [ -n "${VIRTUAL_ENV:-}" ]; then
    python -m pip "$@"
  elif [ -x "${VIRTUAL_ENV}/bin/pip" ]; then
    "${VIRTUAL_ENV}/bin/pip" "$@"
  else
    echo "pip or pip3 not found. Please install Python package manager." >/dev/stderr
    exit 1
  fi
}

setup() {
  # This function sets up the Python environment and installs bitwarden_sdk.
  # In GitHub Actions, it will use pre-built wheels if available in target/wheels.
  # For local development, it falls back to building from source with maturin.

  if ! command -v uv >/dev/null; then
    _pip install uv
    uv --version || { echo "Failed to install uv"; exit 1; }
  fi

  uv venv --python "${PYTHON_VERSION}" "${TMP_DIR}/venv" || {
    echo "Failed to create virtual environment with Python ${PYTHON_VERSION}" >/dev/stderr
    exit 1
  }

  echo "Activating virtual environment..."
  # shellcheck disable=SC1091
  . "${TMP_DIR}/venv/bin/activate"

  # Verify we're in the virtual environment
  if [ -z "${VIRTUAL_ENV:-}" ]; then
    echo "ERROR: Virtual environment activation failed" >/dev/stderr
    exit 1
  fi

  echo "Virtual environment activated successfully at: ${VIRTUAL_ENV}"
  echo "Using Python: $(which python)"

  # Install pip in the virtual environment if it's not available
  if ! command -v pip >/dev/null; then
    echo "Installing pip in virtual environment..."
    python -m ensurepip --upgrade
  fi

  echo "Using pip: $(which pip)"

  if ! command -v maturin >/dev/null; then
    echo "Installing maturin..."
    if [ "$(uname -s)" = "Linux" ]; then
      # Linux requires patchelf for binary compatibility
      _pip install maturin[patchelf]
    else
      _pip install maturin
    fi
  fi

  pushd "${REPO_ROOT}/languages/python" >/dev/null || exit 1

  if ! python -c 'from bitwarden_sdk import BitwardenClient, DeviceType, client_settings_from_dict' >/dev/null 2>&1; then
    echo "Installing bitwarden_sdk..."

    # Check if we're running in GitHub Actions and pre-built wheels are available
    if [ -n "${GITHUB_ACTIONS:-}" ] && [ -d "${REPO_ROOT}/target/wheels" ]; then
      echo "Running in GitHub Actions - looking for pre-built wheels..."

      # Detect current platform for wheel selection
      PLATFORM=""
      case "$(uname -s)" in
        Darwin*)
          if [ "$(uname -m)" = "arm64" ]; then
            PLATFORM="macosx.*arm64"
          else
            PLATFORM="macosx.*x86_64"
          fi
          ;;
        Linux*)
          if [ "$(uname -m)" = "aarch64" ]; then
            PLATFORM="linux_aarch64"
          else
            PLATFORM="linux_x86_64"
          fi
          ;;
        MINGW*|MSYS*|CYGWIN*)
          PLATFORM="win_amd64"
          ;;
      esac

      # Find the appropriate wheel file for the current platform and Python version
      PYTHON_VERSION_SHORT=$(python -c "import sys; print(f'{sys.version_info.major}{sys.version_info.minor}')")

      if [ -n "${PLATFORM}" ]; then
        # Try to find platform-specific wheel first
        WHEEL_FILE=$(find "${REPO_ROOT}/target/wheels" -name "bitwarden_sdk-*-cp${PYTHON_VERSION_SHORT}-*" | grep -E "${PLATFORM}" | head -1)

        # If no platform-specific wheel found, try any wheel
        if [ -z "${WHEEL_FILE}" ]; then
          WHEEL_FILE=$(find "${REPO_ROOT}/target/wheels" -name "bitwarden_sdk-*.whl" | head -1)
        fi
      else
        WHEEL_FILE=$(find "${REPO_ROOT}/target/wheels" -name "bitwarden_sdk-*.whl" | head -1)
      fi

      if [ -n "${WHEEL_FILE}" ] && [ -f "${WHEEL_FILE}" ]; then
        echo "Found pre-built wheel: ${WHEEL_FILE}"
        echo "Installing from wheel..."
        _pip install "${WHEEL_FILE}"
      else
        echo "No compatible pre-built wheel found, falling back to maturin develop..."
        maturin develop
      fi
    else
      echo "Local development or no pre-built wheels available, using maturin develop..."
      maturin develop
    fi
  fi

  popd >/dev/null
}

main() {
  pushd "${REPO_ROOT}" >/dev/null || exit 1
  setup

  echo
  echo "Running Python tests..."
  echo

  python "${REPO_ROOT}/languages/python/test/crud.py"
}

cleanup() {
  rm -rf "${TMP_DIR}"
  # Only popd if we have something on the directory stack
  if dirs -v | grep -q "1"; then
    popd >/dev/null 2>&1 || true
  fi
}

trap cleanup EXIT
main "$@"
