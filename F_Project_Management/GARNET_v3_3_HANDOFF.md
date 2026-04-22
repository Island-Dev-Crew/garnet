# GARNET v3.3 — Handoff for Fresh Session

**Purpose:** Everything a fresh Claude session needs to pick up Garnet v3.3 work cold.
**Last updated:** 2026-04-16 (end of Stage 1, Phase 1A + Security Layer 1 complete)
**Next active phase:** Stage 1, Phase 1B — Swift/Rust/Ruby blend verification + Mini-Spec v1.0

---

## SESSION BOOT SEQUENCE

A new Claude session should read these files in this order, then check in:

1. **This file** (orientation)
2. `GARNET_v3_3_MIT_DEMONSTRATION.md` — narrative of what landed (optional but recommended for full context)
3. `GARNET_v3_3_SLOP_REVERIFICATION.md` — the adversarial audit (what was found, what was fixed)
4. `GARNET_v3_3_SECURITY_THREAT_MODEL.md` — the 15 hardening patterns + novel threat classes
5. `GARNET_v3_3_SECURITY_V1.md` — what Security Layer 1 actually shipped
6. `plans/i-ll-follow-plan-mode-proud-lollipop.md` (in `~/.claude/plans/`) — the full 5-stage plan

Then:
- Run `cargo check --workspace --tests` to confirm the tree still compiles
- Set `export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER="C:/Users/IslandDevCrew/AppData/Local/Microsoft/WinGet/Packages/BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe/mingw64/bin/x86_64-w64-mingw32-gcc.exe"` to unblock test binaries that DO work (actor-runtime)
- Run `cargo test -p garnet-actor-runtime --release` — should show 30 pass / 2 ignored. If it does, environment is healthy.

---

## CURRENT STATE (as of this handoff)

### Repository layout
- **Working directory:** `D:\Projects\New folder\Garnet (1)\GARNET`
- **Rust workspace:** `Garnet/Opus-Gpt-Xai-Opus-Gemini-Opus/Garnet_Final/E_Engineering_Artifacts/`
- **Papers + specs:** `Garnet/Opus-Gpt-Xai-Opus-Gemini-Opus/Garnet_Final/A_Research_Papers/`, `C_Language_Specification/`
- **Project management:** `Garnet/Opus-Gpt-Xai-Opus-Gemini-Opus/Garnet_Final/F_Project_Management/` (this file lives here)

### Workspace crates
| Crate | Role | v3.3 changes |
|-------|------|-------------|
| `garnet-parser-v0.3` | Lexer + parser | **ParseBudget** (triple-axis DOS defense) |
| `garnet-interp-v0.3` | Tree-walk interpreter | **KindGuard** (memory-kind tag); `PRELUDE_SOURCE` const; unwrap hardening |
| `garnet-check-v0.3` | Safe-mode checker | unchanged |
| `garnet-memory-v0.3` | Memory primitives | unchanged |
| `garnet-actor-runtime` | Typed actor runtime | **StateCert** (TypeFingerprint + TaggedState); new blake3 dep |
| `garnet-cli` | `garnet` binary | **CacheHMAC** + **ProvenanceStrategy**; new machine_key.rs + provenance.rs; Episode signs with HMAC |
| `xtask` | 7-run consistency harness | unchanged |

### Test tally
- v3.2 baseline: 857 tests
- v3.3 slop-reverification fixes: +4 tests
- v3.3 Security Layer 1: +57 tests + 1 smoke binary
- **v3.3 total: 918 new tests shipped** (actual pass count pending test-runner env fix — see Known Issues)

### Verification status
- `cargo check --workspace --tests`: ✅ all crates green
- `cargo clippy --workspace --all-targets -- -D warnings`: ✅ zero warnings
- `cargo test -p garnet-actor-runtime --release`: ✅ 30/30 pass + 2 ignored stress tests
- Other crate test binaries (garnet-parser, garnet-interp, garnet-cli): **blocked** by local MinGW/WinLibs ABI mismatch — see Known Issues

### What's in `F_Project_Management/`
| File | Purpose |
|------|---------|
| `GARNET_v3_2_HANDOFF.md` | Prior handoff (v3.2 session) |
| `GARNET_v3_2_VERIFICATION_LOG.md` | Prior verification output |
| `GARNET_v3_3_SLOP_REVERIFICATION.md` | Phase 1A audit report (my forensic re-read) |
| `GARNET_v3_3_SECURITY_THREAT_MODEL.md` | 15-pattern hardening roadmap |
| `GARNET_v3_3_SECURITY_V1.md` | Security Layer 1 implementation deliverable |
| `GARNET_v3_3_HANDOFF.md` | **← this file** |
| `GARNET_v3_3_MIT_DEMONSTRATION.md` | Doctoral-review narrative |

---

## WHAT SHIPPED IN v3.3 (one-line summaries)

### Phase 1A — Slop re-verification (complete)
- 5 real gaps found that Explorer 1's first-pass audit missed
- All 5 fixed: `prelude_hash` now hashes actual prelude content, `Actor::extract_state()` added for real state migration, cross-cwd determinism test, Paper VI C5 wording honest, `eval.rs:176` defensive guard

### Phase 1E — Security Layer 1 (complete)
- **ParseBudget** — triple-axis parser limits (tokens / depth / literal_bytes) closes ParensBomb / StringBlimp DOS
- **KindGuard** — 8-bit runtime tag on memory-kind handles survives future IR discriminant loss
- **StateCert** — `TypeFingerprint` (BLAKE3 of type name + size + align) verified before `Box<dyn Any>` downcast; closes type-confusion cliff Fix #2 would have opened
- **CacheHMAC** — per-machine BLAKE3-keyed MAC on every episode; foreign-key or tampered records silently skipped
- **ProvenanceStrategy** — every strategy carries `justifying_episode_ids`; re-verified at consult time; unjustifiable rules quarantined

### Plan state
All of Stage 1's work is visible in the plan file at `~/.claude/plans/i-ll-follow-plan-mode-proud-lollipop.md`. The plan has 5 stages (v3.3 → v4.2); v3.3 is Stage 1.

---

## WHAT'S NEXT (Stage 1 continuation)

### Phase 1B — Swift/Rust/Ruby blend verification (~6-8 hrs)

Close the research-vision gap at the spec layer. Explorer 2's report identified specific features claimed in the research papers that are NOT yet fully specified in Mini-Spec v0.3.

**Focus areas:**

**Swift inheritance (Paper III §3.1):**
- ARC cycle detection algorithm — spec in Mini-Spec §4 (currently absent)
- Actors + Sendable-equivalent — formal rules in Paper V + Mini-Spec §3.4
- Package/tooling ergonomics — document in Paper VII (currently stub)

**Rust inheritance (Reconciliation Part I + Paper V):**
- Lifetime inference algorithm — spec in Mini-Spec §3.2
- Trait coherence (orphan rule) — spec in Mini-Spec §2.4
- Borrow-checker rules — formalize beyond Paper V's sketch
- Zero-cost abstractions / monomorphization — spec in Mini-Spec §6

**Ruby inheritance (Reconciliation Part II):**
- Blocks + yield semantics — Mini-Spec §2.3 binding/return rules
- `@dynamic` method dispatch table — spec runtime design
- Duck-typing rules at protocol level — Mini-Spec §2.2 formalization
- REPL design — currently missing from all spec docs

**Deliverable:** Promote Mini-Spec v0.3 → v1.0 by filling gaps. Updates to Papers III, V, VII.

### Phase 1C — Paper VI empirical validation plans (4-5 hrs)
Write the 7 experiment protocols (pass@1 LLM study, type-disclosure theorem, compiler-as-agent time-to-fix, kind-aware memory benchmark, error bridging zero-loss audit, hot-reload latency p99, deterministic two-machine hash test).
**Deliverable:** `Paper_VI_Empirical_Validation_Protocol.md`

### Phase 1D — PolarQuant/QJL consolidation (3-4 hrs)
Fold Gemini's Agent-Native Synthesis docx's detailed compression math into `GARNET_Compression_Techniques_Reference.md`. Fold RLM paradigm into Paper IV Appendix.

### Phase 1F — Canonical index + handoff (2 hrs)
Update deliverables index, verification log, v3.3 end-of-stage handoff, fix toolchain issue.

### Stage 1 Gate
All 857 v3.2 tests + 57 new pass (once toolchain fixed), Mini-Spec v1.0 shipped, Paper VI protocols ready, blend matrix 100% accounted at spec level.

---

## KNOWN ISSUES

### 1. MinGW/WinLibs ABI mismatch (environment, not code)

**Symptom:** `cargo test` on crates depending on miette (garnet-parser, garnet-check, garnet-interp, garnet-cli) produces test binaries that crash with `STATUS_ACCESS_VIOLATION` (0xc0000005) before any test runs. `cargo run --example` on CLI crate similarly crashes.

**Root cause:** rustc calls `x86_64-w64-mingw32-gcc` from LLVM-MinGW (`MartinStorsjo`), which lacks `libgcc_eh.a` + `libgcc.a` required by the GNU ABI. The WinLibs MinGW (`BrechtSanders`) does have these libs.

**Workaround:**
```bash
export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER="C:/Users/IslandDevCrew/AppData/Local/Microsoft/WinGet/Packages/BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe/mingw64/bin/x86_64-w64-mingw32-gcc.exe"
```
This unblocks actor-runtime tests. Other crates still crash on test-binary startup (deeper ABI issue inside miette's backtrace init).

**What works:**
- `cargo check --workspace --tests`: type-check without linking
- `cargo clippy --workspace --all-targets -- -D warnings`: clean
- `cargo test -p garnet-actor-runtime --release`: 30/30 pass
- `cargo build -p garnet-cli --release`: produces `garnet.exe`, which RUNS fine (just the test binaries crash)

**Permanent fix (Phase 1F):** Either commit the linker env var to workspace `.cargo/config.toml`, OR investigate which miette transitive dep is triggering the initialization crash and either swap linker toolchain or pin the problematic dep version.

### 2. Documentation for example programs

v3.2 has 3 example .garnet programs (multi_agent_builder, agentic_log_analyzer, safe_io_layer). They parse + check cleanly per my Phase 1A verification. They're not integration-test-referenced in `tests/integration_e2e.rs` yet — the infrastructure exists but the examples aren't bound in. Low priority; deferred to later stages.

---

## REPO CONVENTIONS WORTH KNOWING

- **Plan mode file** lives in `~/.claude/plans/` not in the repo. Updating it is fine from any session.
- **Plan mode vs. auto-memory:** the plan file is project-level and updatable; auto-memory (`~/.claude/projects/.../memory/`) is Claude-session-level.
- **Writes under `.garnet-cache/`** are HMAC'd by v3.3 CacheHMAC. An episode.log committed to git from another machine will be ignored (fail open) rather than honored.
- **Span/AST stability:** the `stable_ast_repr` in `manifest.rs` deliberately excludes `Span` info. Two whitespace-different sources produce the same AST hash — this is intentional.
- **`@safe` vs `def`:** `fn` is safe-mode (Rust-style ownership); `def` is managed-mode (ARC-ish + exceptions). Cross-boundary tests already cover bridging via try/rescue + ?.
- **Keep handoffs versioned:** Every stage ends with `GARNET_v{N}_{HANDOFF,VERIFICATION_LOG}.md`. New session reads the most recent pair.

---

## HOW TO BOOT A FRESH SESSION

From the user's side:
1. Close this session (any exit path)
2. Open a new Claude Code session in `D:\Projects\New folder\Garnet (1)\GARNET`
3. First message: *"Read `Garnet/Opus-Gpt-Xai-Opus-Gemini-Opus/Garnet_Final/F_Project_Management/GARNET_v3_3_HANDOFF.md` and then begin Phase 1B. Verify environment is healthy first (`cargo test -p garnet-actor-runtime --release` should show 30 pass / 2 ignored)."*
4. The new session reads, verifies, and continues.

Nothing else needs transferring. All artifacts are in the repo.

---

*Written by Claude Opus 4.7 at end of v3.3 Phase 1A + Security Layer 1 session — 2026-04-16*
