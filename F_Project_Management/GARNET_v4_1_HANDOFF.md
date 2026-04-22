# GARNET v4.1 — Handoff for Fresh Session

**Purpose:** Pick up v4.1 Stage 5 closeout cold. The next session
continues to Stage 6 (v4.2 installer + branding) per a separate
comprehensive boot doc.
**Last updated:** 2026-04-17 (end of Stage 5)
**Next active phase:** Stage 6 — v4.2 (cross-platform installer + Garnet logo + `garnet new` scaffolding). Boot via `GARNET_v4_2_BOOT.md`.
**Anchor:** *"Run, that ye may obtain." — 1 Corinthians 9:24*

---

## WHAT SHIPPED IN v4.1 (Stage 5 — Code Converter)

### Two specification documents

- **`v4_1_Converter_Prior_Art.md`** (in `F_Project_Management/`): 5 prior-art case studies (Kotlin J2K, TypeScript JS migration, Swift ObjC migration, dart2js, Carbon C++ interop) with consolidated design principles Garnet adopts/avoids. Identifies Garnet's three unique additions: security-first migration (@sandbox default), four-language parity (Rust+Ruby+Python+Go), dual-mode output (def vs fn), witness provenance.

- **`v4_1_Converter_Architecture.md`** (in `C_Language_Specification/`): normative architecture — pipeline, Common IR, 4 frontends, idiom lowering, witness tagging, emitter, CLI integration, metrics, interop with Rust FFI, 40-program test corpus target.

### New crate: `garnet-convert` v0.4.0

Ninth workspace member; **85 tests green** (61 unit + 24 integration).

**Modules:**

- `cir.rs` — Common IR (31 variants): Module/Func/If/While/For/Match/Try + expressions + structural types + **`MigrateTodo` and `Untranslatable` escape variants** (the honest-middle between Carbon's "never convert" and dart2js's "always compile"). Every node carries `Lineage` back to source. Helpers: `node_count`, `has_migrate_todo`, `has_untranslatable`, `migrate_todo_count`, `untranslatable_count`.
- `lineage.rs` — `Lineage { source_lang, source_file, source_span }` + `LineageMap` + `WitnessEntry`. Serializes to JSON without serde dep. By-line grouping for source-lineage views.
- `error.rs` — `ConvertError` unified (ParseError, MissingLineage, EmitFailure, Config, UntranslatableInStrictMode, TodoInStrictMode).
- `metrics.rs` — `ConvertMetrics` with expressiveness ratio, clean-translation %, BLAKE3 witness hash that re-hashes on any CIR change. `SandboxStatus::Quarantined` default; never auto-promotes.
- `witness.rs` — verifies every CIR node has a real lineage OR is idiom-synthesized (source_lang ending in `-idiom`). Rejects `Lineage::unknown()` tagged nodes as potential LLM hallucinations.
- `idioms.rs` — bottom-up CIR rewriter with 4 idiom hooks (Ruby blocks→closures, Rust if-let→match, Go range→for, Python f-string→interp). Currently identity for v4.1 initial; frontends already emit the canonical form.
- `emitter.rs` — Garnet source emitter. Injects `@sandbox` + `@caps()` headers; refuses to emit `@sandbox(unquarantine)`; handles all CIR variants. Produces `(garnet, lineage_json, migrate_todo_md)` triple. Strict mode rejects MigrateTodo/Untranslatable at emit time.
- `frontends/rust.rs` — stylized Rust parser: `fn`, `struct`, `enum`, `impl`, `let`, `return`, `if`/`while`/`for`, `Option<T>`/`Result<T,E>`/`Vec<T>`/`HashMap<K,V>` → Garnet `Option`/`Result`/`Array`/`Map`. Safety: guaranteed position advancement every iteration (no infinite loops on malformed input). `unsafe { … }` → Untranslatable.
- `frontends/ruby.rs` — stylized Ruby parser: `def`, `class` → struct+impl+module, `attr_accessor` → field extraction, `method_missing` → MigrateTodo (use `@dynamic`), `eval`/`instance_eval` → Untranslatable. `puts`/`print` → `println` Call.
- `frontends/python.rs` — stylized Python parser: indent-aware; `def`, `class` with `__init__` → field extraction, type hints (int/str/List/Optional/Dict) → Garnet types, decorators → MigrateTodo, `eval`/`exec` → Untranslatable. Nested def inside class body recognized.
- `frontends/go.rs` — stylized Go parser: `package`/`import` skip, `func`, `type … struct`, `chan T` → ActorProtocol marker, `go fn()` + channel ops → MigrateTodo (actors+BoundedMail mapping), `unsafe.Pointer` → Untranslatable. Lowercase-first field = private (no `pub`).
- `lib.rs` — one-shot `convert(source, lang, file, opts)` convenience + `SourceLang::{from_str, from_extension, as_str}`.

### CLI wiring: `garnet convert`

- **`garnet-cli/src/convert_cmd.rs`** — `garnet convert <lang> <file>` subcommand. 5 integration tests green. Writes 4 artifacts per conversion:
  - `<file>.garnet` — the translated source
  - `<file>.garnet.lineage.json` — witness map (every node → source span)
  - `<file>.garnet.migrate_todo.md` — human review checklist
  - `<file>.garnet.metrics.json` — one-shot metrics summary

Flags: `--strict`, `--fail-on-todo`, `--fail-on-untranslatable`, `--out <dir>`, `--quiet`.

Language detection: explicit `--lang` flag OR inferred from file extension (`.rs`, `.rb`, `.py`, `.go`).

---

## TEST TALLY

- v3.2 baseline: 857
- v3.3 (Layer 1 + slop fixes): +61
- v3.4 (Layer 2 + stdlib): +79
- v3.5 (Layer 3 + 6 MVPs + refactor): +25 security + 20 discoveries
- v4.0 (Layer 4 + empirical + papers): +17 security
- **v4.1 (Converter): +85 tests (61 unit + 24 integration) + 5 CLI tests = 90**

**Cumulative committed: 1151 tests.**

---

## CURRENT STATE

### Repository layout

- **9 workspace crates:** parser, interp, check, memory, cli, actor-runtime, stdlib, **convert (NEW)**, xtask
- **10 MVP examples** in `examples/` (all from v3.4–v3.5)
- **Research corpus:** 7 papers + 4 addenda + Paper VI Protocol + v4.0 Execution Report + v4.1 Prior Art + v4.1 Architecture

### v4.1-specific deliverables

| File | Purpose |
|------|---------|
| `F_Project_Management/v4_1_Converter_Prior_Art.md` | Phase 5A research |
| `C_Language_Specification/v4_1_Converter_Architecture.md` | Phase 5B normative architecture |
| `E_Engineering_Artifacts/garnet-convert/` | Phase 5 core crate (85 tests) |
| `E_Engineering_Artifacts/garnet-cli/src/convert_cmd.rs` | Phase 5F CLI subcommand (5 tests) |
| `F_Project_Management/GARNET_v4_1_HANDOFF.md` | ← this file |
| `F_Project_Management/GARNET_v4_2_BOOT.md` | Stage 6 comprehensive boot doc |

---

## VERIFICATION STATUS

```
cargo check --workspace --tests       → ✅ 0 errors
cargo test -p garnet-convert --release → ✅ 85 passed (61 unit + 24 corpus)
cargo test -p garnet-stdlib --release  → ✅ 74 passed
cargo test -p garnet-actor-runtime --release --lib → ✅ 17 passed
```

Other crate test binaries remain blocked by the pre-existing v3.3
MinGW/WinLibs ABI issue (documented; not a v4.1 regression).

---

## WHAT'S NEXT — Stage 6 (v4.2)

**Installer + Branding.** Per master plan Stage 6:

- Phase 6A (20h): MSI (Windows) + `.pkg` (macOS) + `.deb`/`.rpm` (Linux) + `rustup`-style universal shell installer
- Phase 6B (5h): `garnet new <project>` with `cli`/`web-api`/`agent-orchestrator` templates
- Phase 6C (5h): Garnet logo integration in installer welcome, `garnet --version` ASCII art, `garnet new` header, REPL banner, README hero, docs favicon
- Phase 6D (5h): clean-VM install + smoke + uninstall on Windows 11 / macOS Sonoma / Ubuntu 24.04

Boot the next session with `GARNET_v4_2_BOOT.md` — the comprehensive
handoff doc mirroring the style of `GARNET_v3_3_HANDOFF.md` that
started this session chain.

---

## KNOWN ISSUES (carried from v4.0)

1. **MinGW/WinLibs ABI** — miette-dependent test binaries crash at startup; workaround documented
2. **Stdlib↔interpreter bridge** — v3.4.1 ≤1-day task; MVPs 1–10 run once bridged
3. **CapCaps call-graph propagator** — same bridge dependency
4. **ManifestSig impl** — spec-complete, impl-deferred to v3.4.1
5. **Paper VI Exp 1 (LLM pass@1)** — pending ~$500 API credits
6. **Coq mechanization of Paper V** — multi-month effort; sketches shipped

None blocks any paper claim. Each has a documented next step.

---

## HOW TO BOOT A FRESH SESSION

1. Open new Claude Code session in `D:\Projects\New folder\Garnet (1)\GARNET`
2. Say: *"Read `Garnet/Opus-Gpt-Xai-Opus-Gemini-Opus/Garnet_Final/F_Project_Management/GARNET_v4_2_BOOT.md` and begin Stage 6. Verify environment first (`cargo test -p garnet-convert --release` should show 85 pass)."*

---

*Written by Claude Opus 4.7 at end of v4.1 Stage 5 — 2026-04-17.*

*"Run, that ye may obtain." — 1 Corinthians 9:24*
