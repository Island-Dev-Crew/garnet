#!/usr/bin/env bash
# S104 — reproduce the Garnet ultrapunch end-to-end:
#   ACCEPT (the 4 trust artifacts + a verified transparency-log chain) and
#   REJECT  (a capability widening is refused and NEVER sealed — the punch).
# Exit 0 iff the ultrapunch reproduces; non-zero otherwise. Honest scope: accepted
# on capability + depth evidence only (@caps + @max_depth enforced); the agent is
# simulated/scripted. See C_Language_Specification/GARNET_ULTRAPUNCH.md.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
GARNET="${GARNET:-$ROOT/target/debug/garnet}"
if [ ! -x "$GARNET" ]; then
  echo "reproduce-ultrapunch: build garnet first: cargo build -p garnet-cli --bin garnet" >&2
  exit 2
fi
F="$ROOT/garnet-cli/tests/fixtures/ultrapunch"
OUT="$(mktemp -d)"
trap 'rm -rf "$OUT"' EXIT

echo "== ACCEPT: a safe agent proposal =="
"$GARNET" agent-loop \
  --baseline "$F/baseline.garnet" --proposal "$F/accept_proposal.garnet" \
  --seal-out "$OUT/accept-seal.json" --record-dir "$OUT/accept" \
  --attest agent=scripted-agent-v1 --attest model=simulated --gate-version dogfood-gate-v1
for artifact in capability_manifest.json diff_caps.txt seal.json transparency_log.jsonl decision.md; do
  if [ ! -f "$OUT/accept/$artifact" ]; then
    echo "FAIL: accept did not emit trust artifact: $artifact" >&2
    exit 1
  fi
done
if ! "$GARNET" caps-log --verify "$OUT/accept/transparency_log.jsonl" >/dev/null; then
  echo "FAIL: the transparency-log chain did not verify" >&2
  exit 1
fi

echo "== REJECT: a capability widening (the punch) =="
if "$GARNET" agent-loop \
  --baseline "$F/baseline.garnet" --proposal "$F/reject_widen.garnet" \
  --seal-out "$OUT/reject-seal.json" --record-dir "$OUT/reject" >/dev/null 2>&1; then
  echo "FAIL: a capability-widening proposal was NOT refused" >&2
  exit 1
fi
if [ -f "$OUT/reject/seal.json" ]; then
  echo "FAIL: a widening proposal was sealed (it must never reach attestation)" >&2
  exit 1
fi

echo "OK: ultrapunch reproduced — accept sealed (4 artifacts, chain verified); widening refused, never sealed."
echo "    (accepted on capability + depth evidence; @bounded/memory/time/mailbox/OS-sandbox remain declared-not-enforced)"
