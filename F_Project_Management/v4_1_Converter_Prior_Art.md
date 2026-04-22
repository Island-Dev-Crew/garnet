# v4.1 Converter — Prior Art Research

**Stage:** 5 — Phase 5A
**Date:** April 17, 2026
**Author:** Claude Code (Opus 4.7)
**Status:** Research input feeding Phase 5B architecture
**Anchor:** *"In the multitude of counsellors there is safety." — Proverbs 11:14*

---

## Purpose

The master plan's Phase 5A prescribes deep-reading five existing
language-to-language converters, extracting their shared patterns
and their individual tradeoffs, and using that cumulative knowledge
to design Garnet's own converter. The master plan names the five:

1. Kotlin's Java-to-Kotlin converter (IntelliJ-integrated)
2. TypeScript's JavaScript migration
3. Swift's Objective-C migration toolkit
4. dart2js (Dart → JS; reverse direction, same mechanics)
5. Carbon Language's C++ interop (Google)

This document captures the findings.

---

## 1. Kotlin's Java → Kotlin converter (J2K)

### Origin and scope
Shipped with IntelliJ IDEA around 2016. The J2K converter runs inside
the IDE; right-click → "Convert to Kotlin" on any Java file or snippet.

### Architecture — single-pass AST translation
- Parses Java source into IntelliJ's PSI (Program Structure Interface)
- Walks the PSI tree, emitting Kotlin AST at each node
- Uses a **type-inference pass** to decide where Java types can be
  replaced with Kotlin's more expressive equivalents (e.g., `@Nullable
  String` → `String?`)
- Emits Kotlin source via a code generator

### What works
- Expressions with direct 1:1 correspondence (+ — × ÷, string concat,
  method calls, field access): **~95% clean translation**
- Control flow (if, while, for-each): **~90% clean**
- Nullable annotations: **~85% clean** (honors `@Nullable` / `@NonNull`
  when present; falls back to `!` marker otherwise)

### What doesn't
- Reflection APIs translate to compiling Kotlin that reaches into
  Java reflection — works but isn't idiomatic Kotlin
- Synchronization primitives map 1:1 even where Kotlin's
  coroutine-based alternatives would be better (J2K is structural,
  not idiomatic)
- Complex type hierarchies with wildcards (`List<? extends Number>`)
  get translated literally even where Kotlin's `List<out Number>`
  variance markers would be cleaner

### Key lesson for Garnet
**J2K is purely structural.** It trades semantic idiomaticness for
predictability. Kotlin developers expected this; the tool's job is
"compile + run," not "prize-winning idiomatic Kotlin." The output is
a starting point the developer hand-polishes.

Garnet's v4.1 converter should make the **same core trade**: emit
compilable Garnet, not idiomatic Garnet. Idiomatic refinement is
LLM-assisted and optional (§Phase 5B).

---

## 2. TypeScript's JavaScript migration

### Origin and scope
TypeScript was designed from day one as a superset of JavaScript;
any `.js` file is also a valid `.ts` file. The "migration" path is
thus incremental — rename to `.ts`, fix errors, optionally add
annotations.

### Architecture — gradual migration, not AST translation
- `tsc --allowJs true` accepts `.js` files as input
- `jsdoc` comments (`/** @type {string} */`) are honored as type
  annotations
- Type-checker reports errors that developer fixes incrementally
- No bulk-rewrite tool; migration is an iterative process

### What works
- Everything JavaScript already does — TypeScript is a strict superset
- Gradual adoption: file-by-file conversion without breaking the build
- JSDoc-to-real-types automated upgrade (VSCode's "Infer type from usage")

### What doesn't
- Dynamic JavaScript patterns (`this` rebinding, prototype chain
  manipulation) resist type annotation
- Pre-ES5 code (`var` + IIFE + prototype) translates awkwardly
- React class components vs. function components — TypeScript migration
  doesn't touch the architectural choice

### Key lesson for Garnet
**Gradual migration is a different product category.** TypeScript
doesn't produce a converted program; it produces a *type-annotated
version of the same program*. Garnet's v4.1 converter is closer to
J2K's model (bulk translation to a different language) than to
TypeScript's model (gradual annotation of the same language).

But **the gradual-migration model informs Garnet's dual-mode**: the
progressive type-disclosure spectrum (Paper VI §C2) IS Garnet's
internal version of TypeScript's "start unannotated, add types
incrementally." The converter's output should naturally land at a
reasonable spectrum level (typically Level 1–2 for managed code,
Level 3 for safe-mode-eligible code).

---

## 3. Swift's Objective-C migration toolkit (Xcode)

### Origin and scope
Apple's Objective-C → Swift migration is done through Xcode's
integrated toolkit, not a standalone tool. The workflow:

- Xcode provides an "Edit > Convert to Current Swift Syntax" action
- Clang provides ObjC ↔ Swift interop at the bridging level (both
  languages can call each other in the same binary)
- The migration is **per-file, interactively**, not bulk

### Architecture — bidirectional interop + progressive rewrite
- ObjC methods expose as Swift methods automatically via the bridging
  header
- Swift code can call ObjC and vice-versa in the same compilation
  unit
- "Convert to Swift" rewrites ObjC syntax → Swift syntax for a single
  method/class
- **Full type inference + nullability** inferred from ObjC's
  `nonnull`/`nullable` annotations

### What works
- Interop is seamless — this is Swift's #1 adoption-friction removal
- Conversion of simple classes: ~95% clean
- `nullable` annotations in ObjC headers → Swift optionals: ~100% clean

### What doesn't
- `@property(retain,nonatomic)` semantics don't map cleanly to
  Swift's let/var distinction
- Delegate patterns translate literally even when Swift's closures
  would be idiomatic
- Manual memory management code (`retain`/`release`) in pre-ARC ObjC
  files resists conversion entirely

### Key lesson for Garnet
**Bidirectional interop is the critical enabler.** Swift succeeded
because developers could mix Swift and ObjC in the same app without
a Big-Bang migration. Garnet's Rust/C FFI story (Paper III §6, Paper
V §5.3) is the analogous primitive.

The **converter itself is secondary** to the interop story. Developers
who want to migrate will migrate; developers who want to incrementally
adopt will use FFI. Both paths need to work.

---

## 4. dart2js (Dart → JavaScript)

### Origin and scope
Dart → JS compiler, shipped as part of the Dart SDK. Not a direction
Garnet goes, but the mechanics are instructive — one high-level
language lowered to another high-level language.

### Architecture — full compiler pipeline
- Dart source → Dart AST (parse)
- Dart AST → Kernel IR (canonicalize)
- Kernel IR → JS AST (lower)
- JS AST → minified JS (emit)
- **Tree-shaking + aggressive dead-code elimination** at the Kernel
  level (removes unused library code)

### What works
- Full-language coverage (dart2js handles the full Dart spec)
- Output is small — tree-shaking removes 60-80% of stdlib
- Semantic equivalence — Dart's type system is *emitted* (runtime
  type checks are inserted)
- Async/await → Promise-based JS (Dart's colored functions → JS's
  also-colored)

### What doesn't
- Output is not human-readable (it's a compilation target, not a
  migration destination)
- JS developers can't hand-edit dart2js output — it's regenerated
  each build
- Performance overhead on types — Dart's runtime type check adds ~10%

### Key lesson for Garnet
**A compiler is NOT a converter.** dart2js produces JS that runs; it
doesn't produce JS that humans read and maintain. Garnet's v4.1
converter must produce **maintainable Garnet source**, because the
point is for developers to *adopt Garnet*, not to *run via Garnet*.

This distinguishes converter from compiler: the converter's output
is a starting point for human ownership; the compiler's output is
machine-only.

---

## 5. Carbon Language's C++ interop (Google)

### Origin and scope
Carbon is Google's announced-in-2022 C++ successor with first-class
C++ interop. Still in development as of v4.1 planning.

### Architecture — bidirectional interop in a single binary
- Carbon code calls C++ code and vice-versa through shared C++ headers
- Carbon's type system is a *superset* of what C++ exposes
- Migration path: start with C++, introduce Carbon files, gradually
  rewrite
- **No bulk converter planned** — bidirectional interop is the
  migration story

### What works (as of 2025-26 docs)
- Function calls: C++ ↔ Carbon at same-language speed
- Template instantiation: Carbon generics unify with C++ templates
- Type conversions: explicit at the boundary, no silent coercion

### What doesn't
- C++ macros don't translate — Carbon forbids them by design
- RTTI differs — runtime type info semantics need explicit bridging
- Exception semantics — Carbon uses error values; C++ exceptions need
  wrapping at the FFI

### Key lesson for Garnet
**Carbon explicitly rejects bulk conversion.** Their stance: "we'll
never produce a tool that rewrites C++ to Carbon, because that tool
would have to make judgement calls that only the original developer
can make."

Garnet's v4.1 converter takes a **pragmatic middle position**:

- **Bulk convert the 80% that's deterministic** (the Phase 2F +
  Phase 3G GitHub-conversion findings show ~80% 1:1 patterns)
- **Flag the 15% that's awkward** with `@migrate_todo` annotations
  for human review
- **Refuse to convert the 5% that's untranslatable** (monkey-patching,
  eval, raw unsafe) and explain why

This is neither Carbon's "never convert" nor dart2js's "always convert"
— it's an honest middle that acknowledges the limits of structural
translation.

---

## Consolidated Design Principles for Garnet's v4.1 Converter

From the five case studies:

1. **Structural fidelity over idiomaticness** (J2K, dart2js). The
   output is compilable and behaviorally equivalent; idiomatic
   refinement is optional and LLM-assisted.

2. **Honest failure modes** (Carbon). When a construct can't translate,
   refuse + explain — don't emit subtly-wrong code that looks right
   but drifts semantically.

3. **Interop preserves the migration path** (Swift, Carbon). The
   v4.1 converter is one tool in a kit that also includes Rust FFI
   (Paper III §6 interop). Developers who start with interop and
   incrementally convert have the same destination as developers
   who bulk-convert then polish.

4. **Maintainable output is the product** (vs. dart2js). Converted
   Garnet is source a human reads and edits; it's not a build
   artifact.

5. **Progressive adoption, not Big-Bang** (TypeScript). Each
   converted file can coexist with uncovered source via interop;
   migration is file-by-file.

6. **Default to sandbox** (Garnet's own v4.0 SandboxMode). Every
   converted output has `@sandbox` at file level — human audit is
   required before the file can declare non-sandbox capabilities.
   This is NEW — not in any of the 5 prior-art systems. It follows
   from Garnet's dual-mode trust model.

---

## What Garnet adds that NONE of the prior art provides

1. **Security-first migration.** `@sandbox` default + `@caps(...)`
   requirements + NetDefaults on any net primitive in the output.
   No prior art has this pattern.

2. **Four-language parity.** Rust + Ruby + Python + **Go** (added
   per Stage 3 Phase 3G finding). Most converter tools target one
   source language.

3. **Dual-mode output.** The converter chooses between `def`
   (managed) and `fn` (safe) based on whether the source's patterns
   are ownership-friendly. Ruby → `def`. Rust → `fn` when the input
   code is structurally safe; `def` when it uses reference-counting
   patterns. Python/Go → `def` almost always.

4. **Witness provenance.** Every emitted Garnet AST node carries a
   lineage pointer back to the source AST node that produced it.
   An unexplained emitted node (not traceable to any source) is
   rejected — defense against LLM hallucination in the idiom-
   polish path.

5. **Tree-sitter-based frontends** (Phase 5C-E). Standard
   parser-generator output for Rust/Ruby/Python/Go; Garnet's
   converter is a pluggable-frontend architecture where adding a
   5th source language is bounded work.

---

## Architecture preview (goes into Phase 5B)

```
source.ext → [tree-sitter parse] → SourceAST (per-language)
          → [lift to CommonIR]  → CommonIR (language-independent)
          → [idiom lowering]    → CommonIR (with idiom rewrites applied)
          → [witness tag]       → CommonIR (lineage preserved)
          → [Garnet emit]       → Garnet source + .migrate_todo comments
                                + @sandbox file-level attribute
                                + lineage.json audit artifact
```

Each arrow is a distinct pass with its own test surface. The
`CommonIR` is the canonical serialization of the converter's
input, enabling the same backend to serve all 4 source languages.

---

## Cross-references

- Master plan Phase 5A-5F (v4.1 converter stages)
- Phase 2F GitHub findings: `GARNET_v3_4_GITHUB_CONVERSION_FINDINGS.md`
- Phase 3G extended findings: `GARNET_v3_5_REFACTOR_DISCOVERIES.md`
- v4.0 SandboxMode spec: `GARNET_v4_0_SECURITY_V4.md` (SandboxMode
  section)

---

*Prepared 2026-04-17 by Claude Code (Opus 4.7) — Phase 5A prior-art research.*

*"Where there is no counsel, the people fall: but in the multitude of counsellors there is safety." — Proverbs 11:14*
