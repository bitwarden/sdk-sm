# Bitwarden-wasm

Requirements:

- `wasm32-unknown-unknown` rust target.
- `wasm-bindgen-cli` installed.
- `binaryen` installed for `wasm-opt`.

```bash
rustup target add wasm32-unknown-unknown
cargo install -f wasm-bindgen-cli
brew install binaryen
```

#### Build

```bash
# dev
./build.sh

# release
./build.sh -r
```
