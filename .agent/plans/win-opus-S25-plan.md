# win-opus — S25 Plan: Host-effect composition capstone

**Slot:** win-opus · **Slice:** S25 (Jon-directed; capstone of the S22–S24 arc) ·
**Branch:** `agent-win-opus/s25-host-effect-capstone` (off `origin/main` `3015ace`, S24 merged)
**Baseline:** readiness 83.9% / 36 lanes, `--check-no-regression` exit 0.

## Goal

Prove the runtime surfaces completed across S22–S24 **compose end-to-end** from
Garnet source into one capability-checked agent pipeline, and tell the story.
No new language/stdlib surface — this is a composition + evidence slice (the S20
"novel composition + story" pattern, now with the real host effects).

## Scope (additive only)

- `examples/novel_06_observability_provenance_pipeline.garnet` — a deterministic,
  cross-platform `@caps(fs)` program fusing `std::json` (S22) + `std::log::to_file`
  (S24, durable file sink) + `memory::episodic` (S22) + `crypto::blake3` provenance.
  A combination none of novel_01..05 use. Added to the novel-composition harness
  (6/6). Logs to the gitignored `.garnet-cache/`; the asserted output is derived
  from formatter return values + a blake3 over a fixed JSON string (byte-stable).
- `garnet-interp-v0.3/tests/host_effect_composition.rs` — the marquee proof: a
  `@caps(proc, fs)` program that captures a host command's stdout
  (`std::process::output`, S23), appends a leveled line to a unique temp file
  (`std::log::to_file`, S24), keeps an episodic Mnemos trace (S22), reads the sink
  back (`read_file`), and binds `crypto::blake3` provenance — asserting the composed
  token, recall count, file contents, exit code, and fingerprint. cfg-selected
  command + unique temp path → deterministic on every host; cleans up.
- Story: `C_Language_Specification/GARNET_NOVEL_COMPOSITIONS.md` novel_06 section +
  the "durable, capability-checked observability" narrative.
- `host_effect_composition` readiness lane; baseline surgically extended.
- CHANGELOG, CURRENT_STATE, S25 contract block, ledger (S24 MERGED `3015ace` + S25
  STARTED). **No parser/CST/owned-crate source change** — only a new example, a new
  test, and additive docs/scripts.

## Test proportion (~60/40)
"Code" = the novel_06 program + the integration-test scaffolding. "Test" = the two
behavioral integration tests (process→log→memory→read-back→provenance composition,
asserted) + the novel-harness exact-output case + check-clean assertions.

## Novel discovery
Managed Garnet can emit **durable, capability-checked observability** for an agent
pipeline (logs persisted to disk) AND bind content-addressed provenance to the data
it processed — the loop from S23's process-output capture to S24's file sink, gated
by `@caps`. The auditable agent run: what it did is on disk; what it processed is
fingerprinted.

## Honest scope
- The deterministic novel example omits the process step (host command + output are
  platform-variable); the process leg is proven in the cfg-guarded integration test.
- Still synchronous managed-mode; no async actor / OS-thread runtime claim.

## Dogfood block
```bash
cargo build -p garnet-cli
cargo test -p garnet-interp --test host_effect_composition --no-fail-fast
garnet run examples/novel_06_observability_provenance_pipeline.garnet
python3 scripts/smoke_garnet_novel_compositions.py            # 6/6
python3 -m unittest scripts.test_garnet_novel_compositions
cargo fmt --all -- --check ; cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
```
