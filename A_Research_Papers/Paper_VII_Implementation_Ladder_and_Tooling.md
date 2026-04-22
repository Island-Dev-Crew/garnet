# Paper VII — Implementation Ladder & Tooling Ergonomics
## Phase 1B Stub (v0.1)

**Series:** Garnet Language Research Series
**Date:** April 16, 2026
**Author:** Claude Code (Opus 4.7) at the direction of Jon — Island Development Crew
**Status:** Stub created in Phase 1B; full v1.0 deliverable is a v4.0 commitment per the master plan
**Anchor:** *"Whatsoever thy hand findeth to do, do it with thy might." — Ecclesiastes 9:10*

---

## Foreword (stub status)

This paper is a v3.3 Phase 1B **stub**. Per the master plan (`~/.claude/plans/i-ll-follow-plan-mode-proud-lollipop.md`):

- Phase 1B (v3.3) commits to a stub that documents the garnet CLI design + REPL + tooling ergonomics as a Swift-inheritance close-out.
- Phase 4C (v4.0) commits to the full v1.0 paper: the engineering-ladder narrative from v3.2 → v4.0, the empirical findings from the 7 Paper VI experiments, and the cross-platform installer story (v4.2).

This stub provides enough content to anchor Mini-Spec v1.0 §16's normative single-CLI principle. The full v1.0 will expand each section with measured outcomes from Stages 2–4.

---

## Abstract (stub)

Garnet's research contribution is partly the language and partly the *engineering process* by which the language was built. This paper documents the implementation ladder — the seven rungs from a parsed grammar to a deterministically-built signed binary — and the tooling that makes the ladder accessible to working developers. The thesis is that **language design and toolchain design are co-equal first-order concerns**: a beautifully designed language with fragmented tooling (Ruby's eternal critique) loses ground to a less-elegant language with one coherent CLI (Cargo + rustup as the modern proof). Garnet inherits Cargo and SwiftPM's single-binary discipline as a structural commitment that prevents fragmentation from re-emerging.

---

## 1. Why a separate Paper on Implementation & Tooling

The Garnet research corpus is split across:

- Paper I: Rust deep dive
- Paper II: Ruby deep dive
- Paper III: Synthesis (Garnet's positioning + market wedge)
- Paper IV: Agentic systems (memory cores, harnesses)
- Paper V: Formal grounding (λ_managed + λ_safe + RustBelt grounding) + Paper V Addendum v1.0
- Paper VI: Novel frontiers (the seven contributions)

What's missing — and what Phase 1B identifies as a Swift-inheritance gap — is a paper that treats *the implementation journey itself* as a research artifact. Paper III §3.1 identifies "package tooling and language ergonomics can reinforce one another" as one of three Swift contributions, but offers no extended argument for what that means in Garnet's case.

This paper makes the case explicit: **the choices made on the engineering ladder (rungs 2–7) are research-class decisions** because they determine whether the language can be adopted, audited, and trusted at scale. We document them with the same discipline as the formal sections of Paper V.

---

## 2. The Engineering Ladder (seven rungs)

The Garnet implementation is structured as seven *rungs*. Each rung is a self-contained capability that can be evaluated independently and that the next rung depends on. This is the same shape as Rust's own ladder (lex → parse → typeck → MIR → codegen) but extended to capture the agentic and dual-mode aspects.

| Rung | Capability | Spec reference | Status (April 2026) |
|------|------------|----------------|---------------------|
| 1 | Lexical grammar | Mini-Spec §2 | ✅ Implemented (`garnet-parser-v0.3`) |
| 2 | Surface syntax parser | Mini-Spec §§3–11 | ✅ Implemented (`garnet-parser-v0.3`) |
| 3 | Managed-mode interpreter + REPL | Mini-Spec §§5–7, §15 | ⏳ Interpreter ✅; REPL stub |
| 4 | Safe-mode lowering (NLL + borrow check) | Mini-Spec §§8.5–8.6 | ⏳ Type-check ✅; NLL pending Rung 6 |
| 5 | Sendable + actor isolation | Mini-Spec §9.4 | ⏳ Marker trait spec'd; runtime check pending |
| 6 | Cycle collection + IR lowering | Mini-Spec §4.5; Compiler Architecture Spec | ⏳ Spec'd; not yet in interpreter |
| 7 | Cross-platform installer + signing | Mini-Spec §16; ManifestSig (v3.4) | ⏳ Planned for v3.4–v4.2 |

(The status column is a snapshot of v3.3 end-of-Phase-1B. Progress through the rungs is the subject of Stages 2–6 of the master plan.)

### 2.1 Why seven rungs and not five

The base PL ladder is roughly: grammar → parser → type-checker → IR → codegen. Garnet inserts two additional rungs:

- **Rung 5 (Sendable + actor isolation)** is a separate rung because actors are a first-class language feature in Garnet (per Paper III's Swift inheritance), not a library. Sendable enforcement at protocol declarations (Mini-Spec §9.4) is an additional check beyond the standard type-check / borrow-check pair.
- **Rung 7 (installer + signing)** is a separate rung because the installer is part of the *security model* (manifest signing per v3.4 ManifestSig) and the *adoption model* (one-command install per Mini-Spec §16). Treating it as "deployment" or "post-language" would understate its load.

### 2.2 Rung dependencies

Rungs strictly depend on lower-numbered rungs. Rung 4 cannot ship before Rung 3 because the type checker depends on the parsed AST. But within a rung, components MAY ship independently. For example, Rung 3 ships the interpreter (used now) before the REPL (a v3.3-end-of-Phase-1B pending item).

---

## 3. The Single-CLI Principle (`garnet`)

### 3.1 The principle

A Garnet developer interacts with the toolchain through one binary, `garnet`, with sub-commands for every workflow. The principle is borrowed from Cargo (Rust) and SwiftPM (Swift). The deliberate anti-pattern is Ruby's tooling fragmentation: separate `gem` / `bundle` / `rake` / `rspec` / `rubocop` binaries with overlapping configs, each with its own version-pinning behavior, each with its own lockfile.

The single-CLI principle is normative (Mini-Spec §16.1). Any conformant Garnet implementation MUST expose all standard workflows through the `garnet` binary.

### 3.2 Sub-command catalog

The complete v1.0 sub-command catalog is documented in Mini-Spec §16.1. Highlights:

- `garnet new` / `garnet init` — project creation
- `garnet build` / `garnet build --deterministic` — compilation
- `garnet run` / `garnet test` — execution
- `garnet check` / `garnet fmt` — static analysis + formatting
- `garnet repl` — interactive evaluation (Mini-Spec §15)
- `garnet doc` — API doc generation
- `garnet audit` — `cargo-geiger`-equivalent dep scan (v3.5 FFIGeiger)
- `garnet verify` — manifest signature verification (v3.4 ManifestSig)
- `garnet convert` — code converter from Rust/Ruby/Python (v4.1)

### 3.3 Sub-command extensibility

User-installed sub-commands follow the Cargo convention: an executable named `garnet-foo` on PATH MAY be invoked as `garnet foo`. This permits ecosystem extension without overcrowding the core CLI.

### 3.4 Output discipline

All `garnet` sub-commands MUST produce machine-parseable JSON output when given `--format=json`. Default output is human-readable. This dual output is essential for IDE integration (LSP server in v4.x) and CI integration.

### 3.5 What we deliberately do NOT include

The flat-CLI design means we do NOT have `garnet pkg add`, `garnet pkg remove`, `garnet pkg list` — package operations live as `garnet add`, `garnet remove`, `garnet list` directly. The maximum nesting depth is two (`garnet build --deterministic` is one sub-command with flags, not nested sub-commands).

---

## 4. The REPL as a First-Class Surface

The REPL is specified normatively in Mini-Spec §15. This section provides the *research argument* for treating REPL design as a research artifact rather than an afterthought.

### 4.1 Why the REPL matters

Three claims:

1. **Adoption velocity.** Languages with strong REPLs (Python, Ruby, Clojure, Julia) consistently see faster adoption among individual developers than languages without (Go, early Rust). The REPL is the first thing most newcomers try.
2. **Empirical truth-finding.** A REPL is the cheapest way to falsify or confirm a hypothesis about behavior. For an agent-native language (Paper III §4), this matters more than for a batch-compiled language: agents iterate.
3. **Documentation grounding.** Doc examples that ship as REPL transcripts are auto-tested via `garnet test --doctests` (planned v3.4). Documentation that doesn't pass is rejected by CI.

### 4.2 What Garnet's REPL inherits and what it changes

Inherits from Ruby's `irb`:
- Per-input expression evaluation
- Multiline detection on incomplete expressions
- Persistent bindings across inputs
- Last-expression-printed convention

Inherits from Rust's `evcxr`:
- Type display alongside value
- Module-level redefinition without ceremony

Garnet-specific changes:
- Safe-mode requires module envelope (Mini-Spec §15.4) — preserves the safe mode's compile-time guarantees in an interactive context
- `:type` / `:t` command for type inquiry without evaluation
- `:bench` command for ad-hoc performance measurement
- Hot-reload semantics: redefining a function takes effect immediately for new calls; running actors retain old behavior until reload command (Mini-Spec §15.3)

### 4.3 Performance contract

REPL evaluation latency for trivial expressions MUST be ≤ 10 ms (Mini-Spec §15.9). This is a hard constraint that shapes the implementation: a JIT-compiled REPL would amortize poorly per input. Garnet's REPL uses tree-walk interpretation with on-demand AST caching.

This trades peak throughput for latency, which is the right trade for an interactive surface.

---

## 5. Toolchain Layered Around the Compiler

### 5.1 Build cache (`.garnet-cache/`)

The Garnet build cache lives at `<project>/.garnet-cache/`. It contains:

- `episodes.log` — NDJSON record of every compilation event (per Paper VI Contribution 3, Compiler-as-Agent)
- `strategies.db` — SQLite database of synthesized compilation strategies (per Paper VI C3)
- `knowledge.db` — content-addressed knowledge store (per Paper VI C3)
- `manifest.json` — deterministic-build manifest (per Paper VI C7)

Per v3.3 CacheHMAC + ProvenanceStrategy (Security Layer 1), every record is HMAC-signed with a per-machine key at `~/.garnet/machine.key`. Foreign records (committed cache from another developer) are silently skipped, not honored.

### 5.2 Per-machine key (`~/.garnet/machine.key`)

Generated on first invocation of any `garnet` command that writes to `.garnet-cache/`. 32 random bytes from `getrandom`, file mode `0600` on Unix. Path overridable via `GARNET_MACHINE_KEY_PATH`.

### 5.3 LSP server (planned, v4.x)

A language server protocol implementation is planned for v4.1 or v4.2. The REPL's tree-walk interpreter shares the type-inference engine, so LSP `hover` and `goto-definition` reuse the existing infrastructure. Full design deferred to that release.

### 5.4 Formatter (`garnet fmt`)

`garnet fmt` enforces a single canonical style — no options, no configurable preferences. This is `gofmt`'s discipline: arguments about style are over once and for all by giving everyone the same answer.

The canonical style is documented in `C_Language_Specification/GARNET_Style_Guide.md` (planned for v3.5).

---

## 6. Cross-Platform Distribution (v4.2)

### 6.1 Installer targets

Per Stage 6 of the master plan:

- **Windows:** MSI installer via `cargo-wix`, signed with code-signing certificate, installs to `C:\Program Files\Garnet\`
- **macOS:** `.pkg` installer via `productbuild`, universal binary (x86_64 + arm64), notarized with Apple Developer ID
- **Linux:** `.deb` + `.rpm` via `cargo-deb` + `cargo-rpm`, plus `rustup`-style universal shell installer at `https://sh.garnet-lang.org`

### 6.2 First-run UX target

Time from "begin install" to "running my first Garnet program":

- Windows / macOS / Linux: under 2 minutes on a clean machine with internet
- Includes: install, `garnet new my_project`, `cd my_project`, `garnet build`, `garnet run`

This metric is testable; v4.2 verification includes timed installs on clean VMs of all three OSes.

### 6.3 Logo + brand integration

User has a Garnet logo ready. v4.2 integrates it into:

- Installer welcome screen (Windows MSI, macOS pkg)
- CLI `garnet --version` ASCII art (small)
- `garnet new <name>` output header
- REPL prompt banner
- README hero image
- Docs site favicon

### 6.4 Deferred items

- Public package registry (analogous to crates.io / RubyGems / npm) — design only in v4.x; implementation is post-MIT
- IDE integration beyond LSP — VS Code extension is post-MIT
- Hosted execution / sandbox — out of scope for the language paper

---

## 7. Migration Story (v4.1 Code Converter)

### 7.1 Strategic claim

A new language faces the chicken-and-egg problem: ecosystems require adopters; adopters require ecosystems. The Garnet response is a **bidirectional code converter** for the three closest source languages (Rust, Ruby, Python) shipping in v4.1.

The converter is documented in the v4.1 stage of the master plan; details deferred to that release. The relevant Paper VII commitment: **converter outputs ship in `@sandbox` mode by default** (per Mini-Spec §11 + v4.0 SandboxMode security item). Human audit is required before promoting converted code to non-sandbox status.

### 7.2 Why this matters for the language Paper

A converter is a language artifact — its existence (or absence) shapes the value proposition. Including the converter design in Paper VII rather than treating it as out-of-scope is the explicit acknowledgment that adoption-friction is part of the research deliverable.

---

## 8. Engineering Discipline as a First-Order Concern

Three engineering disciplines have been documented in the v3.2 → v3.3 implementation work, and Paper VII formalizes them as research-class commitments:

### 8.1 Adversarial-audit-before-trust

Every velocity push is followed by an adversarial re-read of the diff with a different question than the test author asked. v3.3 Phase 1A demonstrated this on the v3.2 implementation; details in `GARNET_v3_3_SLOP_REVERIFICATION.md`. The discipline:

- Tests verifying that features exist is not the same as tests that distinguish working from broken implementations.
- Every shipped feature is adversarially audited within one release of shipping.
- Findings are documented honestly with file:line and severity.

### 8.2 Threat-model-before-hardening

Every new attack surface (memory primitive, network primitive, hot-reload channel) is paired with its hardening before it ships. v3.3 Security Layer 1 demonstrated this for the v3.2 → v3.3 cleanup; v3.4 Security Layer 2 will demonstrate it for the new networking stdlib.

The discipline:
- No feature ships before its paired defense.
- Threat models are written explicitly with named attack vectors, not just "we considered security."
- Novel-threat-class identification is part of the threat model — Garnet's *strategy-miner adversarial training* and *Box<dyn Any> hot-reload type confusion* are documented in `GARNET_v3_3_SECURITY_THREAT_MODEL.md` as Garnet-specific novel threats with no prior art.

### 8.3 Sequencing discipline

Ordering matters. Plans don't say "we'll add X later." Plans gate releases on paired deliverables. v3.3 validated five sequencing rules; the master plan documents 11 more across v3.4 → v4.0. The discipline:

- Plans are explicit about ordering constraints.
- Slippage cascades — if a defense slips, its paired feature slips too.
- Plans are revisable (the plan is a living document) but ordering rules are not.

These three disciplines together constitute Garnet's "engineering process as research" claim. They are why this paper exists.

---

## 9. References (stub)

- Mini-Spec v1.0 (`C_Language_Specification/GARNET_v1_0_Mini_Spec.md`)
- Paper III (`A_Research_Papers/Paper_III_Garnet_Synthesis_v2_1.md`) — §3.1 Swift inheritance
- Paper V Addendum v1.0 (`A_Research_Papers/Paper_V_Addendum_v1_0.md`) — formal companions to Mini-Spec §§4.5, 8.5–8.6, 9.4, 11.6
- Paper VI (`A_Research_Papers/Paper_VI_Garnet_Novel_Frontiers.md`) — the seven novel contributions
- Compiler Architecture Spec (`C_Language_Specification/GARNET_Compiler_Architecture_Spec.md`) — implementation details for the seven rungs
- v3.3 handoff and security artifacts (`F_Project_Management/GARNET_v3_3_*.md`)
- Master plan: `~/.claude/plans/i-ll-follow-plan-mode-proud-lollipop.md`

---

## 10. What v1.0 (full Paper VII) Will Add

This stub will be expanded into Paper VII v1.0 in Phase 4C of the master plan (v4.0 stage). Additions:

- Empirical findings from the 7 Paper VI experiments — these will land throughout each rung's "evaluation" subsection
- The full v3.2 → v4.0 engineering journey narrative with timeline + retrospective
- LSP server design (assuming v4.1 or v4.2 ships it)
- Cross-platform installer measurements from Stage 6 verification
- Code converter outcomes from v4.1 — pass rates, idiom-translation findings
- The "GitHub conversion stress test" data from Phases 2F + 3G

The stub is structured so that the full v1.0 grows in place — each section already has its scaffold; the v4.0 paper fills in the measured outcomes.

---

*Stub prepared 2026-04-16 by Claude Code (Opus 4.7) — Phase 1B Swift-inheritance close-out. The Mini-Spec §16 "single-CLI principle" is the spec hook; this paper is the research justification.*

*"Where there is no vision, the people perish: but he that keepeth the law, happy is he." — Proverbs 29:18*
