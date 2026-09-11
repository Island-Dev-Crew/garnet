# Garnet installer for Windows (Windows PowerShell 5.1 or PowerShell 7+).
#
# Public bootstrap:
#   irm https://garnet-lang.org/install.ps1 | iex
#
# Downloads garnet-<version>-x86_64-pc-windows-msvc.zip and SHA256SUMS from the
# GitHub Release, checks the zip's SHA-256 against SHA256SUMS, installs
# garnet.exe into %LOCALAPPDATA%\Programs\Garnet\bin, adds that directory to the
# user PATH and runs `garnet --version`. Windows release assets are published
# starting with v0.8.2. Windows on ARM runs the x86_64 build under emulation.
#
# The script takes no parameters, so it works through `iex`. Override with
# environment variables instead:
#   GARNET_VERSION          release version (default 0.8.2)
#   GARNET_REPO             GitHub owner/name (default Island-Dev-Crew/garnet)
#   GARNET_BASE_URL         release download base: https:// (every redirect
#                           must stay on https), or file:/// for a local path
#                           (network shares are refused)
#   GARNET_PREFIX           install root (default %LOCALAPPDATA%\Programs\Garnet)
#   GARNET_NO_MODIFY_PATH   set to 1 to leave the user PATH unchanged
#
# This script checks integrity against SHA256SUMS. SHA256SUMS itself is
# GPG-signed (SHA256SUMS.asc); to check authenticity as well, follow
# https://github.com/Island-Dev-Crew/garnet/blob/main/docs/release-signing.md

& {
    Set-StrictMode -Version 3
    $ErrorActionPreference = 'Stop'
    $ProgressPreference = 'SilentlyContinue'

    function Say([string]$Message) { Write-Host "garnet-install: $Message" }
    function Fail([string]$Message) { throw "garnet-install: error: $Message" }

    if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) {
        Fail "this installer is for Windows. On macOS and Linux run: curl --proto '=https' --tlsv1.2 -sSf https://garnet-lang.org/install.sh | sh"
    }

    $version = '0.8.2'
    if ($env:GARNET_VERSION) { $version = $env:GARNET_VERSION.TrimStart('v') }
    $repo = 'Island-Dev-Crew/garnet'
    if ($env:GARNET_REPO) { $repo = $env:GARNET_REPO }
    $baseUrl = "https://github.com/$repo/releases/download/v$version"
    if ($env:GARNET_BASE_URL) { $baseUrl = $env:GARNET_BASE_URL.TrimEnd('/') }
    $prefix = Join-Path $env:LOCALAPPDATA 'Programs\Garnet'
    if ($env:GARNET_PREFIX) { $prefix = $env:GARNET_PREFIX }
    $bin = Join-Path $prefix 'bin'

    # A 32-bit PowerShell on 64-bit Windows reports x86 here; the real
    # architecture is then in PROCESSOR_ARCHITEW6432.
    $arch = $env:PROCESSOR_ARCHITECTURE
    if ($env:PROCESSOR_ARCHITEW6432) { $arch = $env:PROCESSOR_ARCHITEW6432 }
    switch ($arch) {
        'AMD64' { $target = 'x86_64-pc-windows-msvc' }
        'ARM64' {
            $target = 'x86_64-pc-windows-msvc'
            Say 'Windows on ARM: installing the x86_64 build, which runs under emulation'
        }
        default { Fail "unsupported Windows architecture: $arch" }
    }

    if ($PSVersionTable.PSVersion.Major -lt 6) {
        [Net.ServicePointManager]::SecurityProtocol =
            [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
    }
    Add-Type -AssemblyName System.Net.Http
    Add-Type -AssemblyName System.IO.Compression
    Add-Type -AssemblyName System.IO.Compression.FileSystem

    function Get-LocalPath([string]$Url) {
        $uri = [Uri]$Url
        if (-not $uri.IsFile -or $uri.IsUnc -or $uri.LocalPath.StartsWith('\\')) {
            Fail "refusing a file URL that is not a local path: $Url"
        }
        return $uri.LocalPath
    }

    # Redirects are followed here, one hop at a time, so every hop must stay on
    # https. Windows PowerShell 5.1's automatic redirects would also follow an
    # https-to-http redirect, which would let both SHA256SUMS and the zip be
    # substituted together.
    function Get-Asset([string]$Url, [string]$OutFile) {
        if ($Url.StartsWith('file:')) {
            Copy-Item -LiteralPath (Get-LocalPath $Url) -Destination $OutFile
            return
        }
        $handler = New-Object System.Net.Http.HttpClientHandler
        $handler.AllowAutoRedirect = $false
        $client = New-Object System.Net.Http.HttpClient($handler)
        try {
            $current = [Uri]$Url
            for ($hop = 0; $hop -le 5; $hop++) {
                if ($current.Scheme -ne 'https') { Fail "refusing a non-https download URL: $current" }
                $response = $client.GetAsync($current).GetAwaiter().GetResult()
                try {
                    $status = [int]$response.StatusCode
                    if ($status -ge 300 -and $status -lt 400 -and $null -ne $response.Headers.Location) {
                        $current = New-Object Uri($current, $response.Headers.Location)
                        continue
                    }
                    if (-not $response.IsSuccessStatusCode) { Fail "download failed with HTTP ${status}: $current" }
                    [IO.File]::WriteAllBytes($OutFile, $response.Content.ReadAsByteArrayAsync().GetAwaiter().GetResult())
                    return
                } finally {
                    $response.Dispose()
                }
            }
            Fail "too many redirects: $Url"
        } finally {
            $client.Dispose()
        }
    }

    $asset = "garnet-$version-$target.zip"
    Say "version  = $version"
    Say "release  = $baseUrl"
    Say "asset    = $asset"

    $work = Join-Path ([IO.Path]::GetTempPath()) ('garnet-install-' + [Guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $work | Out-Null
    try {
        $sums = Join-Path $work 'SHA256SUMS'
        try {
            Get-Asset "$baseUrl/SHA256SUMS" $sums
        } catch {
            Fail ("could not download SHA256SUMS for v$version from $baseUrl ($($_.Exception.Message)). " +
                  'Windows release assets are published starting with v0.8.2; if that release is not out yet, ' +
                  "build the development branch from source (requires Rust): cargo install --git https://github.com/$repo --locked garnet-cli")
        }

        # sha256sum format: 64 hex digits, a space, a mode character (space or
        # '*'), then the file name, compared exactly and case-sensitively.
        $expected = $null
        foreach ($line in Get-Content -LiteralPath $sums) {
            if ($line -cmatch '^(?<hash>[0-9a-fA-F]{64}) (?<mode>[ *])(?<name>.+)$' -and
                [string]::Equals($Matches['name'], $asset, [StringComparison]::Ordinal)) {
                $expected = $Matches['hash'].ToLowerInvariant()
                break
            }
        }
        if (-not $expected) { Fail "SHA256SUMS for v$version lists no $asset" }

        $zip = Join-Path $work $asset
        Say "downloading $baseUrl/$asset"
        Get-Asset "$baseUrl/$asset" $zip
        $actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $zip).Hash.ToLowerInvariant()
        if ($actual -ne $expected) {
            Fail "SHA-256 mismatch for $asset (expected $expected, got $actual); refusing to install an unverified download"
        }
        Say 'SHA-256 verified'

        # Only the root entry named exactly garnet.exe is read, and it is
        # written to one fixed path, so no entry name can steer a write.
        $archive = [IO.Compression.ZipFile]::OpenRead($zip)
        try {
            $entry = $null
            foreach ($candidate in $archive.Entries) {
                if ([string]::Equals($candidate.FullName, 'garnet.exe', [StringComparison]::Ordinal)) { $entry = $candidate; break }
            }
            if ($null -eq $entry) { Fail "$asset does not contain garnet.exe at its root" }
            New-Item -ItemType Directory -Force -Path $bin | Out-Null
            [IO.Compression.ZipFileExtensions]::ExtractToFile($entry, (Join-Path $bin 'garnet.exe'), $true)
        } finally {
            $archive.Dispose()
        }
        Say "installed garnet.exe into $bin"
    } finally {
        Remove-Item -LiteralPath $work -Recurse -Force -ErrorAction SilentlyContinue
    }

    if ($env:GARNET_NO_MODIFY_PATH -ne '1') {
        $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
        $entries = @()
        if ($userPath) { $entries = @($userPath -split ';' | Where-Object { $_ }) }
        if ($entries -notcontains $bin) {
            [Environment]::SetEnvironmentVariable('Path', (@($entries + $bin) -join ';'), 'User')
            Say "added $bin to your user PATH; new terminals pick it up"
        }
    }
    if (@($env:Path -split ';') -notcontains $bin) { $env:Path = "$bin;$env:Path" }

    & (Join-Path $bin 'garnet.exe') --version
    if ($LASTEXITCODE -ne 0) { Fail 'garnet --version failed after install' }
    Say 'install complete'
}
