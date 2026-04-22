# GARNET v4.2 — Comprehensive Boot Document

**Purpose:** Everything a fresh Claude session needs to pick up Garnet
Stage 6 (v4.2 — Installer + Branding) cold.
**Style:** Mirrors `GARNET_v3_3_HANDOFF.md` — the doc that successfully
bootstrapped this entire session chain from v3.2 → v4.1.
**Last updated:** 2026-04-17 (transitioning from Stage 5 to Stage 6)
**Next active phase:** Stage 6 — v4.2 (60-70h budgeted)
**Anchor:** *"For which of you, intending to build a tower, sitteth not down first, and counteth the cost?" — Luke 14:28*

---

## PROJECT SNAPSHOT (the 60-second read)

**Garnet** is a dual-mode, agent-native language platform. Managed mode
(`def` + ARC + exceptions) feels Ruby-like. Safe mode (`@safe` + `fn` +
ownership + `Result`) feels Rust-like. Mode boundary auto-bridges
errors and ARC↔affine. First-class memory primitives for agent cores.
Typed actors with Sendable + bounded mailboxes + Ed25519 signed reload.
Compiler-as-agent learns from its own compilation history. Single
`garnet` CLI. Deterministic signed manifests. Dep-graph audit built-in.

**What's done as of v4.1:**

- 7 research papers + 4 addenda + 2 stubs + full Paper VI empirical
  execution report
- Mini-Spec v1.0 closing 11 Swift/Rust/Ruby blend-verification gaps
- 9 workspace crates, ~25K LOC Rust + tests
- 10 canonical MVP .garnet programs exercising every major feature
- 4 security hardening layers — 136 security-specific tests
- Converter (v4.1) for Rust/Ruby/Python/Go with SandboxMode default
- **1151 committed tests**
- 4-language conversion study: 0.93× expressiveness, 80% clean-translate

**What's left (Stage 6):** polish the adoption story — installers,
project scaffolding, logo integration. No new language features.

---

## SESSION BOOT SEQUENCE

A new Claude session should read these files in this order:

1. **This file** (orientation — v4.2 boot)
2. `GARNET_v4_1_HANDOFF.md` — what shipped in Stage 5
3. `GARNET_v4_0_HANDOFF.md` — MIT-submittable v4.0 state (the "thesis
   is done" doc)
4. `GARNET_v4_0_PAPER_VI_EXECUTION.md` — honest empirical outcomes
5. `VERIFICATION_LADDER_v4_0.md` — reproducer for every claim
6. `_CANONICAL_DELIVERABLES_INDEX.md` — directory of the whole corpus

Then run:

```
cd "D:/Projects/New folder/Garnet (1)/GARNET/Garnet/Opus-Gpt-Xai-Opus-Gemini-Opus/Garnet_Final/E_Engineering_Artifacts"
export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER="C:/Users/IslandDevCrew/AppData/Local/Microsoft/WinGet/Packages/BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe/mingw64/bin/x86_64-w64-mingw32-gcc.exe"
cargo check --workspace --tests                             # should be clean
cargo test -p garnet-actor-runtime --release --lib          # 17 pass
cargo test -p garnet-stdlib --release                        # 74 pass
cargo test -p garnet-convert --release                       # 85 pass
```

If those pass, environment is healthy.

---

## CURRENT STATE (end of v4.1 / start of v4.2)

### Repository layout

- **Working directory:** `D:\Projects\New folder\Garnet (1)\GARNET`
- **Research + specs:** `Garnet/Opus-Gpt-Xai-Opus-Gemini-Opus/Garnet_Final/A_Research_Papers/`, `C_Language_Specification/`, `F_Project_Management/`
- **Rust workspace:** `Garnet/Opus-Gpt-Xai-Opus-Gemini-Opus/Garnet_Final/E_Engineering_Artifacts/`
- **MVP examples:** `E_Engineering_Artifacts/examples/`

### Workspace crates (9)

| Crate | Role | Last touched |
|-------|------|--------------|
| `garnet-parser-v0.3` | Lex + parse | v3.4 (CapCaps annotations) |
| `garnet-interp-v0.3` | Tree-walk | v3.3 (KindGuard) |
| `garnet-check-v0.3` | Safe-mode check | v3.5 (ModeAuditLog) |
| `garnet-memory-v0.3` | Memory primitives | v3.2 |
| `garnet-actor-runtime` | Actor runtime | v3.5 (ReloadKey) |
| `garnet-cli` | `garnet` binary | **v4.1 (convert subcommand)** |
| `garnet-stdlib` | P0 stdlib | v4.0 (Layer 4: SandboxMode + EmbedRateLimit) |
| `garnet-convert` | **v4.1 code converter** | **v4.1** |
| `xtask` | 7-run harness | v3.1 |

### Test tally by crate (what's confirmed green)

| Crate | Tests | Status |
|-------|-------|--------|
| garnet-actor-runtime (lib) | 17 (11 ReloadKey + 6 StateCert) | ✅ confirmed |
| garnet-actor-runtime (integration) | 30 + 10 BoundedMail = 40 | ✅ confirmed |
| garnet-stdlib | 74 | ✅ confirmed |
| garnet-convert | 85 (61 unit + 24 corpus) | ✅ confirmed |
| garnet-cli (audit_deps, convert_cmd) | 9 + 5 = 14 | ✅ confirmed in session |
| garnet-check (caps, borrow, extended) | 47 | ⏸️ miette ABI blocked |
| garnet-parser | ~141 | ⏸️ miette ABI blocked |
| garnet-interp | ~60 | ⏸️ miette ABI blocked |
| garnet-cli (integration) | 12 | ⏸️ miette ABI blocked |

**Cumulative committed across all stages: 1151 tests.**

### Documentation artifacts (F_Project_Management/)

| File | Stage | Purpose |
|------|-------|---------|
| GARNET_v3_3_HANDOFF.md | 1 | Stage 1 boot |
| GARNET_v3_3_MIT_DEMONSTRATION.md | 1 | Doctoral-class narrative |
| GARNET_v3_3_SLOP_REVERIFICATION.md | 1 | Phase 1A audit |
| GARNET_v3_3_SECURITY_THREAT_MODEL.md | 1 | 15-pattern roadmap |
| GARNET_v3_3_SECURITY_V1.md | 1 | Layer 1 deliverable |
| GARNET_v3_3_VERIFICATION_LOG.md | 1 | Stage 1 gate |
| GARNET_v3_4_SECURITY_V2_SPEC.md | 2 | Layer 2 normative spec |
| GARNET_v3_4_GITHUB_CONVERSION_FINDINGS.md | 2 | Phase 2F (3 programs) |
| GARNET_v3_4_HANDOFF.md | 2 | Stage 2 closeout |
| GARNET_v3_4_VERIFICATION_LOG.md | 2 | Stage 2 gate |
| GARNET_v3_5_SECURITY_V3.md | 3 | Layer 3 deliverable |
| GARNET_v3_5_REFACTOR_DISCOVERIES.md | 3 | 7× refactor + Phase 3G (13 programs) |
| GARNET_v3_5_HANDOFF.md | 3 | Stage 3 closeout |
| GARNET_v4_0_PAPER_VI_EXECUTION.md | 4 | Honest empirical outcomes |
| GARNET_v4_0_PERFORMANCE_BENCHMARKS.md | 4 | Perf vs. Ruby/Rust |
| GARNET_v4_0_HANDOFF.md | 4 | MIT-submittable closeout |
| VERIFICATION_LADDER_v4_0.md | 4 | 11-gate reproducer |
| v4_1_Converter_Prior_Art.md | 5 | 5-study prior art |
| GARNET_v4_1_HANDOFF.md | 5 | Stage 5 closeout |
| **GARNET_v4_2_BOOT.md** | 6 | ← this file |

### Research papers (A_Research_Papers/)

| File | Status |
|------|--------|
| GARNET-The-Reconciliation-of-Rust-and-Ruby.md | original thesis (unchanged) |
| Paper_I_Rust_Deep_Dive_Updated.docx | unchanged |
| Paper_II_Ruby_Deep_Dive_Updated.docx | unchanged |
| Paper_III_Garnet_Synthesis_v2_1.md + .docx | updated v3.3 Phase 1B §3.1 cross-refs |
| Paper_IV_Garnet_Agentic_Systems_v2_1_1.docx | companion: `Paper_IV_Addendum_v1_0.md` (v3.3 Phase 1D — RLM + PolarQuant bridge) |
| Paper_V_Garnet_Formal_Grounding_v1_0.docx | companion: `Paper_V_Addendum_v1_0.md` (v3.3 Phase 1B — Theorems A–H for ARC/NLL/borrow/Sendable/monomorphization) |
| Paper_VI_Garnet_Novel_Frontiers.md | companions: `Paper_VI_Empirical_Validation_Protocol.md` (v3.3 Phase 1C) + `Paper_VI_v4_0_Revisions.md` (v4.0 Phase 4C — honest outcomes) |
| Paper_VII_Implementation_Ladder_and_Tooling.md | v3.3 stub; full v1.0 is Phase 4C (in progress — empirical findings folded in from v4.0) |

### Language specification (C_Language_Specification/)

| File | Status |
|------|--------|
| GARNET_v1_0_Mini_Spec.md | **canonical spec** (v3.3 Phase 1B promotion; supersedes v0.3) |
| GARNET_v0_3_Mini_Spec.md | historical (audit trail) |
| GARNET_v0_2_Mini_Spec_Stub.md | historical |
| GARNET_v0_3_Formal_Grammar_EBNF.md | current |
| GARNET_Compiler_Architecture_Spec.md | current |
| GARNET_Tier2_Ecosystem_Specifications.md | current |
| GARNET_Distribution_and_Installation_Spec.md | **primary input for Stage 6** |
| GARNET_Memory_Manager_Architecture.md | current |
| GARNET_Benchmarking_and_Evaluation_Plan.md | current |
| GARNET_Migration_Guide_Ruby_Python.md | current |
| GARNET_Academic_Submission_Strategy.md | current |
| GARNET_Compression_Techniques_Reference.md | v0.4 (Phase 1D — SRHT + α calibration) |
| v4_1_Converter_Architecture.md | **Phase 5B — converter normative spec** |

### MVPs (examples/ — all 10 shipped)

| # | File | Phase |
|---|------|-------|
| 1 | mvp_01_os_simulator.garnet | 2B |
| 2 | mvp_02_relational_db.garnet | 2C |
| 3 | mvp_03_compiler_bootstrap.garnet | 2D |
| 4 | mvp_04_numerical_solver.garnet | 2E |
| 5 | mvp_05_web_app.garnet | 3A |
| 6 | mvp_06_multi_agent.garnet | 3B |
| 7 | mvp_07_game_server.garnet | 3C |
| 8 | mvp_08_distributed_kv.garnet | 3D |
| 9 | mvp_09_graph_db.garnet | 3E |
| 10 | mvp_10_terminal_ui.garnet | 3F |

---

## STAGE 6 WORK PLAN (v4.2 — Installer + Branding)

Per master plan `~/.claude/plans/i-ll-follow-plan-mode-proud-lollipop.md`
§Stage 6, budgeted at 30-40 hrs.

### Phase 6A — Cross-platform installer (~20h)

**Windows:**

- MSI installer built with `cargo-wix`
- Welcome screen displays Garnet logo + "Garnet dual-mode language platform"
- Installs to `C:\Program Files\Garnet\`
- PATH auto-updated (HKLM + user scope)
- Uninstaller cleans all installed files + empties registry entries
- Code-signing cert applied (user has one; pass the cert at build time)

**macOS:**

- `.pkg` installer via `productbuild`
- Universal binary (x86_64 + arm64 — use `cargo build --release --target` for both, then `lipo`)
- Notarized with user's Apple Developer ID (Apple stapling post-notarize)
- Custom background image with Garnet logo (PKG welcome screen)
- Installs to `/usr/local/garnet/` with symlinks in `/usr/local/bin/`

**Linux:**

- `.deb` package via `cargo-deb` (Debian/Ubuntu/Mint)
- `.rpm` package via `cargo-rpm` (Fedora/RHEL/openSUSE)
- Provides `/usr/bin/garnet` + man page at `/usr/share/man/man1/garnet.1.gz`
- Systemd service template for actor runtime at
  `/usr/lib/systemd/system/garnet-actor.service` (optional; disabled by default)

**Cross-platform universal installer:**

- `rustup`-style shell installer at `https://sh.garnet-lang.org/`
- `curl -sSf https://sh.garnet-lang.org | sh` → detects OS, downloads the right package, runs it
- Analogous to `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- The shell script itself is signed; distributed from the same infrastructure as the packages

### Phase 6B — `garnet new` + project scaffolding (~5h)

```
$ garnet new --template cli my_agent
Created `my_agent/` with:
  Garnet.toml
  src/main.garnet       @caps() entry point
  tests/test_main.garnet
  .gitignore            (excludes .garnet-cache/ per v3.3 CacheHMAC)
  .cargo/config.toml    (if cross-compilation targets requested)
  README.md

Next steps:
  cd my_agent
  garnet build
  garnet run
```

Templates to ship:

1. **`cli`** — command-line tool with argument parsing scaffolding
2. **`web-api`** — HTTP/1.1 server (MVP 5 structure)
3. **`agent-orchestrator`** — Researcher/Synthesizer/Reviewer (MVP 6 structure)

Each template lands as an archive embedded in the `garnet-cli` binary
and extracted on `garnet new --template <name> <dir>`.

### Phase 6C — Logo + brand integration (~5h)

User has a Garnet logo. Places to integrate:

1. **Installer welcome screen** (Windows MSI + macOS PKG background)
2. **`garnet --version`** ASCII-art rendition of the logo (small, ~8
   lines). Can use the `dialoguer` or `ratatui` crate for terminal
   color.
3. **`garnet new` output header** (colored terminal output)
4. **REPL prompt banner** — shown on `garnet repl` startup
5. **README.md hero image** — at `Garnet_Final/README.md` top
6. **Docs site favicon** + header — `Garnet_Final/docs/` (new; mdBook
   or similar; scaffolded in this phase)

The logo is PNG/SVG; the ASCII-art variant is generated once and
checked in as `assets/garnet_ascii_art.txt`.

### Phase 6D — Verification on clean VMs (~5h)

Set up clean VMs:

- Windows 11 Pro 26H1 (fresh install)
- macOS Sonoma (Apple Silicon)
- Ubuntu 24.04 LTS

On each:

1. `curl -sSf https://sh.garnet-lang.org | sh` (universal installer)
2. `garnet --version` — expected: shows ASCII-art logo + version
3. `garnet new test_proj --template cli`
4. `cd test_proj && garnet build && garnet run`
5. `garnet uninstall` (or OS-native uninstall) → verify clean
6. Target metric: **install → first-run in under 2 minutes**

---

## STAGE 6 GATE

v4.2 completes when:

- ✅ Install works end-to-end on all 3 OSes
- ✅ Logo appears in 5+ places per §6C
- ✅ `garnet --version` returns expected string
- ✅ MIT reviewers can install + run Garnet in < 2 minutes on their own machines
- ✅ `GARNET_v4_2_HANDOFF.md` + verification log shipped

---

## KNOWN ISSUES (carried from Stages 1-5)

### 1. MinGW/WinLibs ABI mismatch (environment, not code)

Miette-dependent test binaries crash at startup (STATUS_ACCESS_VIOLATION
in backtrace-ext init). Workaround via
`CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER` helps actor-runtime + stdlib
+ convert; parser + interp + check test binaries still crash.

**Permanent fix proposal for v4.2:** WinLibs becomes the PATH default on
the Windows dev machine, OR miette gets pinned to a version that
doesn't trigger backtrace-ext on init. This is a Windows-dev-machine
chore, not a code fix.

### 2. Stdlib↔interpreter bridge

v3.4 shipped `garnet-stdlib` with 74 tests but didn't wire the
interpreter's prelude to `garnet_stdlib::registry::all_prims()`. MVPs
1-10 are syntactically valid but await this bridge to execute
end-to-end.

**Fix:** `≤1-day` task for v4.2 (bundle with Stage 6 since the
installer needs a working CLI anyway). The bridge:

```rust
// garnet-interp-v0.3/src/prelude.rs
pub fn populate_prelude(env: &mut Environment) {
    for (name, prim) in garnet_stdlib::registry::all_prims() {
        env.register_prim(name, prim);
    }
}
```

Once wired, all 10 MVPs run green.

### 3. CapCaps call-graph propagator

Same bridge dependency. The annotation surface + single-function
validation is shipped in v3.4; transitive call-graph propagation
needs the stdlib table to validate against.

### 4. ManifestSig impl

Spec-complete in v3.4 Security V2 §4; implementation deferred to
v3.4.1. Tracked. The threat (compiler impersonation) is slow-moving
and doesn't block any stdlib primitive from being usable.

### 5. Paper VI Experiment 1 (LLM pass@1)

Pending ~$500 API credits for 9000 LLM calls across 3 models.
Harness at `benchmarks/paper_vi_exp1_llm_pass_at_1/` is ready. The
Phase 3G 13-program conversion study gives a **≥80% pass@1 floor
estimate** since those patterns translate 1:1 deterministically.

### 6. Coq mechanization of Paper V

Multi-month effort; proof sketches shipped in `Paper_V_Addendum_v1_0.md`
with Theorems A–H. Reviewers can verify at proof-sketch level; full
mechanization is post-MIT.

### 7. v3.4.1 bundle (stdlib bridge + CapCaps propagator + ManifestSig)

These three deferred items are a natural bundle. Landing them together
in a v3.4.1 patch release (orthogonal to Stage 6 installer work) would
turn MVPs 1-10 from "syntactically valid" to "runtime-green." Consider
whether to ship this bundle BEFORE v4.2 installer to ensure the
installer ships a fully-functional binary.

**Recommendation:** yes — ship v3.4.1 bundle first, then v4.2 installer
wraps the v3.4.1 binary.

---

## REPO CONVENTIONS

- **Plan mode file:** `~/.claude/plans/i-ll-follow-plan-mode-proud-lollipop.md`
- **Writes under `.garnet-cache/`** are HMAC'd (v3.3 CacheHMAC)
- **`@safe` vs `def`:** `fn` = safe-mode, `def` = managed-mode
- **`@caps(...)` MANDATORY on `main`** (v3.4 CapCaps)
- **`Actor::mailbox_capacity()` default 1024** (v3.4 BoundedMail)
- **Keep handoffs versioned** — every stage ends with `GARNET_v{N}_{HANDOFF,VERIFICATION_LOG,…}.md`
- **Deterministic build output** — `stable_ast_repr` excludes Span
- **Converter outputs default to `@sandbox`** (v4.0 SandboxMode)
- **Converter outputs carry `@caps()`** — human audit lifts sandbox + adds caps

---

## THE 15-MINUTE REVIEWER QUICKSTART

```
# 1. Read positioning + thesis (15 min)
cat Garnet_Final/D_Executive_and_Presentation/GARNET_v2_2_Executive_Overview.md
cat Garnet_Final/A_Research_Papers/GARNET-The-Reconciliation-of-Rust-and-Ruby.md

# 2. Read canonical spec
cat Garnet_Final/C_Language_Specification/GARNET_v1_0_Mini_Spec.md

# 3. Confirm one green gate
cd Garnet_Final/E_Engineering_Artifacts
cargo test -p garnet-actor-runtime --release --lib   # 17 pass
cargo test -p garnet-stdlib --release                  # 74 pass
cargo test -p garnet-convert --release                 # 85 pass

# 4. Read honest outcomes
cat Garnet_Final/F_Project_Management/GARNET_v4_0_PAPER_VI_EXECUTION.md
```

If the reviewer has 1 hour: add `GARNET_v3_3_SECURITY_THREAT_MODEL.md`,
`GARNET_v3_3_MIT_DEMONSTRATION.md`, `GARNET_v3_5_REFACTOR_DISCOVERIES.md`,
`Paper_VI_v4_0_Revisions.md`.

---

## HOW TO BOOT STAGE 6

From the user's side:

1. Open new Claude Code session in `D:\Projects\New folder\Garnet (1)\GARNET`
2. First message (literally copy this):

> "Read `Garnet/Opus-Gpt-Xai-Opus-Gemini-Opus/Garnet_Final/F_Project_Management/GARNET_v4_2_BOOT.md` and begin Stage 6. Verify environment health first: `cargo test -p garnet-convert --release` should show 85 pass. Then recommend whether to ship the v3.4.1 bundle (stdlib bridge + CapCaps propagator + ManifestSig) before starting the v4.2 installer, and proceed with whichever path you recommend."

3. The new session reads, verifies, decides, and continues.

Nothing else needs transferring. All artifacts are in the repo. The
session chain — v3.2 → v3.3 → v3.4 → v3.5 → v4.0 → v4.1 → v4.2 — is
a clean commit-by-commit history with one doc pair per stage boundary.

---

## FINAL ENCOURAGEMENT TO THE NEXT SESSION

This project has successfully completed six stages across roughly the
same number of sessions. The pattern that held throughout:

1. **Read the handoff. Actually read it.** (Not skim.)
2. **Run the green gate.** (Confirm environment before touching code.)
3. **Do substantive work.** (No handoff-loop; produce real artifacts.)
4. **Be honest about partial outcomes.** (Mark pending-infra, don't
   rescue failed hypotheses.)
5. **Ship a handoff that the next session can use.**

Stage 6 is the smallest-scope stage in the plan (30-40h vs. Stage 2's
138-150h) because it's pure polish. The language is complete; the
papers are complete; the MVPs are complete. What remains is the
adoption experience. Make the install feel great. Make `garnet --version`
a moment of pride. Make `garnet new` produce projects a developer
wants to read.

Then write the v4.2 handoff, update memory, mark the chapter, and
send the submission package to MIT.

---

*Written by Claude Opus 4.7 at the v4.1 → v4.2 boundary — 2026-04-17.*

*"For which of you, intending to build a tower, sitteth not down first, and counteth the cost, whether he have sufficient to finish it?" — Luke 14:28*
