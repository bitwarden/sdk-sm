# Instructions

This guide is for developers who want to use the Bitwarden Go SDK module in their own Go projects.
Please see the main [README](./README.md) and [example.go](./example/example.go) file for examples.

## Supported Targets

The Bitwarden Go SDK module utilizes FFI calls to the Bitwarden Rust SDK via
[cgo](https://pkg.go.dev/cmd/cgo). The module supports the following statically linked targets:

- Linux `x86-64` & `arm64`
- macOS `x86-64` & `arm64`
- Windows `x86-64`

## Linux

### Prerequisites

- [Go](https://go.dev/dl)
- A C toolchain

We recommend the [MUSL toolchain](https://musl.libc.org). You can install this on most debian based
systems with:

```shell
sudo apt install musl-tools
```

### Set Go Environment Info

#### Enable cgo

```shell
go env -w CGO_ENABLED=1
```

#### Set the C compiler

```shell
go env -w CC=musl-gcc
```

#### Verify

```shell
go env
```

### Install the Bitwarden Go SDK

#### Adding the Module

```shell
go get github.com/bitwarden/sdk-go/v2
```

#### Build

```shell
go build -ldflags '-linkmode external -extldflags "-static -Wl,-unresolved-symbols=ignore-all"'
```

## macOS

### Prerequisites

- [Go](https://go.dev/dl)
- A C toolchain

[Clang](https://clang.llvm.org/get_started.html) is the default C and C++ toolchain on Mac OS. The
easiest way to ensure you have the toolchain is to install the Xcode Command Line tools.

You can install Clang with:

```shell
xcode-select --install
```

### Set Go Environment Info

#### Enable cgo

```shell
go env -w CGO_ENABLED=1
```

#### Set the C & C++ compilers

```shell
go env -w CC=clang CXX=clang++
```

#### Verify

```shell
go env
```

### Install the Bitwarden Go SDK

#### Adding the Module

```shell
go get github.com/bitwarden/sdk-go/v2
```

#### Build

```shell
go build
```

## Windows

### Prerequisites

- [Go](https://go.dev/dl)
- [GCC](https://gcc.gnu.org)

Go [documentation](https://go.dev/wiki/cgo) recommends the mingw-w64 gcc compiler.

We recommend following the Visual Studio Code
[guide](https://code.visualstudio.com/docs/cpp/config-mingw#_installing-the-mingww64-toolchain) for
installing the mingw-w64 toolchain.

### Set Go Environment Info

#### Enable cgo

```shell
go env -w CGO_ENABLED=1
```

#### Set the C & C++ compilers

```shell
go env -w CC=gcc CXX=g++
```

#### Verify

```shell
go env
```

### Install the Bitwarden Go SDK

#### Adding the Module

```shell
go get github.com/bitwarden/sdk-go/v2
```

#### Build

```shell
go build
```
