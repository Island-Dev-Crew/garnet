#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXTENSION_DIR="${ROOT}/editors/vscode"
OUTPUT_DIR="${ROOT}/target/vscode"
COPY_TO_DESKTOP=0
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"

usage() {
  cat <<'USAGE'
Usage: scripts/package_garnet_vscode_extension.sh [--output-dir DIR] [--copy-to-desktop]

Build garnet-lsp, package the VS Code extension with the bundled native server,
and write a manifest-backed evidence bundle. The VSIX filename includes the
current host target because the bundled server is platform-specific.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-dir)
      OUTPUT_DIR="$2"
      shift 2
      ;;
    --copy-to-desktop)
      COPY_TO_DESKTOP=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "required command not found: $1" >&2
    exit 2
  }
}

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
      echo "unsupported VS Code extension packaging host: ${os} ${arch}" >&2
      exit 3
      ;;
  esac
}

server_executable_name() {
  case "$(uname -s 2>/dev/null || printf unknown)" in
    MINGW*|MSYS*|CYGWIN*) printf 'garnet-lsp.exe' ;;
    *) printf 'garnet-lsp' ;;
  esac
}

need_cmd cargo
need_cmd npm
need_cmd node
need_cmd python3

VERSION="$(python3 - <<'PY'
import json
from pathlib import Path

package = json.loads(Path("editors/vscode/package.json").read_text(encoding="utf-8"))
print(package["version"])
PY
)"
TARGET="$(detect_vscode_target)"
SERVER_EXE="$(server_executable_name)"
VSIX_NAME="garnet-${VERSION}-lsp-mvp-${TARGET}.vsix"
OUTPUT_DIR="$(cd "${ROOT}" && mkdir -p "${OUTPUT_DIR}" && cd "${OUTPUT_DIR}" && pwd)"
VSIX_PATH="${OUTPUT_DIR}/${VSIX_NAME}"
EVIDENCE_DIR="${OUTPUT_DIR}/garnet-vscode-release-assets-${STAMP}"

if [[ "${COPY_TO_DESKTOP}" -eq 1 ]]; then
  EVIDENCE_DIR="${HOME}/Desktop/dogfood/garnet-vscode-release-assets-${STAMP}"
  mkdir -p "${EVIDENCE_DIR}"
  VSIX_PATH="${EVIDENCE_DIR}/${VSIX_NAME}"
else
  mkdir -p "${EVIDENCE_DIR}"
fi

echo "==> Building garnet-lsp release binary"
cargo build -p garnet-lsp --release --locked

echo "==> Installing VS Code extension dependencies"
(
  cd "${EXTENSION_DIR}"
  npm ci
)

echo "==> Compiling and packaging VS Code extension (${TARGET})"
(
  cd "${EXTENSION_DIR}"
  npm run compile
  node scripts/bundle-server.mjs
  npx vsce package --out "${VSIX_PATH}"
)

echo "==> Inspecting VSIX contents"
python3 - "${VSIX_PATH}" "${SERVER_EXE}" > "${EVIDENCE_DIR}/vsix-contents.txt" <<'PY'
import sys
from zipfile import ZipFile

vsix_path = sys.argv[1]
server_exe = sys.argv[2]
required = {
    "extension/package.json",
    "extension/dist/extension.js",
    f"extension/server/{server_exe}",
}

with ZipFile(vsix_path) as archive:
    names = sorted(archive.namelist())

for name in names:
    print(name)

missing = sorted(required.difference(names))
if missing:
    print("missing required VSIX entries:", ", ".join(missing), file=sys.stderr)
    sys.exit(4)
PY

python3 - "${ROOT}" "${VSIX_PATH}" "${EVIDENCE_DIR}" "${TARGET}" "${SERVER_EXE}" "${VERSION}" <<'PY'
import hashlib
import json
import os
import platform
import sys
from pathlib import Path

root = Path(sys.argv[1])
vsix = Path(sys.argv[2])
evidence_dir = Path(sys.argv[3])
target = sys.argv[4]
server_exe = sys.argv[5]
version = sys.argv[6]

digest = hashlib.sha256(vsix.read_bytes()).hexdigest()
manifest = {
    "status": "release-asset-ready",
    "boundary": "This is a host-native VSIX artifact. It is not Marketplace/OpenVSX publication and not v0.5.0 tag proof.",
    "repo_root": str(root),
    "version": version,
    "vscode_target": target,
    "server_executable": server_exe,
    "vsix": {
        "path": str(vsix),
        "name": vsix.name,
        "sha256": digest,
        "size_bytes": vsix.stat().st_size,
    },
    "host": {
        "system": platform.system(),
        "machine": platform.machine(),
        "platform": platform.platform(),
    },
}
(evidence_dir / "artifact-manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
(evidence_dir / "README.md").write_text(
    "# Garnet VS Code Release-Asset Evidence\n\n"
    f"- VSIX: `{vsix.name}`\n"
    f"- Target: `{target}`\n"
    f"- SHA-256: `{digest}`\n"
    "- Boundary: release-asset-ready local evidence only; this is not a v0.5.0 tag, GitHub Release asset, Marketplace publication, or OpenVSX publication.\n",
    encoding="utf-8",
)

files = sorted(p for p in evidence_dir.rglob("*") if p.is_file() and p.name != "MANIFEST.sha256")
if vsix not in files:
    files.insert(0, vsix)

entries = []
for path in files:
    entries.append(f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {path}\n")
(evidence_dir / "MANIFEST.sha256").write_text("".join(entries), encoding="utf-8")
PY

echo "vsix=${VSIX_PATH}"
echo "evidence_dir=${EVIDENCE_DIR}"
