# S15 Plan — Trivia-Preserving CST via rowan (`garnet-cst`)

| Field | Value |
|---|---|
| Slot | mac-opus (Claude Code Opus 4.7 1M, macOS) |
| Slice | S15 |
| PRD | `F_Project_Management/PRD_A_S15_CST_MIGRATION.md` |
| Contract | `F_Project_Management/GARNET_v0_7_SLICE_DOGFOOD.md` → §S15 |
| Baseline (main tip `c1fc957`) | `cargo test --workspace` 96 suites ok / 0 failed (exit 0); `cargo clippy --workspace --all-targets -D warnings` exit 0; readiness 78.0% |
| PR count | 2 (PR-1 trait stub, PR-2 substantive) — PRD §"Trait Publication Protocol" |

---

## 0. Build-both-then-compare boundary (PRD §"v0.7 directive", READ FIRST)

This is an **A/B build**, not an override. #221 already merged a hand-rolled
in-parser CST at `garnet-parser-v0.3/src/cst.rs` (+ `tests/cst_round_trip.rs`).
My independence contract for S15:

- **Do NOT read, copy, extend, or delete `garnet-parser-v0.3/src/cst.rs`.** It is
  preserved untouched as the S15-Compare baseline.
- **Do NOT touch `parse_source_cst` / `parse_source_cst_with_budget`** in
  `garnet-parser-v0.3/src/lib.rs` (those drive #221's CST).
- Build the rowan `garnet-cst` crate **cold from the Mini-Spec grammar** (§2–§11,
  already read). The CST *structure* is designed without reference to #221.
- **Allowed shared surfaces (additive, not a violation of independence):** reuse
  `garnet_parser::lex_source*` (already `pub`, trivia-preserving) for tokenization,
  and target `garnet_parser::ast::Module` as the `cst_to_ast` output type. These
  are the lexer and the AST contract — shared by the language, not #221's CST.
- **Reconciliation is NOT my job.** The keep/merge/discard decision is the
  separate S15-Compare checkpoint (Jon, fresh eyes). I stop at dogfood-passing.

---

## 1. Architecture

New crate `garnet-cst/`, depending on `rowan` + `garnet-parser` (path). Dependency
direction is **garnet-cst → garnet-parser** (never the reverse — avoids a cycle and
keeps the parser's AST path pristine).

```
garnet-cst/
  Cargo.toml          # deps: rowan, garnet-parser (path); dev: proptest, criterion
  AGENTS.md           # trait surface, converter, stability tier (PRD §8)
  src/
    lib.rs            # re-exports; parse_cst, Parse<T>, SyntaxError, cst_to_source
    syntax_kind.rs    # SyntaxKind enum + GarnetLanguage: rowan::Language
    nodes.rs          # CstNode trait + typed node wrappers (PR-2)
    builder.rs        # token stream -> GreenNodeBuilder -> SyntaxNode (PR-2)
    convert.rs        # cst_to_ast(&SyntaxNode) -> Module (PR-2)
  tests/
    roundtrip.rs            # proptest: cst_to_source(parse_cst(s)) == s  (>=1000 iters)
    examples_roundtrip.rs   # byte-identical roundtrip over examples/*.garnet
    cst_to_ast_parity.rs    # cst_to_ast(parse_cst(s)) == parse_source(s) on corpus
  benches/
    parse_cst_vs_ast.rs     # criterion; CST path vs AST path on mvp_* examples
```

**Why roundtrip is the easy guarantee and structure is the hard one:** rowan's
`SyntaxNode::text()` concatenates every token (trivia included) in source order.
So as long as the builder emits *every* lexer token (including `Whitespace`,
`Comment`, `Newline`) into the green tree in order, `cst_to_source == input` is
byte-exact for free. The real engineering is the *composite nesting* (Module →
items → blocks → exprs …) that makes the CST useful to the LSP, and the
`cst_to_ast` projection that proves the nesting is faithful.

---

## 2. PR-1 — `S15: garnet-cst trait surface + stub` (small, opens first)

Branch `agent-mac-opus/s15-cst-trait-stub`. Publishes the stable surface S16 will
target; ships a deliberately trivial impl. Per PRD §3.

- `syntax_kind.rs`: the **full** `SyntaxKind` enum (real, not stubbed — it is the
  language definition, mapped 1:1 from `token::TokenKind` + composite node kinds +
  `Root` + `Error`), and `GarnetLanguage: rowan::Language` with
  `kind_from_raw`/`kind_to_raw`. Defining it fully in PR-1 makes the trait surface
  stable so S16 never has to chase enum churn.
- `lib.rs`:
  - `pub trait CstNode { fn syntax(&self) -> &SyntaxNode; fn kind(&self) -> SyntaxKind; }`
  - `pub struct Parse<T> { pub root: T, pub errors: Vec<SyntaxError> }`
  - `pub struct SyntaxError { message, span }`
  - `pub fn parse_cst(input: &str) -> Parse<SyntaxNode>` — **STUB**: a single `Root`
    node wrapping the entire source as one trivia token. (Roundtrips trivially.)
  - `pub fn cst_to_source(node: &SyntaxNode) -> String` (`node.text().to_string()`).
  - `pub type SyntaxNode`, `pub type SyntaxToken`.
- `tests/roundtrip.rs`: stub-level roundtrip (whole-source-as-trivia) + a couple of
  unit assertions on `SyntaxKind` mapping.
- Stability tier documented in a doc comment + `AGENTS.md` as `experimental`
  (the compiler `@stability(...)` annotation is S17/win-opus — wired after S17
  merges; see Risk R3).
- Cross-cutting: add `garnet-cst` to workspace `Cargo.toml` members; CHANGELOG
  `[Unreleased]` bullet; ledger PR-OPEN/MERGED entries.

Target merge: within 24h of open (PRD §"Trait Publication Protocol").

## 3. PR-2 — `S15: trivia-preserving CST via rowan` (substantive)

Branch `agent-mac-opus/s15-cst-rowan`. Replaces the stub with the real builder.

- `builder.rs`: a **second, independent recursive-descent parser** that consumes
  the full token stream (`garnet_parser::lex_source`, trivia included) and drives
  `rowan::GreenNodeBuilder`. Built cold from Mini-Spec §2–§11. Trivia tokens are
  attached as leaf children in source order (standard rowan approach). Covers the
  grammar productions exercised by the canonical corpus: module/use/const, `def`/`fn`,
  params, blocks, statements (`let`/`var`/`const`, control flow, `match`, try/rescue),
  expression grammar with the §2.3 precedence table, patterns, `struct`/`enum`/
  `trait`/`impl`/`type`, `memory`, `actor`/`on`/`protocol`.
- `nodes.rs`: typed wrappers (`Module`, `FnDef`, `ParamList`, `Block`, `ExprStmt`,
  …) implementing `CstNode`, giving S16 ergonomic typed accessors.
- `convert.rs`: `cst_to_ast(&SyntaxNode) -> Module` — the lossy-on-trivia,
  lossless-on-structure projection (PRD §5).
- `tests/`: proptest roundtrip ≥1000 iters; examples roundtrip (byte-identical);
  `cst_to_ast` parity vs `parse_source` on the corpus.
- `benches/parse_cst_vs_ast.rs`: criterion on `mvp_*` examples; commit numbers;
  ≤1.5× AST target, **ship anyway if slower** and document in CHANGELOG (PRD §7).
- `garnet-cst/AGENTS.md` (PRD §8).

### Fidelity bars (calibrated honesty — what "shipped" means)

| Property | Bar | Confidence | If short → honest label |
|---|---|---|---|
| Source preservation (roundtrip) | 1000/1000 proptest + all `examples/*.garnet` byte-identical | high (free via rowan) | — |
| Composite nesting | full for the canonical corpus + parser test corpus | medium-high | "structurally flat for `<production>`; nesting deferred to v0.8" |
| `cst_to_ast` ≡ `parse_source` | green on canonical corpus | medium | "converter covers the corpus; exotic/error-recovery inputs may diverge — best-effort" |
| Error recovery | best-effort; not claimed | — | "recovery from malformed input is best-effort and may diverge" (already an approved anchor) |

Existing consumers (interp/check/vm) are **untouched** — they keep calling
`parse_source`. `cst_to_ast` is additive and validated only inside `garnet-cst`.
This is how "no regression for interp/check/vm" holds.

---

## 4. Cross-cutting edits (per ledger §"Cross-Cutting Files")

- `Cargo.toml` (workspace): add `"garnet-cst",` grouped with the `garnet-c*`
  members. (Note: the existing `members` list is **not** strictly alphabetical;
  I'll group sensibly and not reorder the rest — flagging in case Jon wants a sort.)
- `CHANGELOG.md`: append S15 bullet(s) under `[Unreleased]`. Append-only.
- `CURRENT_STATE.md`: add a `garnet-cst` row to the source map / an S15 note.
  Section-scoped to S15. **Will not** soften the existing "Trivia-preserving CST
  (S15)" #221 entry — I add a distinct rowan entry.
- `GARNET_v0_7_SLICE_DOGFOOD.md` → §S15: advance `State:` through the state machine;
  do not touch other slices' blocks.
- `scripts/garnet_mit_readiness_status.py`: add a **new** lane `parser_cst_migration`
  (distinct from #221's existing "Trivia-preserving CST (S15)" lane). Lane status
  calibrated to evidence — **not** auto-100%; it reports exactly what the dogfood
  block reproduces (roundtrip + converter-on-corpus + bench), with the rest as
  `deferred`. Then regenerate the baseline:
  `python3 scripts/garnet_mit_readiness_status.py --format json > F_Project_Management/GARNET_v0_5_READINESS_BASELINE.json`
  (the `--check-no-regression` gate is one-directional; a new lane must be seeded).

---

## 5. Verification & dogfood (PRD §"Dogfood block")

Self-validate before each PR open (toolchain PATH export required on this box):

```sh
export PATH="/Users/IDC2.5/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo build -p garnet-cst --release
cargo test -p garnet-cst -p garnet-parser-v0.3 --no-fail-fast
cargo bench -p garnet-cst --bench parse_cst_vs_ast        # PR-2 only
cargo test --workspace --no-fail-fast                     # no consumer regression
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings
cargo deny check                                          # rowan must clear (Risk R4)
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
```

Desktop dogfood bundle at `/Users/IDC2.5/Desktop/dogfood/garnet-s15-cst-<UTCstamp>/`
with the 6 sealed files (`dogfood-readiness-report.md`, `change-diff.patch`,
`verification-log.txt`, `artifact-files.txt`, `MANIFEST.sha256`, `manifest-verify.log`).
PR body uses the dogfood-readiness headings the CI greps for. PR-Agent Grep Loop to
≥4/5 before merge.

## 6. PR / branch / auth mechanics (slice-discipline memory)

- Branch from fresh `origin/main`. Push to **fork** (`Navigata1`).
- Open PR `Navigata1:<branch>` → `Island-Dev-Crew/garnet` base `main`.
- Merge needs **IslandDevCrew** account (Navigata1 lacks org merge perms):
  `gh auth switch --user IslandDevCrew` … `--squash --delete-branch` …
  `gh auth switch --user Navigata1`.
- Node-24 action pin minimums apply if any workflow file is touched (don't expect to).

---

## 7. Risks / decisions to flag to Jon (need a call before/at PR time)

- **R1 — Mini-Spec edit is LOCKED.** PRD §8 asks to "update Mini-Spec §X.Y with a
  one-paragraph CST note," but the ledger marks `GARNET_v1_0_Mini_Spec.md` as
  Jon-only (Handoff Request required). → Propose: file a Handoff Request for the
  one-paragraph note, **or** Jon makes that edit. I will not touch the Mini-Spec
  without approval. (Default if undecided: skip the Mini-Spec note in S15, capture
  it in `garnet-cst/AGENTS.md` instead.)
- **R2 — `--mode cst` CLI ownership.** PRD §4 says add a `--mode cst` flag, and the
  full release gate shows `garnet parse --mode cst`. But that CLI surface lives in
  `garnet-cli`, which is **read-only** for me. My S15 dogfood block does **not**
  require the CLI flag. → Propose: S15 ships the rowan path as a **library** API
  (`garnet_cst::parse_cst`); the `garnet parse --mode cst` CLI wiring is deferred to
  the release gate / a Handoff to the garnet-cli owner after S15-Compare. Flagging
  so we don't silently drop a PRD line.
- **R3 — `@stability(experimental)` is S17's deliverable.** The compiler annotation
  doesn't exist until win-opus ships S17. → PR-1 documents the tier in doc comments
  + AGENTS.md now; the real `@stability(...)` annotation is wired after S17 merges
  (soft dep, tracked).
- **R4 — `rowan` vs `cargo deny`.** `rowan` is not yet in `Cargo.lock`. rowan +
  transitive deps (`text-size`, `countme`, `hashbrown`, `rustc-hash`, …) are
  MIT/Apache-2.0, but `deny.toml` may have a strict allowlist. → I'll run
  `cargo deny check` immediately after adding the dep. If it trips, the fix is a
  scoped `deny.toml` license/advisory allowance (config, not a crate). Flagging
  because `deny.toml` is shared config — I'll treat updating it as part of adding
  the dependency unless Jon prefers a Handoff Request.
- **R5 — readiness lane honesty.** The existing "Trivia-preserving CST (S15)" lane
  (from #221) reads `verified 100%`. My new `parser_cst_migration` lane will be
  calibrated to *its own* evidence and explicitly framed as "second, independent
  CST built for A/B; canonical choice deferred to S15-Compare." Two lanes coexist
  by design until reconciliation — matching the contract's intent.

## 8. Out of scope (PRD §"Out of scope")

LSP features (S16); editor updates; migrating interp/check/vm to consume CST
directly (v0.8); perf optimization below 1.5×; **reconciling with #221's CST
(S15-Compare, not S15)**.

## 9. Done criteria (PRD §"Done criteria")

- [ ] PR-1 (trait surface) merged, green CI.
- [ ] PR-2 (full impl) merged, green CI.
- [ ] `garnet-cst` in workspace `Cargo.toml`.
- [ ] Roundtrip proptest green ≥1000 iters.
- [ ] Ledger mac-opus/S15 → MERGED.
- [ ] CHANGELOG `[Unreleased]` updated.
- [ ] Readiness reporter shows the new `parser_cst_migration` lane; baseline regenerated.
