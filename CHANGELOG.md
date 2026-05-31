# Changelog

All notable changes to Garnet are recorded here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This file is updated in the same PR as the work it tracks (per the v0.5 slice
contract). Lines added here are part of the calibrated-honesty record — if a
slice ships labeled "partial," its CHANGELOG entry says so explicitly.

## [Unreleased] — v0.8.0 in flight

> Release-truth note (S31): only `v0.4.2` and `v0.5.0` are tagged. The `v0.6.0`
> and `v0.7.0` "in flight" labels were planning targets that never cut a tag;
> their slices (S11–S30) folded forward and ship under the next tag, **v0.8.0**
> (planned @ S60 per `GARNET_v0_8_SLICE_DOGFOOD.md`). A full CHANGELOG
> restructure + any retroactive tagging is a deferred S31 release-truth decision
> for Jon — not done in this PR.

### Added

- **S36 (capability manifest — derived from annotations):** adds
  `garnet_cli::cap_manifest::CapabilityManifest` (schema
  `garnet-capability-manifest-v1` wrapping the S35 `CapabilitySurface`) and the
  `garnet caps <path>` command. Emits **deterministic JSON** (`{schema,
  aggregate, functions:[{name,caps}], wildcard}`, reusing
  `diagnostics::json_escape`) for a single file (per-program) or every `.garnet`
  under a directory (per-package, via `merge_surfaces` — union aggregate,
  sorted+deduped `(name,caps)`, OR-ed wildcard). Distinct from the build
  `Manifest` (which carries source/AST hashes but no caps). This is the artifact
  S37 `diff-caps` compares across revisions and S38 `seal` embeds. 4 unit + 3
  integration tests (binary via `CARGO_BIN_EXE`). **Honest scope:** captures the
  **declared** surface (S35) — it does not prove the absence of undeclared
  authority (that is the sandbox job, S46) and does not enforce the project
  `[caps]` budget; per-package same-name functions across files both appear
  (honest surface, not a resolver). No new readiness lane (mandated at S37/S38).

- **S35 (source annotations — the canonical capability surface):** the
  `@caps(...)` annotation *syntax* already existed (v3.4 CapCaps —
  `Annotation::Caps`, the `Capability` enum, parsing, transitive propagation), so
  per the contract's actual words this slice adds **the surface the manifest is
  derived from**: a first-class, deterministic `garnet_check::CapabilitySurface`
  (`aggregate`, `per_function`, `has_wildcard`) via `capability_surface(module)`,
  normalized through the canonical `Capability::as_str()` (sorted + deduplicated).
  Consolidates and **fixes** `garnet trust-report`, which built its caps surface
  via `format!("{c:?}").to_lowercase()` — mislabeling `net_internal` / `Other(_)`
  / wildcard caps — so the checker and trust-report now share one normalization.
  7 unit tests; the existing trust-report tests stay green. **Honest scope:** S35
  adds the surface artifact + a consolidation/bug-fix, **not** new syntax; it
  covers top-level functions' declared caps (the diff-caps surface) — actor
  methods, transitive/effective caps, and `[caps]`-budget enforcement are out of
  scope; S36 builds the per-program/package manifest artifact from this surface.

- **S34 (structured diagnostics — machine + human):** adds the reusable
  `garnet_cli::diagnostics` module (`Severity`, `Diagnostic { severity, code,
  message, span }`) and `garnet check [--format human|json]`. `human` (default)
  is unchanged miette/Display output; `json` emits **deterministic, hand-rolled
  JSON** (no `serde`, per the `manifest.rs` determinism stance): a `diagnostics`
  array with stable per-variant `code`s (`check.caps_coverage`,
  `parse.reserved_word`, …), lowercase `severity`, an escaped `message`, and an
  optional `{start,len}` span, plus a `summary`. Documents the **authoritative
  exit code** (0 clean / 1 fatal-diagnostic-or-parse-or-IO / 2 usage). 5 unit + 4
  integration tests (binary run via `CARGO_BIN_EXE`). **Honest scope:** the
  machine form is wired into `garnet check` only — `parse`/`verify` JSON and the
  MCP transport are follow-ups (S34 ships the structured type + its first
  consumer); check diagnostics have no spans yet (the `CheckError` variants are
  message-only), parse diagnostics do.

- **S33 (one-command `garnet verify` — acceptance gate):** adds
  `garnet verify <path>` — a single acceptance gate, distinct from the existing
  2-arg `garnet verify <file> <manifest.json>` deterministic-manifest verify (the
  dispatcher routes on positional-arg count). It runs **edition-aware parse +
  safe-mode check** over a `.garnet` file or every `.garnet` under a directory and
  emits a **fused merge-confidence band**: the internal local band (5 clean / 4
  advisory / 1 fatal) fused by `min` with an optional external-reviewer band
  (`--external-band`, Greptile at PR time) and a **pluggable capability signal**.
  Exits 0 on a clean tree, non-zero on a planted regression. Adds the
  `garnet_verify_gate` readiness lane (committed-truth; headline 88.3% → 88.6%);
  baseline regenerated. **Honest scope:** the capability-signal slot is a **stub
  until S37 `diff-caps`** (it never lowers the fuse while pending); the gate's
  internal band is the LOCAL acceptance signal that feeds the `dogfood-readiness`
  skill's PR-level falsification-ledger + Greptile fusion — `garnet verify` does
  not itself run cargo/CI; `garnet test` execution is not folded into the gate in
  S33.

- **S32 (edition / compatibility model — two-layer mechanism):** installs the
  compatibility-evolution mechanism before users exist to break.
  **Layer 1 (editions, parse-time):** a `garnet_parser::Edition` registry
  (`v1.0` default + a registered `v2.0` that exists *only* to prove the
  mechanism — not a shipped language version), resolved from `[project].edition`
  in `Garnet.toml` (the legacy `[package]` table and `edition = "garnet-0.3"`
  value are accepted as **deprecated aliases** with a one-line warning; an
  unknown edition is a hard error). The single edition-gated surface difference
  is the reserved word `async` (a free identifier under `v1.0`, rejected at lex
  time under `v2.0`), confined to the lexer so the grammar and AST are untouched.
  **One-canonical-IR invariant proven:** source valid in both editions parses to
  a byte-identical AST and an identical capability manifest (`Manifest::build`
  ast_hash). **Layer 2 (runtime settings, GODEBUG-style):** `GARNET_DEBUG=k=v`
  flips a CLI default (`diagnostics=verbose`) without changing the manifest;
  unknown keys warn, never error. Wired into `garnet check` and
  `garnet run --interp`; the three shipped `garnet new` templates move to the
  canonical `[project]` / `edition = "v1.0"` form. Adds the
  `edition_compatibility` readiness lane (committed-truth; headline 88.0% →
  88.3%) and regenerates the baseline. **Honest scope (mechanism + invariant
  only):** no per-edition syntax-migration catalog; `garnet run --vm` uses the
  default edition (the VM has a separate load path, per the S12/S14 split); no
  manifest `[runtime]` table yet (env var only — a `[runtime]` table would be a
  spec change, deferred to a future Handoff); no Mini-Spec edit (§16.3 already
  specifies the canonical edition form).

- **S31 PR-2 (deterministic readiness reporter — committed-truth / local-evidence split):**
  `scripts/garnet_mit_readiness_status.py` no longer mixes machine-independent
  committed evidence with machine-variable live probes. Each lane is tagged
  `evidence_class`: **committed** lanes (scored from committed repo state;
  byte-identical on every machine) feed the headline % and the
  `--check-no-regression` gate; **local** lanes (`windows_linux_distribution`,
  `windows_linux_domain_proof_matrix`, `promo_video` — live Windows/Linux build
  gates, the `~/Desktop` domain-proof bundle, local promo render) are reported in
  a separate "Local evidence (not scored, not gated)" section. This fixes the
  cross-machine false regression (`windows_linux_distribution` 60% on a Mac vs a
  70% baseline captured on a Windows-capable machine) that broke the hand-off
  gate, and makes the headline **byte-identical on every machine** (committed-truth
  **88.0%**). Adds the `reporter_determinism` lane, regenerates the committed
  baseline (43 lanes, `evidence_class`-tagged), and advances the goal ledger
  `s31 → merged`. **Honest scope:** the split is at the aggregation layer; per-lane
  committed-vs-live decomposition inside the wls/promo sub-reporters is future
  work, and the committed docs-site `78.0%` snapshot vs the live `88.0%` is a
  separate release-truth sync, not done here.

- **S31 PR-1 (v0.8 release truth + slice ledger + readiness contract):** lands
  the v0.8 map (`F_Project_Management/SLICE_PLAN_RECONCILED_OPUS_X_CODEX.md`) and
  the per-slice acceptance contracts (`F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md`,
  S31–S40 detailed, S41–S80 banded; v0.8.0@S60 / v0.8.1@S70 / v0.8.2@S80; **1.0
  held past S80**). **Adopts** the upgraded `dogfood-readiness` skill
  (`Navigata1/dogfood-readiness`: internal×external merge-confidence fusion via
  `min`, grep-loop-to-5/5, persisted goal-mode ledger) rather than rebuilding
  readiness machinery, and **reconciles** the garnet PR-body gate
  (`scripts/check_dogfood_pr_body.py`) to accept either `### Desktop dogfood
  bundle` (legacy) or `### Evidence bundle` (skill) — backward-compatible, locked
  by 4 new/updated unit tests. Initializes the S31→S80 goal ledger at
  `.dogfood/goal.json`. **Honest scope:** doctrine + ledger + gate reconciliation
  only — no runtime behavior changes and **no new readiness lane**; the
  deterministic-reporter fix (committed-truth split, machine-independent %) and
  its `reporter_determinism` lane land in **S31-PR2**.

- **S30 (functional-core composition capstone):** with the full functional `core::`
  surface now interpreter-dispatched (S26 result, S27 option, S28 iter), proves they
  **compose** into railway-oriented pipelines from Garnet source.
  `garnet-interp-v0.3/tests/functional_core_composition.rs` exercises BOTH tracks —
  `core::iter` collect/map/fold → 20; `core::result` Ok-railway → 40 and Err-railway
  recovered via `or_else` → 0; `core::option` Some → 80 and None default → 7
  (`[20,40,0,80,7]`). The deterministic, cross-platform
  `examples/novel_07_functional_core_pipeline.garnet` composes the happy path
  (iter → result → option → `novel_07 final: 80`) and joins the novel-composition
  harness (now **7/7**); the story is in
  `C_Language_Specification/GARNET_NOVEL_COMPOSITIONS.md`. New
  `functional_core_composition` readiness lane is `verified`; MIT readiness moves
  **85.9% -> 86.2%** (42 lanes; baseline surgically extended). Additive — a new
  example + a new test + docs; no parser/CST or owned-crate source change.
  **Honest scope:** pure managed-mode compute (no host effects); the host-effect
  composition is the separate S25 capstone.
- **S29 (`@stability` error-level enforcement, opt-in):** ships the Layer Policy §4
  "error-level enforcement is v0.8" line as an opt-in. `garnet-check-v0.3` adds a
  FATAL `CheckError::StabilityError` variant (listed in `CheckReport::ok()`), and
  `stability.rs` promotes experimental/deprecated call sites from non-fatal
  advisories to that fatal error when `GARNET_STABILITY_ERRORS=1` (or `true`) is set
  — frozen stays informational, `stable` silent. The **default is unchanged
  warning-level**, so existing programs and CI stay green. Proven end-to-end through
  the CLI with **no garnet-cli change** (it already exits on `report.ok()`):
  `garnet check examples/novel_04_*.garnet` warns and exits 0, while
  `GARNET_STABILITY_ERRORS=1 garnet check …` prints `stability error:` and exits 1.
  Unit tests cover the policy (experimental/deprecated→error, frozen→info,
  default→warning, via the env-free `advise(error_mode)` core) and the `ok()` fatal
  classification. New `stability_error_enforcement` readiness lane is `verified`; MIT
  readiness moves **85.5% -> 85.9%** (41 lanes; baseline surgically extended).
  **Honest scope:** error mode is process-global via env var; per-source
  `@uses(experimental)` opt-out still needs the S17 parser-annotation handoff.
- **S28 (`core::iter` completion — zip/collect/chain):** dispatches the last three
  registered `core::iter` combinators, so **all 9** are now runnable from Garnet
  source. `stdlib_bridge.rs` adds `core::iter::zip` (pairs two arrays, stopping at
  the shorter), `core::iter::chain` (concatenates), and `core::iter::collect`
  (materializes a sequence — a `Range` expands to its integers, an Array passes
  through). `garnet-interp-v0.3/tests/core_iter_completion_dispatch.rs` proves them
  from source composed with the S21 higher-order `fold`
  (`collect(1..4)`+`collect([10,20])` chained → fold-sum 36; `zip` stops at the
  shorter → `[36,5,3,2,2]`); bridge unit tests cover each plus the non-sequence type
  error. New `core_iter_completion` readiness lane is `verified`; MIT readiness moves
  **85.1% -> 85.5%** (40 lanes; baseline surgically extended). Additive — bridges +
  tests only. **Honest scope:** `collect` materializes a `Range` or passes an Array
  through; there is no lazy iterator protocol to collect (eager `map`/`filter`
  already return arrays).
- **S27 (`core::option` combinator dispatch):** makes the 5 registered
  `core::option` primitives runnable from Garnet source (sibling of S26).
  `stdlib_bridge.rs` dispatches `core::option::{some,none,map,and_then,unwrap_or}`
  at the Value layer over `Option` Variants (Some/None) identical to the prelude
  builders — `map`/`and_then` are higher-order via `call_value`, bound qualified to
  avoid the bare-`map` collision. `garnet-interp-v0.3/tests/core_option_dispatch.rs`
  proves it from source (map over Some, None pass-through, `and_then` chain +
  short-circuit, `unwrap_or` default → `[10,7,6,8,5,99]`); bridge unit tests cover
  each combinator, the None constructor, and the non-`Option` type error. New
  `core_option_dispatch` readiness lane is `verified`; MIT readiness moves
  **84.7% -> 85.1%** (39 lanes; baseline surgically extended). Additive — bridges +
  tests only. **Honest scope:** `and_then` trusts the callee to return an `Option`
  (dynamic typing); ergonomic method syntax is a later follow-on.
- **S26 (`core::result` combinator dispatch):** makes the 6 registered
  `core::result` primitives runnable from Garnet source, continuing the S21/S22
  registry-to-runtime arc. `stdlib_bridge.rs` dispatches
  `core::result::{ok,err,map,and_then,or_else,unwrap_or}` at the Value layer over
  `Result` Variants (Ok/Err) identical to the prelude builders — `map`/`and_then`/
  `or_else` are higher-order via `call_value`, bound under their qualified names so
  `core::result::map` does not collide with the bare `map` (Map constructor) on the
  last-segment fallback. `garnet-interp-v0.3/tests/core_result_dispatch.rs` proves a
  railway-oriented pipeline from source (map over Ok, Err pass-through, `and_then`
  chain + short-circuit, `or_else` recovery, `unwrap_or` default → `[10,7,6,8,0,5,99]`);
  bridge unit tests cover each combinator plus the non-`Result` type error. New
  `core_result_dispatch` readiness lane is `verified`; MIT readiness moves
  **84.3% -> 84.7%** (38 lanes; baseline surgically extended). Additive — bridges +
  tests only; no parser/CST or stdlib-registry change. **Honest scope:** `and_then`/
  `or_else` trust the callee to return a `Result` (dynamic typing); ergonomic method
  syntax (`result.map(..)`) is a later follow-on.
- **S25 (host-effect composition capstone):** proves the runtime surfaces
  completed across S22–S24 compose end-to-end from Garnet source.
  `garnet-interp-v0.3/tests/host_effect_composition.rs` runs a `@caps(proc, fs)`
  program that captures a host command's stdout (`std::process::output`, S23),
  appends a leveled line to a file (`std::log::to_file`, S24), keeps an episodic
  Mnemos trace (`memory::episodic`, S22), reads the sink back (`read_file`), and
  binds `crypto::blake3` provenance — asserting the composed token, recall count,
  file contents, exit code, and fingerprint (cfg-selected command + unique temp
  path → deterministic on every host). The deterministic, cross-platform
  `examples/novel_06_observability_provenance_pipeline.garnet` composes the
  side-effect-free subset (json + file-sink + episodic memory + blake3) and joins
  the novel-composition harness (now **6/6**); the story is in
  `C_Language_Specification/GARNET_NOVEL_COMPOSITIONS.md`. New
  `host_effect_composition` readiness lane is `verified`; MIT readiness moves
  **83.9% -> 84.3%** (37 lanes; baseline surgically extended). Additive only — a
  new example + a new test + docs/scripts; no parser/CST or owned-crate source
  change. **Honest scope:** the deterministic example omits the platform-variable
  process step (proven in the cfg-guarded integration test); execution is still
  synchronous managed-mode (no async actor/OS-thread claim).
- **S24 (`std::log` file sink with `@caps(fs)`):** closes the S22/S23-deferred
  `std::log` file-sink line (also named in the S17 lane's deferred list).
  `garnet-stdlib`'s `log` module adds `to_file(path, level, message)`, which
  formats the same `[LEVEL] message` line as `info`/`warn`/`error`/`debug` and
  **appends** it (plus a newline, create-if-missing) to a file — so unlike the
  pure formatters it requires `@caps(fs)` — and returns the formatted line so a
  caller can both persist and use it. `registry.rs` tags `std::log::to_file`
  Layer 1 / cap `fs` / `@stability(experimental)`; `stdlib_bridge.rs` dispatches
  it (arity 3). Proof is source-level: `garnet-interp-v0.3/tests/stdlib_s24_dispatch.rs`
  has a `@caps(fs)` Garnet `main` write two lines via `std::log::to_file` then read
  them back with `read_file`, asserting ordered contents; the stdlib unit tests
  prove append-not-truncate and the IO error path. New `log_file_sink_runtime`
  readiness lane is `verified`; MIT readiness moves **83.4% -> 83.9%** (36 lanes;
  baseline surgically extended). **Honest scope:** line-append text sink only
  (no rotation, structured/JSON sinks, or async writers); capability enforcement
  remains the checker's job (registry-tagged `@caps(fs)`).
- **S23 (`std::process` structured argv + output capture):** closes the
  S22-deferred process line. `garnet-stdlib`'s `process` module adds
  `spawn_args(program, [args])` — the program and each argument are handed to the
  OS **literally**, so an argument containing spaces is not re-split the way the
  whitespace-delimited `spawn(cmdline)` does — and `output(program, [args])`,
  which runs a child to completion and captures its **stdout/stderr/exit-code**
  (the v0.7 `spawn`/`wait` pair discarded captured output and kept only the exit
  code). `spawn`/`wait`/`exit_code` are unchanged. `stdlib_bridge.rs` dispatches
  both under their qualified names; `std::process::output` returns a
  `{code, stdout, stderr}` map for ergonomic consumption. Proof is source-level:
  `garnet-interp-v0.3/tests/stdlib_s23_dispatch.rs` runs a host command from a
  `@caps(proc)` Garnet `main`, asserts the captured stdout contains the marker and
  reports the exit code, and round-trips `spawn_args` + `wait`; the stdlib unit
  tests prove (on POSIX, via the `printf "%s"` discriminator) that a spaced
  argument survives as a single argv element. New `process_runtime_completion`
  readiness lane is `verified`; MIT readiness moves **82.9% -> 83.4%** (35 lanes;
  baseline surgically extended). **Honest scope:** process stdout/stderr are
  host-dependent (line endings, locale), so the deterministic proof asserts
  substring + exit-code, not byte-exact full output; execution is still
  synchronous managed-mode (no async/OS-thread or streaming-stdout claim);
  capability enforcement remains the checker's job, unchanged.
- **S22 (Stdlib + memory runtime dispatch completion):** closes the S21-deferred
  runtime surface. The parser now accepts selected keywords **only as qualified
  path segments**, so official APIs such as `std::regex::match`,
  `std::process::spawn`, and `memory::working` are callable without making those
  words legal bare identifiers. `stdlib_bridge.rs` now dispatches `std::json`
  (parse/get/set/stringify), `std::regex` (compile/match/find_all/replace),
  `std::uuid` (v4/v5/v7), `std::env` (get/set/vars), `std::process`
  (spawn/wait/exit_code via managed `Process` + `ProcessStatus` values),
  `std::log` (formatting), and `memory::working|episodic|semantic|procedural`
  constructors that return live Mnemos `MemoryStore` handles. Proof is
  source-level, not just host-unit-level: `garnet-interp-v0.3/tests/stdlib_s22_dispatch.rs`
  drives JSON/regex/uuid/log/env/process/memory from Garnet source, and
  `examples/novel_05_s22_stdlib_memory_pipeline.garnet` is included in the
  novel-composition smoke harness (now 5/5). New `stdlib_memory_runtime_dispatch`
  readiness lane is `verified`; MIT readiness moves **82.4% -> 82.9%** (34
  lanes; baseline regenerated). **Honest scope:** `std::env` and `std::process`
  are proven in Rust integration tests rather than the deterministic novel
  example because they mutate or launch host state; `std::process::spawn` still
  uses the v0.7 whitespace-delimited command contract; `std::log` remains
  formatting-only until the v0.8 `@caps(fs)` file-sink lane.

- **S21 (Interpreter dispatch for the S17 Layer-0/1 stdlib + Mnemos × stdlib):**
  closes the S17 deferred line — the new stdlib primitives now **execute from
  Garnet source**, not just sit in the registry. `garnet-interp-v0.3/src/eval.rs`
  resolves the **fully-qualified name first** (backward-compatible — `Storage::read_block`
  → top-level still works), so primitives bind under their full path
  (`core::math::sqrt`) without colliding with bare prelude builtins that share a
  last segment (`map` = Map ctor, `ok`/`err` = Result builders). `stdlib_bridge.rs`
  adds qualified dispatch for **`core::math`** (abs/sqrt/pow/floor/ceil/round —
  dispatches `garnet_stdlib::math`), **`core::cmp`** (min/max/clamp/ordering —
  Value-level), **`core::iter`** (map/filter/fold/take/drop/enumerate — map/filter/fold
  are **higher-order via `call_value`**, i.e. first-class-function combinators
  callable from managed Garnet), and **`std::base64`** (encode/decode — dispatches
  `garnet_stdlib::base64`): 18 newly-runnable primitives. Verified live
  (`core::math::sqrt(16.0)→4`, `core::iter::map([1,2,3,4], double)→[2,4,6,8]`,
  `std::base64::encode("hi")→"aGk="`) and via the new runnable
  `examples/novel_04_dispatched_stdlib_pipeline.garnet` (deterministic;
  `garnet check` emits the expected non-fatal `@stability` warnings on the
  experimental prims, exit 0). `garnet-interp-v0.3/tests/mnemos_stdlib_combination.rs`
  composes the **four Mnemos memory kinds** (working/episodic/semantic/procedural)
  with the dispatched stdlib (BLAKE3 provenance + base64) — real memory-core ×
  stdlib at the system level. New `interp_stdlib_dispatch` readiness lane
  (`verified`); MIT readiness rises; baseline surgically extended (per-lane floors
  preserved). **Honest scope:** `std::json` / `regex` / `uuid` / `env` / `process`
  / `log` dispatch and a full managed-mode `memory::` prim family (Garnet-callable
  Mnemos with a handle Value) are **S22**; `core::cmp`/`core::iter` are Value-level
  bridges (Garnet's dynamic `Value` can't be the stdlib's monomorphic `T`), with
  the `garnet_stdlib` generics as the tested Rust reference.

- **S20 (Novel-composition dogfood + program-execution discovery):** adds three
  runnable `examples/novel_*.garnet` programs that **fuse** multiple Paper-VI
  contributions per program (the existing corpus proves each in isolation):
  `novel_01` fuses capability-budget gating + cognitively-typed memory recall +
  the researcher→synthesizer→reviewer pipeline (capability-aware, memory-gated
  agent → deterministic governance score 16); `novel_02` fuses BLAKE3 signed
  provenance + a multi-stage pipeline + determinism (content-addressed,
  tamper-evident build lineage → verified fingerprint); `novel_03` fuses a
  release-evidence gate + capability budget + provenance + memory recall into a
  multi-signal release-governance quorum (APPROVED quorum 4). `scripts/smoke_garnet_novel_compositions.py`
  (+ `test_garnet_novel_compositions.py`, 8 tests) proves all three `garnet check`
  clean and `garnet run` with **exact deterministic** output; the composition
  story is `C_Language_Specification/GARNET_NOVEL_COMPOSITIONS.md`. New
  `novel_composition_dogfood` readiness lane (`verified`); MIT readiness
  81.3% → **81.9%** (32 lanes; baseline regenerated to the full snapshot, which
  also caught up two lanes that had drifted upward). Complements (does not
  duplicate) the #232 domain proof matrix. **Honest scope:** compositions are
  **modeled deterministically** in managed mode (the proven runnable subset +
  `crypto::blake3`) — they prove the composition *shape* executes and is
  reproducible, not live runtime integration of actor mailboxes / Mnemos stores /
  Ed25519 signing (tracked separately); the new S17 Layer-0/1 stdlib primitives
  are not interpreter-dispatched yet, so these programs use the proven subset.

- **Windows/Linux Studio Domain Proof Matrix:** adds
  `scripts/smoke_garnet_studio_domain_matrix.py`, a manifest-backed
  cross-platform smoke reporter for the current executable example corpus. The
  matrix runs 20 examples through `garnet parse`, `garnet check`, and
  `garnet run`: the 10 canonical MVP domains, signed hot-reload success,
  signed hot-reload mismatch rejection, five agent toolbelt programs, and the
  three agentic design programs. The mismatch case passes only when Garnet
  rejects it with the expected BLAKE3 fingerprint diagnostic. The Tauri Studio
  Release Evidence panel now exposes this reporter as "Domain Proof Matrix";
  MIT readiness tracks the Windows/Linux distribution sub-lane with a separate
  domain-proof lane, which is only `verified` when a manifest-backed matrix
  evidence bundle is present, while keeping Linux package choice, Windows
  ARM64, signing, and winget open.

- **S15 (Trivia-preserving CST via rowan — PR-1: trait surface + stub):** new
  `garnet-cst/` crate (rowan-backed), built **cold** for the v0.7
  build-both-then-compare A/B. Publishes the stable surface S16 (LSP precision)
  targets: the full `SyntaxKind` / `GarnetLanguage: rowan::Language` binding,
  `SyntaxNode` / `SyntaxToken` aliases, the `CstNode` trait, `Parse<T>`,
  `SyntaxError`, `cst_to_source`, and `parse_cst`. In PR-1 `parse_cst` is an
  **intentionally trivial stub** (whole source as one trivia leaf —
  byte-identical round-trip, no structural parsing); the structural
  recursive-descent builder and the `cst_to_ast` projection land in PR-2.
  `u16` ⇄ `SyntaxKind` conversion is `unsafe`-free. Ships 6 round-trip /
  invariant tests + 1 doc-test + a `proptest` proving the stub round-trips any
  UTF-8 input. Registered in workspace `Cargo.toml`, the root `AGENTS.md`
  contract index, and `scripts/check-agent-contracts.py`. Adds `rowan`
  (MIT/Apache-2.0; clears `cargo deny`). **Honest scope:** #221's in-parser CST
  (`garnet-parser-v0.3/src/cst.rs`) is preserved untouched as the S15-Compare
  baseline — this rowan crate is a *second, independent* implementation; the
  canonical-CST choice is the separate S15-Compare checkpoint (Jon), not this
  PR. No readiness lane yet — the `parser_cst_migration` lane + baseline
  regeneration land in PR-2 with the substantive evidence.

- **S15 (Trivia-preserving CST via rowan — PR-2: substantive builder + `cst_to_ast`):**
  `garnet-cst` gains a **direct recursive-descent CST builder** (`builder.rs`)
  over the trivia-preserving token stream, cold from Mini-Spec v1.0 §2–§11 —
  architecturally distinct from #221's AST-projection CST (the
  build-both-then-compare A/B). `parse_cst` now produces real composite
  structure (items, signatures, blocks, the 11-level expression tower,
  patterns, types) and round-trips **byte-identically** across the canonical
  example corpus + a `proptest` over arbitrary UTF-8 (round-trip is guaranteed
  by construction — every token emitted in order, plus a trailing flush, so it
  holds even for malformed input). Adds typed-node wrappers (`nodes.rs`,
  the S16-facing surface) and `cst_to_ast` (`convert.rs`) projecting onto
  `garnet_parser::ast::Module`, validated by **span-normalized structural
  parity** vs `parse_source` across the corpus (`tests/cst_to_ast_parity.rs`).
  New Criterion bench `parse_cst_vs_ast`: the CST path measures **≈0.99× the
  AST path** (well under the 1.5× gate). New readiness lane
  `parser_cst_migration` (`verified`); MIT readiness 78.0% → 78.8%; baseline
  regenerated. **Honest scope:** error-recovery parsing is best-effort
  (round-trip always holds; structure may flatten on malformed input);
  incremental re-parsing and CST-first migration of interp/check/vm are v0.8
  (consumers stay on `parse_source`, untouched); the **canonical-CST choice is
  the separate S15-Compare checkpoint (Jon)** — this is the second of two
  independent CSTs by design, not yet reconciled.

- **S15-Compare (CST reconciliation):** canonical CST decision recorded:
  the rowan-backed `garnet-cst/` crate is the v0.7/S16 target, while #221's
  in-parser CST stays temporarily as a legacy migration oracle. The useful part
  of #221 was preserved rather than discarded: `garnet-cst/src/tokens.rs` now
  exposes `TokenInfo`, `token_infos`, `token_kind`, `token_span`, and
  `identifier_spans`, giving LSP consumers the same `TokenKind` payload +
  byte-span ergonomics on top of rowan. New
  `tests/parser_cst_token_parity.rs` proves the rowan token view matches #221's
  parser CST token stream across the example corpus, excluding the parser's
  zero-width EOF sentinel. `garnet parse --mode cst <file>` now routes through
  the canonical rowan path and reports token count, CST error count, root kind,
  and byte-identical round-trip status; default `garnet parse <file>` remains
  AST mode.

- **S17 (Stdlib expansion + layer policy + `@stability`):** codifies Garnet's
  five-layer stdlib model in `C_Language_Specification/GARNET_STDLIB_LAYER_POLICY.md`
  (layer model, promotion/deprecation policy, the `@stability` semantics table,
  and the "capability surface + spec volatility = layer assignment" first-order
  principle). Expands `garnet-stdlib` from **24 → 77 primitives**: new Layer-0
  `core::` combinators (`iter`, `result`, `option`, `cmp`, `math`) and Layer-1
  `std::` modules (`json`, `regex`, `base64`, `env`, `process`, `uuid`, `log`),
  each a real Rust host function with behavioral unit tests (138 stdlib tests).
  Every primitive now carries an explicit `Layer` + `Stability` tier in
  `registry.rs` (existing 24 → `stable`; the 53 additions → `experimental`);
  `garnet_stdlib_layer_gate.py` enforces ≥ 50 primitives and ≥ 95% explicit
  `@stability` (live: 100%). Adds a compiler-enforced `@stability` advisory in
  `garnet-check-v0.3/src/stability.rs` — calls into `experimental`/`deprecated`
  primitives warn, `frozen` is info — **non-fatal** (exit code unchanged), read
  from the registry. Adds `@caps(env)` as a known capability (for `std::env`).
  New `stdlib_layer_policy` readiness lane (`verified`); MIT readiness
  79.6% → **80.4%** (on top of S16's lane; baseline regenerated to the full
  28-lane snapshot). New deps `serde_json`/`regex`/`rand`
  (already in the lockfile) + `sha1` (RustCrypto sibling of `sha2`); `base64`
  hand-rolled. **Honest scope:** `@stability` enforcement is **warning-level**
  for backwards compat (error-level is v0.8); source-level `@stability(...)` /
  `@uses(experimental)` / `@migration(...)` on **user-defined** functions is
  **pending a parser handoff** to mac-opus (the annotation parser rejects
  unknown names today) — primitive-stability enforcement ships now, user-function
  enforcement in a follow-up; the new Layer-0/1 primitives ship as registry
  surface + Rust host impls + unit tests, while **interpreter dispatch** of them
  to Garnet source is v0.8 (`garnet-interp` is outside S17's ownership);
  Layer-2 `@garnet-lang/*` packages are S18.

- **S16 (Rowan-backed LSP precision):** `garnet-lsp` now consumes the canonical
  rowan `garnet-cst` token/span surface for rename and semantic tokens while
  preserving parser/check diagnostics. The precision smoke
  `scripts/smoke_garnet_lsp_precision.py` proves document symbols, workspace
  symbols, cross-file function rename, scoped parameter rename, three code
  actions (`Add @caps`, long-parameter refactor, inferred return type), and
  semantic-token categories for `capability`, `attribute`, and `parameter`.
  `editors/vscode` is bumped to `0.7.0`, packages
  `garnet-0.7.0-lsp-precision.vsix`, and exposes the three Garnet quick-fix
  commands. MIT readiness gains an `editor_lsp_precision` lane and moves
  78.8% -> 79.6%. **Honest scope:** precision is managed-mode only;
  cross-package rename, safe-mode precision, per-project token themes, and
  Marketplace/OpenVSX publication remain v0.8/follow-up work.

- **S13 (Registry stub v0.1):** new `garnet-registry-stub/` crate — a
  filesystem-backed registry where an `index.json` (serde) maps
  `name → version → { path, BLAKE3-per-file }` over `<name>/<version>/`
  package directories. `garnet-registry-stub build|verify` generates and
  checks the index deterministically. `garnet add --registry <location>
  <name>@<version>` (in `garnet-cli/src/cmd/add.rs`) loads the index,
  resolves the exact version, verifies every file's BLAKE3 (refusing any
  index `path` that canonicalizes outside the registry root), and vendors
  the package into `.garnet/vendor/<name>/` with a registry-shaped
  `Garnet.toml` entry + `Garnet.lock` provenance. Because the S12 resolver
  loads vendored deps at `garnet run` time, a registry-resolved
  `use <name>::*` resolves end-to-end (`examples/registry_stub_fixture/` +
  `garnet-cli/tests/registry_add.rs`, 3 integration tests; 6 stub-crate
  unit tests incl. tamper-detection + path-traversal refusal). New
  "Registry stub v0.1 (S13)" lane in `garnet_mit_readiness_status.py`
  (verified 100 %); MIT lane count 23 → 24, headline 74.3 % → 75.4 %.
  Documented in `C_Language_Specification/GARNET_REGISTRY_v0_1.md`. Honest
  deferred list: HTTP(S) transport (filesystem / `file://` only); tarball
  packaging (packages are directories); auth / accounts / publish flow;
  signature verification (the index `signature` field is reserved but
  unread); SemVer ranges (exact `<name>@<version>` only); multi-registry
  resolution; transitive dependency resolution from the registry.

- **S14 (Bytecode VM v0.2 — explicit call-frame stack + ABI v0.2):**
  `garnet-vm/src/vm.rs` now executes native function calls on an explicit,
  heap-allocated call-frame stack (`Frame` + `run_frames` + `step`) instead
  of recursing in the host (Rust) language. Before S14, deep Garnet recursion
  overflowed the Rust stack (`countdown(100000)` via `--vm` aborted with a
  stack overflow); after S14, `countdown(200000)` and mutual recursion to
  depth 500 run to completion. The codec magic is version-bumped
  `GARNVM01` → `GARNVM02` and each function record carries an explicit
  `arity` field that the deserializer cross-checks against the parameter
  vector. New `garnet run --vm --dump-lowering` flag prints the
  native/fallback ratio (`lowered: N%`);
  `examples/mvp_function_call_demo.garnet` reports `lowered: 100%`. New
  workspace test `garnet-vm/tests/function_call.rs` (8 cases: deep recursion,
  mutual recursion deep + shallow, mixed arity, nested returns, ABI v0.2
  round-trip, arity-mismatch rejection, truncation rejection). New Criterion
  bench case for the call hot path. New
  "Bytecode VM v0.2 function-call lowering (S14)" lane in
  `garnet_mit_readiness_status.py` (verified 100 %); MIT lane count 22 → 23,
  headline 73.2 % → 74.3 %. Documented in
  `C_Language_Specification/GARNET_BYTECODE_v0_2.md` (v0.1 stays for archival
  reference). Honest deferred list: tail-call optimization (each call costs
  one heap frame); closures / captured environments / dynamic-receiver
  dispatch, pattern matching, try/rescue/ensure, struct/enum constructors all
  still fall back; `and`/`or` short-circuit native lowering (Ruby-style
  operand-returning semantics need value-preserving conditional-jump + `Dup`
  opcodes); `--vm`-path vendored-dependency pre-load (the S12 resolver is
  `--interp` only); stable cross-version bytecode ABI (`GARNVM02` is
  tightened, not frozen); production native-compiler proof.

- **S12 (Package-manager resolver contract):** `garnet-cli/src/cmd/run.rs::preload_dependencies`
  reads `Garnet.toml`'s `[dependencies]` table via the new
  `garnet-cli/src/cmd/add.rs::read_dependency_table`, walks each declared
  vendor directory, and pre-loads every `.garnet` source into the
  interpreter's global environment **before** the user source is loaded.
  `Item::Use(_)` in the interpreter stays a no-op; the vendored symbols
  are already in scope by the time `use <dep>::*` is reached. New
  workspace integration test `garnet-cli/tests/run_resolver.rs` covers
  the end-to-end round trip (and a guard test that bare-file runs
  outside any project still work). Four inline unit tests in
  `garnet-cli/src/cmd/run::tests` cover the `strip_top_level_main`
  defence that prevents a vendored dep's own `main` from shadowing the
  user's entry point. New "Package-manager resolver (S12)" lane in
  `garnet_mit_readiness_status.py` (verified 100 %); MIT lane count
  21 → 22, headline 71.9 % → 73.2 %. **Closes the S3 deferred line
  on resolver** (the existing "Garnet manifest + vendored deps" lane's
  deferred list no longer mentions resolver). Honest deferred list for
  S12: qualified-path resolution (`local_lib::hello()`), remote sources,
  transitive deps, SemVer matching, workspace mode, VM-path pre-load
  (S14 will harmonize), lockfile BLAKE3 verification at run time,
  name-collision handling between deps (last-loaded wins today),
  module-scoped `use local_lib::Foo::bar` paths.

- **S11 (v0.6 slice contract scaffold):** new
  `F_Project_Management/GARNET_v0_6_SLICE_DOGFOOD.md` ports the v0.5
  contract pattern (state machine, common verification primitives,
  cross-slice gates, PR body template, integration-with-scripts table,
  honesty anchors) to v0.6 and defines the v0.6.0 release gate plus
  contracts for S12–S16. New
  `F_Project_Management/ROADMAPS/GARNET_v0_6_LANGUAGE_RUNTIME_ROADMAP.md`
  records the v0.6 thesis ("v0.5 shipped scaffolds; v0.6 makes them
  load-bearing"), the confirmed slice order, what's explicitly deferred
  to v0.7+, the target lane delta (`71.9 % / 21 lanes / 12 verified` →
  `≥ 80 % / ≥ 25 lanes / ≥ 17 verified` after S16), and v0.6 honesty
  anchors. `F_Project_Management/DOGFOOD/GARNET_DOGFOOD_READINESS_SKILL.md`
  is refreshed in place from its v0.4.2 / 86 slices / `6e945d6` pulse to
  the current v0.5.x / 87 slices / `e43d378` pulse, with both readiness
  reporters distinguished (implementation-plan vs. MIT-lane). Scaffolding
  only — no reporter lanes added (those land with their respective
  slices, matching the S0 pattern); no baseline regeneration.

- **S18 (First five Layer-2 packages — local-registry source-ready):**
  adds `tools/garnet-lang-template/` as the reusable official-package scaffold
  and `examples/garnet_lang_registry_seed/` with local filesystem-registry
  v0.1.0 seeds for `http-client`, `llm`, `cli`, `test-property`, and `log`.
  `examples/mvp_18_all_official_packages/` plus
  `scripts/smoke_garnet_lang_packages_seed.py` vendors all five packages
  through the existing S13 registry stub and runs one primitive from each.
  New readiness lane `official_packages_seed` is labeled
  `local-registry-source-ready` (85.0%). **Honest scope:** this is source and
  filesystem-registry proof, not public package publication:
  `github.com/garnet-lang/` is a Jon/manual org step and is not visible to the
  active token yet; the five external `github.com/garnet-lang/*` repos and
  their CI are still pending. Source-level `@stability(...)` remains pending
  the parser annotation handoff, so v0.7 package stability is declared in
  `Garnet.toml`/docs while package source stays runnable.

- **S19 (Compiler-as-agent LLM tier — feature-gated source-ready):** new
  `garnet-suggest-llm/` crate behind the non-default `llm` Cargo feature. The
  crate runs S10 deterministic suggestions first, builds a prompt that treats
  those findings as ground truth, emits separate `LlmSuggestion` advisories
  tagged `@stability(non-deterministic)`, and writes
  `.garnet-cache/llm-suggest-log.jsonl` with prompt hash, provider/model,
  temperature, raw response, emitted suggestions, timestamp, token budget, and
  warnings. Provider-compatible Anthropic, OpenAI, and Ollama clients use an
  explicit `LlmTransport` boundary; no API key is written to the repro log.
  `scripts/check_determinism_no_llm.py` and its CI hook fail if the
  determinism workflow ever contains `--llm`. The Paper VI Exp 3 harness ships
  at `benchmarks/paper_vi_exp3_compiler_as_agent/` with ten codebase snapshots,
  stateless/history-aware runners, and aggregate/analyze scripts. New readiness
  lane `compiler_agent_llm_tier` is labeled
  `feature-gated-source-ready` (85.0%); after the S17 merge on current
  `origin/main`, the combined live MIT readiness pulse reports 82.1%.
  **Honest scope:** this is not a shipped end-to-end CLI claim yet:
  `garnet-cli` is read-only for mac-codex, so `garnet check --suggest --llm`
  is filed as a ledger handoff; the shared `garnet-lang/llm` package trait
  waits on S18 after S17; streaming, tools/function calling, vision, and
  provider-specific edge features remain v0.8+; running Paper VI Exp 3 to
  produce h3 results is v0.7.1 work.

### Fixed

- **CHANGELOG.md merge-conflict markers:** resolved the live
  `<<<<<<< HEAD / ======= / >>>>>>> 407e6ec (S3: garnet add …)` markers
  under `[Unreleased] — v0.5.1 in flight`. Both the S7 (PR
  [#213](https://github.com/Island-Dev-Crew/garnet/pull/213)) and S3 (PR
  [#211](https://github.com/Island-Dev-Crew/garnet/pull/211)) entries
  are legitimate; S7 merged first, and the conflict against the
  already-merged S7 CHANGELOG addition slipped through PR #211's merge.
  Resolution: drop the markers, keep both entries in merge order (S7
  first, then S3). No content changes to either entry.

## [Unreleased] — v0.5.1 in flight

### Added

- **S7 (Actor OS-thread bridge / `trust-report`):** new
  `garnet trust-report <file.garnet>` command (`garnet-cli/src/cmd/trust_report.rs`)
  produces a structural trust report including the literal line
  `actors: N / threads: N`, matching the contract's dogfood grep. The
  count is structural — `garnet-actor-runtime/src/runtime.rs` already
  spawns one OS thread per actor (its header documents the
  "Spawn-and-mailbox runtime" contract); S7 lands the CLI bridge that
  surfaces what the runtime does. New
  `examples/agent_orchestrator_3thread.garnet` is the three-actor
  fixture; `garnet-cli/tests/trust_report.rs` asserts the dogfood block
  on every `cargo test --workspace`. New "Actor OS-thread bridge" lane
  in `garnet_mit_readiness_status.py` (verified 100%). Honest deferred
  list documents that live-runtime instrumentation, mailbox/Sendable
  audit, and transitive caps aggregation are out of scope. Closes
  Paper VI Contribution 4's CLI-bridge surface gap.
- **S3 (`garnet add` + Manifest Spec v0.1):** new `garnet-cli/src/cmd/add.rs`
  implements `garnet add [--name <id>] <path>` to vendor a local Garnet
  directory into `.garnet/vendor/<name>/`, update `Garnet.toml`'s
  `[dependencies]` table, and write `Garnet.lock` with BLAKE3-per-file
  hashes. Lockfile output is deterministic (alpha-sorted deps, lex-sorted
  files, lowercase hex). Format documented in
  `C_Language_Specification/GARNET_MANIFEST_v0_1.md`. New
  "Garnet manifest + vendored deps" lane in
  `garnet_mit_readiness_status.py` (verified 100%). Honest deferred list
  documents that the interpreter does NOT yet resolve `use <dep>::*` at
  `garnet run` time, remote sources / transitive deps / SemVer matching /
  workspace mode / `garnet verify-deps` are all out of scope until later
  slices.
- **S6 (Memory eviction policy benchmarks):** `garnet-memory-v0.3/benches/eviction.rs`
  is a Criterion bench harness exercising `MemoryPolicy::score` +
  `should_retain` per Mnemos kind (working / episodic / semantic /
  procedural) against a naive FIFO baseline. `scripts/garnet_memory_eviction_status.py`
  inventories per-kind coverage; `scripts/test_garnet_memory_eviction_status.py`
  asserts the harness keeps all four kinds covered with both branches.
  New "Memory eviction policy benchmarks" lane in
  `garnet_mit_readiness_status.py` (verified 100%). Closes the S6 contract
  surface and half of Paper VI Contribution 3's production-allocator gap.
  Honest deferred list documents that a fresh Criterion measurement run,
  end-to-end store-throughput benches, and the production allocator path
  itself remain separate work.
- **S4 (Formatter idempotent baseline):** `garnet-cli/tests/fmt_idempotency.rs`
  proves that two passes of `garnet fmt --stdout` over every canonical
  `examples/{mvp_,det_}*.garnet` produce byte-identical output, and that
  three runs on the same input produce identical bytes. This makes the
  S4 contract goal (deterministic, idempotent source formatter) workspace-
  test-enforced. New "Formatter idempotent baseline" lane in
  `garnet_mit_readiness_status.py` (verified 100%). Honest deferred list
  documents that AST-driven semantic formatting, comment-preserving
  round-trip, and workspace-level fmt are NOT in scope until the parser
  grows a trivia-preserving CST.

## [0.5.0] — 2026-05-20

### Added

- **v0.5.0 organization release validation:** the `v0.5.0` GitHub Release now
  exists at `13a5805250dc0777ca9212f2214fff5d07247e7b` with Linux `.deb`/`.rpm`
  packages, macOS aarch64/x86_64 CLI tarballs, unified `SHA256SUMS`, and
  darwin-arm64/linux-x64 VSIX assets from green tag workflows. Release-only
  M5 evidence is sealed at
  `/Users/idc2.0/Desktop/dogfood/garnet-v0-5-release-validation-20260520T142443Z`:
  `scripts/verify_org_release_smoke.sh` passed against the org release without
  source fallback, the installer honestly fell back from the unavailable `.pkg`
  to the aarch64 tarball, `garnet new --template cli` / `garnet test` /
  `garnet run` passed from the installed release binary, and the published
  darwin-arm64 VSIX produced the injected standalone VS Code diagnostic. This
  is still not Apple Developer ID notarization, a signed/notarized macOS `.pkg`,
  Marketplace/OpenVSX publication, or Windows/Linux target-runtime proof.
- **v0.5 macOS release tarball path:** `.github/workflows/linux-packages.yml`
  now stages macOS CLI tarballs for `aarch64-apple-darwin` and
  `x86_64-apple-darwin`, then composes one release-time `SHA256SUMS` covering
  Linux `.deb`/`.rpm` packages plus those tarballs. This closes the pre-tag
  workflow gap that would make an M5 Mac release-only installer smoke fail
  after publication. The tag-time publication and release-only smoke evidence
  are recorded in the v0.5.0 organization release validation entry above; this
  remains not a signed/notarized `.pkg`. Fresh local M5 file-backed release-mode
  evidence is sealed at
  `/Users/idc2.0/Desktop/dogfood/garnet-macos-cli-tarball-release-assets-20260520T135703Z`.
- **v0.5 release-backed VSIX path:** `scripts/package_garnet_vscode_extension.sh`
  now builds `garnet-lsp`, packages the VS Code extension with the bundled
  native server, writes host-labeled VSIX evidence, and can copy a sealed bundle
  to Desktop. `.github/workflows/vscode-extension.yml` builds those VSIX
  artifacts on PR/main/tag runs and publishes them as GitHub Release assets on
  `v*` tag pushes. `scripts/verify_org_release_smoke.sh` now fails the release
  smoke unless the matching release-backed VSIX asset exists and contains the
  extension entry point plus bundled server. The tag-time publication and
  release-backed diagnostic proof are recorded in the v0.5.0 organization
  release validation entry above; this remains not Marketplace publication or
  OpenVSX publication. Fresh M5 local evidence is sealed at
  `/Users/idc2.0/Desktop/dogfood/garnet-vscode-release-assets-20260520T133747Z`
  for `garnet-0.5.0-lsp-mvp-darwin-arm64.vsix`.
- **v0.5 release-gate evidence:** post-merge public installer source-fallback
  proof is recorded in
  `/Users/idc2.0/Desktop/dogfood/garnet-v0-5-rc-merged-20260520T121820Z`, and
  Mac-local Cursor/VSIX diagnostic proof is recorded in
  `/Users/idc2.0/Desktop/dogfood/garnet-v0-5-editor-gate-20260520T122611Z`.
  The latter includes the local `garnet-0.5.0-lsp-mvp.vsix`, installed
  `island-dev-crew.garnet@0.5.0` extension evidence, a screenshot showing
  `1 problem in this file` / `Errors: 1`, and protocol smoke JSON for
  diagnostics, hover, and go-to-definition. Clean standalone VS Code 1.121.0
  diagnostic proof is recorded in
  `/Users/idc2.0/Desktop/dogfood/garnet-v0-5-standalone-vscode-gate-20260520T130303Z`:
  the locally packaged VSIX contains `extension/server/garnet-lsp`, installs
  into isolated user-data/extensions directories, launches without
  `garnet.lsp.path`, and shows the injected syntax-error diagnostic.
- **S8 (Signed hot-reload BLAKE3 demo):** `examples/mvp_11_signed_hotreload.garnet`
  and `examples/mvp_11_signed_hotreload_mismatch.garnet` are runnable
  managed-mode demonstrations of the BLAKE3 fingerprint check that drives the
  Rust-runtime `actor.reload_signed` path. The success example exits 0 with
  `reloaded successfully` on stdout; the mismatch example exits 1 with
  `BLAKE3 fingerprint mismatch` on stderr. New "Signed hot-reload BLAKE3 demo"
  lane in `garnet_mit_readiness_status.py` (verified 100%). Honest deferred
  list documents that managed-mode `actor.reload_signed` syntax is NOT
  exposed yet — the demos use `crypto::blake3` and `raise` to reproduce the
  fingerprint-mismatch behaviour at the program level. Closes Paper VI
  Contribution 5 surface gap.
- **S10 (Compiler advisory mode, rules-based):** `garnet-check-v0.3/src/suggest.rs`
  ships a deterministic, no-LLM suggestion engine with three rules today —
  `managed-fn-missing-caps`, `long-parameter-list`, and `empty-function-body`.
  `garnet check --suggest <file.garnet>` surfaces them prefixed with the
  literal `compiler suggested:` so downstream tooling can grep. Corpus test
  `garnet-check-v0.3/tests/suggest_corpus.rs` proves ≥ 3 distinct rules fire on
  3 fixture programs. New "Compiler advisory mode (rules-based)" lane in
  `garnet_mit_readiness_status.py` (verified 100%). Closes Paper VI
  Contribution 7 surface for the rules-based tier; the LLM tier remains
  pending-infra.
- **S5 (Parser fuzz harness):** `garnet-parser-v0.3/fuzz/` cargo-fuzz
  sub-workspace with a single `parse_input` target wrapping every call to
  `garnet_parser::parse_source_with_budget` in a strict `ParseBudget`
  (4096-byte source cap, 1024-token cap, 32-depth cap, 512-byte literal
  cap). New `.github/workflows/fuzz-nightly.yml` runs `cargo +nightly
  fuzz run parse_input -- -max_total_time=3600` nightly + on-demand;
  crashes upload as artifacts for triage. Seed corpus is populated from
  canonical `examples/*.garnet` files. New "Parser fuzz harness
  (nightly)" lane in `garnet_mit_readiness_status.py` (`verified` 100%).
  `scripts/garnet_proof_benchmark_status.py` also inventories the fuzz
  harness as evidence while keeping accumulated nightly fuzz hours unclaimed.
  The fuzz sub-workspace carries explicit license metadata and a scoped
  `cargo deny --manifest-path garnet-parser-v0.3/fuzz/Cargo.toml check`
  record for `libfuzzer-sys`'s permissive NCSA component. Honest deferred list
  documents that the interpreter, checker, and archived v0.2 parser are NOT in
  scope today.
- **S9 (Determinism CI):** `.github/workflows/determinism.yml` builds
  `examples/det_fixture_01.garnet` with `garnet build --deterministic --sign
  <key>` on a matrix of ubuntu-latest and macos-latest. A `prepare-key` job
  generates a single short-lived ed25519 signing key and uploads it as an
  artifact so both OSs sign with identical key bytes; the `compare` job
  diffs the resulting per-OS SHA-256 manifest hashes and fails CI with an
  `::error::` annotation on divergence. Closes Paper VI Contribution 6
  verification gap. New "Determinism CI cross-machine" lane in
  `garnet_mit_readiness_status.py` (`verified` 100%); honest deferred list
  documents that Windows runner and Linux aarch64 are not yet in the
  cross-OS matrix.
- **S0 (housekeeping):** `scripts/garnet_conformance_matrix_check.py` — file-existence
  check on the conformance matrix's evidence column. Advisory by default; `--strict`
  opts into CI-fail behavior. Lands the gate before the existing matrix shorthand
  is repaired so future drift is catchable.
- **S0 (housekeeping):** `--check-no-regression` flag on
  `scripts/garnet_mit_readiness_status.py`. Compares live lane percentages against
  a committed baseline at
  `F_Project_Management/GARNET_v0_5_READINESS_BASELINE.json` and exits 1 on any
  drop. Lanes absent from the live output (slice removed/renamed) also trigger
  failure.
- **S0 (housekeeping):** baseline snapshot
  `F_Project_Management/GARNET_v0_5_READINESS_BASELINE.json` captured at
  54.2 % overall / 12 lanes from the 2026-05-20 main tip.
- **v0.5 slice contract:** `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md` —
  single source of truth for every v0.5 PR. State machine, dogfood blocks,
  honesty anchors, PR template.
- **S1 (LSP MVP):** source-present `garnet-lsp/` language server and
  `editors/vscode/` extension launcher for diagnostics, hover, and basic
  go-to-definition. `scripts/smoke_garnet_lsp_protocol.py` proves those paths
  over stdio; local VSIX packaging now bundles `server/garnet-lsp` from the
  release build, and local install smoke passed in Cursor plus standalone VS
  Code 1.121.0 on this Mac.
- **S2 (Bytecode VM scaffold):** source-present `garnet-vm/` crate with a
  deterministic bytecode serializer, 15 native opcode families, function-level
  tree-walk fallback, `garnet run --vm` / `--interp` dispatch, a bounded
  Criterion VM/interpreter comparison harness, and
  `C_Language_Specification/GARNET_BYTECODE_v0_1.md`. The proof/benchmark
  reporter now inventories the VM harness, and the MIT reporter's proof lane is
  more granular while the overall objective remains active-partial.

### Honest partials

- The `v0.5.0` tag and GitHub Release exist with release-backed installer and
  darwin-arm64 VSIX diagnostic evidence, but that is not proof of Apple
  Developer ID notarization, a signed/notarized macOS `.pkg`,
  Marketplace/OpenVSX publication, or Windows/Linux target-runtime evidence.
- The current Mac has Cursor as `/usr/local/bin/code`, not the standalone VS
  Code CLI. Clean standalone VS Code diagnostic proof exists through an
  isolated downloaded VS Code 1.121.0 app, including the release-backed
  darwin-arm64 VSIX installed from the GitHub Release.
- The S1 LSP slice is source-present until Marketplace/OpenVSX publication and
  full manual VSCode hover/go-to-definition screenshots are attached to later
  review/release evidence. Safe-mode hover, workspace symbols, rename, and
  CST-grade incremental precision remain deferred.
- The S2 VM is a scaffold, not a production VM. It covers 15 opcode families for
  the MVP fixtures, falls back to the tree-walk interpreter at unsupported
  function boundaries, and does not claim a stable bytecode ABI, production
  native compiler proof, full safe-mode lowering, or standing benchmark
  measurements in the status reporter.
- The S5 fuzz harness is source-present with local 60-second dogfood evidence
  and scheduled nightly coverage, not a claim that one-hour nightly fuzz has
  already accumulated or that parser correctness is proven.

### Known Advisory Gates (inherited, not yet fixed)

- Conformance matrix shorthand: 9 path-like references in
  `C_Language_Specification/GARNET_v0_4_2_Conformance_Matrix.md` do not resolve
  to files on disk today. The new check surfaces these as advisory findings; a
  future slice will fix the matrix and flip the gate to strict.

## Historical record

For the v0.4.2 (research-grade) verification ledger and earlier phase logs, see
`F_Project_Management/GARNET_v4_2_HANDOFF.md` and the dated `GARNET_v*_HANDOFF`
files. Pre-CHANGELOG history was tracked in those handoff documents; from v0.5
onward this file is the canonical entry point.
