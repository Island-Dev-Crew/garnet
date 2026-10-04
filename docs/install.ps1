# Garnet installer for Windows (Windows PowerShell 5.1 or PowerShell 7+).
#
# Public bootstrap:
#   irm https://garnet-lang.org/install.ps1 | iex
#
# Downloads garnet-<version>-x86_64-pc-windows-msvc.zip, SHA256SUMS and
# SHA256SUMS.asc from the GitHub Release, checks the signature on SHA256SUMS
# when gpg is installed, checks the zip's SHA-256 against SHA256SUMS, installs
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
#   GARNET_VERIFY_SIGNATURE set to 0 to skip the signature check (for local
#                           test assets); SHA256SUMS is then trusted on
#                           integrity only
#   GARNET_SIGNING_KEYS_URL where the release public keys come from (default
#                           https://garnet-lang.org/garnet-release-keys.asc)
#   GARNET_SIGNING_KEY_FPR  pin a different key, for a mirror you sign yourself
#
# When gpg is on PATH, SHA256SUMS.asc must be a detached signature over
# SHA256SUMS by the release key this script pins for the version. The keys
# file may hold other keys, but a signature by any of them is refused, so a
# swapped keys file cannot pass. A missing, tampered or wrong-key signature
# stops the install. Without gpg the script warns and trusts SHA256SUMS on
# integrity only; Gpg4win provides gpg. Releases before v0.8.1 were never
# signed. See docs/release-signing.md.

& {
    Set-StrictMode -Version 3
    $ErrorActionPreference = 'Stop'
    $ProgressPreference = 'SilentlyContinue'

    function Say([string]$Message) { Write-Host "garnet-install: $Message" }
    function Fail([string]$Message) { throw "garnet-install: error: $Message" }
    function Warn([string]$Message) { Write-Warning "garnet-install: $Message" }

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
    $keysUrl = 'https://garnet-lang.org/garnet-release-keys.asc'
    if ($env:GARNET_SIGNING_KEYS_URL) { $keysUrl = $env:GARNET_SIGNING_KEYS_URL }
    # The rotated release key, pinned for v0.8.3 and later. Empty until the new
    # key exists; while it is empty, a later version is refused when gpg is present.
    $releaseKey083 = ''

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

    function Get-PinnedSigningKey([string]$Version) {
        if ($env:GARNET_SIGNING_KEY_FPR) { return $env:GARNET_SIGNING_KEY_FPR }
        if ($Version -eq '0.8.1' -or $Version -eq '0.8.2') { return '04D56F91F03817DDFFEBC62AC14DF6E713956ED1' }
        return $releaseKey083
    }

    # True when the version is older than v0.8.1, the first release that
    # shipped SHA256SUMS.asc. A version that does not parse counts as signed.
    function Test-PredatesSignedReleases([string]$Version) {
        if ($Version -notmatch '^(?<major>\d{1,6})\.(?<minor>\d{1,6})(\.(?<patch>\d{1,6}))?') { return $false }
        $major = [int]$Matches['major']
        $minor = [int]$Matches['minor']
        $patch = 0
        if ($Matches['patch']) { $patch = [int]$Matches['patch'] }
        if ($major -ne 0) { return $false }
        return ($minor -lt 8) -or ($minor -eq 8 -and $patch -lt 1)
    }

    # Windows PowerShell 5.1 turns a native command's redirected stderr into
    # errors under 'Stop', so the preference is relaxed inside this function
    # only. The status lines come back; the exit code decides.
    # Git for Windows, MSYS2 and Cygwin ship gpg as a POSIX program with cygpath
    # beside it. It reads C:\... as a relative path, and its gpg-agent, which
    # starts from /, needs an absolute home, so it is given POSIX paths. Gpg4win's
    # gpg has no cygpath beside it and takes Windows paths as they are.
    function ConvertTo-GpgPath([string]$GpgPath, [string]$Path) {
        $cygpath = Join-Path (Split-Path -Parent $GpgPath) 'cygpath.exe'
        if (-not (Test-Path -LiteralPath $cygpath -PathType Leaf)) { return $Path }
        $ErrorActionPreference = 'Continue'
        $converted = @(& $cygpath -u $Path 2>$null)
        if ($LASTEXITCODE -ne 0 -or $converted.Count -ne 1 -or -not "$($converted[0])".StartsWith('/')) {
            Fail "cannot convert $Path to a path for $GpgPath"
        }
        return "$($converted[0])"
    }

    # stdout carries the status lines; gpg's own messages (stderr) are kept so a
    # refusal can say why gpg failed.
    function Invoke-Gpg([string]$GpgPath, [string]$GnupgHome, [string[]]$Arguments) {
        $ErrorActionPreference = 'Continue'
        $output = @(& $GpgPath --homedir (ConvertTo-GpgPath $GpgPath $GnupgHome) --batch --no-tty @Arguments 2>&1)
        $code = $LASTEXITCODE
        $lines = @($output | Where-Object { $_ -isnot [System.Management.Automation.ErrorRecord] } | ForEach-Object { "$_" })
        $messages = @($output | Where-Object { $_ -is [System.Management.Automation.ErrorRecord] } | ForEach-Object { "$_".Trim() } | Where-Object { $_ })
        return [pscustomobject]@{ ExitCode = $code; Lines = $lines; Messages = $messages }
    }

    function Format-GpgMessages($Result) {
        if ($Result.Messages.Count -eq 0) { return "gpg exited $($Result.ExitCode)" }
        return "gpg exited $($Result.ExitCode): $($Result.Messages -join ' | ')"
    }

    function Confirm-SumsSignature([string]$Sums, [string]$SignatureUrl, [string]$Work) {
        if ($env:GARNET_VERIFY_SIGNATURE -eq '0') {
            Warn 'signature verification is off (GARNET_VERIFY_SIGNATURE=0); SHA256SUMS is trusted on integrity only'
            return
        }
        if (-not $env:GARNET_SIGNING_KEY_FPR -and (Test-PredatesSignedReleases $version)) {
            Warn "v$version predates signed releases (the first is v0.8.1); SHA256SUMS is trusted on integrity only"
            return
        }
        $gpg = Get-Command gpg -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($null -eq $gpg) {
            Warn 'gpg not found: SHA256SUMS.asc is not verified, so SHA256SUMS is trusted on integrity only; install gpg (Gpg4win) to check authenticity (docs/release-signing.md)'
            return
        }

        $fpr = Get-PinnedSigningKey $version
        if (-not $fpr) {
            Fail "no release signing key is pinned for v$version in this installer; refusing to trust SHA256SUMS (GARNET_VERIFY_SIGNATURE=0 proceeds on integrity only)"
        }
        $gnupg = Join-Path $Work 'gnupg'
        New-Item -ItemType Directory -Path $gnupg | Out-Null
        $signature = Join-Path $Work 'SHA256SUMS.asc'
        try {
            Get-Asset $SignatureUrl $signature
        } catch {
            Fail "SHA256SUMS.asc is missing for v$version ($SignatureUrl); refusing an unsigned SHA256SUMS"
        }
        $keys = Join-Path $Work 'garnet-release-keys.asc'
        try {
            Get-Asset $keysUrl $keys
        } catch {
            Fail "cannot fetch the release signing keys from $keysUrl"
        }
        $import = Invoke-Gpg $gpg.Path $gnupg @('--quiet', '--import', (ConvertTo-GpgPath $gpg.Path $keys))
        if ($import.ExitCode -ne 0) {
            Fail "cannot import the release signing keys from $keysUrl ($(Format-GpgMessages $import))"
        }
        # A detached signature is required: gpg refuses an inline-signed message
        # when given the data file, so another signed text cannot stand in.
        $result = Invoke-Gpg $gpg.Path $gnupg @('--status-fd', '1', '--verify', (ConvertTo-GpgPath $gpg.Path $signature), (ConvertTo-GpgPath $gpg.Path $Sums))
        if ($result.ExitCode -ne 0) { Fail "SHA256SUMS.asc does not verify against SHA256SUMS; refusing to install ($(Format-GpgMessages $result))" }
        # VALIDSIG carries the signing key's fingerprint and, last, its primary key's.
        $signedByPin = $false
        foreach ($line in $result.Lines) {
            $fields = @("$line" -split ' ' | Where-Object { $_ })
            if ($fields.Count -ge 3 -and $fields[0] -eq '[GNUPG:]' -and $fields[1] -eq 'VALIDSIG' -and
                ($fields[2] -eq $fpr -or $fields[$fields.Count - 1] -eq $fpr)) {
                $signedByPin = $true
            }
        }
        if (-not $signedByPin) { Fail "SHA256SUMS.asc is not signed by the pinned release key $fpr; refusing to install" }
        Say "SHA256SUMS signature verified (key $fpr)"
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
                  'Windows release assets are published starting with v0.8.2. ' +
                  "To build this version from source instead (requires Rust): cargo install --git https://github.com/$repo --tag v$version --locked garnet-cli")
        }
        Confirm-SumsSignature $sums "$baseUrl/SHA256SUMS.asc" $work

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
