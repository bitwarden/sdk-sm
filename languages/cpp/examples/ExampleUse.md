# Examples

## Prerequisites

### Bitwarden Libraries

Have the two Bitwarden libraries at the same path:

- `BitwardenClient`
- `bitwarden_c`

For each OS the library files will be the following:

- macOS: `libBitwardenClient.dylib` and `libbitwarden_c.dylib`
- Linux: `libBitwardenClient.so` and `libbitwarden_c.so`
- Windows: `BitwardenClient.dll` and `bitwarden_c.dll`

Follow the [cmake build guide](../CMakeBuild.md) to create the libraries locally.

### Include Directory

`include` directory contains:

- `BitwardenLibrary.h`
- `BitwardenClient.h`
- `BitwardenSettings.h`
- `CommandRunner.h`
- `Projects.h`
- `Secrets.h`
- `schemas.hpp`

### Other Libraries

- `nlohmann-json` (<https://github.com/nlohmann/json>)
- `boost` (<https://www.boost.org/>)

### Compiling

Use g++/clang++ for compiling.

Example of the folder structure (macOS):

```text
--root
    --build
        `libBitwardenClient.dylib`
        `libbitwarden_c.dylib`
    --include
        --`BitwardenLibrary.h`
        --`BitwardenClient.h`
        --`BitwardenSettings.h`
        --`CommandRunner.h`
        --`Projects.h`
        --`Secrets.h`
        --`schemas.hpp`
    --examples
        --`Wrapper.cpp`
```

Set the environment variable path for the Bitwarden libraries.

For macOS:

```bash
export DYLD_LIBRARY_PATH=/path/to/your/library:$DYLD_LIBRARY_PATH
```

For Linux:

```bash
export LD_LIBRARY_PATH=/path/to/your/library:$LD_LIBRARY_PATH
```

For Windows:

```shell
 set "PATH=%PATH%;C:\path\to\your\library"
```

Set environment variables used in `Wrapper.cpp`:

```bash
export ACCESS_TOKEN=<"access-token">
export ORGANIZATION_ID=<"organization-id">
export API_URL=http://localhost:4000
export IDENTITY_URL=http://localhost:33656
```

Compile:

```bash
cd examples
clang++ -std=c++20 -I../include -I/path/to/include/nlohmann -I/path/to/include/boost -L../build/ -o MyBitwardenApp Wrapper.cpp -lBitwardenClient -ldl
```

for Windows `-ldl` should be excluded,

for macOS nlohmann and boost libraries installed with homebrew the following can be used:

```bash
-I/opt/homebrew/include
```

The result is `MyBitwardenApp` in the `examples` directory, and can be ran from the `examples`
directory:

```bash
./MyBitwardenApp
```
