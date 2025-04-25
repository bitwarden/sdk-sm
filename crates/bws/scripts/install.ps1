param (
  [switch]$Uninstall
)

$ErrorActionPreference = "Stop"

$defaultBwsVersion = "1.0.0"
$bwsVersion = if ($env:bwsVersion) { $env:bwsVersion } else { $defaultBwsVersion }
$installDir = [Environment]::GetFolderPath([Environment+SpecialFolder]::LocalApplicationData) | Join-Path -ChildPath "Programs" | Join-Path -ChildPath "Bitwarden"

# https://learn.microsoft.com/en-us/windows/win32/cimwin32prov/win32-processor#properties
$processorArch = (Get-CimInstance -ClassName Win32_Processor).Architecture
if ($processorArch -eq 9) {
  $arch = "x86_64"
} elseif ($processorArch -eq 12) {
  $arch = "aarch64"
} else {
  throw "Unsupported architecture: $processorArch"
}

function Test-BwsInstallation {
  $existingBws = Get-Command bws -ErrorAction SilentlyContinue
  if ($null -ne $existingBws) {
    $userInput = Read-Host "bws is already installed at $($existingBws.Source). Do you want to overwrite it? (Y/N)"
    if ($userInput -ne "Y") {
      Write-Host "Installation cancelled by user."
      exit
    }
  }
}

function Invoke-BwsDownload {
  Write-Host "Detected architecture: $arch"

  $bwsUrl = "https://github.com/bitwarden/sdk-sm/releases/download/bws-v$bwsVersion/bws-$arch-pc-windows-msvc-$bwsVersion.zip"
  Write-Host "Downloading bws from: $bwsUrl"
  $outputPath = Join-Path $env:TEMP "bws.zip"
  Invoke-WebRequest -Uri $bwsUrl -OutFile $outputPath
  return $outputPath
}

function Test-Checksum {
  param($zipPath)
  Write-Host "Validating checksum..."

  $checksumUrl = "https://github.com/bitwarden/sdk-sm/releases/download/bws-v$bwsVersion/bws-sha256-checksums-$bwsVersion.txt"
  $checksumFile = Join-Path $env:TEMP "bws-checksums.txt"
  Invoke-WebRequest -Uri $checksumUrl -OutFile $checksumFile

  $expectedChecksum = (Get-Content $checksumFile | Where-Object { $_ -match "bws-$arch-pc-windows-msvc-$bwsVersion.zip" }).Split(" ")[0]
  $actualChecksum = (Get-FileHash -Algorithm SHA256 -Path $zipPath).Hash

  if ($actualChecksum -ne $expectedChecksum) {
    throw "Checksum validation failed. Expected: $expectedChecksum, Actual: $actualChecksum"
  } else {
    Write-Host "Checksum validation successful."
  }
}

function Install-Bws {
  param($zipPath)
  Write-Host "Installing bws..."
  New-Item -ItemType Directory -Force -Path $installDir | Out-Null
  Expand-Archive -Force $zipPath -DestinationPath $installDir
  Write-Host "bws installed to $installDir"
  setx PATH "$env:PATH;$installDir"
  Write-Host "$installDir has been added to your PATH"
  Write-Host "Please restart your shell to use bws"
}

function Test-Bws {
  Write-Host "Checking bws..."
  $bwsPath = Join-Path $installDir "bws.exe"
  if (Test-Path $bwsPath) {
    Write-Host "bws is installed at $bwsPath"
  } else {
    throw "bws is not installed"
  }
}

function Remove-Bws {
  Write-Host "Uninstalling bws..."

  if (Test-Path $installDir) {
    Remove-Item -Path $installDir -Recurse -Force
    Write-Host "bws uninstalled from $installDir"
  } else {
    Write-Host "bws installation directory not found at $installDir. Skipping removal."
  }

  $configDir = "$env:USERPROFILE\.bws"
  if (Test-Path $configDir -PathType Container) {
    Remove-Item -Path $configDir -Recurse -Force
    Write-Host "bws config directory removed from $configDir"
  } else {
    Write-Host "bws config directory not found at $configDir. Skipping removal."
  }
}

if ($Uninstall) {
  Remove-Bws
} else {
  Test-BwsInstallation
  $zipPath = Invoke-BwsDownload
  Test-Checksum -zipPath $zipPath
  Install-Bws -zipPath $zipPath
  Test-Bws
}
