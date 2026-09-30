# CI check for D-26: docs/install.ps1 verifies SHA256SUMS.asc against the
# pinned release key. Run from the repository root on a Windows runner, after
# the windows-cli-zip job has staged dist/ (the zip and an unsigned SHA256SUMS).
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$releaseKey = '04D56F91F03817DDFFEBC62AC14DF6E713956ED1'
$installer = (Resolve-Path docs/install.ps1).Path
$keys = ([Uri](Resolve-Path docs/garnet-release-keys.asc).Path).AbsoluteUri

# Git for Windows ships gpg in usr\bin. A missing gpg would make every case
# below pass on integrity alone, so this fails instead of skipping.
if (-not (Get-Command gpg -CommandType Application -ErrorAction SilentlyContinue)) {
    $env:Path = "$env:ProgramFiles\Git\usr\bin;$env:Path"
}
$gpg = Get-Command gpg -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
if ($null -eq $gpg) { throw 'gpg is not on PATH; the signature check would be skipped' }
Write-Host "gpg: $($gpg.Path)"
Write-Host "PowerShell: $($PSVersionTable.PSVersion)"

function Invoke-Installer([hashtable]$Env) {
    $names = @('GARNET_VERSION', 'GARNET_BASE_URL', 'GARNET_SIGNING_KEY_FPR', 'GARNET_VERIFY_SIGNATURE')
    foreach ($name in $names) { Remove-Item -Path "env:$name" -ErrorAction SilentlyContinue }
    $env:GARNET_SIGNING_KEYS_URL = $keys
    $env:GARNET_PREFIX = Join-Path $env:RUNNER_TEMP ('garnet-sig-' + [Guid]::NewGuid().ToString('N'))
    $env:GARNET_NO_MODIFY_PATH = '1'
    foreach ($key in $Env.Keys) { Set-Item -Path "env:$key" -Value $Env[$key] }
    $failure = $null
    $log = ''
    try {
        $log = & $installer *>&1 | Out-String
    } catch {
        $failure = $_.Exception.Message
    }
    foreach ($key in $Env.Keys) { Remove-Item -Path "env:$key" -ErrorAction SilentlyContinue }
    $installed = Test-Path (Join-Path $env:GARNET_PREFIX 'bin\garnet.exe')
    return [pscustomobject]@{ Log = $log; Failure = $failure; Installed = $installed }
}

function Expect-Refusal([string]$Case, [hashtable]$Env, [string]$Pattern) {
    $result = Invoke-Installer $Env
    Write-Host "${Case}: install.ps1 said: $($result.Failure)"
    if (-not $result.Failure -or $result.Failure -notmatch $Pattern) { throw "${Case}: expected a failure matching '$Pattern'" }
    if ($result.Installed) { throw "${Case}: garnet.exe was installed despite the refusal" }
}

# 1. The real v0.8.2 release installs through its real signature.
$result = Invoke-Installer @{ GARNET_VERSION = '0.8.2' }
Write-Host $result.Log
if ($result.Failure) { throw "v0.8.2 install failed: $($result.Failure)" }
if ($result.Log -notmatch "SHA256SUMS signature verified \(key $releaseKey\)") { throw 'v0.8.2 installed without the signature check' }
if (-not $result.Installed) { throw 'v0.8.2 reported success but garnet.exe is missing' }

# 2. The same real signature, with a different key pinned, is refused.
Expect-Refusal 'other key' @{ GARNET_VERSION = '0.8.2'; GARNET_SIGNING_KEY_FPR = ('0' * 40) } 'not signed by the pinned release key'

# The staged zip and its unsigned SHA256SUMS, served from a local directory.
$version = (Select-String -Path Cargo.toml -Pattern '^version = "([^"]+)"' | Select-Object -First 1).Matches[0].Groups[1].Value
$staged = Join-Path $env:RUNNER_TEMP 'garnet-sig-staged'
Remove-Item -Recurse -Force $staged -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $staged | Out-Null
Copy-Item dist/* $staged
$stagedUrl = ([Uri]$staged).AbsoluteUri

# 3. No SHA256SUMS.asc beside SHA256SUMS: refused, never installed on integrity alone.
Expect-Refusal 'missing' @{ GARNET_VERSION = $version; GARNET_BASE_URL = $stagedUrl; GARNET_SIGNING_KEY_FPR = $releaseKey } 'SHA256SUMS\.asc is missing'

# 4. A real signature by the pinned key over a different SHA256SUMS: refused.
Invoke-WebRequest -UseBasicParsing -Uri 'https://github.com/Island-Dev-Crew/garnet/releases/download/v0.8.2/SHA256SUMS.asc' -OutFile (Join-Path $staged 'SHA256SUMS.asc')
Expect-Refusal 'tampered' @{ GARNET_VERSION = $version; GARNET_BASE_URL = $stagedUrl; GARNET_SIGNING_KEY_FPR = $releaseKey } 'SHA256SUMS\.asc does not verify'

# 5. Without gpg on PATH it warns and installs on the checksum alone.
$savedPath = $env:Path
$env:Path = (@($env:Path -split ';' | Where-Object { $_ -and -not (Test-Path (Join-Path $_ 'gpg.exe')) }) -join ';')
try {
    if (Get-Command gpg -CommandType Application -ErrorAction SilentlyContinue) { throw 'could not take gpg off PATH' }
    $result = Invoke-Installer @{ GARNET_VERSION = $version; GARNET_BASE_URL = $stagedUrl }
} finally {
    $env:Path = $savedPath
}
if ($result.Failure) { throw "the no-gpg install failed: $($result.Failure)" }
if ($result.Log -notmatch 'gpg not found: SHA256SUMS\.asc is not verified') { throw 'the no-gpg install did not warn' }
if (-not $result.Installed) { throw 'the no-gpg install reported success but garnet.exe is missing' }

Write-Host 'install.ps1 signature checks passed'
