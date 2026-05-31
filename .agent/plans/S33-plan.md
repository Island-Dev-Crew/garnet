# S33 Plan — one-command `garnet verify` (acceptance gate)

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S33.
Map: `SLICE_PLAN_RECONCILED_OPUS_X_CODEX.md` → S33 (pluggable capability signal +
min-fusion graft).
Branch: `codex/s33-garnet-verify`.

## Decision: disambiguate the two `verify`s by arg count
`garnet verify <file> <manifest.json>` (2 positional args) already exists =
deterministic-manifest verify (unchanged). S33 adds the acceptance gate as
`garnet verify <path>` (exactly 1 positional arg = a `.garnet` file or a project
dir). The dispatcher routes on positional-arg count; 2 → manifest verify, 1 →
acceptance gate. Documented in `print_help` and both usages.

## What the gate does (bounded, honest)
`garnet verify <path>`:
1. Collect targets: a single `.garnet` file, or every `.garnet` under a dir
   (excluding `target/`, vendor/dep dirs).
2. Per target: resolve project edition (S32) → parse → `garnet_check::check_module`.
   Aggregate fatal errors (parse failure / `CheckReport::ok()==false`) and
   non-fatal advisories.
3. **Internal acceptance band (1–5)** — local signal:
   - 5: every target parses + checks clean (0 diagnostics).
   - 4: clean except non-fatal advisories.
   - 1: any fatal error / parse failure.
4. **Pluggable capability signal**: a `CapabilitySignal` slot that returns
   `Pending` (a stub) until S37 `diff-caps` wires it in. Never contributes to the
   fused min while pending.
5. **External reviewer slot**: optional `--external-band <1..5>` (default None;
   in CI/PR this is Greptile). 
6. **Fused band = min** over the present signals (internal always present;
   external + capability only when supplied/wired).
7. Emit: per-target check lines + `Merge confidence (fused): N/5` + a component
   breakdown + the honest label.
8. **Exit:** 0 on a clean tree (no fatal errors), non-zero on a planted
   regression (≥1 fatal error). The band is emitted on both paths.

## Crates touched (writable)
- `garnet-cli`: new `verify_gate.rs` (band + `min`-fusion, unit-tested pure
  logic), new `cmd/verify_gate.rs` (the CLI gate, reusing edition resolution +
  `garnet_check::check_module`), dispatcher arm + `print_help`.
- `garnet-check-v0.3` — **read-only** (reuse `check_module`/`CheckReport`).

## Load-bearing dogfood (per contract)
- `garnet verify <clean-file>` → exit 0, emits band 5/5.
- `garnet verify <file-with-planted-check-error>` → exit ≠ 0, emits a degraded band.
- `--external-band 3` lowers the fused band to 3 via `min` (proves fusion).
- Capability-signal slot reports `stub until S37` and does not raise the band.

## End-state / gates
- New readiness lane `garnet_verify_gate` (committed-truth) + baseline regen.
- `cargo fmt`/`clippy -D warnings`/`test --workspace` green; `--check-no-regression`
  exit 0; `cargo doc -D warnings`; `cargo deny`.
- CHANGELOG `[Unreleased]` entry; contract S33 state updated.
- Dogfood bundle → PR (Navigata1) → CLI-merge (IslandDevCrew) → ledger advance
  `s33` rides with the S34 PR.

## Honest scope / out of scope
- "Until S37, the capability-signal slot is a stub — the fused band uses the
  internal local band (+ an optional external band) only."
- The gate's internal band is the LOCAL acceptance signal; the full PR
  falsification ledger + Greptile fusion is the `dogfood-readiness` skill's job,
  which this gate feeds. `garnet verify` does not itself run cargo/CI.
- Test-execution integration (`garnet test`) is not folded into the gate in S33
  (parse + safe-mode check is the acceptance signal); a later slice may add it.
