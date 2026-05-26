#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GARNET_BIN="${GARNET_BIN:-garnet}"
LLM_PROVIDER="${LLM_PROVIDER:-anthropic}"
OUT_DIR="${OUT_DIR:-$ROOT/out/history-aware}"
HISTORY_DIR="$OUT_DIR/.garnet-cache"

mkdir -p "$OUT_DIR" "$HISTORY_DIR"
: > "$HISTORY_DIR/episodes.log"

if [[ "${GARNET_EXP3_EXECUTE:-0}" != "1" ]]; then
  echo "harness-only: set GARNET_EXP3_EXECUTE=1 to run provider-backed history-aware trials"
  for snapshot in "$ROOT"/codebase_versions/v*/main.garnet; do
    echo "GARNET_CACHE_DIR=$HISTORY_DIR $GARNET_BIN check --suggest --llm $LLM_PROVIDER $snapshot"
  done
  exit 0
fi

for snapshot in "$ROOT"/codebase_versions/v*/main.garnet; do
  name="$(basename "$(dirname "$snapshot")")"
  log="$OUT_DIR/$name.jsonl"
  GARNET_CACHE_DIR="$HISTORY_DIR" "$GARNET_BIN" check --suggest --llm "$LLM_PROVIDER" "$snapshot" > "$OUT_DIR/$name.out" 2> "$OUT_DIR/$name.err"
  printf '{"snapshot":"%s","stdout":"%s","stderr":"%s"}\n' "$name" "$OUT_DIR/$name.out" "$OUT_DIR/$name.err" >> "$HISTORY_DIR/episodes.log"
  if [[ -f "$HISTORY_DIR/llm-suggest-log.jsonl" ]]; then
    tail -1 "$HISTORY_DIR/llm-suggest-log.jsonl" >> "$log"
  else
    printf '{"snapshot":"%s","status":"missing-llm-log"}\n' "$name" >> "$log"
  fi
done
