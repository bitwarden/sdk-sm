#!/usr/bin/env bash
# shellcheck disable=SC1090
set -euo pipefail

TMP_DIR="$(mktemp -d)"
PYTHON_VERSIONS="${PYTHON_VERSIONS:-3.13}"

echo "Running Python SDK tests..."

check_requirements() {
  echo "Checking requirements..."

  if ! command -v python3 >/dev/null 2>&1; then
    echo "Error: python3 is required but not installed" >&2
    exit 1
  fi

  if ! command -v uv >/dev/null 2>&1; then
    echo "Error: uv is required but not installed" >&2
    exit 1
  fi

  if [ ! -f "bitwarden_sdk/schemas.py" ]; then
    echo "Error: schemas.py not found. Please run ./setup.sh first"
    exit 1
  fi

  echo "✓ All requirements met"
}

source_venv() {
  # for Windows compatibility...
  set -x
  source "$TMP_DIR/.venv-$python_version/bin/activate" || source "$TMP_DIR/.venv-$python_version/Scripts/activate" || {
    echo "Error: Failed to activate virtual environment for $python_version" >&2
    ls -halR "$TMP_DIR/.venv-$python_version" || true
    exit 1
  }
  set +x

}

# Install Python dependencies
setup_python_environment() {
  python_version=$1
  echo "Setting up Python virtual environment $python_version..."

  # Create virtual environment if it doesn't exist
  if [ ! -d "$TMP_DIR/.venv-$python_version" ]; then
    uv venv "$TMP_DIR/.venv-$python_version" --python "$python_version"
    echo "✓ Created Python virtual environment for $python_version"
  fi

  # Activate virtual environment
  source_venv

  # Upgrade pip and install maturin
  uv pip install --upgrade pip
}

# Build the Python package
build_package() {
  python_version=$1
  echo "Building Python package for $python_version..."

  # Activate virtual environment
  source_venv

  # Build the package in development mode
  if [ "$(uname -s)" = "Linux" ]; then
    # Linux requires patchelf for binary compatibility
    uv pip install .[dev-linux]
  else
    uv pip install .[dev]
  fi
  echo "✓ Built Python package"
}

# Run the CRUD test script
run_crud_test() {
  echo "Running CRUD test for Python $python_version..."

  # Activate virtual environment
  source_venv

  # Run the CRUD test
  python3 test/crud.py
}

# Main test function
main() {
  check_requirements
  for python_version in $PYTHON_VERSIONS; do
    setup_python_environment "$python_version"
    build_package "$python_version"
    run_crud_test "$python_version"
  done
}

main "$@"
