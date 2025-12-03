# PHP Installation

## Introduction

Composer is used to build the PHP Bitwarden client library.

## Prerequisites

- PHP >= 8.0
- FFI extension enabled in PHP configuration
- Composer
- Bitwarden SDK native library.
  - Expected in one of below locations, depending on the OS and architecture. The `src` is relative
    path to the [src](./src) directory.
    - Windows x86_64: `src\lib\windows-x64\bitwarden_c.dll`
    - Linux x86_64: `src/lib/linux-x64/libbitwarden_c.so`
    - macOS x86_64: `src/lib/macos-x64/libbitwarden_c.dylib`
    - macOS aarch64: `src/lib/macos-arm64/libbitwarden_c.dylib`
  - If you prefer to build the SDK yourself, see the [SDK README.md](../../README.md) for
    instructions.

## Build Commands

```shell
composer install
```

## Example

### macOS

#### Install Prerequisites

Use brew Composer and PHP

```shell
brew install php
brew install composer
```

#### Build Commands

```shell
composer install
```

## Example SDK Usage Project

```shell
export ACCESS_TOKEN="<access_token>"
export STATE_FILE="<state_file>"
export ORGANIZATION_ID="<organization_id>"
export API_URL="https://api.bitwarden.com"
export IDENTITY_URL="https://identity.bitwarden.com"

php example.php
```
