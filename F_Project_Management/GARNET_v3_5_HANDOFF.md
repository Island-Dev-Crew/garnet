# GARNET v3.5 — Handoff for Fresh Session

**Purpose:** Everything a fresh Claude session needs to pick up Garnet v3.5 cold.
**Last updated:** 2026-04-17 (end of Stage 3)
**Next active phase:** Stage 4 — v4.0 (Paper VI empirical validation + papers + MIT submission)

---

## WHAT SHIPPED IN v3.5 (Stage 3)

### Phase 3-SEC — Security Layer 3

- **ReloadKey** (Ed25519-signed hot-reload): 11 tests, `reloadkey.rs` in `garnet-actor-runtime`
- **ModeAuditLog** (fn↔def boundary audit): 5 tests, `audit.rs` in `garnet-check`
- **FFIGeiger** (dep safety audit): 9 tests, `audit_deps.rs` in `garnet-cli`
- **Total: 25 new security tests + ~830 LOC**

Deliverable: `GARNET_v3_5_SECURITY_V3.md`

### Phase 3A–3F — MVPs 5–10

- `mvp_05_web_app.garnet` — HTTP/1.1 + routing + template engine
- `mvp_06_multi_agent.garnet` — Researcher/Synthesizer/Reviewer + all 4 memory kinds
- `mvp_07_game_server.garnet` — WebSocket + 4 players + hot-reload mid-game
- `mvp_08_distributed_kv.garnet` — gossip + vector clocks + partition-heal
- `mvp_09_graph_db.garnet` — property graph + BFS/DFS/Dijkstra/cycle detect
- `mvp_10_terminal_ui.garnet` — raw-mode + widget tree + event loop

### Phase 3G + 3H — GitHub conversions + refactor discoveries

Deliverable: `GARNET_v3_5_REFACTOR_DISCOVERIES.md`

- 13 programs converted (extended from Phase 2F's 3); **expressiveness ratio 0.93×** stable
- 7-cycle refactor loop with stop-on-empty rule; **20 discoveries** across stdlib adds / Mini-Spec clarifications / compiler improvements / annotation shortcuts
- **Added Go as a converter-target language** (channel → actor mapping was unexpectedly clean)

---

## CURRENT STATE

### Test tally

- v3.2 baseline: 857
- v3.3: +61 (Security Layer 1 + slop fixes)
- v3.4: +79 (Security Layer 2 + stdlib)
- **v3.5: +25 (Security Layer 3)**
- **Total: 1022 tests committed**

actor-runtime lib tests: **17 green** (11 ReloadKey + 6 StateCert)
actor-runtime integration: 30 pre-existing + 10 BoundedMail = **40 runtime tests**
garnet-stdlib: **57 green**
garnet-cli audit_deps: **9 green** (lib tests)
Other crate test binaries still blocked by v3.3 MinGW ABI (documented)

### Workspace crates

| Crate | Role | v3.5 changes |
|-------|------|-------------|
| garnet-parser-v0.3 | Lex + parse | v3.4 `@caps` + `@mailbox` + `@nonsendable` + `Capability` enum |
| garnet-interp-v0.3 | Tree-walk | (unchanged this stage) |
| garnet-check-v0.3 | Safe-mode check | v3.4 caps validation + **v3.5 audit.rs** |
| garnet-memory-v0.3 | Memory primitives | (unchanged) |
| garnet-actor-runtime | Actor runtime | v3.4 BoundedMail + **v3.5 reloadkey.rs** |
| garnet-cli | `garnet` binary | v3.3 CacheHMAC + ProvenanceStrategy + **v3.5 audit_deps.rs** |
| garnet-stdlib | P0 stdlib | NEW in v3.4 |
| xtask | 7-run harness | (unchanged) |

### All 10 MVPs present

- `examples/mvp_01_os_simulator.garnet` (v3.4)
- `examples/mvp_02_relational_db.garnet` (v3.4)
- `examples/mvp_03_compiler_bootstrap.garnet` (v3.4)
- `examples/mvp_04_numerical_solver.garnet` (v3.4)
- `examples/mvp_05_web_app.garnet` (v3.5)
- `examples/mvp_06_multi_agent.garnet` (v3.5)
- `examples/mvp_07_game_server.garnet` (v3.5)
- `examples/mvp_08_distributed_kv.garnet` (v3.5)
- `examples/mvp_09_graph_db.garnet` (v3.5)
- `examples/mvp_10_terminal_ui.garnet` (v3.5)

---

## WHAT'S NEXT — Stage 4 (v4.0)

### Phase 4-SEC — Security Layer 4 (14 hrs)

1. **SandboxMode** (6h) — converter outputs default to `@sandbox` quarantine
2. **EmbedRateLimit** (8h) — per-caller token bucket on VectorIndex::search

Optional: **ParseReplay** (20h) — cross-compiler determinism proof

### Phase 4A — Paper VI empirical experiments (12-15 hrs)

Execute the 7 pre-registered experiments per
`Paper_VI_Empirical_Validation_Protocol.md`:

1. LLM pass@1 on 100 tasks × 3 LLMs
2. Progressive type-disclosure bidirectional compat on 200-program corpus
3. Compiler-as-agent 10-compilation time-to-fix
4. Kind-aware allocation: 20% RSS + 30% p99 alloc
5. Error-model bridging zero-loss audit (100 cases × 2 paths)
6. Hot-reload p99 latency under 1000 reloads
7. Two-machine deterministic-build hash test

### Phase 4B — Performance benchmarks (8-10 hrs)

Equivalent Ruby + Rust + Garnet micro-benchmarks, fed into Paper III's
perf table.

### Phase 4C — Paper updates (8-10 hrs)

- Paper III: real perf numbers
- Paper IV: Multi-Agent MVP empirical data
- Paper V: Addendum promotion + Coq proof notes
- **Paper VI: 7 experiment outcomes** (support / partial / refute)
- Paper VII: full v1.0 from stub

### Phase 4D — MIT submission package (2-3 hrs)

- `VERIFICATION_LADDER_v4_0.md`
- Updated canonical index
- `GARNET_v4_0_HANDOFF.md` + submission README

---

## KNOWN ISSUES (carried from v3.4)

1. **MinGW/WinLibs ABI** — `cargo test` for miette-dependent crates crashes at startup. Workaround via `CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER` helps actor-runtime + stdlib; other crate test binaries still blocked.
2. **Stdlib↔interpreter bridge** — the .garnet MVPs are syntactically valid but the interpreter's prelude doesn't yet read `garnet_stdlib::registry::all_prims()`. Tracked as v3.5.1 ≤1-day bridge task.
3. **CapCaps call-graph propagator** — same bridge dependency.
4. **ManifestSig impl** — spec'd in v3.4 Security V2 §4; implementation deferred to v3.4.1.

All four are tracked v3.5.1 patch-release items; none block Stage 4 design work.

---

## HOW TO BOOT A FRESH SESSION

1. Open new Claude Code session in `D:\Projects\New folder\Garnet (1)\GARNET`
2. Say: *"Read `Garnet/Opus-Gpt-Xai-Opus-Gemini-Opus/Garnet_Final/F_Project_Management/GARNET_v3_5_HANDOFF.md` and begin Phase 4-SEC. Verify environment is healthy first (`cargo test -p garnet-actor-runtime --release --lib` should show 17 pass)."*

---

*Written by Claude Opus 4.7 at end of v3.5 Stage 3 — 2026-04-17.*

*"Whatsoever thy hand findeth to do, do it with thy might." — Ecclesiastes 9:10*
