# Bitwarden Secrets Manager CLI

A Rust CLI for interacting with the
[Bitwarden Secrets Manager](https://bitwarden.com/products/secrets-manager/). This is a beta release
and might be missing some functionality.

## Install

We offer three ways to install bws:

### Cargo (crates.io)

Download bws via `cargo` from [crates.io](https://crates.io):

```bash
cargo install bws --locked
```

### Install Script (from GitHub Releases)

Linux/macOS: `curl https://bws.bitwarden.com/install | sh`

Windows: `iwr https://bws.bitwarden.com/install | iex`

An optional `-u/--uninstall` flag can be passed to the POSIX script to uninstall the CLI. The
PowerShell version accepts an equivalent `-Uninstall` flag. The uninstallation process will remove
the `bws` binary and the configuration directory (`~/.bws`).

### GitHub Releases (Manual)

Download a pre-built binary from the [Releases](https://github.com/bitwarden/sdk-sm/releases) page.

## Usage

```bash
bws --help
```

## How to enable shell autocompletions

### Zsh

If completion is not enabled already, you need to enable it first:

```zsh
echo "autoload -U compinit; compinit" >> ~/.zshrc
```

Enable autocompletions for the current user:

```zsh
echo 'source <(/path/to/bws completions zsh)' >> ~/.zshrc
```

### Bash

Enable autocompletions for the current user:

```zsh
echo 'source <(/path/to/bws completions bash)' >> ~/.bashrc
```

For more detailed documentation, please refer to the
[Secrets Manager CLI help article](https://bitwarden.com/help/secrets-manager-cli/).

## Docker

We also provide a docker image preloaded with the `bws` cli.

```bash
# From the root of the repository
docker build -f crates/bws/Dockerfile -t bitwarden/bws .

docker run --rm -it bitwarden/bws --help
```

To use a configuration file, utilize docker
[bind mounting](https://docs.docker.com/storage/bind-mounts/) to expose it to the container:

```bash
docker run --rm -it -v "$HOME"/.bws:/home/app/.bws bitwarden/bws --help
```

## How to build manpages

The manpages get built during compilation of the `bws` crate through the use of a build script. The
output path of this build script can be located as follows:

```
MANPAGES_DIR=$(cargo build -p bws --message-format json | jq -r --slurp '.[] | select (.reason == "build-script-executed") | select(.package_id|contains("crates/bws")) .out_dir')
```

After running the provided commands, the built manpages should be located in
`$MANPAGES_DIR/manpages`
