# S68 Plan — capability transparency log stub

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S68 (v0.8.1 band).
Map: reconciled plan §167 (GRAFT) — transparency log + cross-language manifest seed.
Branch: `codex/s68-transparency-log`. Base: `origin/main` @ `4c48a2e` (S67).

## Approach
Append-only, BLAKE3-chained capability log (CT/Rekor in spirit). Local +
tamper-evident; honest "not a distributed/witnessed log".

## Deliverables
- `garnet-cli/src/cmd/caps_log.rs` + dispatch + help: `caps-log <file> --log <p>`
  appends an entry {index, program, caps, caps_blake3, prev_blake3} chained by
  blake3 of the prior line; `--verify <log>` recomputes the chain (exit 1 on
  break). Reuses cap_manifest + blake3.
- `C_Language_Specification/GARNET_CAPABILITY_TRANSPARENCY.md` — log + schema seed
  + honest "local stub, not Rekor".
- `garnet-cli/tests/caps_log.rs` — 2 cross-OS tests (intact verifies; tamper -> exit 1).

## Dogfood
- append 2 entries (hello, c_stat) -> verify "chain intact"; flip a byte ->
  "CHAIN BROKEN" exit 1.

## Honest scope (do not soften)
- LOCAL hash-chained STUB, NOT distributed/witnessed (no log server / signed tree
  head / witness / inclusion proof). Tamper-evidence for a local file, not Rekor.
  No new readiness lane.

## Gates
- Rust tests + ladder (workspace 0 failed; clippy clean). Ledger: `s67 →
  merged(5)` advanced; `s68` rides with S69.
