# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **BREAKING:** Updated MSRV to `1.88.0` (#1426)
- Errors are now printed as a short `Error:` line with an optional `Hint:` line instead of a full
  error report. Set `BWS_DEBUG=1` (or `RUST_BACKTRACE=1`) to see the full report. (#1578)
- Network and server errors no longer print raw response bodies. (#1578)
- Clearer messages for missing or invalid access tokens, config file problems, and login failures.
  (#1579)
- The state directory warning is now a single line with a hint. (#1579)
- **BREAKING:** An access token that isn't associated with an organization now exits with code 1
  instead of 0. (#1579)
- Clearer messages for secret, project, `run`, `config` and `completions` errors, including
  not-found and validation errors. (#1580)
- `bws config` now rejects server URLs that don't start with `http://` or `https://`. (#1580)

### Fixed

- No longer panic if access token has access to no secrets (#1255)
- No longer panic when writing to stdout fails, e.g. when piping into `head` or when the disk is
  full. (#1578)
- `bws config <name>` without a value now reports the missing value instead of a missing name.
  (#1580)
- `bws run` with no command on empty stdin now fails instead of silently succeeding. (#1580)
- `bws run` no longer fails when there are no secrets to inject. (#1580)

## [1.0.0] - 2024-09-26

### Added

- The ability to edit unassigned secrets with direct permissions. (#906)
- The `run` command, to run commands with secrets (#621)

### Changed

- Updated MSRV `1.75.0` (#980)
- Use state files by default. You can opt out of this behavior with the new `state_opt_out` key.
  (#930)
  - Opt out documentation can be found
    [here](https://bitwarden.com/help/secrets-manager-cli/#config-state)

### Removed

- The deprecated `action type` commands are now removed. Please use `type action` instead. (#836)

## [0.5.0] - 2024-04-26

### Added

- Add a `BWS_CONFIG_FILE` environment variable to specify the location of the config file (#571)
- The `bws` CLI is now available as a Docker image (`docker run -it bitwarden/bws --help`) (#305)
- The `bws` CLI releases are now code signed on Windows and Mac (#534, #535)

### Fixed

- Re-add output options to the help menu after they were accidentally removed (#477)

### Changed

- Switched TLS backend to `rusttls`, removing the dependency on `OpenSSL` (#374)
- Updated MSRV for `bws` to `1.71.0` (#589)

## [0.4.0] - 2023-12-21

### Added

- Ability to output secrets in an `env` format with `bws` (#320)
- Basic state to avoid reauthenticating every run, used when setting the `state_file_dir` key in the
  config (#388)

## [0.3.1] - 2023-10-13

### Added

- Support for shell autocompletion with the `bws completions` command (#103)
- When running `bws` with no args, the help text is now printed to `stderr` instead of `stdout` to
  be consistent with `bws subcommand` behavior (#190)

## [0.3.0] - 2023-07-26

### Deprecated

- Switched command order from `action type` to `type action`, please re-read the help documentation
  (#76)

### Added

- Ability to create and edit projects (#53)
- Ability to create and edit secrets (#77)
- Support `NO_COLOR` environment variable to disable CLI colors (#61)
- Support for `CLICOLOR_FORCE` (#74)

### Fixed

- Improve login error handling (#109)
- Respect users color choice for errors (#61)

## [0.2.1] - 2023-03-22

### Fixed

- Add user agent to login requests (#11)
