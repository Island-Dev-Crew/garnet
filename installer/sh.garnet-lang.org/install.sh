#!/bin/sh
# Garnet universal installer — v4.2 Phase 6A.
#
# Usage:
#   curl --proto '=https' --tlsv1.2 -sSf https://sh.garnet-lang.org | sh
#
# Detects the host OS and arch, downloads the corresponding package
# from https://releases.garnet-lang.org, and runs the native installer.
# Shape mirrors rustup's shell installer (https://sh.rustup.rs) —
# familiar to developers who installed Rust via rustup.
#
# This script itself is served over HTTPS only (HSTS enforced at the
# edge). The downloaded package is verified two ways:
#   1. TLS transport integrity
#   2. Content SHA-256 compared against the manifest at
#      https://releases.garnet-lang.org/<version>/SHA256SUMS
#      (and the SHA256SUMS file is itself minisigned by the project's
#      release key — see RELEASE_SIGNING.md)

set -eu

# ─── Configuration ────────────────────────────────────────────────────

GARNET_VERSION="${GARNET_VERSION:-0.4.2}"
GARNET_CHANNEL="${GARNET_CHANNEL:-stable}"
GARNET_BASE_URL="${GARNET_BASE_URL:-https://releases.garnet-lang.org}"
GARNET_PREFIX="${GARNET_PREFIX:-}"   # empty = OS-appropriate default

# ─── Wordmark (mirrors garnet-cli/src/lib.rs GARNET_WORDMARK) ──────────

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

say()  { printf 'garnet-install: %s\n' "$1"; }
warn() { printf 'garnet-install: warning: %s\n' "$1" >&2; }
err()  { printf 'garnet-install: error: %s\n' "$1" >&2; exit 1; }

need_cmd() {
    command -v "$1" >/dev/null 2>&1 || err "required command '$1' not found on PATH"
}

# ─── Detect OS + arch ─────────────────────────────────────────────────

detect_triple() {
    _uname="$(uname -s 2>/dev/null || printf unknown)"
    _arch="$(uname -m 2>/dev/null || printf unknown)"
    case "$_uname" in
        Linux)
            case "$_arch" in
                x86_64|amd64)  printf 'x86_64-unknown-linux-gnu';;
                aarch64|arm64) printf 'aarch64-unknown-linux-gnu';;
                *) err "unsupported Linux arch: $_arch";;
            esac
            ;;
        Darwin)
            case "$_arch" in
                x86_64)        printf 'x86_64-apple-darwin';;
                arm64)         printf 'aarch64-apple-darwin';;
                *) err "unsupported macOS arch: $_arch";;
            esac
            ;;
        *)
            err "unsupported OS: $_uname"
            ;;
    esac
}

# Decide which packaging format to download based on the host.
# Returns one of: "deb", "rpm", "pkg", "tar".
detect_format() {
    case "$(uname -s 2>/dev/null || printf unknown)" in
        Linux)
            if   command -v dpkg >/dev/null 2>&1;        then printf 'deb'
            elif command -v rpm  >/dev/null 2>&1;        then printf 'rpm'
            else                                               printf 'tar'
            fi
            ;;
        Darwin) printf 'pkg';;
        *) printf 'tar';;
    esac
}

# ─── Download + checksum verification ─────────────────────────────────

download() {
    _url="$1"
    _out="$2"
    if command -v curl >/dev/null 2>&1; then
        curl --proto '=https' --tlsv1.2 -fL "$_url" -o "$_out"
    elif command -v wget >/dev/null 2>&1; then
        wget --https-only -q "$_url" -O "$_out"
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
        err "SHA-256 mismatch on $_file
  expected: $_expected
  got:      $_actual
  aborting install — do not run an unverified binary"
    fi
    say "SHA-256 verified: $_file"
}

# Resolve the expected SHA-256 for $1 (asset filename) from the
# per-version SHA256SUMS manifest.
lookup_expected_sha256() {
    _asset="$1"
    _sums_url="${GARNET_BASE_URL}/${GARNET_VERSION}/SHA256SUMS"
    _tmp="$(mktemp 2>/dev/null || printf '/tmp/garnet-sums-%s' "$$")"
    download "$_sums_url" "$_tmp"
    # Format: "<sha256>  <filename>"
    _sha="$(awk -v f="$_asset" '$2==f {print $1}' "$_tmp")"
    rm -f "$_tmp"
    [ -n "$_sha" ] || err "asset '$_asset' not listed in $_sums_url"
    printf '%s' "$_sha"
}

# ─── Installers per format ────────────────────────────────────────────

install_deb() {
    _file="$1"
    need_cmd dpkg
    say "installing $_file via dpkg (sudo required)"
    sudo dpkg -i "$_file" || {
        say "dpkg reported missing dependencies; attempting apt-get fix"
        sudo apt-get install -f -y
    }
}

install_rpm() {
    _file="$1"
    if command -v dnf >/dev/null 2>&1; then
        say "installing $_file via dnf (sudo required)"
        sudo dnf install -y "$_file"
    elif command -v yum >/dev/null 2>&1; then
        say "installing $_file via yum (sudo required)"
        sudo yum install -y "$_file"
    else
        err "no dnf or yum available"
    fi
}

install_pkg() {
    _file="$1"
    need_cmd installer
    say "installing $_file via /usr/sbin/installer (sudo required)"
    sudo installer -pkg "$_file" -target /
}

install_tar() {
    _file="$1"
    _prefix="${GARNET_PREFIX:-$HOME/.local}"
    mkdir -p "$_prefix/bin"
    tar -xzf "$_file" -C "$_prefix/bin" garnet
    chmod 0755 "$_prefix/bin/garnet"
    say "extracted garnet into $_prefix/bin"
    case ":$PATH:" in
        *":$_prefix/bin:"*) ;;
        *) warn "$_prefix/bin is not on your PATH — add it to your shell rc";;
    esac
}

# ─── Main ─────────────────────────────────────────────────────────────

main() {
    say_banner
    say "channel  = ${GARNET_CHANNEL}"
    say "version  = ${GARNET_VERSION}"

    _triple="$(detect_triple)"
    _format="$(detect_format)"
    say "detected = ${_triple} / ${_format}"

    case "$_format" in
        deb)  _ext='deb'; _installer=install_deb;;
        rpm)  _ext='rpm'; _installer=install_rpm;;
        pkg)  _ext='pkg'; _installer=install_pkg;;
        tar)  _ext='tar.gz'; _installer=install_tar;;
        *)    err "unknown format: $_format";;
    esac

    _asset="garnet-${GARNET_VERSION}-${_triple}.${_ext}"
    _url="${GARNET_BASE_URL}/${GARNET_VERSION}/${_asset}"
    _dest="$(mktemp 2>/dev/null || printf '/tmp/%s' "$_asset")"
    trap 'rm -f "$_dest"' EXIT INT HUP TERM

    say "downloading ${_url}"
    download "$_url" "$_dest"

    say "fetching SHA256SUMS"
    _expected_sha="$(lookup_expected_sha256 "$_asset")"
    verify_sha256 "$_dest" "$_expected_sha"

    $_installer "$_dest"

    # Sanity check.
    if command -v garnet >/dev/null 2>&1; then
        say "install complete"
        garnet --version | head -10
    else
        warn "installer completed but 'garnet' is not on PATH; open a new shell"
    fi
}

main "$@"
