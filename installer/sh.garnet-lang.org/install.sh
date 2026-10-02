#!/bin/sh
# Garnet universal installer, v0.8.2 stable-release default.
#
# Public bootstrap URL:
#   curl --proto '=https' --tlsv1.2 -sSf https://garnet-lang.org/install.sh | sh
#
# The script detects the host OS and architecture, prefers the matching native
# package from GitHub Releases, verifies it against SHA256SUMS, installs it, and
# runs `garnet --version`, failing if the installed binary does not run. If the
# requested release asset is unavailable, does not run on this host, or the host
# glibc is older than the requested release's Linux assets need (2.39 through
# v0.8.2, 2.34 from v0.8.3), auto mode falls back to a source install through
# `cargo install --path ... --locked`.
#
# Authenticity (D-26): when gpg is installed, SHA256SUMS is trusted only after
# SHA256SUMS.asc verifies and was made by the release key this script pins for
# the requested version. The public keys are fetched from
# GARNET_SIGNING_KEYS_URL, but a signature by any other key is refused, so a
# swapped keys file cannot pass. A missing, tampered or wrong-key signature
# stops the install; it never falls back to a source build. Without gpg the
# installer warns and trusts SHA256SUMS on integrity only; releases before
# v0.8.1 were never signed. GARNET_VERIFY_SIGNATURE=0 turns the check off (for
# local test assets). GARNET_SIGNING_KEY_FPR pins a different key, for a mirror
# you sign yourself. GARNET_BASE_URL and GARNET_CHECKSUM_URL move the asset
# source and the checksum source together; point them only at a mirror you
# trust. See docs/release-signing.md.

set -eu

GARNET_VERSION="${GARNET_VERSION:-0.8.2}"
case "${GARNET_VERSION}" in
  v*)
    GARNET_VERSION="${GARNET_VERSION#v}"
    ;;
esac
GARNET_CHANNEL="${GARNET_CHANNEL:-stable}"
GARNET_REPO="${GARNET_REPO:-Island-Dev-Crew/garnet}"
GARNET_TAG="${GARNET_TAG:-v${GARNET_VERSION}}"
GARNET_BASE_URL="${GARNET_BASE_URL:-https://github.com/${GARNET_REPO}/releases/download/${GARNET_TAG}}"
GARNET_PREFIX="${GARNET_PREFIX:-}"
GARNET_FORMAT="${GARNET_FORMAT:-}"
GARNET_INSTALL_MODE="${GARNET_INSTALL_MODE:-auto}"
GARNET_SOURCE_FALLBACK="${GARNET_SOURCE_FALLBACK:-1}"
GARNET_SOURCE_REF="${GARNET_SOURCE_REF:-}"
GARNET_SOURCE_REPO_URL="${GARNET_SOURCE_REPO_URL:-https://github.com/${GARNET_REPO}.git}"
GARNET_VERIFY_SIGNATURE="${GARNET_VERIFY_SIGNATURE:-1}"
GARNET_SIGNING_KEYS_URL="${GARNET_SIGNING_KEYS_URL:-https://garnet-lang.org/garnet-release-keys.asc}"
GARNET_SIGNING_KEY_FPR="${GARNET_SIGNING_KEY_FPR:-}"
# The rotated release key, pinned for v0.8.3 and later. Empty until the new key
# exists; while it is empty, a later version is refused when gpg is present.
GARNET_RELEASE_KEY_0_8_3=''
GARNET_INSTALLED_BIN=""

say_banner() {
    printf '%s\n' ''
    printf '%s\n' '   ####   ###  ####  #   # ####### ##### ######'
    printf '%s\n' '  #    # #   # #   # ##  # #         #     #'
    printf '%s\n' '  #      ##### ####  # # # #####     #     #'
    printf '%s\n' '  #  ### #   # #  #  #  ## #         #     #'
    printf '%s\n' '  #    # #   # #   # #   # #         #     #'
    printf '%s\n' '   ####  #   # #   # #   # #######   #     #'
    printf '%s\n' ''
    printf '%s\n' '  Rust Rigor. Ruby Velocity. One Coherent Language.'
    printf '%s\n' ''
}

say() { printf 'garnet-install: %s\n' "$1"; }
warn() { printf 'garnet-install: warning: %s\n' "$1" >&2; }
err() { printf 'garnet-install: error: %s\n' "$1" >&2; exit 1; }

need_cmd() {
    command -v "$1" >/dev/null 2>&1 || err "required command '$1' not found on PATH"
}

as_root() {
    if [ "$(id -u 2>/dev/null || printf 1)" = "0" ]; then
        "$@"
    else
        need_cmd sudo
        sudo "$@"
    fi
}

mktemp_file() {
    mktemp 2>/dev/null || printf '/tmp/garnet-%s-%s' "$$" "$1"
}

detect_triple() {
    _uname="$(uname -s 2>/dev/null || printf unknown)"
    _arch="$(uname -m 2>/dev/null || printf unknown)"

    case "$_uname" in
        Linux)
            case "$_arch" in
                x86_64|amd64) printf 'x86_64-unknown-linux-gnu' ;;
                aarch64|arm64) printf 'aarch64-unknown-linux-gnu' ;;
                *) err "unsupported Linux architecture: $_arch" ;;
            esac
            ;;
        Darwin)
            case "$_arch" in
                x86_64) printf 'x86_64-apple-darwin' ;;
                arm64|aarch64) printf 'aarch64-apple-darwin' ;;
                *) err "unsupported macOS architecture: $_arch" ;;
            esac
            ;;
        *)
            err "unsupported OS: $_uname"
            ;;
    esac
}

detect_format() {
    if [ -n "$GARNET_FORMAT" ]; then
        case "$GARNET_FORMAT" in
            deb|rpm|pkg|tar) printf '%s' "$GARNET_FORMAT" ;;
            *) err "unsupported GARNET_FORMAT: $GARNET_FORMAT" ;;
        esac
        return
    fi

    case "$(uname -s 2>/dev/null || printf unknown)" in
        Linux)
            if command -v dpkg >/dev/null 2>&1; then
                printf 'deb'
            elif command -v rpm >/dev/null 2>&1; then
                printf 'rpm'
            else
                printf 'tar'
            fi
            ;;
        # No signed .pkg is published; GARNET_FORMAT=pkg still requests one.
        Darwin) printf 'tar' ;;
        *) printf 'tar' ;;
    esac
}

asset_name() {
    _triple="$1"
    _format="$2"

    case "$_format" in
        deb)
            case "$_triple" in
                x86_64-unknown-linux-gnu) printf 'garnet_%s-1_amd64.deb' "$GARNET_VERSION" ;;
                aarch64-unknown-linux-gnu) printf 'garnet_%s-1_arm64.deb' "$GARNET_VERSION" ;;
                *) err "no Debian package mapping for $_triple" ;;
            esac
            ;;
        rpm)
            case "$_triple" in
                x86_64-unknown-linux-gnu) printf 'garnet-%s-1.x86_64.rpm' "$GARNET_VERSION" ;;
                aarch64-unknown-linux-gnu) printf 'garnet-%s-1.aarch64.rpm' "$GARNET_VERSION" ;;
                *) err "no RPM package mapping for $_triple" ;;
            esac
            ;;
        pkg)
            case "$_triple" in
                x86_64-apple-darwin|aarch64-apple-darwin) printf 'garnet-%s-universal.pkg' "$GARNET_VERSION" ;;
                *) err "no macOS package mapping for $_triple" ;;
            esac
            ;;
        tar)
            printf 'garnet-%s-%s.tar.gz' "$GARNET_VERSION" "$_triple"
            ;;
        *)
            err "unknown package format: $_format"
            ;;
    esac
}

try_download() {
    _url="$1"
    _out="$2"

    case "$_url" in
        file://*)
            _path="${_url#file://}"
            [ -f "$_path" ] || return 1
            cp "$_path" "$_out" || return 1
            return
            ;;
    esac

    if command -v curl >/dev/null 2>&1; then
        curl --proto '=https' --tlsv1.2 -fL "$_url" -o "$_out" || return 1
    elif command -v wget >/dev/null 2>&1; then
        wget --https-only -q "$_url" -O "$_out" || return 1
    else
        return 1
    fi
}

download() {
    _url="$1"
    _out="$2"

    try_download "$_url" "$_out" || err "failed to download $_url"
}

verify_sha256() {
    _file="$1"
    _expected="$2"

    if command -v sha256sum >/dev/null 2>&1; then
        _actual="$(sha256sum "$_file" | awk '{print $1}')"
    elif command -v shasum >/dev/null 2>&1; then
        _actual="$(shasum -a 256 "$_file" | awk '{print $1}')"
    else
        err "need sha256sum or shasum to verify $_file"
    fi

    if [ "$_actual" != "$_expected" ]; then
        err "SHA-256 mismatch for $_file
  expected: $_expected
  got:      $_actual
  refusing to run an unverified installer"
    fi

    say "SHA-256 verified"
}

expected_sha256() {
    _sums="$1"
    _asset="$2"
    awk -v f="$_asset" '
        {
            name = $2
            sub(/^\*/, "", name)
            base = name
            sub(/^.*\//, "", base)
            if (name == f || base == f) {
                print $1
                exit
            }
        }
    ' "$_sums"
}

# The release signing key pinned for a version (D-26), or nothing.
pinned_signing_key() {
    case "$1" in
        0.8.1|0.8.2) printf '%s' '04D56F91F03817DDFFEBC62AC14DF6E713956ED1' ;;
        *) printf '%s' "$GARNET_RELEASE_KEY_0_8_3" ;;
    esac
}

# True when version $1 is older than $2.$3.$4. A version that does not parse
# counts as not older.
version_before() {
    _vb_major="${1%%.*}"
    _vb_rest="${1#*.}"
    _vb_minor="${_vb_rest%%.*}"
    case "$_vb_rest" in
        *.*) _vb_patch="${_vb_rest#*.}" ;;
        *) _vb_patch=0 ;;
    esac
    _vb_patch="${_vb_patch%%[!0-9]*}"
    case "${_vb_major}.${_vb_minor}.${_vb_patch}" in
        *[!0-9.]*|.*|*..*|*.) return 1 ;;
    esac
    [ "$_vb_major" -lt "$2" ] && return 0
    [ "$_vb_major" -gt "$2" ] && return 1
    [ "$_vb_minor" -lt "$3" ] && return 0
    [ "$_vb_minor" -gt "$3" ] && return 1
    [ "$_vb_patch" -lt "$4" ]
}

# True when the version is older than v0.8.1, the first release that shipped
# SHA256SUMS.asc. A version that does not parse counts as signed (fail closed).
predates_signed_releases() {
    version_before "$1" 0 8 1
}

verify_sums_signature() {
    _sums="$1"
    _sig_url="$2"

    if [ "$GARNET_VERIFY_SIGNATURE" = "0" ]; then
        warn "signature verification is off (GARNET_VERIFY_SIGNATURE=0); SHA256SUMS is trusted on integrity only"
        return 0
    fi
    if [ -z "$GARNET_SIGNING_KEY_FPR" ] && predates_signed_releases "$GARNET_VERSION"; then
        warn "v${GARNET_VERSION} predates signed releases (the first is v0.8.1); SHA256SUMS is trusted on integrity only"
        return 0
    fi
    if ! command -v gpg >/dev/null 2>&1; then
        warn "gpg not found: SHA256SUMS.asc is not verified, so SHA256SUMS is trusted on integrity only; install gpg to check authenticity (docs/release-signing.md)"
        return 0
    fi

    _fpr="${GARNET_SIGNING_KEY_FPR:-$(pinned_signing_key "$GARNET_VERSION")}"
    [ -n "$_fpr" ] || err "no release signing key is pinned for v${GARNET_VERSION} in this installer; refusing to trust SHA256SUMS (GARNET_VERIFY_SIGNATURE=0 proceeds on integrity only)"

    _keyring="$(mktemp -d 2>/dev/null)" || err "cannot create a temporary keyring"
    if ! try_download "$_sig_url" "$_keyring/SHA256SUMS.asc"; then
        rm -rf "$_keyring"
        err "SHA256SUMS.asc is missing for v${GARNET_VERSION} (${_sig_url}); refusing an unsigned SHA256SUMS"
    fi
    if ! try_download "$GARNET_SIGNING_KEYS_URL" "$_keyring/keys.asc"; then
        rm -rf "$_keyring"
        err "cannot fetch the release signing keys from ${GARNET_SIGNING_KEYS_URL}"
    fi
    if ! gpg --homedir "$_keyring" --batch --quiet --import "$_keyring/keys.asc" >/dev/null 2>&1; then
        rm -rf "$_keyring"
        err "cannot import the release signing keys from ${GARNET_SIGNING_KEYS_URL}"
    fi
    # A detached signature is required: gpg refuses an inline-signed message when
    # given the data file, so another signed text cannot stand in for this one.
    if ! _status="$(gpg --homedir "$_keyring" --batch --status-fd 1 --verify "$_keyring/SHA256SUMS.asc" "$_sums" 2>/dev/null)"; then
        rm -rf "$_keyring"
        err "SHA256SUMS.asc does not verify against SHA256SUMS; refusing to install"
    fi
    rm -rf "$_keyring"
    # VALIDSIG carries the signing key's fingerprint and, last, its primary key's.
    printf '%s\n' "$_status" | awk -v fpr="$_fpr" '
        $1 == "[GNUPG:]" && $2 == "VALIDSIG" && ($3 == fpr || $NF == fpr) { found = 1 }
        END { exit !found }
    ' || err "SHA256SUMS.asc is not signed by the pinned release key ${_fpr}; refusing to install"
    say "SHA256SUMS signature verified (key ${_fpr})"
}

install_deb() {
    _file="$1"
    need_cmd dpkg
    say "installing $_file via dpkg"
    as_root dpkg -i "$_file" || {
        say "dpkg reported missing dependencies; attempting apt-get repair"
        as_root apt-get install -f -y
    }
}

install_rpm() {
    _file="$1"
    if command -v dnf >/dev/null 2>&1; then
        say "installing $_file via dnf"
        as_root dnf install -y "$_file"
    elif command -v yum >/dev/null 2>&1; then
        say "installing $_file via yum"
        as_root yum install -y "$_file"
    else
        err "no dnf or yum available"
    fi
}

install_pkg() {
    _file="$1"
    need_cmd installer
    say "installing $_file via /usr/sbin/installer"
    as_root installer -pkg "$_file" -target /
}

install_tar() {
    _file="$1"
    _prefix="${GARNET_PREFIX:-$HOME/.local}"
    _scratch="$(mktemp -d 2>/dev/null || printf '/tmp/garnet-install-%s' "$$")"

    mkdir -p "$_prefix/bin" "$_scratch"
    if tar -tzf "$_file" | grep -qx 'garnet'; then
        tar -xzf "$_file" -C "$_scratch" garnet
        cp "$_scratch/garnet" "$_prefix/bin/garnet"
    elif tar -tzf "$_file" | grep -qx 'bin/garnet'; then
        tar -xzf "$_file" -C "$_scratch" bin/garnet
        cp "$_scratch/bin/garnet" "$_prefix/bin/garnet"
    else
        rm -rf "$_scratch"
        err "tarball does not contain 'garnet' or 'bin/garnet'"
    fi
    rm -rf "$_scratch"

    chmod 0755 "$_prefix/bin/garnet"
    GARNET_INSTALLED_BIN="$_prefix/bin/garnet"
    say "extracted garnet into $_prefix/bin"

    case ":$PATH:" in
        *":$_prefix/bin:"*) ;;
        *) warn "$_prefix/bin is not on PATH; add it to your shell startup file" ;;
    esac
}

run_version_check() {
    if [ -n "$GARNET_INSTALLED_BIN" ] && [ -x "$GARNET_INSTALLED_BIN" ]; then
        _bin="$GARNET_INSTALLED_BIN"
    elif command -v garnet >/dev/null 2>&1; then
        _bin="garnet"
    else
        warn "installer completed but 'garnet' is not on PATH; open a new shell"
        return 0
    fi

    # Keep the exit status of `garnet --version` itself. Piping it straight
    # into `head` reported head's status and hid a binary that cannot run.
    _rc=0
    _out="$("$_bin" --version 2>&1)" || _rc=$?
    if [ "$_rc" -ne 0 ]; then
        printf '%s\n' "$_out" | head -10 >&2
        warn "'$_bin --version' failed (exit $_rc); the installed binary does not run on this host"
        return "$_rc"
    fi
    say "install complete"
    printf '%s\n' "$_out" | head -10
}

# The newest glibc 2.x the requested release's Linux assets may need, as x.
# v0.8.3 and later are built on Ubuntu 22.04 under a GLIBC_2.34 floor
# (scripts/check_glibc_floor.sh); earlier releases need 2.39.
linux_assets_glibc_minor() {
    if version_before "$GARNET_VERSION" 0 8 3; then
        printf '39'
    else
        printf '34'
    fi
}

glibc_too_old() {
    # Succeeds only when the host is Linux, reports a glibc version, and that
    # version is older than the requested release's floor. musl and unknown
    # hosts are not judged here.
    [ "$(uname -s 2>/dev/null || printf unknown)" = "Linux" ] || return 1
    _glibc="$(getconf GNU_LIBC_VERSION 2>/dev/null | awk '{print $2}')"
    [ -n "$_glibc" ] || return 1
    _glibc_major="${_glibc%%.*}"
    _glibc_rest="${_glibc#*.}"
    _glibc_minor="${_glibc_rest%%.*}"
    case "${_glibc_major}${_glibc_minor}" in
        ''|*[!0-9]*) return 1 ;;
    esac
    _glibc_floor="$(linux_assets_glibc_minor)"
    [ "$_glibc_major" -lt 2 ] || { [ "$_glibc_major" -eq 2 ] && [ "$_glibc_minor" -lt "$_glibc_floor" ]; }
}

source_install() {
    need_cmd git
    need_cmd cargo

    _prefix="${GARNET_PREFIX:-$HOME/.local}"
    _scratch="$(mktemp -d 2>/dev/null || printf '/tmp/garnet-source-%s' "$$")"
    _src="$_scratch/garnet"
    _chosen_ref=""

    if [ -n "$GARNET_SOURCE_REF" ]; then
        _candidate_refs="$GARNET_SOURCE_REF"
    else
        # Fail-closed: build ONLY the pinned release tag, never silently fall back
        # to the moving `main` branch — a user who asked for a pinned version must
        # not end up with unverified main. To build main deliberately, set
        # GARNET_SOURCE_REF=main explicitly. [Jon-gated: installer-security policy.]
        _candidate_refs="$GARNET_TAG"
    fi

    say "install  = source via cargo install --locked"

    for _ref in $_candidate_refs; do
        say "source   = ${GARNET_SOURCE_REPO_URL} (${_ref})"
        if git clone --depth 1 --branch "$_ref" "$GARNET_SOURCE_REPO_URL" "$_src"; then
            _chosen_ref="$_ref"
            break
        fi
        rm -rf "$_src"
        warn "failed to clone ${GARNET_SOURCE_REPO_URL} at ${_ref}"
    done

    if [ -z "$_chosen_ref" ]; then
        err "failed to clone ${GARNET_SOURCE_REPO_URL} at requested source refs"
    fi

    cargo install --path "$_src/garnet-cli" --root "$_prefix" --locked ||
        err "failed to install Garnet from source"

    rm -rf "$_scratch"
    GARNET_INSTALLED_BIN="$_prefix/bin/garnet"
    case ":$PATH:" in
        *":$_prefix/bin:"*) ;;
        *) warn "$_prefix/bin is not on PATH; add it to your shell startup file" ;;
    esac
}

release_install_for_format() {
    _triple="$1"
    _format="$2"
    _asset="$(asset_name "$_triple" "$_format")"
    # Not `_url`: try_download sets that global, and the signature check below
    # downloads the .asc and the keys through it.
    _asset_url="${GARNET_BASE_URL}/${_asset}"
    _dest="$(mktemp_file "$_asset")"

    _sums="$(mktemp_file sums)"
    _sums_url="${GARNET_CHECKSUM_URL:-${GARNET_BASE_URL}/SHA256SUMS}"

    trap 'rm -f "$_dest" "$_sums"' EXIT INT HUP TERM

    say "detected = ${_triple} / ${_format}"
    say "asset    = ${_asset}"
    say "fetching SHA256SUMS"
    try_download "$_sums_url" "$_sums" || return 1
    # Runs in this shell, not a $(...) subshell: a signature failure exits the
    # installer instead of reading as "assets unavailable" and falling back.
    verify_sums_signature "$_sums" "${GARNET_SIGNATURE_URL:-${_sums_url}.asc}"
    _expected_sha="$(expected_sha256 "$_sums" "$_asset")"
    [ -n "$_expected_sha" ] || return 1

    say "downloading ${_asset_url}"
    try_download "$_asset_url" "$_dest" || return 1
    verify_sha256 "$_dest" "$_expected_sha"

    # A caller's `if` suspends errexit, so failures are returned explicitly.
    case "$_format" in
        deb) install_deb "$_dest" || return 1 ;;
        rpm) install_rpm "$_dest" || return 1 ;;
        pkg) install_pkg "$_dest" || return 1 ;;
        tar) install_tar "$_dest" || return 1 ;;
        *) err "unknown package format: $_format" ;;
    esac

    run_version_check || return 1
}

release_install() {
    # err inside $(...) exits only the subshell, and a caller's `if` suspends
    # errexit, so a detection failure is returned explicitly.
    _triple="$(detect_triple)" || return 1
    _format="$(detect_format)" || return 1

    if glibc_too_old; then
        warn "host glibc ${_glibc} is older than 2.${_glibc_floor}, which the v${GARNET_VERSION} Linux release assets need; skipping them"
        return 1
    fi

    if release_install_for_format "$_triple" "$_format"; then
        return
    fi

    if [ -z "$GARNET_FORMAT" ] && [ "$_format" != "tar" ]; then
        warn "native $_format release asset unavailable; trying tarball release asset"
        release_install_for_format "$_triple" "tar"
        return
    fi

    return 1
}

main() {
    say_banner
    say "channel  = ${GARNET_CHANNEL}"
    say "version  = ${GARNET_VERSION}"
    say "mode     = ${GARNET_INSTALL_MODE}"
    say "release  = ${GARNET_BASE_URL}"

    case "$GARNET_INSTALL_MODE" in
        auto|release|source) ;;
        *) err "unsupported GARNET_INSTALL_MODE: $GARNET_INSTALL_MODE" ;;
    esac

    # Windows gets its own installer; stop before any download or fallback.
    # (GARNET_INSTALL_MODE=source still builds from source in a POSIX shell.)
    if [ "$GARNET_INSTALL_MODE" != "source" ]; then
        case "$(uname -s 2>/dev/null || printf unknown)" in
            MINGW*|MSYS*|CYGWIN*|Windows_NT)
                err "Windows detected; install from PowerShell instead: irm https://garnet-lang.org/install.ps1 | iex"
                ;;
        esac
    fi

    if [ "$GARNET_INSTALL_MODE" = "source" ]; then
        source_install
        run_version_check
        return
    fi

    if release_install; then
        return
    fi

    if [ "$GARNET_INSTALL_MODE" = "release" ] || [ "$GARNET_SOURCE_FALLBACK" = "0" ]; then
        err "release install failed; set GARNET_INSTALL_MODE=source to build from source"
    fi

    warn "release assets are unavailable; falling back to source install"
    source_install
    run_version_check
}

main "$@"
