#requires -Version 5.1

[CmdletBinding()]
param(
    [switch]$Silent,
    [switch]$KeepInstaller,
    [switch]$ResolveOnly
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

if ($env:OS -ne 'Windows_NT') {
    throw 'ClipLingo installer supports Windows only.'
}

$architecture = if ($env:PROCESSOR_ARCHITEW6432) {
    $env:PROCESSOR_ARCHITEW6432
} else {
    $env:PROCESSOR_ARCHITECTURE
}

if ($architecture -ne 'AMD64') {
    throw "ClipLingo currently qualifies Windows x64 only. Detected architecture: $architecture"
}

try {
    [Net.ServicePointManager]::SecurityProtocol =
        [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
} catch {
    # PowerShell 7+ already uses modern TLS defaults.
}

$repository = 'howlil/cliplingo'
$releaseApi = "https://api.github.com/repos/$repository/releases?per_page=20"
$headers = @{
    Accept = 'application/vnd.github+json'
    'User-Agent' = 'ClipLingo-PowerShell-Installer'
    'X-GitHub-Api-Version' = '2022-11-28'
}

if ($env:GITHUB_TOKEN) {
    $headers.Authorization = "Bearer $env:GITHUB_TOKEN"
}

function Get-LatestClipLingoRelease {
    $releases = @(Invoke-RestMethod -Uri $releaseApi -Headers $headers -Method Get)
    $release = $releases |
        Where-Object { -not $_.draft -and $_.published_at } |
        Sort-Object { [DateTimeOffset]$_.published_at } -Descending |
        Select-Object -First 1

    if (-not $release) {
        throw 'No published ClipLingo release was found.'
    }

    return $release
}

function Get-ClipLingoInstallerAsset {
    param([Parameter(Mandatory)]$Release)

    $assets = @($Release.assets | Where-Object {
        $_.name -match '^ClipLingo_.+_x64-setup\.exe$'
    })

    if ($assets.Count -ne 1) {
        throw "Expected exactly one x64 installer asset in $($Release.tag_name); found $($assets.Count)."
    }

    return $assets[0]
}

function Get-ExpectedSha256 {
    param(
        [Parameter(Mandatory)]$Release,
        [Parameter(Mandatory)]$Installer
    )

    $digestProperty = $Installer.PSObject.Properties['digest']
    $digest = if ($digestProperty) { [string]$digestProperty.Value } else { '' }

    if ($digest -match '^sha256:([0-9a-fA-F]{64})$') {
        return $Matches[1].ToUpperInvariant()
    }

    $sidecarName = "$($Installer.name).sha256"
    $sidecars = @($Release.assets | Where-Object { $_.name -eq $sidecarName })
    if ($sidecars.Count -ne 1) {
        throw "Release $($Release.tag_name) does not expose trusted SHA256 metadata for $($Installer.name)."
    }

    $response = Invoke-WebRequest -Uri $sidecars[0].browser_download_url -Headers $headers -UseBasicParsing
    if ([string]$response.Content -notmatch '(?i)\b([0-9a-f]{64})\b') {
        throw "Checksum sidecar $sidecarName does not contain a SHA256 digest."
    }

    return $Matches[1].ToUpperInvariant()
}

Write-Host 'Resolving latest published ClipLingo release...'
$release = Get-LatestClipLingoRelease
$installer = Get-ClipLingoInstallerAsset -Release $release
$expectedSha256 = Get-ExpectedSha256 -Release $release -Installer $installer

Write-Host "Release:   $($release.tag_name)"
Write-Host "Installer: $($installer.name)"
Write-Host "SHA256:    $expectedSha256"

if ($ResolveOnly) {
    return
}

$tempRoot = Join-Path ([IO.Path]::GetTempPath()) ("cliplingo-install-" + [guid]::NewGuid().ToString('N'))
$installerPath = Join-Path $tempRoot $installer.name
New-Item -ItemType Directory -Path $tempRoot -Force | Out-Null

try {
    Write-Host 'Downloading installer from the canonical GitHub Release asset...'
    Invoke-WebRequest -Uri $installer.browser_download_url -Headers $headers -UseBasicParsing -OutFile $installerPath

    $actualSha256 = (Get-FileHash -Path $installerPath -Algorithm SHA256).Hash.ToUpperInvariant()
    if ($actualSha256 -ne $expectedSha256) {
        throw "Installer SHA256 mismatch. Expected $expectedSha256 but received $actualSha256."
    }

    Write-Host 'Installer integrity verified.'
    Write-Host "Installing ClipLingo $($release.tag_name)..."

    if ($Silent) {
        $process = Start-Process -FilePath $installerPath -ArgumentList '/S' -Wait -PassThru
    } else {
        $process = Start-Process -FilePath $installerPath -Wait -PassThru
    }

    if ($process.ExitCode -ne 0) {
        throw "ClipLingo installer exited with code $($process.ExitCode)."
    }

    Write-Host "ClipLingo $($release.tag_name) installed successfully."
} finally {
    if ($KeepInstaller) {
        Write-Host "Installer kept at: $installerPath"
    } elseif (Test-Path $tempRoot) {
        Remove-Item -Path $tempRoot -Recurse -Force -ErrorAction SilentlyContinue
    }
}
