# GARNET v0.5 SLICE DOGFOOD CONTRACTS

Date: 2026-05-20
Purpose: Single source of truth for every v0.5 PR. Read by Claude Code,
Codex Desktop, Antigravity 2.0, Greptile/PR-Agent, and Jon. Update this
file in the same commit as the work it tracks.

---

## Slice State Machine

Every slice moves through:

  not-started → planned → in-progress → review-ready → dogfood-passing → merged

| Transition | Required artifact |
|---|---|
| not-started → planned | Plan file at `.codex/plans/S<N>-plan.md` or `.claude/plans/S<N>-plan.md` referencing this contract by section |
| planned → in-progress | Draft PR open with title `S<N>: <short>` |
| in-progress → review-ready | CI green · PR body uses the template below · dogfood block run locally with output committed |
| review-ready → dogfood-passing | PR-Agent confidence ≥ 4/5 · Jon reviewed |
| dogfood-passing → merged | Squash-merged · CHANGELOG.md updated · status reporter output committed if % moved |

Backward moves are allowed and require a one-line "regression note" in the PR body.

---

## Common Verification Primitives

Every slice's CI run executes these on top of its own block:


```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
cargo deny check
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
python3 scripts/garnet_conformance_matrix_check.py  # add in S0 housekeeping
```



---

## Cross-Slice Gates (every PR)

| Gate | Where enforced | Failure mode |
|---|---|---|
| `@caps` declared on new authority | `garnet check` in CI | Hard fail |
| Determinism preserved | S9 cross-machine matrix | Hard fail |
| No new ambient `unsafe` | `cargo clippy` + audit | Hard fail |
| Honest voice in docs | Jon review | Block until corrected |
| Reproduction block present | PR template lint | Block until added |
| `cargo deny check` clean | CI | Hard fail |

---

## Slice Contracts

### S1 — LSP MVP

**Goal:** Diagnostics, hover, go-to-def in VSCode for managed mode.

**New surfaces:** `garnet-lsp/` crate · `editors/vscode/` extension · workspace `Cargo.toml` update.

**Deps added:** `tower-lsp`, `tokio`. Must pass `cargo deny`.

**Dogfood block:**

```bash
cargo build -p garnet-lsp --release
python3 scripts/smoke_garnet_lsp_protocol.py target/release/garnet-lsp
(cd editors/vscode && npm install && npm run package)
code --install-extension editors/vscode/garnet-*.vsix

# Manual confirmation — required in PR body as screenshots:
#   (a) Open examples/mvp_01_*.garnet; inject syntax error; diagnostic appears
#   (b) Hover on `def greet` shows signature panel
#   (c) Go-to-def from a call site lands on definition
```


**Honest partial labels available:** "safe-mode hover not in MVP" · "workspace symbols deferred to S1.1" · "rename deferred."

**State as of 2026-05-20:** merged.

---

### S2 — Bytecode VM Scaffold

**Goal:** Instruction set + serializer + loader + execution for top 10–15 opcodes; tree-walk fallback for the rest.

**New surfaces:** `garnet-vm/` crate · `C_Language_Specification/GARNET_BYTECODE_v0_1.md` spec · `garnet-vm/benches/`.

**Dogfood block:**

```bash
cargo bench -p garnet-vm --bench parse_compile_execute > /tmp/vm-bench.txt
# Expect: bytecode path measurably faster than tree-walk on mvp_01..mvp_05

for f in examples/mvp_0{1,2,3,4,5}_*.garnet; do
  target/release/garnet run --vm     "$f" > /tmp/vm.out
  target/release/garnet run --interp "$f" > /tmp/interp.out
  diff /tmp/vm.out /tmp/interp.out || exit 1
done
```



**Honest partial labels available:** "X of N opcodes native, rest fall back to tree-walk" — must be quantified in the PR.

**State as of 2026-05-20:** merged.

---

### S3 — `garnet add` + Manifest Spec  (v0.5.1 acceptable)

**Goal:** Vendored deps with content-addressed lockfile. No central registry yet.

**New surfaces:** `garnet-cli/src/cmd/add.rs` · `C_Language_Specification/GARNET_MANIFEST_v0_1.md` · template updates to scaffold `garnet.toml`.

**Dogfood block:**

```bash
mkdir /tmp/test-add && cd /tmp/test-add
garnet new --template cli demo && cd demo
mkdir ../local-lib && echo 'def hello() { "hi" }' > ../local-lib/lib.garnet
garnet add ../local-lib
grep -q "local-lib" garnet.lock
garnet run src/main.garnet   # uses the added lib
```



**State:** not-started.

---

### S4 — `garnet fmt`  (v0.5.1 acceptable)

**Goal:** Deterministic, idempotent source formatter.

**Dogfood block:**

```bash
for f in examples/*.garnet; do
  garnet fmt "$f" > /tmp/once
  garnet fmt /tmp/once > /tmp/twice
  diff /tmp/once /tmp/twice || { echo "fmt not idempotent: $f"; exit 1; }
done
```



**State:** not-started.

---

### S5 — Parser Fuzz Harness

**Goal:** `cargo fuzz` integration with seed corpus from `examples/`.

**New surfaces:** `garnet-parser-v0.3/fuzz/` · `.github/workflows/fuzz-nightly.yml`.

**Dogfood block (local):**

```bash
cd garnet-parser-v0.3
cargo +nightly fuzz run parse_input -- -max_total_time=60
# Expect: 0 panics, 0 hangs, memory bounded under default sanitizer limits
```



**CI:** runs ≥1 hour nightly; artifacts uploaded on any crash.

**Security tie-in:** primary defense against the agent-generated-Garnet adversarial corpus described in the v0.5 security plan.

**State as of 2026-05-20:** merged via PR #189; local Mac dogfood, proof-reporter inventory, and scoped license-check evidence recorded in the S5 evidence closure.

---

### S6 — Memory Eviction Policy Benchmarks

**Goal:** Each Mnemos memory kind (working/episodic/semantic/procedural) gets a measured eviction strategy with Criterion benchmarks vs a naive baseline.

**New surfaces:** `garnet-memory-v0.3/benches/eviction.rs` · `scripts/garnet_memory_eviction_status.py` · update to `C_Language_Specification/MEMORY_CORE_ROADMAP.md`.

**Dogfood block:**

```bash
cargo bench -p garnet-memory-v0.3 --bench eviction > /tmp/evict.txt
python3 scripts/garnet_memory_eviction_status.py
# Expect: per-kind numbers committed as evidence artifact
```



**Closes:** half of Paper VI Contribution 3's "production allocator path" gap.

**State:** not-started.

---

### S7 — Actor OS-Thread Bridge

**Goal:** Close the "managed source bridge active, full OS-thread CLI bridge staged" gap from CURRENT_STATE.md.

**New surfaces:** `garnet-actor-runtime` OS-thread driver · new example `examples/agent_orchestrator_3thread.garnet`.

**Dogfood block:**

```bash
garnet run examples/agent_orchestrator_3thread.garnet
garnet trust-report examples/agent_orchestrator_3thread.garnet \
  | grep -q "actors: 3 / threads: 3"
```



**Closes:** Paper VI Contribution 4 partial → supported.

**State:** not-started.

---

### S8 — Signed Hot-Reload Demo

**Goal:** Two runnable examples — one success, one BLAKE3 fingerprint mismatch — demonstrating `actor.reload_signed`.

**New surfaces:** `examples/mvp_11_signed_hotreload.garnet` · `examples/mvp_11_signed_hotreload_mismatch.garnet`.

**Dogfood block:**

```bash
garnet run examples/mvp_11_signed_hotreload.garnet
# Expect: exit 0, "reloaded successfully" in stdout

garnet run examples/mvp_11_signed_hotreload_mismatch.garnet
# Expect: exit 1, "BLAKE3 fingerprint mismatch" in stderr
```



**Closes:** Paper VI Contribution 5 surface gap (the implementation worked; the demo didn't exist).

**State as of 2026-05-20:** merged via PR #193. Local Mac dogfood proves the
success fixture exits 0 with `reloaded successfully` and the mismatch fixture
exits 1 with `BLAKE3 fingerprint mismatch`; CI's canonical MVP examples gate
now treats `*_mismatch.garnet` as an expected-failure fixture instead of a
positive example. Honest boundary: this is a BLAKE3 program-level demo for the
existing Rust-runtime `actor.reload_signed` path, not managed-mode
`actor.reload_signed` syntax or full Ed25519 payload verification.

---

### S9 — Determinism CI Cross-Machine

**Goal:** GitHub Actions matrix proves byte-identical deterministic builds across OSs.

**New surfaces:** `.github/workflows/determinism.yml` · `examples/det_fixture_01.garnet`.

**Dogfood block (runs in CI on matrix `[ubuntu-latest, macos-latest]`):**

```bash
garnet build --deterministic --sign /tmp/keys/test.key examples/det_fixture_01.garnet
sha256sum examples/det_fixture_01.garnet.manifest.json > /tmp/hash-$RUNNER_OS.txt
# Upload as artifact; comparison job diffs the two hashes and fails if they differ.
```



**Closes:** Paper VI Contribution 6 verification gap.

**State as of 2026-05-20:** merged via PR #187. The GitHub Actions
determinism workflow is active and compares ubuntu-latest vs macos-latest
manifest hashes produced by `garnet build --deterministic --sign` with a shared
short-lived signing key. Honest boundary: Windows, Linux aarch64, multi-key
rotation, and native-backend binary determinism are deferred.

---

### S10 — Compiler-as-Agent Advisory Mode

**Goal:** Deterministic, rules-based suggestion engine emitting "compiler suggested" rewrites from compilation history. No LLM yet — establishes the seam where the LLM plugs in when Paper VI Exp 1 unblocks.

**New surfaces:** `garnet-check-v0.3/src/suggest.rs` · `garnet-cli/src/cmd/check.rs` (`--suggest` flag) · `garnet-check-v0.3/tests/suggest_corpus/`.

**Dogfood block:**

```bash
garnet check --suggest examples/mvp_03_*.garnet | grep -q "compiler suggested"
cargo test -p garnet-check-v0.3 --test suggest_corpus
# Expect: at least 3 detectable patterns produce suggestions on the corpus
```



**Closes:** Paper VI Contribution 7 surface (rules-based advisory tier; LLM tier remains pending-infra).

**State as of 2026-05-20:** merged via PR #191. The rules-based advisory tier
is active through `garnet check --suggest` and the `compiler suggested:` output
contract; the test corpus proves at least three deterministic patterns. Honest
boundary: provider-backed or LLM-derived suggestions remain pending-infra.

---

## v0.5.0 Release Gate

Tag v0.5.0 only when all of:

- [x] S1, S2, S5, S8, S9, S10 in `merged` state
- [x] `scripts/garnet_mit_readiness_status.py` reports a higher AND more granular %
      (`67.9%` across 17 lanes after S8, up from the v0.4.2/productization
      pulse while preserving blocked/deferred lanes)
- [x] `CHANGELOG.md` updated with each merged slice
- [x] `docs/blog/2026-Qx-garnet-v0-5.md` drafted using the substance-over-surface framing
- [x] Pre-tag clean-machine/source-fallback and local release-asset reproduction passes:

```bash
  rm -rf /tmp/clean && mkdir /tmp/clean && cd /tmp/clean
  curl -sSf https://garnet-lang.org/install.sh | sh
  garnet new --template cli demo && cd demo
  garnet test && garnet run src/main.garnet
  # S4 (if merged): garnet fmt --check src/main.garnet
  code --install-extension <published-garnet-vsix>
  # Confirm: VSCode shows diagnostics on injected syntax error
```

Current state as of 2026-05-20: the pre-tag release gate is satisfied and the
`v0.5.0` tag is authorized, but release completion is not yet proven. The
release-candidate preflight retargets the canonical package version and public
installer default to v0.5.0, but source-fallback proof is not release-asset
publication proof.
Post-merge public installer source-fallback proof is recorded in
`/Users/idc2.0/Desktop/dogfood/garnet-v0-5-rc-merged-20260520T121820Z`:
`curl -fsSL https://garnet-lang.org/install.sh` installed `garnet 0.5.0` into
an isolated prefix with a configured Rust toolchain, then `garnet new`,
`garnet test`, and `garnet run` passed. Mac-local editor evidence is recorded
in `/Users/idc2.0/Desktop/dogfood/garnet-v0-5-editor-gate-20260520T122611Z`:
`island-dev-crew.garnet@0.5.0` installed into Cursor, the injected syntax-error
fixture showed `1 problem in this file` / `Errors: 1`, and the LSP protocol
smoke proved diagnostics, hover, and go-to-definition over stdio.
Clean standalone VS Code diagnostic evidence is recorded in
`/Users/idc2.0/Desktop/dogfood/garnet-v0-5-standalone-vscode-gate-20260520T130303Z`:
standalone VS Code 1.121.0 arm64 installed the locally packaged
`garnet-0.5.0-lsp-mvp.vsix` into isolated user-data/extensions directories,
launched the bundled `extension/server/garnet-lsp` without `garnet.lsp.path`,
and showed `1 problem in this file` for an injected syntax error. The sealed
bundle includes the screenshot, VSIX, installed-server path, trace log,
protocol smoke JSON for diagnostics/hover/go-to-definition, and
`MANIFEST.sha256`.

Release-backed VSIX publication is now staged, not claimed:
`scripts/package_garnet_vscode_extension.sh` builds host-native
`garnet-<version>-lsp-mvp-<target>.vsix` artifacts with the bundled server and
manifested evidence, `.github/workflows/vscode-extension.yml` publishes those
VSIX files as GitHub Release assets on `v*` tag pushes, and
`scripts/verify_org_release_smoke.sh` fails until the matching release-backed
VSIX asset exists and contains the extension entry point plus bundled server.
Fresh local M5 evidence is recorded in
`/Users/idc2.0/Desktop/dogfood/garnet-vscode-release-assets-20260520T133747Z`;
its manifest verifies `garnet-0.5.0-lsp-mvp-darwin-arm64.vsix` as
release-asset-ready local evidence, not publication.

The macOS CLI release asset path is also staged, not claimed:
`.github/workflows/linux-packages.yml` builds
`garnet-<version>-aarch64-apple-darwin.tar.gz` and
`garnet-<version>-x86_64-apple-darwin.tar.gz` on macOS, then publishes those
tarballs with Linux `.deb`/`.rpm` packages behind one release-time
`SHA256SUMS`. This is required because the public installer falls back from
unsigned/unpublished `.pkg` assets to tarballs on macOS. It is workflow proof,
not publication proof, until the `v0.5.0` tag workflow completes and
`scripts/verify_org_release_smoke.sh` passes against the organization release.
Fresh local M5 file-backed release-mode evidence is recorded in
`/Users/idc2.0/Desktop/dogfood/garnet-macos-cli-tarball-release-assets-20260520T135703Z`:
the `garnet-0.5.0-aarch64-apple-darwin.tar.gz` tarball verified against
`SHA256SUMS`, installed into an isolated prefix, reported `garnet 0.5.0`, then
`garnet new --template cli`, `garnet test`, and `garnet run` passed.

Tag authorization is ready after PR #200. The current Mac has Cursor as
`/usr/local/bin/code`, but standalone VS Code diagnostic proof now exists
through the isolated `/tmp` downloaded app. Release-backed package proof still
requires the tag workflow to publish assets and the release-only smoke to pass.
Marketplace/OpenVSX publication is not claimed by this gate.

Post-tag validation required before closing the v0.5.0 goal:

- [ ] `v0.5.0` tag workflow publishes Linux packages, macOS CLI tarballs,
      `SHA256SUMS`, and host-native VSIX assets.
- [ ] `scripts/verify_org_release_smoke.sh` passes against
      `Island-Dev-Crew/garnet` release `v0.5.0` without source fallback.
- [ ] Clean release install runs `garnet new --template cli`, `garnet test`,
      and `garnet run src/main.garnet`.
- [ ] Release-backed `garnet-0.5.0-lsp-mvp-darwin-arm64.vsix` installs into
      standalone VS Code/Cursor and shows diagnostics on an injected syntax
      error.



S3, S4, S6, S7 may land in v0.5.1 if not ready at tag time. Their `not-started → merged` arc follows the same contract.

---

## PR Body Template


```markdown
## Slice
S<N>: <short>

## Goal
<paste the Goal line from GARNET_v0_5_SLICE_DOGFOOD.md>

## State transition
<previous-state> → <new-state>

## What's in
- 
- 

## What's NOT in (honest partial)
- 
- 

## Reproduction
```bash
<paste the slice's dogfood block, run it, paste output below>
```

## Status reporter delta
```


Before: <paste relevant scripts/garnet_*_status.py output>
After:  <paste same after this PR>

```

## Conformance matrix delta
<paste diff if S2 / S6 / S7 / S10 affects spec coverage>

## PR-Agent / Greptile loop
- Initial confidence: <N>/5
- Final confidence:   <N>/5
- Iterations to 5/5:  <count>

## Cargo deny
<paste `cargo deny check` summary>

## Regression note (if state moved backward)
<one line>
```


---

## Integration with Existing Scripts

Every slice's `merged` transition must verify against the appropriate status reporter:

| Slice | Status reporter consulted |
|---|---|
| S1 | `garnet_mit_readiness_status.py` (LSP becomes a tracked sub-gate) |
| S2 | `garnet_proof_benchmark_status.py` (VM benches feed into this) |
| S3 | new: `garnet_package_manager_status.py` |
| S4 | new: `garnet_formatter_status.py` |
| S5 | `garnet_proof_benchmark_status.py` (fuzz hours become evidence) |
| S6 | new: `garnet_memory_eviction_status.py` |
| S7 | `garnet_mit_readiness_status.py` (actor concurrency sub-gate) |
| S8 | `garnet_mit_readiness_status.py` (hot-reload demo sub-gate) |
| S9 | `garnet_mit_readiness_status.py` (determinism cross-machine sub-gate) |
| S10 | `garnet_converter_llm_feasibility.py` (advisory tier graduates to "active") |

New reporters are written in the same Python style and discipline as existing ones: deterministic, manifest-backed, no claims beyond their evidence.

---

## Honesty Anchors (do not soften)

These phrases stay verbatim in the README, status outputs, and release blog through v0.5 — they are brand equity:

- "research-grade prototype (v0.x.x) — not production-complete"
- "tracked-slice ledger is complete, but that is not full MIT/productization completion"
- Paper VI scorecard: "4 supported, 2 partial (downgraded honestly), 0 refuted, 1 pending-infra"
- "production allocator path tracked in MEMORY_CORE_ROADMAP.md"
- "human/aesthetic acceptance remains open"

If a slice would let one of these soften, the slice's PR body says so explicitly and Jon decides.
