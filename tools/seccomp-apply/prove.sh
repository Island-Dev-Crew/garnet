#!/usr/bin/env bash
# Reproduce the Garnet seccomp-apply proof on a Linux host:
#   Garnet GENERATES the @caps(fs) policy (S46) -> the harness APPLIES it as a real
#   seccomp filter -> a denied syscall (socket) is deterministically TRAPPED (EPERM),
#   while @caps(fs,net) ALLOWS socket (policy-driven, not blanket-deny).
# Requires: a Linux kernel (CONFIG_SECCOMP) + cc + libseccomp-dev + the garnet binary.
# First proven on the UTM Debian-12 ARM64 kernel — see PROOF_utm_debian12_aarch64.txt.
set -euo pipefail

if [ "$(uname -s)" != "Linux" ]; then
  echo "prove: requires a Linux kernel (seccomp); this host is $(uname -s). See PROOF_*.txt." >&2
  exit 2
fi
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
GARNET="${GARNET:-$ROOT/target/debug/garnet}"
if [ ! -x "$GARNET" ]; then
  echo "prove: build garnet first (cargo build -p garnet-cli --bin garnet), or set GARNET=." >&2
  exit 2
fi
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

cc -O2 -o "$WORK/seccomp_apply" "$ROOT/tools/seccomp-apply/seccomp_apply.c" -lseccomp

allow() {
  printf '@caps(%s)\ndef main() { 1 }\n' "$1" > "$WORK/p.garnet"
  "$GARNET" sandbox --format json "$WORK/p.garnet" \
    | python3 -c "import json,sys; print('\n'.join(json.load(sys.stdin)['seccomp']['allow']))"
}
allow "fs" > "$WORK/fs.txt"
allow "fs, net" > "$WORK/fsnet.txt"

echo "== @caps(fs): socket must be DENIED (the trap) =="
"$WORK/seccomp_apply" "$WORK/fs.txt" denied
echo "== @caps(fs, net): socket must be ALLOWED (policy-driven) =="
"$WORK/seccomp_apply" "$WORK/fsnet.txt" allowed

echo "OK: Garnet's generated seccomp policy is APPLIED and TRAPS on this Linux kernel."
