param (
  [switch]$Uninstall
)

$ErrorActionPreference = "Stop"

$defaultBwsVersion = "2.1.0"
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
      Write-Output "Installation cancelled by user."
      exit
    }
  }
}

function Invoke-BwsDownload {
  Write-Output "Detected architecture: $arch"

  $bwsUrl = "https://github.com/bitwarden/sdk-sm/releases/download/bws-v$bwsVersion/bws-$arch-pc-windows-msvc-$bwsVersion.zip"
  Write-Output "Downloading bws from: $bwsUrl"
  $outputPath = Join-Path $env:TEMP "bws.zip"
  Invoke-WebRequest -Uri $bwsUrl -OutFile $outputPath
  return $outputPath
}

function Test-Checksum {
  param($zipPath)
  Write-Output "Validating checksum..."

  $checksumUrl = "https://github.com/bitwarden/sdk-sm/releases/download/bws-v$bwsVersion/bws-sha256-checksums-$bwsVersion.txt"
  $checksumFile = Join-Path $env:TEMP "bws-checksums.txt"
  Invoke-WebRequest -Uri $checksumUrl -OutFile $checksumFile

  $expectedChecksum = (Get-Content $checksumFile | Where-Object { $_ -match "bws-$arch-pc-windows-msvc-$bwsVersion.zip" }).Split(" ")[0]
  $actualChecksum = (Get-FileHash -Algorithm SHA256 -Path $zipPath).Hash

  if ($actualChecksum -ne $expectedChecksum) {
    throw "Checksum validation failed. Expected: $expectedChecksum, Actual: $actualChecksum"
  } else {
    Write-Output "Checksum validation successful."
  }
}

function Install-Bws {
  param($zipPath)
  Write-Output "Installing bws..."
  New-Item -ItemType Directory -Force -Path $installDir | Out-Null
  Expand-Archive -Force $zipPath -DestinationPath $installDir
  Write-Output "bws installed to $installDir"
  Add-BwsToUserPath
  Write-Output "Please restart your shell to use bws"
}

function Add-BwsToUserPath {
  # Update only the user PATH in the registry. Don't use `setx PATH "$env:PATH;..."`:
  # $env:PATH also contains the machine PATH, and setx truncates values to 1024 characters.
  $envKey = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey("Environment", $true)
  try {
    # Read the raw value so %VAR% references in the user PATH are kept.
    $userPath = $envKey.GetValue("Path", "", [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
    $entries = @($userPath -split ";" | Where-Object { $_ -ne "" })

    if ($entries -contains $installDir) {
      Write-Output "$installDir is already in your PATH"
      return
    }

    $newPath = ($entries + $installDir) -join ";"
    $envKey.SetValue("Path", $newPath, [Microsoft.Win32.RegistryValueKind]::ExpandString)
  } finally {
    $envKey.Dispose()
  }

  # Setting a user variable through .NET broadcasts WM_SETTINGCHANGE,
  # so new shells started from Explorer see the updated PATH.
  [Environment]::SetEnvironmentVariable("BWS_INSTALL_TMP", "1", "User")
  [Environment]::SetEnvironmentVariable("BWS_INSTALL_TMP", $null, "User")

  Write-Output "$installDir has been added to your PATH"
}

function Test-Bws {
  Write-Output "Checking bws..."
  $bwsPath = Join-Path $installDir "bws.exe"
  if (Test-Path $bwsPath) {
    Write-Output "bws is installed at $bwsPath"
  } else {
    throw "bws is not installed"
  }
}

function Remove-Bws {
  Write-Output "Uninstalling bws..."

  if (Test-Path $installDir) {
    Remove-Item -Path $installDir -Recurse -Force
    Write-Output "bws uninstalled from $installDir"
  } else {
    Write-Output "bws installation directory not found at $installDir. Skipping removal."
  }

  $configDir = "$env:USERPROFILE\.bws"
  if (Test-Path $configDir -PathType Container) {
    Remove-Item -Path $configDir -Recurse -Force
    Write-Output "bws config directory removed from $configDir"
  } else {
    Write-Output "bws config directory not found at $configDir. Skipping removal."
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
