# S32 Plan — Edition / compatibility model (two-layer)

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S32.
Map: `SLICE_PLAN_RECONCILED_OPUS_X_CODEX.md` → S32 (two-layer graft).
Branch: `codex/s32-editions-compat`.

## Resolved decisions (Jon, 2026-05-31)
1. **Canonical edition form = spec form**: `[project]` table, `edition = "v1.0"`.
   Editions are **named compatibility epochs** (Rust 2015/2018/2021 style),
   decoupled from the rolling compiler version. The legacy template form
   (`[package]` table, `edition = "garnet-0.3"`) is accepted as a **deprecated
   alias** (warn, do not break) and the shipped template is fixed to the spec
   form. **No Mini-Spec edit** — §16.3 already specifies the canonical form.
2. **Runtime-settings layer = `GARNET_DEBUG` env var** for v0.8 (Go GODEBUG
   style). No manifest `[runtime]` table yet → no spec change. The manifest
   `[runtime]` table is a future Handoff-gated addition.
3. **Scope = mechanism + invariant only.** No per-edition syntax-migration
   catalog (future, "resolution decreases past S60").

## Layer 1 — Editions (parse-time)
- New `Edition` registry (e.g. `garnet-cli/src/edition.rs`): an enum of named
  editions (`V1_0` = current/default; `Next` = a second registered edition to
  prove the mechanism), `parse(&str) -> Result<Edition, EditionError>`,
  `current()`/`default()`, and the legacy-alias mapping.
- Read `[project].edition` (alias `[package].edition`) wherever the manifest is
  already loaded (`garnet-cli/src/cmd/add.rs::read_dependency_table` neighbor /
  `cmd/run.rs` project-root walk). Unknown edition → clear hard error;
  legacy form/value → one-line deprecation warning, then proceed.
- Thread `Edition` into the parser entry (`garnet_parser::parse_*`) as an
  explicit argument (default = `V1_0` so all existing callers/examples are
  unchanged).
- **One demonstrable parse-time difference**, minimal + contained: an
  **edition-gated reserved word** — an identifier that is free in `V1_0` but
  reserved in `Next` (so `let <word> = 1` parses under V1_0, errors under Next).
  Chosen to touch only the lexer/keyword table, not the grammar.
- **One-canonical-IR invariant**: for source valid in BOTH editions, the
  produced AST is byte-identical (editions only gate the front-end surface;
  check/interp/vm never see an edition). Proven by an AST-equality test.

## Layer 2 — Runtime settings (GODEBUG-style, semantic-time)
- `garnet-cli` (or a small `runtime_settings.rs`): parse `GARNET_DEBUG=k=v,k2=v2`
  into a settings struct; unknown keys warn (forward-compat), never error.
- **One real toggle** that flips a runtime *default* without changing program
  meaning or authority (candidate: a deterministic diagnostic/printing default
  already present in the interpreter; final pick made against live code so it is
  genuinely behavioral, not cosmetic). Documented + tested.

## The load-bearing invariant + dogfood
- **Capability semantics are edition- AND toggle-invariant.** Test:
  - Build the capability manifest (`garnet-cli/src/manifest.rs::Manifest::build`)
    for a program under edition `V1_0` and under `Next` → **identical** manifest
    (same `[caps]`, same `@caps` coverage).
  - Flip a `GARNET_DEBUG` toggle → capability manifest **unchanged**.
- Dogfood block (per contract): an edition-N and edition-N+1 program produce the
  same capability manifest; a GODEBUG toggle changes a default, not the manifest.

## Crates touched (writable for this slice)
- `garnet-cli` (edition + runtime-settings parse, wiring, template fix, tests).
- `garnet-parser-v0.3` (edition arg on the parse entry + the gated reservation).
- `garnet-check-v0.3` — **read-only** (capability semantics are the invariant we
  preserve; do NOT change the cap set).
- `garnet-interp-v0.3` — only if the chosen runtime toggle lives there; minimal.

## End-state / gates
- New readiness lane `edition_compatibility` (committed-truth) + baseline regen.
- `cargo fmt`/`clippy`/`test --workspace` green; `--check-no-regression` exit 0.
- dogfood-readiness skill: fused 5/5 (grep loop); goal ledger advance `s32`.
- CHANGELOG `[Unreleased]` entry. PR as Navigata1 → dogfood pass → Chrome-merge.

## Honest scope / out of scope
- Mechanism + invariant only; per-edition migration catalog is future.
- Manifest `[runtime]` table + its spec text = future Handoff (env-var only now).
- No change to capability set/semantics — that is the invariant, not a target.

## Risks
- Parser is large; keep the edition-gated difference to the keyword table to
  bound blast radius. Default edition = V1_0 so every existing example/test is
  unaffected (validated by `cargo test --workspace`).
