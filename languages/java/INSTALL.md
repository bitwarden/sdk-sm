# Java build

## Introduction

Gradle is used to build Java Bitwarden client library.

The output of the build is placed in `build/libs` directory and should contain `BitwardenSDK.jar` file.

## Prerequisites

- JDK 17 installed.
- Bitwarden SDK native library build. See [SDK README.md](../../README.md) for instructions.

## Build Commands

```shell
./gradlew build
```

## Example

### macOS

#### Install Prerequisites

Use brew to install JDK 17.

```shell
brew install --cask temurin@17
brew install jenv
export PATH="$HOME/.jenv/bin:$PATH"
eval "$(jenv init -)"
jenv add /Library/Java/JavaVirtualMachines/temurin-17.jdk/Contents/Home
jenv shell 17
```

#### Build Commands

```shell
./gradlew build
```

### Windows

Download JDK17

Go to System Environment Variables and point JAVA_HOME to where the JDK software is located, for example: JAVA_HOME=C:\Program Files\Java\jdk1.6.0_02.

## Example SDK Usage Project

```shell
export ACCESS_TOKEN="<access_token>"
export ORGANIZATION_ID="<organization_id>"
export API_URL="https://api.bitwarden.com"
export IDENTITY_URL="https://identity.bitwarden.com"

./gradlew :example:run
```
