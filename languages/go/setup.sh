#!/usr/bin/env bash
set -euo pipefail

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
GO_ARCH="$(uname -m | sed 's/x86_64/x64/' | sed 's/aarch64/arm64/')"

mkdir -p "$REPO_ROOT"/languages/go/internal/cinterface/lib/{darwin,linux,windows}-{arm64,x64}

if [ ! -f ./target/debug/libbitwarden_c.a ]; then
  echo "Building bitwarden_c..."
  cargo build --quiet -p bitwarden-c
fi

# windows can be either mingw, msys, or cygwin
if [[ "$OS" = *"mingw"* ]] || [[ "$OS" = *"msys"* ]] || [[ "$OS" = *"cygwin"* ]]; then
  OS="windows" # normalize to windows
  ln -f "$REPO_ROOT/target/debug/bitwarden_c.dll" "$REPO_ROOT/languages/go/internal/cinterface/lib/$OS-$GO_ARCH/bitwarden_c.dll" || {
    echo "Failed to symlink bitwarden_c.dll"
    exit 1
  }
else
  ln -f "$REPO_ROOT/target/debug/libbitwarden_c.a" "$REPO_ROOT/languages/go/internal/cinterface/lib/$OS-$GO_ARCH/libbitwarden_c.a" || {
    echo "Failed to symlink libbitwarden_c.a"
    exit 1
  }
fi
