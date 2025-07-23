# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **BREAKING:** Updated MSRV to `1.82.0` (#1267)

## [1.0.0] - 2024-09-26

### Added

- Support for secrets sync (#678)
- Password generator (#986)

### Changed

- `ClientSettings` and `DeviceType` is now exported in the root module (#805)
- Secrets Manager now requires `bitwarden::secrets_manager::ClientSecretsExt` and
  `bitwarden::secrets_manager::ClientProjectsExt` to be imported in order to access `secrets()` and
  `projects` on the client (#798)
- Updated MSRV `1.75.0` (#980)

### Removed

- The deprecated `client.access_token_login()` is now removed. Please use
  `client.auth().login_access_token()` instead. (#806)

## [0.5.0] - 2024-04-26

### Changed

- Switched TLS backend to `rustls`, removing the dependency on `OpenSSL`. (#374)
- `client::AccessToken` is now `auth::AccessToken`. (#656)

### Fixed

- Fix renew for service account access token logins (#702)

## [0.4.0] - 2023-12-21

### Added

- Support for basic state to avoid reauthenticating when creating a new `Client`. This is a breaking
  change because of adding `state_file` to the `AccessTokenLoginRequest` struct. (#388)

### Deprecated

- `client.access_token_login()` is now deprecated and will be removed in a future release. Please
  use `client.auth().login_access_token()` instead. (#319)

## [0.3.1] - 2023-10-13

### Changed

- `auth::request::AccessTokenLoginRequest` moved to `auth::login::AccessTokenLoginRequest` (#178)
- Support for fetching multiple secrets by ids (#150)

## [0.3.0] - 2023-07-26

### Deprecated

- The secrets manager SDK is now hidden behind a `secrets` feature flag. Make sure to enable this
  flag in your `Cargo.toml` file. At the moment the flag is enabled by default for compatibility
  reasons, but this is considered deprecated and the flag will be made opt-in eventually.

### Added

- Support for creating and editing secrets (#77)
- Support for creating and editing projects (#53)

### Changed

- Folder structure, update `use` declarations (#68)

### Fixed

- Improve login error handling (#109)

## [0.2.1] - 2023-03-22

### Fixed

- Add user agent to login requests (#11)
