#!/bin/sh

##################################################
# An installer for the bws command line utility. #
##################################################

DEFAULT_BWS_VERSION="1.0.0"
BWS_VERSION="${BWS_VERSION:-$DEFAULT_BWS_VERSION}"

main() {
  case "$1" in
  -u | --uninstall)
    uninstall_bws
    ;;
  *)
    check_required
    platform_detect
    arch_detect
    download_bws
    validate_checksum
    install_bws
    ;;
  esac
}

error() {
  echo "$1" >&2
  echo "Exiting..." >&2
  exit 1
}

check_required() {
  if ! command -v curl >/dev/null && ! command -v wget >/dev/null; then
    error "curl or wget is required to download bws."
  fi

  if ! command -v unzip >/dev/null; then
    error "unzip is required to install bws."
  fi
}

can_sudo() {
  if command -v sudo >/dev/null; then
    echo "Attempting to install bws with sudo. Please enter your password if prompted."
    if sudo -v 2>/dev/null; then
      echo "sudo is available and we have the necessary permissions."
      echo "Installing bws to /usr/local/bin..."
      return 0
    else
      echo "sudo is available, but we failed to authenticate."
      return 1
    fi
  else
    echo "sudo is not available."
    return 1
  fi
}

platform_detect() {
  if [ "$(uname -s)" = "Linux" ]; then
    PLATFORM="unknown-linux-gnu"
  elif [ "$(uname -s)" = "Darwin" ]; then
    PLATFORM="apple-darwin"
  else
    error "Unsupported platform: $(uname -s)"
  fi
}

arch_detect() {
  if [ "$(uname -m)" = "x86_64" ]; then
    ARCH="x86_64"
  elif [ "$(uname -m)" = "aarch64" ]; then # Linux uname output
    ARCH="aarch64"
  elif [ "$(uname -m)" = "arm64" ]; then # Darwin uname output
    ARCH="aarch64"
  else
    error "Unsupported architecture: $(uname -m)"
  fi
}

checksum() {
  if command -v sha256sum >/dev/null; then
    sha256sum "$1"
  else
    shasum -a 256 "$1"
  fi
}

downloader() {
  if command -v curl >/dev/null; then
    curl -L -o "$2" "$1"
  else
    wget -O "$2" "$1"
  fi
}

extract() {
  unzip -o "$1" -d "$2"
}

download_bws() {
  bws_url="https://github.com/bitwarden/sdk-sm/releases/download/bws-v${BWS_VERSION}/bws-${ARCH}-${PLATFORM}-${BWS_VERSION}.zip"
  echo "Downloading bws from: $bws_url"
  tmp_dir="$(mktemp -d)"
  downloader "$bws_url" "$tmp_dir/bws.zip"
}

validate_checksum() {
  checksum_url="https://github.com/bitwarden/sdk-sm/releases/download/bws-v${BWS_VERSION}/bws-sha256-checksums-${BWS_VERSION}.txt"
  echo "Downloading checksum file from: $checksum_url"
  checksum_file="$tmp_dir/bws-checksums.txt"
  downloader "$checksum_url" "$checksum_file"

  expected_checksum="$(grep "bws-${ARCH}-${PLATFORM}-${BWS_VERSION}.zip" "$checksum_file" | awk '{print $1}')"
  actual_checksum="$(checksum "$tmp_dir/bws.zip" | awk '{print $1}')"

  if [ "$actual_checksum" != "$expected_checksum" ]; then
    error "Checksum validation failed. Expected: $expected_checksum, Actual: $actual_checksum"
  else
    echo "Checksum validation successful."
  fi
}

install_bws() {
  echo "Installing bws..."
  extract "$tmp_dir/bws.zip" "$tmp_dir"
  chmod +x "$tmp_dir/bws"

  if can_sudo; then
    sudo install -m 755 "$tmp_dir/bws" /usr/local/bin/bws

    if ! command -v bws >/dev/null; then
      error "Installation failed. bws was not found in /usr/local/bin"
    fi

    echo "bws installed to /usr/local/bin/bws"
  else
    echo "Installing to your \$HOME directory..."
    user_bin_dir="${HOME}/.local/bin"
    mkdir -p "${user_bin_dir}"
    install -m 755 "$tmp_dir/bws" "${user_bin_dir}/bws"

    if ! command -v "${user_bin_dir}/bws" >/dev/null; then
      error "Installation failed. bws was not found in ${user_bin_dir}"
    fi

    echo "bws installed at ${user_bin_dir}/bws"
    echo "Please add ${user_bin_dir} to your PATH by adding the following line to your ~/.profile or shell rc file:"
    echo "export PATH=\"\$PATH:${user_bin_dir}\""
  fi

  rm -rf "$tmp_dir"
}

uninstall_bws() {
  if command -v bws >/dev/null; then
    echo "Uninstalling bws..."
    if can_sudo; then
      sudo rm "$(command -v bws)"
    else
      rm "$(command -v bws)"
    fi

    # Safely remove the configuration directory
    if [ -n "$HOME" ]; then
      echo "Removing bws configuration directory at ${HOME}/.bws"
      echo "If you use another directory for your configuration, you may want to remove it manually."
      rm -rf "${HOME}/.bws"
    else
      echo "HOME environment variable is not set. Cannot safely remove .bws directory."
    fi

    echo "bws uninstalled successfully."
  else
    echo "bws is not installed."
  fi
  exit 0
}

main "$@"
