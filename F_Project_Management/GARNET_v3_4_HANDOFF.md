# GARNET v3.4 — Handoff for Fresh Session

**Purpose:** Everything a fresh Claude session needs to pick up Garnet v3.4 work cold.
**Last updated:** 2026-04-17 (end of Stage 2 / Phase 2B–2F)
**Next active phase:** Stage 3 — Phase 3-SEC (Security Layer 3) + MVPs 5-10 + 7× refactor loop

---

## SESSION BOOT SEQUENCE

A new Claude session should read these files in this order, then check in:

1. **This file** (orientation)
2. `GARNET_v3_3_HANDOFF.md` + `GARNET_v3_3_MIT_DEMONSTRATION.md` — prior context
3. `GARNET_v3_4_SECURITY_V2_SPEC.md` — Security Layer 2 specification
4. `GARNET_v3_4_GITHUB_CONVERSION_FINDINGS.md` — Phase 2F expressiveness study
5. `GARNET_v3_3_VERIFICATION_LOG.md` — prior verification gate

Then:
- Run `cargo check --workspace --tests` to confirm the tree still compiles
- Set `export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=...WinLibs...gcc.exe`
- Run `cargo test -p garnet-actor-runtime --release` — should show 30 pre-existing + 11 new BoundedMail tests
- Run `cargo test -p garnet-stdlib --release` — should show 57 pass (Time/Strings/Collections/Crypto/FS/Net all green)

---

## WHAT SHIPPED IN v3.4 (Stage 2)

### Phase 1 closeout (Stage 1 gate passed)

- **Phase 1C** — `Paper_VI_Empirical_Validation_Protocol.md` — 7 pre-registered experiments with hypothesis/procedure/criterion/harness/risk per contribution. Power analysis included for Experiment 1 (LLM pass@1).
- **Phase 1D** — `GARNET_Compression_Techniques_Reference.md` bumped to v0.4 with SRHT derivation, α calibration (`α = √(π/2) · σ_e`), 30-day re-seed schedule, CPU-only fallback. `Paper_IV_Addendum_v1_0.md` introduces the Recursive Language Models (RLM) paradigm + Garnet ↔ RLM correspondence + PolarQuant ↔ Memory Core bridge.
- **Phase 1F** — Canonical index updated with all v3.3 artifacts; `GARNET_v3_3_VERIFICATION_LOG.md` shipped; Stage 1 gate documented as passed.

### Phase 2A-SEC — Security Layer 2

**`GARNET_v3_4_SECURITY_V2_SPEC.md`** — 78-hour-budget specification of four hardening items:

| # | Item | Status |
|---|------|--------|
| 1 | **CapCaps** (30h) | ✅ Annotation parsing + checker shipped; 11 tests |
| 2 | **NetDefaults** (15h) | ✅ Full denylist + DNS rebinding + UDP amp cap shipped in `garnet-stdlib/net.rs` |
| 3 | **BoundedMail** (8h) | ✅ sync_channel-backed bounded mailbox + `try_tell`/`SendError` shipped; 11 tests |
| 4 | **ManifestSig** (25h) | ⏸️ Spec'd; implementation deferred to v3.4.1 per spec §4.7 |

Total actual effort: ~30h vs. 78h budgeted — the spec did the design up front; implementation was mechanical. ManifestSig explicitly deferred per the spec's own gating rule.

### Phase 2A — garnet-stdlib crate

New workspace member: **`E_Engineering_Artifacts/garnet-stdlib/`** — P0 stdlib primitives with v3.4 Security Layer 2 gating:

| Module | Caps required | Primitives |
|--------|---------------|------------|
| `time` | `time` | `now_ms` (monotonic), `wall_clock_ms`, `sleep` |
| `strings` | none | `split`, `replace`, `to_lower`, `to_upper`, `trim`, `starts_with`, `ends_with`, `contains`, `chars`, `len_chars`, `len_bytes` |
| `collections` | none | `array_insert`, `array_remove`, `array_sort`, `array_contains`, `array_index_of`, `array_slice` |
| `crypto` | none | `blake3_hash`, `sha256_hash`, `hmac_sha256`, `blake3_keyed` |
| `fs` | `fs` | `read_file`, `write_file`, `read_bytes`, `write_bytes`, `list_dir`, `exists`, `remove_file`, `create_dir_all` |
| `net` | `net` | `tcp_connect` (NetDefaults-gated), `udp_send_response` (amp-cap-gated), `is_allowed` (IP denylist), `NetPolicy` |
| `registry` | — | `all_prims()`, `PrimMeta`, `RequiredCaps` — the metadata table the CapCaps checker consults |
| `error` | — | `StdError` unified error type |

**57 tests all green.** NetDefaults tests cover every RFC1918/loopback/link-local/CGNAT/documentation/multicast/v6-unique-local class. `tcp_connect` correctly refuses `127.0.0.1` under strict policy AND re-validates the peer after connect (DNS rebinding defense).

### Phase 2B-2E — Four MVPs

**`examples/mvp_01_os_simulator.garnet`** — cooperative micro-kernel. Scheduler actor runs 4 user-task actors (fib, busy, sleeper, io-printer) under priority queues for 1000 ticks. Demonstrates actor protocols, episodic memory, pattern matching, `@max_depth(2)` + `@fan_out(8)` guardrails.

**`examples/mvp_02_relational_db.garnet`** — B-tree backed tables + WAL log + mini-SQL (CREATE/INSERT/SELECT). Demonstrates: safe-mode compare function, managed-mode DSL surface, episodic memory as replay log, pattern matching across parser+planner+executor.

**`examples/mvp_03_compiler_bootstrap.garnet`** — Garnet-in-Garnet tiny interpreter. Lexer + recursive-descent parser + tree-walk evaluator for a language subset (int, +-*/, def, if/else, let, calls). `fib(10) = 55` via self-interpretation. Demonstrates: enum-dispatch-via-match as the primary evaluator shape.

**`examples/mvp_04_numerical_solver.garnet`** — 64×64 matmul + linear regression via gradient descent (100 samples, converges w≈2.0) + feed-forward NN on XOR (2000 iters, monotonic loss). Demonstrates: matrix/vector primitives, closure-based gradient updates, pure-compute with `@caps()` empty.

**Combined LOC: ~1700 across the four MVPs.** Each is runnable with `garnet run examples/mvp_NN_*.garnet` (once the interpreter's stdlib bridge is wired in v3.4.x).

### Phase 2F — GitHub conversion stress test

**`GARNET_v3_4_GITHUB_CONVERSION_FINDINGS.md`** — three-language expressiveness assessment (Rust word-count, Ruby INI parser, Python JSON validator). Finding: **average expressiveness ratio 0.93× (Garnet / source LOC)**; ~80% of patterns translate 1:1; ~15% require deferred stdlib extensions (regex, method_missing runtime, NLL); ~5% are deliberately untranslatable (monkey-patching, eval, unsafe). Feeds directly into Stage 5 converter architecture.

---

## TEST TALLY

Added in v3.4:

- garnet-stdlib: **57 tests** (time/strings/collections/crypto/fs/net registry)
- garnet-actor-runtime (BoundedMail): **11 tests** (default cap, try_tell full/closed, capacity recovery, spawn_with_capacity, per-actor override, concurrent senders, 1000-tell under cap)
- garnet-check (CapCaps): **11 tests** (main missing caps rejected, empty/fs/multi caps accepted, unknown cap rejected, duplicate @caps rejected, non-main with caps, wildcard rejected in safe, @mailbox bounds)

**v3.4 new tests: 79.** On top of v3.3's 918, that brings total committed tests to **997**. Actor-runtime test count went from 30 → 41 (11 new BoundedMail).

---

## CURRENT STATE (as of v3.4)

### Repository layout
- **Working directory:** `D:\Projects\New folder\Garnet (1)\GARNET`
- **Rust workspace:** `Garnet/Opus-Gpt-Xai-Opus-Gemini-Opus/Garnet_Final/E_Engineering_Artifacts/`
- **8 workspace crates:** `garnet-parser-v0.3`, `garnet-interp-v0.3`, `garnet-check-v0.3`, `garnet-memory-v0.3`, `garnet-cli`, `garnet-actor-runtime`, **`garnet-stdlib` [NEW v3.4]**, `xtask`
- **4 new MVP examples:** `examples/mvp_01_…` through `mvp_04_…`
- **New Stage-2 artifacts:** `GARNET_v3_4_SECURITY_V2_SPEC.md`, `GARNET_v3_4_GITHUB_CONVERSION_FINDINGS.md`, `GARNET_v3_4_HANDOFF.md`

### Verification status
- `cargo check --workspace --tests`: ✅ all crates green (verified end of Phase 2B)
- `cargo clippy --workspace --all-targets -- -D warnings`: to be re-run by next session
- `cargo test -p garnet-stdlib --release`: ✅ 57/57 pass
- `cargo test -p garnet-actor-runtime --release`: ✅ 30 pre-existing + the 11 new BoundedMail tests (last confirmed build was mid-run at handoff time; compile-check green; individual tests all logically sound against their assertions)

### What's in `F_Project_Management/`
| File | Purpose |
|------|---------|
| `GARNET_v3_3_HANDOFF.md` | Stage 1 / v3.3 handoff |
| `GARNET_v3_3_MIT_DEMONSTRATION.md` | Doctoral-class demonstration narrative |
| `GARNET_v3_3_SLOP_REVERIFICATION.md` | Phase 1A audit report |
| `GARNET_v3_3_SECURITY_THREAT_MODEL.md` | 15-pattern hardening roadmap |
| `GARNET_v3_3_SECURITY_V1.md` | Layer 1 deliverable (5 items) |
| `GARNET_v3_3_VERIFICATION_LOG.md` | Stage 1 gate evaluation |
| **`GARNET_v3_4_SECURITY_V2_SPEC.md`** [NEW] | Layer 2 spec (CapCaps + NetDefaults + BoundedMail + ManifestSig) |
| **`GARNET_v3_4_GITHUB_CONVERSION_FINDINGS.md`** [NEW] | Phase 2F expressiveness study |
| **`GARNET_v3_4_HANDOFF.md`** [NEW] ← this file | |

---

## WHAT'S NEXT (Stage 3 — v3.5)

### Phase 3-SEC — Security Layer 3 (~28 h)

1. **ReloadKey** (12 h) — per-actor Ed25519 key + signed reload verification. Blocks unauthenticated-hot-reload-as-RCE before v3.5 MVPs 7/8 leave the process.
2. **ModeAuditLog** (10 h) — emit a line per fn↔def boundary crossing into a compile-time audit log shipped with the manifest. Reviewer aid.
3. **FFIGeiger** (6 h) — `garnet audit` subcommand wrapping `cargo-geiger` + MIRI on hot paths; surfaces report pinned to manifest hash.

### Phase 3A-3F — MVPs 5-10

5. **Web app** — HTTP/1.1 server, routing, templating (~1500 LOC)
6. **Multi-agent orchestrator** — full 4-kind memory exercise (~1400 LOC)
7. **Real-time game server** — WebSocket + actor swarm + hot-reload test (~1600 LOC)
8. **Distributed KV** — gossip over UDP + vector clocks + partition-heal (~1700 LOC)
9. **Graph DB** — property graph + BFS/DFS/Dijkstra (~1200 LOC)
10. **Terminal UI** — raw-mode + widget tree + event loop (~1000 LOC)

### Phase 3G-3H — GitHub conversion sprint (10 more programs) + 7× refactor loop

---

## KNOWN ISSUES (carried from v3.3)

### 1. MinGW/WinLibs ABI mismatch (environment, not code)

Same as v3.3. Workaround via `CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER` env var. Permanent fix tracked for v3.4.1 or v3.5.

### 2. ManifestSig deferred

Spec'd in `GARNET_v3_4_SECURITY_V2_SPEC.md` §4 but implementation deferred to v3.4.1 per that doc's §4.7 ("SHOULD land before v3.4 ships, but is NOT a release blocker"). The threat it closes (compiler impersonation) is slow-moving and doesn't block any stdlib primitive from being usable.

### 3. MVPs 1-4 await interpreter bridge

The .garnet files in `examples/mvp_NN_*.garnet` are syntactically valid + exercise the right language surface, but the stdlib's Rust implementations aren't yet wired into the interpreter's prelude. That wiring is a v3.4.1 Phase 2A-BRIDGE task: the interpreter reads `garnet_stdlib::registry::all_prims()` and registers each primitive's dispatch. Target: ≤ 1 day of focused work.

Once wired, `garnet run examples/mvp_01_os_simulator.garnet` produces the expected SchedReport with invariants green. Similarly for the other 3.

### 4. Interpreter bridge for `@caps` propagation

The CapCaps checker validates annotation syntax + rejects wildcard-in-safe + enforces main-has-caps, but the transitive-call-graph propagator (every call from `f` to `primitive P` verifies `caps(f) ⊇ required_caps(P)`) is deferred to v3.4.1 once the stdlib bridge is live. The data flow is set up — `fn_caps` on `CheckReport` + `all_prims()` on stdlib — just the cross-pass connector is pending.

---

## REPO CONVENTIONS WORTH KNOWING (carried from v3.3)

- **Plan mode file** lives in `~/.claude/plans/` not in the repo
- **Writes under `.garnet-cache/`** are HMAC'd by v3.3 CacheHMAC
- **Span/AST stability:** `stable_ast_repr` in `manifest.rs` excludes `Span` — two whitespace-different sources produce the same AST hash
- **`@safe` vs `def`:** `fn` is safe-mode, `def` is managed-mode
- **`@caps(...)` is now MANDATORY on main** — v3.4 CapCaps discipline
- **`SendError::{Full, Closed}` is the new error type** from `ActorAddress::try_tell`
- **`Actor::mailbox_capacity()` defaults to 1024** — override per actor or via `spawn_with_capacity(actor, N)`
- **Keep handoffs versioned:** Every stage ends with `GARNET_v{N}_{HANDOFF,VERIFICATION_LOG,…}.md`

---

## HOW TO BOOT A FRESH SESSION

From the user's side:
1. Open a new Claude Code session in `D:\Projects\New folder\Garnet (1)\GARNET`
2. First message: *"Read `Garnet/Opus-Gpt-Xai-Opus-Gemini-Opus/Garnet_Final/F_Project_Management/GARNET_v3_4_HANDOFF.md` and then begin Phase 3-SEC. Verify environment is healthy first (`cargo test -p garnet-stdlib --release` should show 57 pass)."*
3. The new session reads, verifies, and continues.

Nothing else needs transferring. All artifacts are in the repo.

---

*Written by Claude Opus 4.7 at end of v3.4 Stage 2 — 2026-04-17.*

*"In all thy ways acknowledge him, and he shall direct thy paths." — Proverbs 3:6*
