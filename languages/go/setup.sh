#!/usr/bin/env bash
set -euo pipefail

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
GO_ARCH="$(uname -m | sed 's/x86_64/x64/' | sed 's/aarch64/arm64/')"

mkdir -p "$REPO_ROOT"/languages/go/internal/cinterface/lib/{darwin,linux,windows}-{arm64,x64}

if [ ! -f ./target/debug/libbitwarden_c.a ]; then
  echo "Building libbitwarden_c.a..."
  cargo build --quiet -p bitwarden-c
fi

ln -f "$REPO_ROOT/target/debug/libbitwarden_c.a" "$REPO_ROOT/languages/go/internal/cinterface/lib/$OS-$GO_ARCH/libbitwarden_c.a" || {
  echo "Failed to link libbitwarden_c.a"
  exit 1
}
