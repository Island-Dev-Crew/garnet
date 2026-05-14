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

if ! command -v gh >/dev/null 2>&1; then
  echo "gh CLI is required for release verification" >&2
  exit 3
fi

echo "verifying organization release exists: ${REPO} ${TAG}"
if ! gh release view "${TAG}" --repo "${REPO}" >/dev/null; then
  echo "release ${TAG} is not available on ${REPO}" >&2
  exit 10
fi

echo "running network-backed installer smoke in release-only mode"
GARNET_INSTALL_MODE=release GARNET_VERSION="${VERSION}" sh "${INSTALLER}"

expected_version="garnet ${SEMVER}"
echo "verifying launcher is release-backed (${expected_version})"
if ! garnet --version | grep -q "${expected_version}"; then
  echo "unexpected garnet version after installer smoke" >&2
  exit 11
fi

echo "release smoke PASSED"
