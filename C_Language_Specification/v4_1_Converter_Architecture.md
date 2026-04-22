# v4.1 Converter — Architecture Specification

**Stage:** 5 — Phase 5B
**Date:** April 17, 2026
**Author:** Claude Code (Opus 4.7)
**Status:** Normative architecture for the `garnet-convert` crate
**Anchor:** *"Let all things be done decently and in order." — 1 Corinthians 14:40*

---

## Purpose

Normative architecture for Garnet's v4.1 code converter — the
`garnet convert` CLI subcommand and its backing `garnet-convert`
crate. Builds on Phase 5A prior-art findings to specify a
parse-to-emit pipeline with a language-independent Common IR.

---

## 1. High-level pipeline

```
                   ┌──────────────────┐
  source.rs ─────→│  rust_frontend   │
  source.rb ─────→│  ruby_frontend   │
  source.py ─────→│  python_frontend │── lifts to ──→ CommonIR
  source.go ─────→│  go_frontend     │
                   └──────────────────┘
                                           │
                                           ▼
                                    ┌───────────────┐
                                    │ idiom_lowering │
                                    └───────────────┘
                                           │
                                           ▼
                                    ┌───────────────┐
                                    │ witness_tagger │
                                    └───────────────┘
                                           │
                                           ▼
                                    ┌───────────────┐
                                    │ garnet_emitter │
                                    └───────────────┘
                                           │
                                           ▼
                                    source.garnet
                                    source.garnet.lineage.json
                                    source.garnet.migrate_todo.md
```

Each pass is:
- Independently testable (has its own unit-test suite)
- Idempotent on its output
- Emits structured diagnostics on failure

---

## 2. Common IR (CIR)

The CIR is Garnet's canonical intermediate representation for
conversion. Superset of all four source languages' common concepts,
with Garnet-specific annotations attached where necessary.

```rust
enum Cir {
    // Control
    Module { name: String, items: Vec<Cir>, sandbox: bool },
    Func { name: String, params: Vec<Param>, return_ty: CirTy, body: Vec<Cir>, mode: FuncMode },
    If { cond: Box<Cir>, then_b: Vec<Cir>, else_b: Option<Vec<Cir>> },
    While { cond: Box<Cir>, body: Vec<Cir> },
    For { var: String, iter: Box<Cir>, body: Vec<Cir> },
    Match { scrutinee: Box<Cir>, arms: Vec<MatchArm> },
    Return { value: Option<Box<Cir>> },
    Try { body: Vec<Cir>, catches: Vec<CatchArm>, finally: Option<Vec<Cir>> },

    // Expressions
    Literal(CirLit),
    Ident(String),
    Call { func: Box<Cir>, args: Vec<Cir> },
    MethodCall { recv: Box<Cir>, name: String, args: Vec<Cir> },
    FieldAccess { recv: Box<Cir>, name: String },
    BinOp { op: String, lhs: Box<Cir>, rhs: Box<Cir> },
    UnOp { op: String, operand: Box<Cir> },
    Assign { lhs: Box<Cir>, rhs: Box<Cir> },
    Lambda { params: Vec<Param>, body: Vec<Cir> },

    // Structure
    Struct { name: String, fields: Vec<FieldDecl> },
    Enum { name: String, variants: Vec<VariantDecl> },
    Impl { target: String, methods: Vec<Cir> },

    // Collection ops
    ArrayLit(Vec<Cir>),
    MapLit(Vec<(Cir, Cir)>),
    Index { recv: Box<Cir>, key: Box<Cir> },

    // Migration-escape
    Untranslatable { reason: String, source_lineage: Lineage },
    MigrateTodo { placeholder: Box<Cir>, note: String, source_lineage: Lineage },
}

enum FuncMode { Managed, Safe, Unspecified }
enum CirTy { Inferred, Concrete(String), Optional(Box<CirTy>), Generic(String) }
struct Param { name: String, ty: CirTy, ownership: Ownership }
enum Ownership { Borrowed, Owned, MovedIn, Default }
struct Lineage { source_lang: String, source_file: String, source_span: (usize, usize) }
```

The IR is **explicitly lossy** at the structural level but
**non-lossy at the intent level** — every node either translates
cleanly or is marked `Untranslatable` / `MigrateTodo` with its
source lineage preserved.

---

## 3. Frontends (per source language)

Each frontend's contract:

```rust
trait Frontend {
    fn parse(source: &str, filename: &str) -> Result<ParsedAst, ConvertError>;
    fn lift_to_cir(ast: ParsedAst) -> Result<Cir, ConvertError>;
    fn source_lang() -> &'static str;
}
```

### 3.1 Rust frontend

- **Parser:** `syn` crate (already widely used, maintained by the Rust team) OR `tree-sitter-rust` (for multi-version support)
- **Lifting rules:**
  - `fn` + `&` parameters → `Cir::Func { mode: Safe, params: [{ ownership: Borrowed }] }`
  - `&mut` → `ownership: Owned` with note
  - `Option<T>` → `CirTy::Optional(T)`
  - `Result<T, E>` → Garnet's native `Result<T, E>`
  - `impl Trait for Type` → `Cir::Impl`
  - `unsafe { … }` → `Cir::Untranslatable { reason: "unsafe block" }`
  - `extern "C"` → `Cir::Untranslatable { reason: "ffi boundary" }`
- **Expected clean rate:** 92-95% per Phase 3G stats

### 3.2 Ruby frontend

- **Parser:** `ripper` (stdlib) or `tree-sitter-ruby`
- **Lifting rules:**
  - `def name(args)` → `Cir::Func { mode: Managed }`
  - Blocks `do |x| … end` → `Cir::Lambda` (passed implicitly via §5.4 Mini-Spec)
  - `Class.new` → `Cir::Struct { name }` + `Cir::Impl`
  - `method_missing` → `Cir::MigrateTodo { note: "needs @dynamic" }`
  - `class Foo; end; Foo.new.define_method(:x) { … }` → similar
  - `eval(s)` → `Cir::Untranslatable { reason: "Garnet has no eval" }`
- **Expected clean rate:** 80-90% per Phase 3G

### 3.3 Python frontend

- **Parser:** Python's stdlib `ast` module (via FFI or tree-sitter-python)
- **Lifting rules:**
  - `def name(…):` → `Cir::Func { mode: Managed }`
  - `class Foo:` → `Cir::Struct + Cir::Impl`
  - `__init__` → emitted as constructor `def new(…)`
  - `if isinstance(x, T):` → `match x { T => … }` (tag dispatch)
  - Type hints → Garnet's Level 2 annotations
  - `*args`/`**kwargs` → `Cir::MigrateTodo { note: "Garnet has fixed arity" }`
  - Decorators → closure-wrap pattern
- **Expected clean rate:** 90-98% per Phase 3G (JSON validator 98%)

### 3.4 Go frontend (added per Phase 3G finding)

- **Parser:** Go's stdlib `go/parser` (via FFI) or `tree-sitter-go`
- **Lifting rules:**
  - `func foo(…)` → `Cir::Func { mode: Safe }` (Go's ownership maps clean)
  - `chan T` → Garnet actor protocol with typed message
  - `go fn()` → `spawn Actor { … }`
  - `select { case … }` → Garnet `match` on actor receive
  - `interface{}` → `dyn Trait` or structural `protocol`
  - `unsafe.Pointer` → `Cir::Untranslatable`
- **Expected clean rate:** 92-96% (channel → actor was unexpectedly clean per Phase 3G)

---

## 4. Idiom lowering

A CIR-to-CIR rewrite pass that applies language-specific idiom
corrections. Examples:

- `if let Some(x) = expr` in Rust → emitted as Garnet `match expr { Some(x) => … }` directly
- Ruby `each_with_index do |x, i|` → `for (i, x) in enumerate(xs)` (requires stdlib `enumerate`)
- Python `lambda x: x + 1` → Garnet `|x| x + 1` (closure syntax)
- Go `for _, v := range xs` → `for v in xs`

Each idiom is a `fn rewrite(cir: Cir) -> Cir` in `garnet-convert::idioms`.

---

## 5. Witness tagging

Every emitted CIR node carries `Lineage { source_lang, source_file,
source_span }`. After idiom lowering, the witness tagger:

- Walks the CIR post-idiom tree
- Verifies every node has a Lineage OR is a direct idiom-rewrite
  (which inherits lineage from the rewritten node)
- Rejects untagged nodes as potential hallucinations
- Emits `source.garnet.lineage.json` with the node-to-source mapping

This provides the **audit trail** Carbon's "never bulk convert"
position was suspicious of. A reviewer can trace any line of
converted Garnet back to its source origin.

---

## 6. Garnet emitter

`emit(cir: Cir, opts: EmitOpts) -> EmittedSource` produces:

- `source.garnet` — Garnet source with `@sandbox` at file level
  (v4.0 SandboxMode default for all converted code)
- `source.garnet.migrate_todo.md` — human-readable checklist of
  every `MigrateTodo` site and its note
- `source.garnet.lineage.json` — witness mapping per §5

Emitter discipline:

- Emit Garnet style conventions (2-space indent, no trailing whitespace)
- Never emit `unsafe` (would violate SandboxMode)
- Never emit `extern "C"` (same reason)
- Cap nested depth at 128 (sandbox budget)
- Use `@caps()` on `main` (empty — sandboxed code declares no caps
  until human audit)

---

## 7. SandboxMode default tagging

Every converted file starts with:

```garnet
@sandbox                # v4.0 SandboxMode — human audit required
@caps()                 # no capabilities until reviewer lifts sandbox
module ConvertedFromRust {
  …
}
```

To lift the sandbox, a human reviewer:

1. Audits the file end-to-end
2. Resolves every `@migrate_todo(...)` annotation
3. Replaces `@sandbox` with `@sandbox(unquarantine)` — this IS the
   audit-complete signal per v4.0 spec
4. Adds appropriate `@caps(...)` based on what the reviewed code
   actually does

The converter **cannot** emit `@sandbox(unquarantine)` by design.
This is the defense against malicious source crafting unsafe Garnet.

---

## 8. CLI integration

```
$ garnet convert rust src/foo.rs
converted: src/foo.garnet (sandboxed)
  - 142 Garnet AST nodes emitted
  - 3 @migrate_todo annotations
  - 0 @untranslatable constructs
  - lineage: src/foo.garnet.lineage.json
  - checklist: src/foo.garnet.migrate_todo.md

  review the file then change @sandbox to @sandbox(unquarantine)
  and add @caps(...) based on your audit.
```

Flags:

- `--fail-on-todo` — nonzero exit if any `MigrateTodo` appears
- `--fail-on-untranslatable` — nonzero exit if any construct is
  rejected entirely
- `--strict` — both of the above
- `--quiet` — suppress per-file human-readable output, still emit
  structured JSON
- `--out <dir>` — target directory (default: alongside source)

---

## 9. Test corpus

Per language, 10 programs in `garnet-convert/tests/corpus/<lang>/`:

- Rust: `word_count`, `toml_parser`, `cli_argparser`, `state_machine`,
  `channel_fan_in`, `retry_backoff`, `simple_server`, `json_pretty`,
  `csv_reader`, `ini_parser`
- Ruby: `ini_parser`, `dsl_rake`, `http_mock`, `each_patterns`,
  `struct_methods`, `block_yield`, `method_missing_demo`, `regex_use`,
  `file_walker`, `json_to_ruby`
- Python: `json_validator`, `csv_sql`, `decorators`, `dataclasses`,
  `type_hints`, `iter_gens`, `file_walk`, `http_client`, `regex_find`,
  `argparse_cli`
- Go: `chan_fan_in`, `goroutine_pool`, `http_server`, `select_loops`,
  `error_handling`, `interface_subtyping`, `context_cancel`,
  `json_encode`, `ticker_loop`, `simple_repl`

Each program in the corpus has:

- `source.<ext>` — original input
- `expected.garnet` — expected converter output (hand-maintained)
- `expected.lineage.json` — expected witness mapping

Conversion test: `cargo test --test corpus_roundtrip` compares.

**Success gate for v4.1:** ≥ 70% of test corpus produces output
equal to `expected.garnet` (modulo whitespace + comments). Remaining
≤ 30% produce partial output with `MigrateTodo` annotations at the
expected sites.

---

## 10. Interop path

The v4.1 converter is **one tool** alongside:

- Garnet FFI (Paper III §6 — call Rust/C directly from Garnet)
- Mini-Spec v1.0 §16 tooling (garnet new / build / run — production flow)

A migrating team can:

1. Keep most source in original language
2. FFI-call from Garnet to the original
3. Incrementally run `garnet convert` on individual files
4. Review + un-sandbox each converted file
5. Replace the FFI with native Garnet as conversion progresses

This mirrors Swift's ObjC migration pattern (Phase 5A §3).

---

## 11. Metrics surface

Every conversion emits a JSON metrics file:

```json
{
  "source_lang": "rust",
  "source_file": "src/foo.rs",
  "target_file": "src/foo.garnet",
  "source_loc": 142,
  "target_loc": 158,
  "expressiveness_ratio": 1.11,
  "clean_translation_percent": 87.3,
  "migrate_todo_count": 3,
  "untranslatable_count": 0,
  "sandbox_status": "quarantined",
  "converter_version": "0.4.0",
  "witness_hash": "3f2a...ce1e"
}
```

Aggregated metrics feed back into Paper VI §C1 (LLM pass@1) as a
floor estimate and into Paper III §7 as expressiveness data.

---

## 12. Versioning

- `garnet-convert` 0.4.0 — v4.1 initial release
- Each source-language frontend versioned independently
  (`rust_frontend 0.4.0`, `ruby_frontend 0.4.0`, etc.)
- Common IR versioned separately (`CIR_VERSION = "1"` in every
  lineage file); increment on any backwards-incompatible change

---

## 13. Cross-references

- Phase 5A prior art: `v4_1_Converter_Prior_Art.md`
- v4.0 SandboxMode: `GARNET_v4_0_SECURITY_V4.md` (SandboxMode §)
- Phase 2F + 3G GitHub findings (empirical input)
- Mini-Spec v1.0 §16 (single-CLI principle)
- Paper III §6 (migration-aware interop)

---

*Prepared 2026-04-17 by Claude Code (Opus 4.7) — Phase 5B architecture
specification.*

*"Through wisdom is an house builded; and by understanding it is established." — Proverbs 24:3*
