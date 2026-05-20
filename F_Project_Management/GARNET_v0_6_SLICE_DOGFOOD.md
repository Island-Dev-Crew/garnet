# GARNET v0.6 SLICE DOGFOOD CONTRACTS

Date: 2026-05-20
Purpose: Single source of truth for every v0.6 PR. Read by Claude Code,
Codex Desktop, Antigravity 2.0, Greptile/PR-Agent, and Jon. Update this
file in the same commit as the work it tracks.

The v0.5 successor of `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md`.
v0.5 is closed; this file governs every PR titled `S11:`–`S16:` and any
later v0.6 slice added under § Slice Contracts.

---

## v0.6 thesis

v0.5 shipped the scaffolds — bytecode VM (S2), LSP MVP (S1), `garnet add`
manifest + vendored deps (S3), formatter idempotent baseline (S4), parser
fuzz harness (S5), memory eviction benchmarks (S6), OS-thread trust report
(S7), signed hot-reload demo (S8), determinism CI cross-machine (S9), and
rules-based compiler advisory (S10).

Each one carries a calibrated-honest `deferred` list. v0.6 closes the
load-bearing entries on those lists and adds the missing registry stub.
Concretely:

- The interpreter has to actually consume `.garnet/vendor/` at `garnet run`
  time (S12 closes S3's deferred line #1).
- A static registry stub has to exist so `garnet add` is a complete loop
  end-to-end, not just a local-path vendor (S13).
- The bytecode VM has to lower function calls natively so the
  function-boundary fallback stops swallowing the common case (S14).
- The parser has to grow a trivia-preserving CST so LSP precision,
  semantic formatter, and richer trust-report stop being gated on the same
  missing layer (S15).
- LSP has to surface workspace symbols, rename, code actions, and semantic
  tokens — and wire S10 advisory suggestions into code actions (S16).

If all five land, the v0.6.0 release gate fires.

---

## Slice State Machine

Every slice moves through:

  not-started → planned → in-progress → review-ready → dogfood-passing → merged

| Transition | Required artifact |
|---|---|
| not-started → planned | Plan file at `.codex/plans/S<N>-plan.md` or `.claude/plans/S<N>-plan.md` referencing this contract by section |
| planned → in-progress | Draft PR open with title `S<N>: <short>` |
| in-progress → review-ready | CI green · PR body uses the dogfood-readiness headings · dogfood block run locally with output committed |
| review-ready → dogfood-passing | Jon reviewed |
| dogfood-passing → merged | Squash-merged · CHANGELOG.md updated · status reporter output committed if % moved · readiness baseline regenerated if a lane was added |

Backward moves are allowed and require a one-line "regression note" in the PR body.

---

## Common Verification Primitives

Every slice's CI run executes these on top of its own block. These gates
were stood up in v0.5 (S0 introduced the `--check-no-regression` flag and
the conformance matrix check); v0.6 inherits them unchanged:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
cargo deny check
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
python3 scripts/garnet_conformance_matrix_check.py
python3 -m unittest discover scripts/ -p 'test_*.py'
```

---

## Cross-Slice Gates (every PR)

| Gate | Where enforced | Failure mode |
|---|---|---|
| `@caps` declared on new authority | `garnet check` in CI | Hard fail |
| Determinism preserved | S9 cross-machine matrix | Hard fail |
| No new ambient `unsafe` | `cargo clippy` + audit | Hard fail |
| Honest voice in docs | Jon review | Block until corrected |
| Dogfood-readiness headings present | `.github/workflows/dogfood-readiness.yml` greps PR body | Hard fail |
| `cargo deny check` clean | CI | Hard fail |
| Node-24 action minimums | `scripts/test_github_actions_node24_readiness.py` | Hard fail |
| `rustfmt` clean per package | `cargo fmt --all -- --check` | Hard fail |
| MIT-readiness baseline regenerated if a lane was added | S0 `--check-no-regression` | Hard fail |
| Bytecode ABI stability respected (v0.6+) | S14 ABI test | Block on review |

---

## Slice Contracts

### S11 — v0.6 slice contract scaffold

**Goal:** Land this file, the v0.6 roadmap, and the refreshed dogfood-readiness skill so every downstream v0.6 slice has the contract surface and discipline reference it needs.

**New surfaces:**
- `F_Project_Management/GARNET_v0_6_SLICE_DOGFOOD.md` (this file).
- `F_Project_Management/ROADMAPS/GARNET_v0_6_LANGUAGE_RUNTIME_ROADMAP.md`.
- Refresh of `F_Project_Management/DOGFOOD/GARNET_DOGFOOD_READINESS_SKILL.md` (v0.4.2 pulse → v0.5.x pulse).
- `CHANGELOG.md` opens `## [Unreleased] — v0.6.0 in flight` block.

**Dogfood block:**

```bash
python3 scripts/garnet_mit_readiness_status.py --check-no-regression  # pass
python3 scripts/garnet_readiness_status.py | head -20                  # still 87/87
python3 scripts/garnet_conformance_matrix_check.py                     # pass
python3 -m unittest discover scripts/ -p 'test_*.py'                   # pass
```

**Honest partial labels available:** "documentation-only — no reporter lanes added; no baseline regeneration; v0.6 slice lanes get added by their respective slices."

**State:** in-progress (this PR).

---

### S12 — Package-manager resolver contract

**Goal:** Wire `garnet run` to read `Garnet.toml`'s `[dependencies]` and resolve `use <dep>::<sym>` through `.garnet/vendor/<dep>/`. Closes the deferred line #1 from S3 ("interpreter does NOT yet load `.garnet/vendor/` deps at `garnet run` time").

**New surfaces:**
- `garnet-cli/src/cmd/run.rs` (dep manifest loader hook).
- `garnet-interp-v0.3/src/modules.rs` or equivalent (vendor-aware module resolver).
- `garnet-parser-v0.3/src/use_path.rs` (verify it already exists; if not, add minimal `use <ident>::<ident>` parsing).
- New lane in `scripts/garnet_mit_readiness_status.py`: `pkg_resolver_v0_2`.
- Regenerated `F_Project_Management/GARNET_v0_5_READINESS_BASELINE.json` (or successor v0.6 file — TBD by S12's plan).
- Examples: `examples/pkg_resolver_demo_main.garnet` + `examples/pkg_resolver_demo_lib.garnet` with a working `use demo_lib::*` resolution.

**Deps added:** none new at this layer (S3 already vendored). Must pass `cargo deny`.

**Dogfood block:**

```bash
mkdir /tmp/test-resolver && cd /tmp/test-resolver
garnet new --template cli demo && cd demo
mkdir ../local-lib && echo 'def hello() { "hi from local-lib" }' > ../local-lib/lib.garnet
garnet add ../local-lib
# Edit src/main.garnet to `use local_lib::*; @caps() def main() { print(hello()) }`
garnet run src/main.garnet | grep -q "hi from local-lib"   # PASS (was deferred under S3)
```

**Honest partial labels available:** "local-path resolver only — remote sources, transitive deps, SemVer matching, and workspace mode remain deferred to v0.7 or later." "Module re-export semantics out of scope — only direct `use` of vendored top-level symbols."

**State:** not-started.

---

### S13 — Registry stub v0.1

**Goal:** Specify a static `index.json` registry format, build a `garnet-registry-stub` crate that serves it from the file system, and add `garnet add --registry <url> <name>@<version>` that fetches a tarball, verifies its content-address against the index, and vendors it under `.garnet/vendor/<name>/`. End-to-end honest only after S12 merges.

**New surfaces:**
- `C_Language_Specification/GARNET_REGISTRY_v0_1.md` — `index.json` schema + tarball naming + signature surface (signature reserved, not yet verified).
- `garnet-registry-stub/` crate (new) — minimal static-server CLI + `index.json` generator from a directory tree.
- `garnet-cli/src/cmd/add.rs` — extend with `--registry <url>` flag.
- New lane in `scripts/garnet_mit_readiness_status.py`: `registry_stub_v0_1`.
- Regenerated baseline.
- Example registry fixture under `examples/registry_stub_fixture/`.

**Deps added:** lightweight HTTP client (`ureq` or `reqwest::blocking`) for the fetch path. Must pass `cargo deny`.

**Dogfood block:**

```bash
# Start the stub registry locally
cargo run -p garnet-registry-stub -- serve examples/registry_stub_fixture --port 8765 &
REG=http://127.0.0.1:8765

# Add a registry-resolved dep
mkdir /tmp/test-registry && cd /tmp/test-registry
garnet new --template cli demo && cd demo
garnet add --registry "$REG" hello-lib@0.1.0
grep -q "registry = \"$REG\"" Garnet.toml
grep -q "\"hello-lib\"" Garnet.lock

# After S12 merges, the resolver loop closes:
garnet run src/main.garnet                                   # consumes the registry-fetched lib
```

**Honest partial labels available:** "static index only — no central registry, no auth, no publish flow, no signature verification beyond BLAKE3 content-address." "Single registry per project — multi-registry resolution deferred." "Signature surface reserved in `index.json` but not yet verified — Ed25519 verification meshes with the notarization slice."

**State:** not-started.

---

### S14 — Bytecode VM v0.2 function-call lowering

**Goal:** Lower full managed-mode function calls into native bytecode. Eliminate the function-boundary fallback for the common case (parameterized functions returning MVP values). Tighten the bytecode ABI and version-bump the magic to `GARNVM02`.

**New surfaces:**
- `garnet-vm/src/compiler.rs` — function-call lowering, frame management, return-value handling.
- `garnet-vm/src/vm.rs` — `Call` opcode now executes natively for non-fallback functions.
- `garnet-vm/src/codec.rs` — magic header `GARNVM02`; function table extended with a stable parameter-arity field and a deterministic local-slot ordering.
- `C_Language_Specification/GARNET_BYTECODE_v0_2.md` (new) — v0.2 spec; v0.1 stays for archival reference.
- `garnet-vm/tests/function_call.rs` (new) — integration test asserting native lowering for a mixed-arity corpus.
- New lane in `scripts/garnet_mit_readiness_status.py`: `vm_function_call_lowering`.
- Regenerated baseline.
- New Criterion case under `garnet-vm/benches/parse_compile_execute.rs` covering function-call hot paths.

**Dogfood block:**

```bash
cargo build -p garnet-vm --release
cargo build -p garnet-cli --bin garnet --release
cargo bench -p garnet-vm --bench parse_compile_execute > /tmp/vm-v0-2-bench.txt

for f in examples/mvp_0{1,2,3,4,5}_*.garnet examples/mvp_function_call_demo.garnet; do
  target/release/garnet run --vm     "$f" > /tmp/vm.out
  target/release/garnet run --interp "$f" > /tmp/interp.out
  diff /tmp/vm.out /tmp/interp.out || exit 1
done

# Native lowering ratio reported by the compiler:
target/release/garnet run --vm --dump-lowering examples/mvp_function_call_demo.garnet \
  | grep -q "lowered: 100%"
```

**Honest partial labels available:** "Closures, captured environments, and method dispatch on dynamic receivers still fall back." "Pattern matching, try/rescue/ensure, and struct/enum constructors still fall back." "Bytecode ABI v0.2 is more stable than v0.1 but not yet a cross-version ABI promise — version-bump on schema change is in v0.7."

**State:** not-started.

---

### S15 — Trivia-preserving CST in `garnet-parser-v0.3`

**Goal:** Add a green-tree / CST layer in `garnet-parser-v0.3` that preserves whitespace, comments, and (where present) tab vs space distinctions. The AST stays the primary semantic surface; the CST becomes the trivia-faithful spine the LSP, formatter, and richer trust-report consume.

**New surfaces:**
- `garnet-parser-v0.3/src/cst.rs` — green/red tree (consider `rowan` crate; hand-rolled if it can be kept ≤ 600 LOC).
- `garnet-parser-v0.3/src/trivia.rs` — trivia capture path from the lexer.
- `garnet-parser-v0.3/src/lib.rs` — public API exposes the CST root alongside the existing AST root.
- `garnet-parser-v0.3/tests/cst_round_trip.rs` — corpus assertion: every `examples/*.garnet` is byte-identical after `parse → cst → emit_source`.
- New lane in `scripts/garnet_mit_readiness_status.py`: `parser_cst_layer`.
- Regenerated baseline.

**Deps added:** `rowan` (if chosen). Must pass `cargo deny`.

**Dogfood block:**

```bash
cargo test -p garnet-parser-v0.3 --test cst_round_trip
# Expect: every example in examples/{mvp_,det_}*.garnet round-trips byte-identical
# through parse → cst → emit_source.

cargo bench -p garnet-parser-v0.3 --bench cst_overhead > /tmp/cst-overhead.txt
# Expect: trivia-capturing parse < 2x trivia-dropping parse on the corpus.
```

**Honest partial labels available:** "Round-trip is source-preserving for canonical examples; recovery from malformed input is best-effort and may diverge." "AST is still the semantic reference; CST consumers must not extract semantics directly from the green tree."

**State:** not-started.

---

### S16 — LSP v0.2 on the CST

**Goal:** Build workspace symbols, rename, code actions, and semantic tokens on top of S15's CST. Surface S10's advisory `compiler suggested:` rules as LSP code-actions. Updates the VSCode extension to consume the new capabilities.

**New surfaces:**
- `garnet-lsp/src/workspace.rs` — workspace symbol index over the project's `.garnet` files.
- `garnet-lsp/src/rename.rs` — rename-symbol with CST-precise edits.
- `garnet-lsp/src/code_actions.rs` — code-actions backed by S10 `garnet-check-v0.3/src/suggest.rs`.
- `garnet-lsp/src/semantic_tokens.rs` — token classification consuming the CST.
- `editors/vscode/` — package version bump; declare `documentSymbolProvider`, `renameProvider`, `codeActionProvider`, `semanticTokensProvider` capabilities.
- `editor_lsp_adoption` lane in `scripts/garnet_mit_readiness_status.py` advances from `source-present 60 %` toward `verified 100 %`.
- Regenerated baseline.

**Dogfood block:**

```bash
cargo build -p garnet-lsp --release
python3 scripts/smoke_garnet_lsp_protocol.py target/release/garnet-lsp \
  --capabilities document_symbol,rename,code_action,semantic_tokens
(cd editors/vscode && npm install && npm run package)
code --install-extension editors/vscode/garnet-*.vsix

# Manual confirmation — required in PR body as screenshots:
#   (a) Workspace symbol search (Ctrl+T) lists every top-level def across the workspace
#   (b) Rename on a function symbol updates every call site in the workspace
#   (c) Code-action lightbulb on a `compiler suggested:` rule applies the fix
#   (d) Semantic tokens highlight `@caps` annotations, capability names, and actor types distinctly
```

**Honest partial labels available:** "Workspace symbol/rename works across files in the same `Garnet.toml` project root; cross-project rename out of scope." "Code-actions wired only to the three S10 rules; LLM-tier advisories still pending-infra (Paper VI Exp 1)." "Semantic tokens use a static classification scheme; per-project token themes deferred."

**State:** not-started.

---

## v0.6.0 Release Gate

Tag v0.6.0 only when all of:

- [ ] S12, S13, S14, S15, S16 in `merged` state.
- [ ] `scripts/garnet_mit_readiness_status.py` reports a higher AND more
      granular % than the v0.5.1 close (71.9 % / 21 lanes / 12 verified).
      Target: ≥ 80 % / ≥ 25 lanes after S16 merges.
- [ ] `CHANGELOG.md` updated with each merged slice; `## [Unreleased] —
      v0.6.0 in flight` block named every entry.
- [ ] `docs/blog/2026-Qx-garnet-v0-6.md` drafted using the
      substance-over-surface framing.
- [ ] Pre-tag clean-machine reproduction passes the v0.6 contract loop:

```bash
rm -rf /tmp/clean && mkdir /tmp/clean && cd /tmp/clean
curl -sSf https://garnet-lang.org/install.sh | sh
garnet new --template cli demo && cd demo

# S12 resolver: vendored local lib resolves at run time
mkdir ../local-lib && echo 'def hello() { "hi" }' > ../local-lib/lib.garnet
garnet add ../local-lib
# Edit src/main.garnet to use local_lib::*
garnet run src/main.garnet | grep -q "hi"

# S13 registry: registry stub end-to-end (after S12)
cargo run -p garnet-registry-stub -- serve <fixture> --port 8765 &
garnet add --registry http://127.0.0.1:8765 hello-lib@0.1.0
garnet run src/main.garnet | grep -q "hello-lib OK"

# S14 VM v0.2: native function-call lowering
garnet run --vm src/main.garnet
diff <(garnet run --vm src/main.garnet) <(garnet run --interp src/main.garnet)

# S16 LSP v0.2: workspace symbols + rename + code-actions in VSCode
code --install-extension <published-garnet-vsix-v0.6.0>
# Manual: rename a symbol across two files; apply a compiler-suggested code action
```

- [ ] Release-asset workflows (Linux packages, macOS tarballs, VSIX) publish
      `v0.6.0` assets with a unified `SHA256SUMS`.
- [ ] `scripts/verify_org_release_smoke.sh` passes against
      `Island-Dev-Crew/garnet` release `v0.6.0`.

S12–S16 may land out of order under the strict slice-per-PR discipline as
long as S15 merges before S16 opens (S16 consumes the CST API) and S12
merges before S13's end-to-end loop is honest (S13 can still land its
crate + spec independently, with the deferred line "end-to-end resolution
gates on S12").

---

## PR Body Template

```markdown
## Slice
S<N>: <short>

## Goal
<paste the Goal line from GARNET_v0_6_SLICE_DOGFOOD.md>

## State transition
<previous-state> → <new-state>

## What's in
- 
- 

## What's NOT in (honest partial)
- 
- 

## Dogfood Readiness

### Current truth
- [ ] origin/main tip: …
- [ ] readiness status: …

### Local verification
- [ ] cargo fmt --all -- --check
- [ ] cargo clippy --workspace --all-targets -- -D warnings
- [ ] cargo test --workspace --no-fail-fast
- [ ] python3 scripts/garnet_mit_readiness_status.py --check-no-regression
- [ ] python3 scripts/garnet_conformance_matrix_check.py
- [ ] <slice-specific dogfood block output attached>

### Remote verification
- [ ] PR dogfood evidence
- [ ] cargo test (matrix)
- [ ] clippy
- [ ] cargo-deny check

### Desktop dogfood bundle
- [ ] Bundle path: /Users/IDC2.5/Desktop/dogfood/garnet-<slug>-<UTCstamp>/
- [ ] MANIFEST.sha256 sealed; manifest-verify.log records OK.

### Deferred / out of scope
- [ ] <verbatim from honest-partial labels>

## Status reporter delta
Before: <paste relevant scripts/garnet_*_status.py output>
After:  <paste same after this PR>

## Honesty anchors
- This PR does not claim production ARC complete.
- <slice-specific anchor>

## Regression note (if state moved backward)
<one line>
```

---

## Integration with Existing Scripts

Every slice's `merged` transition must verify against the appropriate status
reporter:

| Slice | Status reporter consulted |
|---|---|
| S11 | `garnet_mit_readiness_status.py` (no lane delta; scaffolding only) |
| S12 | `garnet_mit_readiness_status.py` (new lane: `pkg_resolver_v0_2`) |
| S13 | `garnet_mit_readiness_status.py` (new lane: `registry_stub_v0_1`) |
| S14 | `garnet_mit_readiness_status.py` (new lane: `vm_function_call_lowering`) + `garnet_proof_benchmark_status.py` (VM v0.2 bench feeds in) |
| S15 | `garnet_mit_readiness_status.py` (new lane: `parser_cst_layer`) |
| S16 | `garnet_mit_readiness_status.py` (`editor_lsp_adoption` advances 60 % → verified) |

New reporters are written in the same Python style and discipline as
existing ones: deterministic, manifest-backed, no claims beyond their
evidence.

---

## Honesty Anchors (carry forward from v0.5, plus v0.6 additions)

These phrases stay verbatim in the README, status outputs, and release blog
through v0.6 — they are brand equity:

Carried from v0.5:

- "research-grade prototype (v0.x.x) — not production-complete"
- "tracked-slice ledger is complete, but that is not full MIT/productization
  completion"
- Paper VI scorecard: "4 supported, 2 partial (downgraded honestly), 0
  refuted, 1 pending-infra"
- "production allocator path tracked in MEMORY_CORE_ROADMAP.md"
- "human/aesthetic acceptance remains open"

New for v0.6:

- "bytecode ABI v0.2 is more stable than v0.1 but not yet a cross-version
  ABI promise"
- "registry stub serves a static index; no central registry, no auth, no
  publish flow"
- "package-manager resolver is local-path-first; remote sources,
  transitive deps, SemVer matching, and workspace mode remain deferred"
- "CST round-trip is source-preserving for canonical examples; recovery
  from malformed input is best-effort"

If a slice would let one of these soften, the slice's PR body says so
explicitly and Jon decides.
