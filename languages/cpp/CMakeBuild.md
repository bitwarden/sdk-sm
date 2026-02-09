# CMake Build

## Introduction

Cmake is used to build the C++ Bitwarden client library. Output should be placed in the build
directory. The output contains two dynamic libraries:

- The C++ client `BitwardenClient`
- The Bitwarden library used by the C++ client `bitwarden_c`.

See how to use these libraries in the [example use guide](./examples/ExampleUse.md)

## Prerequisites

- Cmake installed, minimum version 3.15
- `schemas.hpp` generated into `include` directory
- installed `nlohmann-json` library
- installed `boost` library

## Build Commands

One should be in the root directory of the C++ wrapper (the same level where is CMakeLists.txt
placed). Paths of the three libraries should be placed inside the cmake build command:

```bash
mkdir -p build
cd build
cmake .. -DNLOHMANN=/path/to/include/nlohmann -DBOOST=/path/to/include/boost -DTARGET=relative/path/to/libbitwarden_c
cmake --build .
```

## IDE Support

You may need to manually set the CMake `TARGET` variable for your IDE. For CLion, add the following
to the CMake options settings:

```bash
# macOS example
-DTARGET=../../target/release/libbitwarden_c.dylib
```

## Example

### macOS

#### Install Prerequisites

```bash
brew install cmake
brew install boost
brew install nlohmann-json
```

#### Build

```bash
mkdir -p build
cd build
cmake .. -DNLOHMANN=/opt/homebrew/include -DBOOST=/opt/homebrew/include -DTARGET=../../target/release/libbitwarden_c.dylib
cmake --build .
```
