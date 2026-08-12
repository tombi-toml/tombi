<#
.SYNOPSIS
Installs Tombi from GitHub Releases.

.PARAMETER Version
The Tombi version to install. Defaults to the embedded latest stable version.

.PARAMETER InstallDir
The directory where tombi.exe will be installed.

.PARAMETER Checksum
An optional SHA256 checksum, either as hexadecimal or sha256:<hex>.
#>
[CmdletBinding()]
param(
    [string]$Version,
    [string]$InstallDir,
    [string]$Checksum
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$ChecksumWasSpecified = $PSBoundParameters.ContainsKey("Checksum")
$InstallDirWasSpecified = $PSBoundParameters.ContainsKey("InstallDir")

$LatestStableVersion = "1.3.4"
$ReleaseBaseUrl = "https://github.com/tombi-toml/tombi/releases/download"
$ExecutableName = "tombi.exe"

function Write-Step {
    param([string]$Message)

    [Console]::Error.WriteLine("==> {0}", $Message)
}

function Resolve-Version {
    param([string]$RequestedVersion)

    if ([string]::IsNullOrWhiteSpace($RequestedVersion) -or $RequestedVersion -eq "latest") {
        return $LatestStableVersion
    }

    $ResolvedVersion = $RequestedVersion -replace "^v", ""
    if ($ResolvedVersion -notmatch "^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$") {
        throw "Invalid version '$RequestedVersion'. Expected a semantic version or 'latest'."
    }

    return $ResolvedVersion
}

function Resolve-Architecture {
    $Architecture = $env:PROCESSOR_ARCHITEW6432
    if ([string]::IsNullOrWhiteSpace($Architecture)) {
        $Architecture = $env:PROCESSOR_ARCHITECTURE
    }

    switch ($Architecture.ToUpperInvariant()) {
        "AMD64" { return "x86_64" }
        "X86_64" { return "x86_64" }
        "ARM64" { return "aarch64" }
        default { throw "Unsupported Windows architecture: $Architecture" }
    }
}

function ConvertTo-NormalizedChecksum {
    param([string]$ChecksumValue)

    if ([string]::IsNullOrWhiteSpace($ChecksumValue)) {
        throw "Invalid checksum: value must not be empty."
    }

    $OriginalChecksum = $ChecksumValue
    if ($ChecksumValue.StartsWith("sha256:", [StringComparison]::OrdinalIgnoreCase)) {
        $ChecksumValue = $ChecksumValue.Substring("sha256:".Length)
    }
    elseif ($ChecksumValue.Contains(":")) {
        throw "Unsupported checksum format '$OriginalChecksum'. Only sha256:<hex> is supported."
    }

    if ([string]::IsNullOrWhiteSpace($ChecksumValue)) {
        throw "Invalid checksum: SHA256 value must not be empty."
    }

    if ($ChecksumValue.Length -ne 64) {
        throw "Invalid checksum '$OriginalChecksum': expected 64 hex characters for SHA256, got $($ChecksumValue.Length)."
    }

    if ($ChecksumValue -notmatch "^[0-9A-Fa-f]{64}$") {
        throw "Invalid checksum '$OriginalChecksum': SHA256 must contain only hexadecimal characters."
    }

    return $ChecksumValue.ToLowerInvariant()
}

function Test-PathEntry {
    param(
        [string]$PathValue,
        [string]$Entry
    )

    if ([string]::IsNullOrWhiteSpace($PathValue)) {
        return $false
    }

    $NormalizedEntry = [IO.Path]::GetFullPath($Entry).TrimEnd("\")
    foreach ($Candidate in $PathValue.Split(";")) {
        if ([string]::IsNullOrWhiteSpace($Candidate)) {
            continue
        }

        try {
            $ExpandedCandidate = [Environment]::ExpandEnvironmentVariables($Candidate)
            $NormalizedCandidate = [IO.Path]::GetFullPath($ExpandedCandidate).TrimEnd("\")
        }
        catch {
            continue
        }

        if ($NormalizedCandidate.Equals($NormalizedEntry, [StringComparison]::OrdinalIgnoreCase)) {
            return $true
        }
    }

    return $false
}

function Assert-TombiVersion {
    param(
        [string]$BinaryPath,
        [string]$ExpectedVersion
    )

    $VersionOutputLines = @(& $BinaryPath --version 2>&1)
    $VersionExitCode = $LASTEXITCODE
    if ($VersionExitCode -ne 0) {
        throw "$BinaryPath cannot be executed."
    }
    $VersionOutput = ($VersionOutputLines | Select-Object -First 1).ToString()
    if ($VersionOutput -notmatch "^tombi v?(\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?)") {
        throw "Unable to determine the installed Tombi version from: $VersionOutput"
    }
    if ($Matches[1] -ne $ExpectedVersion) {
        throw "Installed version mismatch: expected $ExpectedVersion, but got $($Matches[1])."
    }
}

if ($env:OS -ne "Windows_NT") {
    throw "install.ps1 only supports Windows."
}

$ResolvedVersion = Resolve-Version -RequestedVersion $Version
$Architecture = Resolve-Architecture
$Target = "$Architecture-pc-windows-msvc"
$ExpectedChecksum = $null
if ($ChecksumWasSpecified) {
    $ExpectedChecksum = ConvertTo-NormalizedChecksum -ChecksumValue $Checksum
}

if ($InstallDirWasSpecified -and [string]::IsNullOrWhiteSpace($InstallDir)) {
    throw "Invalid installation directory: value must not be empty."
}
if (-not $InstallDirWasSpecified) {
    if ([string]::IsNullOrWhiteSpace($env:LOCALAPPDATA)) {
        throw "LOCALAPPDATA is not available. Specify the installation directory with -InstallDir."
    }
    $InstallDir = Join-Path $env:LOCALAPPDATA "tombi\bin"
}
$InstallDir = [IO.Path]::GetFullPath($InstallDir)
if (-not (Test-PathEntry -PathValue $env:PATH -Entry $InstallDir)) {
    Write-Step "$InstallDir is not in your PATH. Consider adding it to your user PATH."
}

$AssetName = "tombi-cli-$ResolvedVersion-$Target.zip"
$DownloadUrl = "$ReleaseBaseUrl/v$ResolvedVersion/$AssetName"
$TemporaryDirectory = Join-Path ([IO.Path]::GetTempPath()) ("tombi-install-" + [Guid]::NewGuid())
$ArchivePath = Join-Path $TemporaryDirectory $AssetName
$ExtractDirectory = Join-Path $TemporaryDirectory "extract"
$InstalledBinary = Join-Path $InstallDir $ExecutableName

Write-Step "Detected system: $Target"
Write-Step "Installing tombi $ResolvedVersion to $InstallDir"

try {
    New-Item -ItemType Directory -Path $TemporaryDirectory | Out-Null
    New-Item -ItemType Directory -Path $ExtractDirectory | Out-Null
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

    Write-Step "Downloading $DownloadUrl"
    $PreviousSecurityProtocol = [Net.ServicePointManager]::SecurityProtocol
    try {
        [Net.ServicePointManager]::SecurityProtocol = $PreviousSecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -UseBasicParsing -Uri $DownloadUrl -OutFile $ArchivePath
    }
    finally {
        [Net.ServicePointManager]::SecurityProtocol = $PreviousSecurityProtocol
    }

    if ($null -ne $ExpectedChecksum) {
        $ActualChecksum = (Get-FileHash -Algorithm SHA256 -LiteralPath $ArchivePath).Hash.ToLowerInvariant()
        if ($ActualChecksum -ne $ExpectedChecksum) {
            throw "Checksum verification failed. Expected: $ExpectedChecksum Actual: $ActualChecksum"
        }
        Write-Step "Checksum verification passed."
    }

    Expand-Archive -LiteralPath $ArchivePath -DestinationPath $ExtractDirectory -Force
    $ExtractedBinary = Get-ChildItem -LiteralPath $ExtractDirectory -Filter $ExecutableName -File -Recurse |
        Select-Object -First 1
    if ($null -eq $ExtractedBinary) {
        throw "$ExecutableName was not found in the downloaded archive."
    }
    Assert-TombiVersion -BinaryPath $ExtractedBinary.FullName -ExpectedVersion $ResolvedVersion

    $StagedBinary = Join-Path $InstallDir ("." + $ExecutableName + "." + [Guid]::NewGuid() + ".tmp")
    try {
        Copy-Item -LiteralPath $ExtractedBinary.FullName -Destination $StagedBinary
        Move-Item -LiteralPath $StagedBinary -Destination $InstalledBinary -Force
    }
    finally {
        Remove-Item -LiteralPath $StagedBinary -Force -ErrorAction SilentlyContinue
    }

    Write-Step "Tombi $ResolvedVersion was installed successfully."
    Write-Output $InstalledBinary
}
finally {
    Remove-Item -LiteralPath $TemporaryDirectory -Recurse -Force -ErrorAction SilentlyContinue
}
