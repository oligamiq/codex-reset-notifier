$ErrorActionPreference = 'Stop'

$Repo = 'oligamiq/codex-reset-notifier'
$Asset = 'codex-reset-notifier-windows-x64.exe'
$BaseUrl = "https://github.com/$Repo/releases/latest/download"
$InstallDir = if ($env:CODEX_NOTIFY_INSTALL_DIR) {
    $env:CODEX_NOTIFY_INSTALL_DIR
} else {
    Join-Path $env:LOCALAPPDATA 'Programs\codex-reset-notifier'
}

if (-not [Environment]::Is64BitOperatingSystem) {
    throw 'Only 64-bit Windows is currently supported.'
}

$TempDir = Join-Path ([IO.Path]::GetTempPath()) ([Guid]::NewGuid())
New-Item -ItemType Directory -Path $TempDir | Out-Null
try {
    $BinaryPath = Join-Path $TempDir $Asset
    $ChecksumsPath = Join-Path $TempDir 'SHA256SUMS.txt'
    Write-Host "Downloading $Asset..."
    Invoke-WebRequest -UseBasicParsing "$BaseUrl/$Asset" -OutFile $BinaryPath
    Invoke-WebRequest -UseBasicParsing "$BaseUrl/SHA256SUMS.txt" -OutFile $ChecksumsPath

    $Line = Get-Content $ChecksumsPath | Where-Object { $_ -match "\s+$([regex]::Escape($Asset))$" } | Select-Object -First 1
    if (-not $Line) { throw "Checksum for $Asset not found." }
    $Expected = ($Line -split '\s+')[0].ToLowerInvariant()
    $Actual = (Get-FileHash -Algorithm SHA256 $BinaryPath).Hash.ToLowerInvariant()
    if ($Actual -ne $Expected) { throw 'SHA256 mismatch.' }

    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    $Destination = Join-Path $InstallDir 'codex-reset-notifier.exe'
    Move-Item -Force $BinaryPath $Destination

    $UserPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    $Parts = @($UserPath -split ';' | Where-Object { $_ })
    if ($Parts -notcontains $InstallDir) {
        $NewUserPath = (($Parts + $InstallDir) -join ';')
        [Environment]::SetEnvironmentVariable('Path', $NewUserPath, 'User')
        Write-Host "Added $InstallDir to your user PATH."
    }
    if (($env:Path -split ';') -notcontains $InstallDir) {
        $env:Path = "$env:Path;$InstallDir"
    }

    Write-Host "Installed codex-reset-notifier to $Destination"
    Write-Host 'Next: set CODEX_NOTIFY_NTFY_TOPIC and run codex-reset-notifier --test-notification.'
}
finally {
    Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $TempDir
}
