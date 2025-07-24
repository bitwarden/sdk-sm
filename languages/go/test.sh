#!/usr/bin/env bash

pushd "$REPO_ROOT"/languages/go > /dev/null || exit 1

go test

popd > /dev/null || exit 1
