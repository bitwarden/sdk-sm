#!/usr/bin/env bash
export DOTNET_CLI_TELEMETRY_OPTOUT=true
export DOTNET_NOLOGO=false

dotnet build --verbosity quiet
