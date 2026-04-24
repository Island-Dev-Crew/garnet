#!/bin/sh
# Garnet universal installer, v0.4.2.
#
# Public bootstrap URL:
#   curl --proto '=https' --tlsv1.2 -sSf https://garnet-lang.org/install.sh | sh
#
# The script detects the host OS and architecture, downloads the matching
# native package from GitHub Releases, verifies it against SHA256SUMS, installs
# it, and runs `garnet --version`. If the release, checksum manifest, or asset
# is missing, it exits with a clear error.

set -eu

GARNET_VERSION="${GARNET_VERSION:-0.4.2}"
GARNET_CHANNEL="${GARNET_CHANNEL:-stable}"
GARNET_REPO="${GARNET_REPO:-Island-Dev-Crew/garnet}"
GARNET_TAG="${GARNET_TAG:-v${GARNET_VERSION}}"
GARNET_BASE_URL="${GARNET_BASE_URL:-https://github.com/${GARNET_REPO}/releases/download/${GARNET_TAG}}"
GARNET_PREFIX="${GARNET_PREFIX:-}"
GARNET_FORMAT="${GARNET_FORMAT:-}"
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
        Darwin) printf 'pkg' ;;
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

download() {
    _url="$1"
    _out="$2"

    case "$_url" in
        file://*)
            _path="${_url#file://}"
            [ -f "$_path" ] || err "missing local release file: $_path"
            cp "$_path" "$_out" || err "failed to copy $_path"
            return
            ;;
    esac

    if command -v curl >/dev/null 2>&1; then
        curl --proto '=https' --tlsv1.2 -fL "$_url" -o "$_out" ||
            err "failed to download $_url"
    elif command -v wget >/dev/null 2>&1; then
        wget --https-only -q "$_url" -O "$_out" ||
            err "failed to download $_url"
    else
        err "need curl or wget to download $_url"
    fi
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

lookup_expected_sha256() {
    _asset="$1"
    _sums_url="${GARNET_CHECKSUM_URL:-${GARNET_BASE_URL}/SHA256SUMS}"
    _tmp="$(mktemp_file sums)"

    download "$_sums_url" "$_tmp"
    _sha="$(awk -v f="$_asset" '
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
    ' "$_tmp")"
    rm -f "$_tmp"

    [ -n "$_sha" ] || err "asset '$_asset' is not listed in $_sums_url"
    printf '%s' "$_sha"
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
    if command -v garnet >/dev/null 2>&1; then
        say "install complete"
        garnet --version | head -10
    elif [ -n "$GARNET_INSTALLED_BIN" ] && [ -x "$GARNET_INSTALLED_BIN" ]; then
        say "install complete"
        "$GARNET_INSTALLED_BIN" --version | head -10
    else
        warn "installer completed but 'garnet' is not on PATH; open a new shell"
    fi
}

main() {
    say_banner
    say "channel  = ${GARNET_CHANNEL}"
    say "version  = ${GARNET_VERSION}"
    say "release  = ${GARNET_BASE_URL}"

    _triple="$(detect_triple)"
    _format="$(detect_format)"
    _asset="$(asset_name "$_triple" "$_format")"
    _url="${GARNET_BASE_URL}/${_asset}"
    _dest="$(mktemp_file "$_asset")"

    trap 'rm -f "$_dest"' EXIT INT HUP TERM

    say "detected = ${_triple} / ${_format}"
    say "asset    = ${_asset}"
    say "fetching SHA256SUMS"
    _expected_sha="$(lookup_expected_sha256 "$_asset")"

    say "downloading ${_url}"
    download "$_url" "$_dest"
    verify_sha256 "$_dest" "$_expected_sha"

    case "$_format" in
        deb) install_deb "$_dest" ;;
        rpm) install_rpm "$_dest" ;;
        pkg) install_pkg "$_dest" ;;
        tar) install_tar "$_dest" ;;
        *) err "unknown package format: $_format" ;;
    esac

    run_version_check
}

main "$@"
