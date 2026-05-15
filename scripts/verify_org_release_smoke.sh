#!/usr/bin/env bash
set -euo pipefail

REPO="${REPO:-Island-Dev-Crew/garnet}"
VERSION="${VERSION:-0.4.2}"
SEMVER="${VERSION#v}"
TAG="${TAG:-v${SEMVER}}"
INSTALLER="${INSTALLER:-installer/sh.garnet-lang.org/install.sh}"

if [[ ! -x "$INSTALLER" && ! -f "$INSTALLER" ]]; then
  echo "install script not found: ${INSTALLER}" >&2
  exit 2
fi

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

if [ -n "${GARNET_PREFIX:-}" ]; then
  :
else
  rm -rf "${SMOKE_PREFIX}"
fi

echo "release smoke PASSED"
