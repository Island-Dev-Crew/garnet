#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GARNET_BIN="${GARNET_BIN:-garnet}"
LLM_PROVIDER="${LLM_PROVIDER:-anthropic}"
OUT_DIR="${OUT_DIR:-$ROOT/out/stateless}"

mkdir -p "$OUT_DIR"

if [[ "${GARNET_EXP3_EXECUTE:-0}" != "1" ]]; then
  echo "harness-only: set GARNET_EXP3_EXECUTE=1 to run provider-backed stateless trials"
  for snapshot in "$ROOT"/codebase_versions/v*/main.garnet; do
    echo "$GARNET_BIN check --suggest --llm $LLM_PROVIDER $snapshot"
  done
  exit 0
fi

for snapshot in "$ROOT"/codebase_versions/v*/main.garnet; do
  name="$(basename "$(dirname "$snapshot")")"
  log="$OUT_DIR/$name.jsonl"
  "$GARNET_BIN" check --suggest --llm "$LLM_PROVIDER" "$snapshot" > "$OUT_DIR/$name.out" 2> "$OUT_DIR/$name.err"
  if [[ -f .garnet-cache/llm-suggest-log.jsonl ]]; then
    tail -1 .garnet-cache/llm-suggest-log.jsonl >> "$log"
  else
    printf '{"snapshot":"%s","status":"missing-llm-log"}\n' "$name" >> "$log"
  fi
done
