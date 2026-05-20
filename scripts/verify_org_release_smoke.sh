#!/usr/bin/env bash
set -euo pipefail

REPO="${REPO:-Island-Dev-Crew/garnet}"
VERSION="${VERSION:-0.5.0}"
SEMVER="${VERSION#v}"
TAG="${TAG:-v${SEMVER}}"
INSTALLER="${INSTALLER:-installer/sh.garnet-lang.org/install.sh}"
SKIP_VSIX_CHECK="${GARNET_SKIP_VSIX_CHECK:-0}"

if [[ ! -x "$INSTALLER" && ! -f "$INSTALLER" ]]; then
  echo "install script not found: ${INSTALLER}" >&2
  exit 2
fi

detect_vscode_target() {
  local os arch
  os="$(uname -s 2>/dev/null || printf unknown)"
  arch="$(uname -m 2>/dev/null || printf unknown)"

  case "${os}:${arch}" in
    Darwin:arm64|Darwin:aarch64) printf 'darwin-arm64' ;;
    Darwin:x86_64|Darwin:amd64) printf 'darwin-x64' ;;
    Linux:x86_64|Linux:amd64) printf 'linux-x64' ;;
    Linux:aarch64|Linux:arm64) printf 'linux-arm64' ;;
    MINGW*:x86_64|MSYS*:x86_64|CYGWIN*:x86_64) printf 'win32-x64' ;;
    MINGW*:aarch64|MSYS*:aarch64|CYGWIN*:aarch64|MINGW*:arm64|MSYS*:arm64|CYGWIN*:arm64) printf 'win32-arm64' ;;
    *)
      echo "unsupported VSIX smoke host: ${os} ${arch}" >&2
      exit 13
      ;;
  esac
}

server_entry_for_vscode_target() {
  case "$1" in
    win32-*) printf 'extension/server/garnet-lsp.exe' ;;
    *) printf 'extension/server/garnet-lsp' ;;
  esac
}

echo "verifying organization release exists: ${REPO} ${TAG}"
release_available=0
if command -v gh >/dev/null 2>&1 && gh release view "${TAG}" --repo "${REPO}" >/dev/null 2>&1; then
  release_available=1
else
  echo "gh CLI check unavailable or failed; checking public API directly"
  status="$(curl -sS -o /tmp/garnet-release-check.json -w '%{http_code}' "https://api.github.com/repos/${REPO}/releases/tags/${TAG}" || true)"
  if [ "${status}" != "200" ]; then
    if [ "${status}" = "404" ]; then
      echo "release ${TAG} is not available on ${REPO}" >&2
      exit 10
    fi
    echo "release check failed (HTTP ${status}); could not verify availability" >&2
    exit 12
  fi
  release_available=1
fi

if [ "${release_available}" -ne 1 ]; then
  echo "release ${TAG} is not available on ${REPO}" >&2
  exit 10
fi

echo "running network-backed installer smoke in release-only mode"
SMOKE_PREFIX="${GARNET_PREFIX:-$(mktemp -d "${TMPDIR:-/tmp}/garnet-release-smoke.XXXXXX")}"
CHECKSUM_URL="${GARNET_CHECKSUM_URL:-https://github.com/${REPO}/releases/download/${TAG}/SHA256SUMS?garnet_smoke=$(date +%s)}"
GARNET_CHECKSUM_URL="${CHECKSUM_URL}" GARNET_REPO="${REPO}" GARNET_PREFIX="${SMOKE_PREFIX}" GARNET_INSTALL_MODE=release GARNET_VERSION="${VERSION}" sh "${INSTALLER}"

expected_version="garnet ${SEMVER}"
echo "verifying launcher is release-backed (${expected_version})"
if [ -x "${SMOKE_PREFIX}/bin/garnet" ]; then
  "${SMOKE_PREFIX}/bin/garnet" --version | grep -q "${expected_version}" || {
    echo "unexpected garnet version after installer smoke" >&2
    exit 11
  }
elif command -v garnet >/dev/null 2>&1; then
  garnet --version | grep -q "${expected_version}" || {
    echo "unexpected garnet version after installer smoke" >&2
    exit 11
  }
else
  echo "garnet launcher not found after installer smoke" >&2
  exit 11
fi

if [ "${SKIP_VSIX_CHECK}" = "1" ]; then
  echo "skipping release-backed VSIX check because GARNET_SKIP_VSIX_CHECK=1"
else
  echo "verifying release-backed VSIX asset"
  vsix_target="${GARNET_VSIX_TARGET:-$(detect_vscode_target)}"
  vsix_asset="${GARNET_VSIX_ASSET:-garnet-${SEMVER}-lsp-mvp-${vsix_target}.vsix}"
  vsix_url="${GARNET_VSIX_URL:-https://github.com/${REPO}/releases/download/${TAG}/${vsix_asset}?garnet_smoke=$(date +%s)}"
  vsix_tmp="$(mktemp "${TMPDIR:-/tmp}/garnet-vsix-smoke.XXXXXX")"
  contents_tmp="$(mktemp "${TMPDIR:-/tmp}/garnet-vsix-contents.XXXXXX")"

  curl -fsSL "$vsix_url" -o "$vsix_tmp" || {
    echo "release-backed VSIX asset is not available: ${vsix_asset}" >&2
    rm -f "$vsix_tmp" "$contents_tmp"
    exit 14
  }

  unzip -l "$vsix_tmp" > "$contents_tmp" || {
    echo "downloaded VSIX is not a readable zip archive: ${vsix_asset}" >&2
    rm -f "$vsix_tmp" "$contents_tmp"
    exit 15
  }

  grep -q "extension/package.json" "$contents_tmp" || {
    echo "VSIX missing extension/package.json: ${vsix_asset}" >&2
    rm -f "$vsix_tmp" "$contents_tmp"
    exit 16
  }
  grep -q "extension/dist/extension.js" "$contents_tmp" || {
    echo "VSIX missing extension/dist/extension.js: ${vsix_asset}" >&2
    rm -f "$vsix_tmp" "$contents_tmp"
    exit 16
  }
  server_entry="$(server_entry_for_vscode_target "$vsix_target")"
  grep -q "$server_entry" "$contents_tmp" || {
    echo "VSIX missing bundled server ${server_entry}: ${vsix_asset}" >&2
    rm -f "$vsix_tmp" "$contents_tmp"
    exit 16
  }
  rm -f "$vsix_tmp" "$contents_tmp"
fi

if [ -n "${GARNET_PREFIX:-}" ]; then
  :
else
  rm -rf "${SMOKE_PREFIX}"
fi

echo "release smoke PASSED"
