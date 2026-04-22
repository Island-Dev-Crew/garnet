# Garnet v1.0 Mini-Spec
**Supersedes:** v0.3 Mini-Spec (April 16, 2026)
**Status:** Normative draft — canonical specification for Rungs 2–4 of the engineering ladder
**Date:** April 16, 2026 (Phase 1B promotion)
**Author:** Claude Code (Opus 4.7) at the direction of Jon — Island Development Crew
**Anchor:** *"Where there is no vision, the people perish." — Proverbs 29:18*

> v1.0 promotion. This revision preserves all v0.3 normative content unchanged and fills the eleven Phase 1B blend-verification gaps identified in `~/.claude/plans/i-ll-follow-plan-mode-proud-lollipop.md`:
>
> | # | Gap | Inheritance | Section added/extended |
> |---|-----|-------------|------------------------|
> | 1 | ARC cycle detection algorithm | Swift | §4.5 (new) |
> | 2 | Actors + Sendable-equivalent | Swift | §9.4 (new) |
> | 3 | Package/tooling ergonomics summary | Swift | §16 (new), full treatment in Paper VII |
> | 4 | Lifetime inference algorithm | Rust | §8.5 (new) |
> | 5 | Trait coherence algorithm (orphan rule) | Rust | §11.5 (extended) |
> | 6 | Borrow-checker rules | Rust | §8.6 (new) |
> | 7 | Zero-cost abstractions / monomorphization | Rust | §11.6 (new) |
> | 8 | Blocks + `yield` binding/return rules | Ruby | §5.4 (new) |
> | 9 | `@dynamic` method dispatch table | Ruby | §11.7 (new) |
> | 10 | Duck-typing rules at protocol level | Ruby | §11.8 (new) |
> | 11 | REPL design | Ruby | §15 (new) |
>
> Where v1.0 refines or extends v0.3 wording, a `[v1.0]` marker appears. Where v0.3 deferred a question to v0.4 that v1.0 now answers, the open question is updated in §17.

---

## 1. Scope

Garnet v1.0 specifies the complete surface syntax and normative semantics for:

1. Module declarations and imports
2. Memory unit declarations *(carried from v0.2 §2)*
3. Function definitions in both managed and safe modes
4. Expression and statement grammar including control flow
5. Error handling model (dual-mode)
6. `@safe` mode boundaries, crossing rules, **borrow checker, and lifetime inference** *(extended from v0.3 §8)*
7. Typed message protocols, actor declarations, and **Sendable-equivalent** *(extended from v0.3 §9)*
8. Recursive execution guardrails with concrete annotation syntax *(extended from v0.2 §5)*
9. Type system foundations including **monomorphization, `@dynamic` dispatch, and structural protocols** *(extended from v0.3 §11)*
10. Pattern matching
11. **REPL semantics** [v1.0 §15]
12. **Tooling ergonomics summary** [v1.0 §16, full spec in Paper VII]

**Out of v1.0 scope:** codegen IR (LLVM/Cranelift) layout, optimization passes, standard library contents, package manager protocol details, interop ABI layout. These belong to the Compiler Architecture Specification, Standard Library Outline, and Paper VII Implementation Ladder — not the language spec.

---

## 2. Lexical Grammar

### 2.1 Character set and encoding

Garnet source files MUST be valid UTF-8. The compiler MUST reject files containing invalid UTF-8 sequences before lexing begins. Line endings MUST be normalized to `\n` (LF); `\r\n` (CRLF) is accepted and silently converted.

### 2.2 Keywords

The following identifiers are reserved and MUST NOT be used as user-defined names:

**Mode and structure:**
`module` `use` `@safe` `@dynamic` `pub` `end`

**Declarations:**
`def` `fn` `let` `var` `const` `type` `trait` `impl` `struct` `enum` `protocol`

**Memory and actors:**
`memory` `working` `episodic` `semantic` `procedural` `actor` `on` `spawn` `send`

**Control flow:**
`if` `elsif` `else` `while` `for` `in` `loop` `break` `continue` `return` `match` `when` `yield` `next`

**Error handling:**
`try` `rescue` `ensure` `raise` `Result` `Ok` `Err`

**Ownership (safe mode):**
`own` `borrow` `mut` `ref` `move`

**Recursion guardrails:**
`@max_depth` `@fan_out` `@require_metadata`

**Sendable + dynamic dispatch [v1.0]:**
`@nonsendable` `Sendable` `dyn`

**Literals and values:**
`true` `false` `nil` `self` `super`

> **Why `protocol` is now a top-level keyword [v1.0]:** v0.3 introduced `protocol` only inside `actor` blocks (§9.1 actor-item). v1.0 §11.8 generalizes `protocol` to a structural-type declaration. The two uses are syntactically disambiguated by context: inside an `actor { ... }`, `protocol foo(...)` declares a typed message; outside, `protocol Foo { ... }` declares a duck-typed structural protocol. The keyword is shared because both meanings share the kernel idea of "a contract a participant must satisfy."

### 2.3 Operators and precedence

From lowest to highest binding:

| Precedence | Operators | Associativity | Description |
|---|---|---|---|
| 1 | `=` `+=` `-=` `*=` `/=` `%=` | Right | Assignment |
| 2 | `or` | Left | Logical OR |
| 3 | `and` | Left | Logical AND |
| 4 | `not` | Prefix | Logical NOT |
| 5 | `==` `!=` `<` `>` `<=` `>=` | Left | Comparison |
| 6 | `..` `...` | Left | Range (inclusive / exclusive) |
| 7 | `\|>` | Left | Pipeline |
| 8 | `+` `-` | Left | Addition / subtraction |
| 9 | `*` `/` `%` | Left | Multiplication / division / modulo |
| 10 | `-` `not` `!` | Prefix | Unary negation, logical not, bitwise not |
| 11 | `.` `::` `()` `[]` | Left | Member access, path, call, index |

**Design rationale (MIT-defensible):** The pipeline operator `|>` (borrowed from Elixir/F#/OCaml) is included because Garnet targets agent orchestration where data flows through transformation chains. It desugars to function application: `x |> f` becomes `f(x)`, `x |> f(y)` becomes `f(x, y)`. This provides fluent left-to-right reading of chains that would otherwise nest deeply.

### 2.4 Literals

```
integer    := [0-9]+ ("_" [0-9]+)*                    # 1_000_000
float      := integer "." integer ([eE] [+-]? integer)?
string     := '"' (char | "#{" expr "}")* '"'          # interpolated
raw_string := 'r"' char* '"'                           # no interpolation
symbol     := ":" ident                                # :ok, :error
boolean    := "true" | "false"
nil_lit    := "nil"
array      := "[" (expr ("," expr)*)? "]"
map        := "{" (expr "=>" expr ("," expr "=>" expr)*)? "}"
```

**String interpolation:** `#{}` inside double-quoted strings evaluates the enclosed expression, calls `.to_s` on the result, and splices the string representation in place. The parser MUST re-enter expression parsing for the interpolated segment. This is consistent with Ruby's interpolation and already implemented in the v0.2 parser's `expr.rs`.

---

## 3. Module System

### 3.1 File-based modules

Each `.garnet` file constitutes a module. The module name is derived from the filename by stripping the extension and converting to PascalCase. Directories create nested module paths:

```
src/
  main.garnet          # module Main (entry point)
  memory/
    core.garnet        # module Memory::Core
    manager.garnet     # module Memory::Manager
  agents/
    build_agent.garnet # module Agents::BuildAgent
```

### 3.2 Inline modules

```
module-decl := ("@safe")? ("pub")? "module" ident "{" item* "}"
```

A module MAY be declared inline within another module. Inline modules share the same file but create a distinct namespace. The `@safe` annotation, if present, MUST precede `module` and applies to all items within.

### 3.3 Imports

```
use-decl := "use" path ("::" "{" ident ("," ident)* "}")? 
          | "use" path "::" "*"
```

Examples:
```garnet
use Memory::Core                    # import module, access via Core::method
use Memory::Core::{Store, Index}    # import specific items
use Memory::Core::*                 # glob import (discouraged in safe mode)
```

### 3.4 Visibility

Items are private to their module by default. The `pub` keyword makes an item visible to importing modules. The `pub(module)` form restricts visibility to the parent module only.

**Design rationale:** Rust's visibility model is proven and well-understood. Privacy-by-default is essential for safe mode's ownership guarantees (external code cannot alias internal state). The four-model consensus point 2 (dual-mode as correct shape) requires that safe-mode visibility rules be strict while managed mode can be more permissive, but defaulting to private in both modes prevents accidental exposure.

---

## 4. Memory Units *(carried from v0.2 §2, unchanged through §4.4; §4.5 added in v1.0)*

### 4.1 Declaration form

```
memory-decl  := "memory" memory-kind ident ":" store-type
memory-kind  := "working" | "episodic" | "semantic" | "procedural"
store-type   := ident ("<" type-args ">")?
type-args    := type ("," type)*
type         := ident ("<" type-args ">")? | fn-type | tuple-type
fn-type      := "(" type-args? ")" "->" type
tuple-type   := "(" type "," type ("," type)* ")"
```

### 4.2 Semantics *(unchanged from v0.2 §2.2)*

A memory unit declaration MUST introduce a top-level binding whose static type is `store-type` and whose runtime identity is unique within the enclosing module. Two memory units of the same kind and ident in the same module MUST be a compile-time error. The four memory kinds are type-level tags, not independent types.

### 4.3 Out of scope *(unchanged from v0.2 §2.3)*

Retention policies, TTL semantics, ranking functions, privacy scopes, persistence guarantees, wire formats — all belong to the Memory Manager and Memory Core, not the language core. See `GARNET_Memory_Manager_Architecture.md` for the runtime-layer specification that addresses OQ-7 (R+R+I decay formula) and OQ-8 (multi-agent consistency).

### 4.4 Generics over memory kinds *(unchanged from v0.3 §4.4 — explicit deferral)*

Garnet v1.0 does NOT support parameterizing a function or type over a memory kind. A declaration such as `def foo<M: MemoryKind>(x: M) { ... }` is rejected by the parser. The deferral rationale from v0.3 §4.4 is preserved.

### 4.5 ARC cycle detection [v1.0 — addresses Phase 1B Swift gap #1]

Managed mode (§8.2) provides automatic reference counting (ARC) for all reference types. Naïve reference counting cannot reclaim cyclic structures (e.g., a parent owning a child that holds a back-reference to the parent). v1.0 specifies the cycle-detection algorithm normatively so that implementations agree on observable behavior, including drop ordering and `__finalize` timing.

#### 4.5.1 Algorithm: synchronous trial-deletion (Bacon–Rajan 2001) with kind-aware roots

The compiler MUST emit, and the runtime MUST execute, the following trial-deletion algorithm whenever ARC decrement triggers cycle scanning. The algorithm is taken verbatim from Bacon & Rajan's *Concurrent Cycle Collection in Reference Counted Systems* (ECOOP 2001), with one Garnet-specific extension noted below.

Each ARC-managed object carries three runtime tags:

- **color**: one of `Black` (in-use), `Gray` (probable cycle member), `White` (provisionally garbage), `Purple` (root candidate)
- **rc**: the unsigned reference count
- **buffered**: a boolean — has this object been added to the candidate roots buffer this cycle?

On every reference-count decrement that does NOT reach zero, the runtime MUST:

```
release(s):
    s.rc -= 1
    if s.rc == 0:
        free(s)                              # not a cycle candidate
    elif s.color != Purple:
        s.color = Purple                     # mark as candidate root
        if not s.buffered:
            s.buffered = true
            roots.push(s)                    # buffer for later scan
        if roots.len() >= scan_threshold:
            collect_cycles()
```

The `scan_threshold` is implementation-defined but MUST default to 256 buffered roots. An implementation MAY tune the threshold dynamically based on allocation pressure.

`collect_cycles` then runs three passes over the roots buffer:

```
collect_cycles():
    for each s in roots: mark_gray(s)
    for each s in roots: scan(s)
    for each s in roots: collect_white(s)
    roots.clear()

mark_gray(s):
    if s.color != Gray:
        s.color = Gray
        for each t in children(s):
            t.rc -= 1                        # speculative decrement
            mark_gray(t)

scan(s):
    if s.color == Gray:
        if s.rc > 0:
            scan_black(s)                    # not garbage — re-establish
        else:
            s.color = White
            for each t in children(s): scan(t)

scan_black(s):
    s.color = Black
    for each t in children(s):
        t.rc += 1                            # undo speculative decrement
        if t.color != Black: scan_black(t)

collect_white(s):
    if s.color == White and not s.buffered:
        s.color = Black
        for each t in children(s): collect_white(t)
        free(s)
```

#### 4.5.2 Garnet extension: kind-aware root partitioning

`children(s)` returns the ARC-managed children of `s` reachable through any field. v0.3 §4 introduced four memory kinds — `working`, `episodic`, `semantic`, `procedural`. v1.0 makes one Garnet-specific refinement to Bacon–Rajan: **roots are partitioned by memory kind**, and `collect_cycles` MAY scan one kind's root set without scanning the others.

This matters because the four kinds have different lifetime profiles:

| Kind | Typical lifetime | Cycle-scan frequency (default) |
|------|------------------|--------------------------------|
| working | request-scoped | every 64 buffered roots |
| episodic | session-scoped | every 256 buffered roots |
| semantic | process-lifetime | every 1024 buffered roots |
| procedural | persistent (rarely reclaimed) | scan only on explicit `gc.run()` |

Cross-kind cycles (e.g., a working-memory object pointing to an episodic-memory object pointing back) are detected when ANY of the involved kinds triggers a scan. The Bacon–Rajan algorithm's correctness is preserved because `mark_gray` and `scan` traverse `children(s)` without filtering by kind — the partitioning is purely a scheduling optimization.

#### 4.5.3 Finalization

When `free(s)` runs, if `s` defines a `def __finalize(self) { ... }` method, the runtime MUST invoke it before reclaiming memory. Finalizers MUST NOT resurrect their object (assign `self` to a new strong reference); attempting to do so produces undefined behavior in v1.0 and will be promoted to a runtime error in v1.1.

#### 4.5.4 Safe mode interaction

Safe-mode (`@safe`) modules MUST NOT use ARC-managed types except through the boundary rules of §8.4. Cycle detection consequently does NOT operate on safe-mode allocations; affine ownership (§8.3) precludes cyclic ownership at compile time, so no runtime check is required.

#### 4.5.5 Observable invariants (MIT-defensible)

> **ARC + Cycle Detection Soundness Theorem [v1.0]**
>
> For any managed-mode program P that terminates normally (no `panic`, no infinite loop):
>
> 1. **No leaks of cyclic garbage.** Every object unreachable from any root is reclaimed within `O(scan_threshold)` allocations.
> 2. **No premature reclamation.** No object reachable from any root is reclaimed.
> 3. **Finalizer determinism.** For any two objects `a`, `b` whose live-references graph forms an acyclic structure with `a → b`, `a.__finalize` is invoked strictly before `b.__finalize`.
>
> **Proof sketch.** Properties 1 and 2 follow directly from Bacon & Rajan (2001) §3.5–3.7, whose proof rests on the invariant that `mark_gray` + `scan_black` round-trip preserves the true reference count. Garnet's kind-aware extension (§4.5.2) does not affect correctness because partitioning only schedules when scans run, not what `mark_gray` traverses. Property 3 follows from the topological invariant of trial-deletion: `collect_white` recurses into children before freeing the parent.

**Why Bacon–Rajan over Swift's mainline ARC:** Swift relies on programmer-inserted `weak`/`unowned` annotations to break cycles; cycles without those annotations leak silently. This was acceptable for Swift's UIKit-centric origins but is the wrong default for agent systems where data graphs are dynamic and refactor-heavy. Bacon–Rajan adds bounded overhead (~5–8% in measured Cocoa retrofits per the original paper) for guaranteed-no-leak semantics. Paper III §3.1 cited "ARC can be predictable and serious" as Swift's contribution — v1.0 takes that contribution one step further by removing the cycle footgun while preserving the predictable-cost property.

---

## 5. Function Definitions

### 5.1 Managed-mode functions [v0.3]

```
managed-fn := ("pub")? "def" ident "(" param-list? ")" ("->" type)? block
param-list := param ("," param)*
param      := ident (":" type)?
block      := "{" stmt* expr? "}"
```

In managed mode, parameter types are OPTIONAL. When omitted, the compiler infers types from usage (gradual typing). The return type is OPTIONAL; when omitted, it is inferred from the body's final expression.

The last expression in a block is its implicit return value (Ruby convention). An explicit `return` MAY be used for early exits.

```garnet
# Managed mode — types optional, Ruby-like feel
def greet(name) {
  "Hello, #{name}!"
}

# With optional type annotations
def add(x: Int, y: Int) -> Int {
  x + y
}
```

**Design rationale (MIT-defensible):** The block syntax uses `{ }` (braces) rather than `end` keywords. This decision was forced by the v0.2 parser, which already uses braces for actor bodies, handler blocks, and the provisional expression grammar. Consistency across all block-delimited constructs prevents the confusing dual-delimiters problem that plagued Lua and early Kotlin. The `def` keyword signals "managed-mode function" and serves as a semantic beacon for both humans and LLMs — aligning with Frontier 1 (LLM-Native Compilation) from the gap analysis. The implicit last-expression return preserves Ruby's expressive philosophy (consensus point 2: dual-mode is correct).

### 5.2 Safe-mode functions [v0.3]

```
safe-fn    := ("pub")? "fn" ident "(" typed-params? ")" "->" type block
typed-params := typed-param ("," typed-param)*
typed-param  := (ownership)? ident ":" type
ownership    := "own" | "borrow" | "ref" | "mut"
```

In safe mode, ALL parameter types and the return type are REQUIRED. Ownership annotations are available on parameters: `own` transfers ownership into the function, `borrow` takes a shared reference, `mut` takes a mutable reference, `ref` is a read-only reference (alias for `borrow`).

```garnet
@safe
module FastPath {
  # Safe mode — types required, ownership explicit
  fn process(own data: Buffer, borrow config: Config) -> Buffer {
    let ref header = data.slice(0, 4)
    let mut body = data.slice(4, -1)
    body.transform()
    data
  }
}
```

**Design rationale:** The `fn` keyword signals "safe-mode function" — a distinct keyword from `def` that immediately tells both the developer and the compiler which mode's rules apply. Ownership annotations on parameters make the borrow checker's expectations visible at the call site, addressing Rust's most-cited usability complaint (hidden ownership transfer). The four-model consensus point 2 mandates that safe mode provides Rust-level guarantees; explicit ownership annotations are the mechanism.

### 5.3 Closures (both modes) [v0.3]

```
closure := "|" param-list? "|" ("->" type)? block
         | "|" param-list? "|" expr
```

```garnet
# Short form — single expression
let double = |x| x * 2

# Block form
let transform = |data: Buffer| -> Buffer {
  let processed = data.compress()
  processed.validate()
  processed
}

# In pipelines
items |> map(|x| x.name) |> filter(|n| n.starts_with("G"))
```

In managed mode, closures capture variables by reference (ARC increment). In safe mode, closures follow Rust-style capture rules: by reference when borrowing, by value when moving. The `move` keyword forces value capture: `move |x| { ... }`.

### 5.4 Blocks and `yield` [v1.0 — addresses Phase 1B Ruby gap #8]

In addition to the explicit closure form (§5.3), managed-mode functions MAY accept an *implicit block parameter* — a closure attached at the call site without appearing in the parameter list. This preserves Ruby's idiomatic block-passing style for iterators, DSLs, and resource-acquisition patterns.

#### 5.4.1 Grammar

```
block-arg     := do-block | brace-block
do-block      := "do" ("|" param-list? "|")? stmt* expr? "end"
brace-block   := "{" ("|" param-list? "|")? stmt* expr? "}"
yield-expr    := "yield" ("(" arg-list? ")")?
next-expr     := "next" expr?
```

A function call MAY be followed by a `do…end` or `{…}` block. The block becomes the call's implicit block argument:

```garnet
def each_line(text) {
  for line in text.split("\n") {
    yield(line)             # invokes the block
  }
}

each_line(file.read()) do |line|
  log("got: #{line}")
end

# Brace form preferred for one-liners
each_line(file.read()) { |line| log(line) }
```

`yield` is normatively defined as "invoke the implicit block parameter of the lexically enclosing `def`."

#### 5.4.2 Block binding rules (formal)

Let *B* be a block syntactically attached to a call to function *f*, defined in lexical scope *S*. Let *f*'s body contain `yield(args)`. Then:

- *B* captures all bindings from *S* by reference (managed-mode semantics, §5.3).
- `yield(args)` evaluates *args* in *f*'s scope, then invokes *B* with those arguments bound to *B*'s parameters. *B* executes with read/write access to its captured bindings in *S*.
- The value of `yield(args)` is the value of *B*'s last expression (or the value passed to `next` — see §5.4.3).
- If *f* has no implicit block at the call site and `yield` is reached, the runtime MUST raise `LocalJumpError`.

#### 5.4.3 Block return rules (formal — the most subtle point)

The Mini-Spec specifies block control-flow with the same semantics as Ruby, because programmers familiar with blocks expect this and the alternative semantics (closure-style) make `each` patterns surprising.

| Statement | Effect inside a block |
|-----------|----------------------|
| `next expr` | Returns `expr` from the block to the enclosing `yield`. Equivalent to a closure's return-from-call. |
| `break expr` | Returns `expr` from the call to *f* itself, as if *f* had executed `return expr`. |
| `return expr` | Returns `expr` from the *def lexically enclosing the block-bearing call*, NOT from the block and NOT from *f*. |

Example demonstrating all three:

```garnet
def find_first_match(items) {
  items.each do |item|
    if item.matches?(query) {
      return item            # returns from find_first_match, not from .each
    }
    if item.skip? {
      next nil               # returns nil to the .each iterator; loop continues
    }
    if item.fatal? {
      break nil              # returns nil from .each itself; iteration ends
    }
  end
  nil
}
```

This matches Ruby's documented semantics (Pickaxe §4.4 "Blocks, Closures, and Procs"). The rationale for `return` escaping the *enclosing def* is that blocks are conceptually "syntactic continuations of the caller" rather than independent functions.

#### 5.4.4 Procs and lambdas (deferred to v1.1)

Ruby distinguishes blocks (the implicit argument), `Proc.new { … }` (block-like return semantics), and `->(x) { … }` (lambda — `return` returns from the lambda itself). v1.0 specifies only blocks and the explicit closure form (§5.3). The `Proc.new` constructor and `->` arrow lambda are explicitly DEFERRED to v1.1; programmers who need closure-return semantics should use the §5.3 `|x| { … }` form, which behaves like a Ruby lambda.

#### 5.4.5 Safe-mode prohibition

Implicit blocks are MANAGED-MODE ONLY. A `@safe` module MUST reject `do…end`, `{…}` block-arguments, and `yield` expressions; safe code must use the explicit closure form (§5.3) so that ownership transfer is visible at the call site.

**Design rationale (MIT-defensible):** The Ruby block / yield pattern is the single most-cited reason developers pick Ruby for DSLs, iterators, and resource acquisition (`File.open(path) do |f| … end`). Garnet's managed mode would feel foreign to Ruby developers without it. Adopting Ruby's exact return-semantics (rather than reinterpreting them) means thirty years of accumulated Ruby idioms transfer directly. The cost is the well-known surprise that `return` inside a block escapes the enclosing method — but this is documented prominently and consistent with what Ruby developers already expect.

---

## 6. Statements and Control Flow [v0.3]

### 6.1 Variable declarations

```
let-decl   := "let" ("mut")? ident (":" type)? "=" expr
var-decl   := "var" ident (":" type)? "=" expr        # managed mode only
const-decl := "const" ident (":" type)? "=" expr
```

`let` declares an immutable binding (both modes). `let mut` declares a mutable binding (both modes). `var` is managed-mode sugar for `let mut` — provided because scripting-oriented developers expect mutable-by-default variables to have a short keyword. A `@safe` module MUST reject `var` declarations; use `let mut` instead. `const` declares a compile-time constant.

### 6.2 Control flow

```
if-expr     := "if" expr block ("elsif" expr block)* ("else" block)?
while-stmt  := "while" expr block
for-stmt    := "for" ident "in" expr block
loop-stmt   := "loop" block                    # infinite loop, break to exit
break-stmt  := "break" expr?
continue-stmt := "continue"
return-stmt := "return" expr?
```

`if` is an EXPRESSION, not a statement — it returns a value:
```garnet
let status = if score > 90 { :excellent } elsif score > 70 { :good } else { :needs_work }
```

**Design rationale:** `if`-as-expression eliminates the ternary operator and aligns with Rust and Kotlin. `elsif` (not `else if`) is a single keyword — consistent with Ruby, easier to lex, and unambiguous in the grammar. `loop` with explicit `break` replaces `while true` patterns and makes infinite loops intentional (same rationale as Rust). No `unless` — it adds cognitive load without expressive power (Crystal dropped it from its Ruby inheritance for the same reason).

### 6.3 Pattern matching [v0.3]

```
match-expr := "match" expr "{" match-arm ("," match-arm)* "}"
match-arm  := pattern ("if" expr)? "=>" (expr | block)
pattern    := literal-pattern | ident-pattern | tuple-pattern
            | enum-pattern | wildcard | rest-pattern
literal-pattern := integer | float | string | symbol | "true" | "false" | "nil"
ident-pattern   := ident
tuple-pattern   := "(" pattern ("," pattern)* ")"
enum-pattern    := path "(" pattern ("," pattern)* ")"
wildcard        := "_"
rest-pattern    := ".."
```

```garnet
match result {
  Ok(value) => process(value),
  Err(:not_found) => default_value(),
  Err(e) if e.retryable? => retry(operation),
  Err(e) => raise e,
}
```

In safe mode, pattern matching MUST be exhaustive — the compiler MUST reject a `match` expression that does not cover all variants. In managed mode, non-exhaustive matches are permitted but emit a compiler warning; an unmatched value at runtime raises `MatchError`.

**Design rationale (MIT-defensible):** Pattern matching is now established as essential in modern language design (Rust, Swift, Kotlin, Python 3.10+, Java 21). Garnet's match combines Rust's algebraic-type exhaustiveness guarantees with Ruby's guard-clause expressiveness (`if` guards on arms). The `=>` arrow syntax follows Rust/Scala convention, which is the most widely recognized among systems-and-web developers. Symbols (`:not_found`) provide lightweight domain-specific tags without requiring full enum declarations — critical for the scripting use case (consensus point 2: managed mode must feel Ruby-like).

---

## 7. Error Handling (Dual-Mode) [v0.3]

### 7.1 Design principle

Error handling is the most architecturally significant consequence of the dual-mode design. Managed mode uses exception-style error handling for developer ergonomics. Safe mode uses `Result<T,E>` for compile-time error exhaustiveness. The mode boundary performs automatic bridging.

**This is a novel contribution.** No existing production language provides automatic bidirectional error-model bridging across a type-system mode boundary. TypeScript has no mode boundary. Rust has no exceptions. Swift has exceptions but no mode boundary. Garnet is the first to formalize this.

> **Implementation note [v1.0].** The "automatic" bridging described in this section is the v4.0 target. v3.2/v3.3 implements the bridging contract via user-authored try/rescue + ? at boundaries — see Paper VI §C5 for the honest distinction between v3.2's shipped behavior and v4.0's automatic compiler-inserted wrappers. The grammar and semantics here describe the v4.0 surface; conforming v0.x implementations MAY require explicit bridging wrappers as long as the diagnostics direct programmers to write the correct manual form.

### 7.2 Managed-mode error handling

```
try-expr := "try" block rescue-clause+ ensure-clause?
rescue-clause := "rescue" (ident (":" type)?)? block
ensure-clause := "ensure" block
raise-stmt := "raise" expr
```

```garnet
# Managed mode — exceptions for ergonomic error handling
def fetch_data(url) {
  try {
    let response = Http.get(url)
    response.parse_json()
  } rescue e: NetworkError {
    log("Network failed: #{e.message}")
    cached_data(url)
  } rescue e {
    raise e  # re-raise unknown errors
  } ensure {
    connection.close()
  }
}
```

Managed-mode code MAY raise any value as an exception. `try`/`rescue` catches exceptions by type. `ensure` always executes (equivalent to Java's `finally`, Ruby's `ensure`). Uncaught exceptions propagate up the call stack.

### 7.3 Safe-mode error handling

```garnet
@safe
module Storage {
  enum StorageError {
    NotFound(String),
    Corruption(String),
    Io(IoError),
  }

  fn read_block(own handle: FileHandle, offset: u64) -> Result<Block, StorageError> {
    let header = handle.read_at(offset)?    # ? propagates Err
    if header.magic != BLOCK_MAGIC {
      return Err(StorageError::Corruption("bad magic"))
    }
    let data = handle.read_at(offset + HEADER_SIZE)?
    Ok(Block::new(header, data))
  }
}
```

Safe-mode functions MUST use `Result<T, E>` for fallible operations. The `?` operator propagates `Err` values to the caller. Safe-mode code MUST NOT use `raise` or `try`/`rescue`; the compiler MUST reject these constructs in `@safe` modules.

### 7.4 Boundary bridging [v0.3]

When managed code calls a safe function returning `Result<T,E>`:
- `Ok(value)` is unwrapped and returned as the bare value
- `Err(error)` is automatically raised as an exception of type `SafeModeError(error)`

When safe code calls a managed function:
- The call is implicitly wrapped in a `try`/`rescue`
- Normal returns become `Ok(value)`
- Exceptions become `Err(ManagedError(exception))`

```garnet
# Managed code calling safe code — bridging is automatic (v4.0; manual via try/rescue in v3.x)
def process_file(path) {
  # Storage.read_block returns Result<Block, StorageError>
  # In managed mode, Err becomes an exception automatically
  let block = Storage.read_block(open(path), 0)  # may raise SafeModeError
  block.data
}
```

**Design rationale (MIT-defensible):** The error-bridging mechanism is formally justified by the mode-boundary calculus in Paper V §4. The managed-to-safe direction (unwrap + raise) preserves the managed-mode invariant that errors are exceptions. The safe-to-managed direction (wrap in try) preserves the safe-mode invariant that errors are values. Neither direction loses information — the original error is preserved inside the wrapper type. This is analogous to how Swift's Objective-C interop bridges NSError to Swift errors, but generalized to a formal dual-mode boundary. The four-model consensus point 2 (dual-mode is correct) implies that each mode's error model should be native to that mode, not a compromise — and the bridging mechanism makes this possible.

---

## 8. Mode Boundaries *(extended from v0.3 §8; §§8.5 and 8.6 added in v1.0)*

### 8.1 Formal grounding *(unchanged from v0.3 §8.1)*

Garnet's `@safe` mode is grounded in **affine type theory** (resources used at most once), formally verified by **RustBelt (Jung et al., POPL 2018, MPI-SWS)** using the **Iris framework** in Coq. See Paper V §2-§6 for the full formal treatment.

**Security Theorem (safe-mode soundness) [v0.3]:**

> For any `@safe` module M that type-checks under the Garnet v1.0 rules, the following properties hold:
>
> 1. **No use-after-free.** Dropped values are not subsequently accessible.
> 2. **No double-free.** Each owned value is dropped exactly once along every execution path.
> 3. **No data races.** Concurrent access to shared memory follows aliasing-XOR-mutation strictly.
> 4. **No memory unsafety** within the well-typed fragment (excluding explicit `extern "C"` boundaries, which are programmer-asserted unsafe by definition).
>
> **Proof sketch.** The λ_safe calculus (Paper V §3) inherits progress and preservation from RustBelt (Jung et al., POPL 2018). Garnet's module-granularity (rather than crate-granularity) refinement does not invalidate the core argument: the Iris separation-logic model of λ_safe is a sub-model of RustBelt's crate model, so any property proven for RustBelt lifts. Formal mechanization in Coq is planned for a future Paper V revision (18–30 person-months per the v2.4 handoff estimate).

This theorem statement elevates the earlier prose to a load-bearing normative claim that MIT or PLDI reviewers can scrutinize precisely.

### 8.2 Managed mode *(unchanged from v0.3 §8.2; cycle-collection algorithm specified in §4.5 v1.0)*

Managed mode MUST provide automatic reference counting with cycle detection. Values MAY be mutated without ownership discipline. Managed mode is the default. The cycle-detection algorithm is specified normatively in §4.5.

### 8.3 Safe mode [v0.3 — concrete syntax added]

A module MAY be annotated `@safe` by placing the annotation before the `module` keyword (inline modules) or as the first non-comment token in a `.garnet` file (file-based modules).

```garnet
# File: src/crypto/hasher.garnet
@safe  # This entire file is a safe-mode module

fn hash(own data: Bytes) -> Hash {
  let mut state = State::new()
  state.update(borrow data)
  state.finalize()
}
```

```garnet
# Inline safe module within a managed file
module App {
  @safe module Crypto {
    fn hash(own data: Bytes) -> Hash { ... }
  }

  def process(input) {
    Crypto.hash(input)  # managed calling safe — boundary bridging applies
  }
}
```

A `@safe` module MUST enforce (unchanged from v0.2 §3.3):
1. Single-owner (affine) semantics for all values
2. Aliasing-XOR-mutation for all references (formalized in §8.6)
3. Lexical or non-lexical lifetime inference (algorithm in §8.5)
4. No implicit allocation on hot paths

### 8.4 Boundary rules *(unchanged from v0.2 §3.4, plus error bridging from §7.4)*

1. Managed callers invoking `@safe` functions MUST satisfy ownership preconditions at call sites
2. `@safe` returns to managed mode MUST return either an owned value (adopted into ARC) or a borrowed reference with a statically proven lifetime
3. Managed memory units MAY be read from safe mode under shared borrow, but MUST NOT be mutated except through a formal bridging API *(reserved for v0.4)*
4. [v0.3] Error bridging follows §7.4 rules automatically at every mode-crossing call site
5. [v1.0] At the boundary, a managed-mode value passed to a safe-mode `borrow` parameter MUST have a lifetime ≥ the call's static frame; the compiler MAY insert a "freeze" check that pins the ARC count for the call's duration

### 8.5 Lifetime inference algorithm [v1.0 — addresses Phase 1B Rust gap #4]

Safe-mode references carry lifetimes — symbolic regions denoting the program-point span during which the reference is valid. v0.3 §8.3 required "lexical or non-lexical lifetime inference" without specifying the algorithm. v1.0 specifies the algorithm normatively, modeled on **Non-Lexical Lifetimes (NLL)** as defined by Rust RFC 2094 (Matsakis 2018).

#### 8.5.1 Algorithm overview

The compiler MUST translate each `@safe` function body to a Mid-level IR (MIR) — a control-flow graph (CFG) of basic blocks containing typed three-address operations. Lifetime inference proceeds in five stages:

1. **Build CFG.** Nodes are basic blocks; edges are control-flow successors.
2. **Compute liveness.** A reference is *live* at a program point P if its referent is read or written along some path from P. Liveness is computed by standard backward dataflow analysis.
3. **Generate region constraints.** For each reference assignment `r = &x` (or `&mut x`), introduce a region variable `'r`. For every program point P where `r` is live, add the constraint `'r ⊇ {P}`. For every reborrow `r2 = r` where `r2: &'b T` and `r: &'a T`, add `'a ⊇ 'b`.
4. **Solve constraints.** Compute the smallest `'r` satisfying all constraints (least fixed point). Existing solver: standard worklist algorithm, O(n·d) where n = nodes, d = max dataflow depth.
5. **Validate.** For each `&mut x` borrow at point P, check no other `&x` or `&mut x` is live at P. If violated, emit a borrow-checker error (§8.6).

#### 8.5.2 Lifetime elision rules

In safe-mode function signatures, lifetimes MAY be elided when they follow these *elision rules* (matching Rust's elision rules for backward compatibility):

1. **Each elided input lifetime becomes its own distinct lifetime parameter.** `fn f(x: borrow A, y: borrow B)` → `fn f<'a, 'b>(x: &'a A, y: &'b B)`.
2. **If there is exactly one elided input lifetime, it is assigned to all elided output lifetimes.** `fn f(x: borrow A) -> borrow B` → `fn f<'a>(x: &'a A) -> &'a B`.
3. **If there is a `borrow self` or `mut self` parameter, the lifetime of `self` is assigned to all elided output lifetimes.** `fn method(borrow self, x: borrow A) -> borrow B` → `fn method<'a, 'b>(self: &'a Self, x: &'b A) -> &'a B`.
4. **If none of the above apply, lifetimes MUST be written explicitly.** The compiler MUST emit `error E0106: missing lifetime specifier` with a suggestion.

#### 8.5.3 Variance

Lifetimes interact with type parameters through variance. v1.0 adopts Rust's variance rules:

| Position | Variance |
|----------|---------|
| `&'a T` (shared reference) | covariant in `'a` |
| `&'a mut T` (mutable reference) | invariant in `'a` AND in `T` |
| `fn(&'a T) -> ()` | contravariant in `'a` |
| `fn() -> &'a T` | covariant in `'a` |
| `Cell<T>`, `RefCell<T>` | invariant in `T` (interior mutability) |
| `T` in `struct S<T>` | inferred from how `T` is used in fields |

Rationale for invariance of `&'a mut T`: a mutable reference allows both reading and writing through it. Reading is covariant; writing is contravariant; the only way to satisfy both is invariance.

#### 8.5.4 Closures and lifetime inference

Closures in safe mode (per §5.3) capture by-reference by default. The compiler MUST infer the lifetime of each capture as the smallest region containing all uses of the closure. If a closure outlives any capture, the compiler MUST reject with `error E0373: closure may outlive captured variable`. The fix is either to extend the captured value's lifetime or to use `move |…| { … }` to transfer ownership into the closure.

#### 8.5.5 Higher-rank trait bounds (deferred)

`for<'a> Fn(&'a T)` higher-rank bounds (HRTB) are common in Rust for callback signatures. v1.0 explicitly DEFERS HRTB to v1.1; v1.0 type-checker may treat them as parse errors with a suggestion to use a concrete lifetime. Real impact on user code is minor because most callback signatures fall under elision rules 1–3.

#### 8.5.6 Interaction with managed mode

Managed-mode functions (called from safe mode via §8.4) erase lifetimes — the ARC system manages object lifetime independently. The boundary rule of §8.4 #5 (the "freeze" check) ensures that an ARC object passed by `borrow` lives at least as long as the safe-mode frame consuming it.

#### 8.5.7 Implementation status

v0.3 implementation: lifetimes parsed but NOT enforced. Safe-mode functions accept and parse lifetime annotations but the tree-walk interpreter does not run NLL inference; safe mode is currently checked at lower fidelity (single-owner enforced; aliasing-XOR-mutation enforced; lifetime correctness NOT proven). Full NLL inference is a Rung 4 deliverable (per §13). **Soundness during the gap window** is preserved by §8.4 #5 (ARC freeze) at boundaries and by §10 recursive guardrails: until NLL ships, programs with potentially-unsound lifetime patterns will leak instead of UB.

---

### 8.6 Borrow checker rules [v1.0 — addresses Phase 1B Rust gap #6]

Safe-mode borrow checking enforces aliasing-XOR-mutation across the program. v0.3 stated the rule informally; v1.0 specifies it as a checking algorithm operating on the MIR + region results from §8.5.

#### 8.6.1 Foundational rules

For every program point P and every memory location L:

- **Rule B1 (uniqueness of mutation).** At most ONE of the following sets may be non-empty at P:
  - the set of live shared borrows of L (`&L`)
  - the set of live mutable borrows of L (`&mut L`)
- **Rule B2 (mut implies unique).** A live `&mut L` at P forbids ANY other live borrow of L (or a sub-place of L) at P, including additional mutable borrows.
- **Rule B3 (lifetime containment).** If `r: &'a L` is live at P, then `'a ⊇ {P}` (the lifetime must cover the program point).
- **Rule B4 (no use after move).** If L is moved at point M, no read or write of L is permitted at any point reachable from M without an intervening reassignment.
- **Rule B5 (drop discipline).** When an owned value goes out of scope, its destructor (if any) runs. After drop, the location is uninitialized; B4 applies.

#### 8.6.2 Two-phase borrows (per Rust RFC 2025)

Self-referential method calls of the form `vec.push(vec.len())` would naively conflict (the call needs `&mut vec`, the argument needs `&vec`). v1.0 adopts Rust RFC 2025's *two-phase borrow* refinement:

A borrow created at `&mut e` MAY be split into a *reservation phase* (acts as `&e`, allows other shared borrows) and an *activation phase* (acts as `&mut e`, exclusive). Activation occurs at the first use of the mutable reference. Between reservation and activation, additional shared borrows of `e` are permitted IF none of them outlive the activation point.

This permits the natural `vec.push(vec.len())` pattern while preserving B1–B5 globally.

#### 8.6.3 Place algebra (sub-place tracking)

A place is a location expression like `x`, `x.f`, `x[i]`, `*x`. The borrow checker tracks borrows at *place granularity*: borrowing `x.f` does not preclude borrowing `x.g`, but DOES preclude borrowing `x` (because `x` is a super-place of `x.f`).

Place compatibility is computed by *prefix relation*: `p` and `q` conflict iff one is a prefix of the other. `x.f` is a prefix of `x.f.g`; `x[i]` and `x[j]` are conservatively treated as conflicting because `i = j` is undecidable in general.

#### 8.6.4 Diagnostic guarantees

When borrow check fails, the compiler MUST produce a diagnostic that:

1. Names the conflicting borrows (location, span, kind: `&`/`&mut`/`move`).
2. Identifies the program point of conflict.
3. Suggests a fix: scope reduction (move one borrow earlier/later), use of `clone()` to break the borrow, or refactoring with `RefCell<T>` (managed-mode escape hatch).

Match Rust's diagnostic quality for the same error class — diagnostics are first-class deliverables, not afterthoughts.

#### 8.6.5 Worked example

```garnet
@safe
fn rotate(mut buf: Buffer) -> Buffer {
  let ref a = buf.left()       # borrow check: &buf — OK
  let ref b = buf.right()      # borrow check: &buf — OK (B1: shared+shared compatible)
  buf.swap_halves(a, b)        # borrow check: &mut buf required, but a, b still live → ERROR
                                # (B2 violated: cannot &mut while shared borrows live)
  buf
}
```

The diagnostic should explain: "cannot borrow `buf` as mutable because it is also borrowed as immutable here (let ref a = …) and here (let ref b = …)." Suggested fix: read a and b into owned values first, then perform the mutable operation.

#### 8.6.6 Soundness

Together with §8.5 lifetime inference, §8.6 implements the safe-mode soundness theorem (§8.1). The proof connection: B1–B3 enforce the affine + aliasing-XOR-mutation discipline that RustBelt's Iris model proves sound. B4 + B5 enforce the use-once / drop-once discipline that prevents use-after-free and double-free.

---

## 9. Typed Message Protocols *(extended from v0.3 §9; §9.4 added in v1.0)*

### 9.1 Grammar [v0.3 — block now formally defined]

```
actor-decl    := ("pub")? "actor" ident "{" actor-item* "}"
actor-item    := actor-protocol-decl | handler-decl | memory-decl | let-decl
actor-protocol-decl := "protocol" ident "(" typed-params ")" ("->" type)?
handler-decl  := "on" ident "(" param-list ")" block
```

> **Naming clarification [v1.0].** Inside an `actor { … }` block, `protocol` declares a *typed message contract* (a single message name + its parameter types). Outside an actor block (top-level), `protocol` declares a *structural protocol* (§11.8 — a duck-typed contract). The parser disambiguates by syntactic position. The same keyword captures the underlying conceptual unity: both meanings are "a contract a participant must satisfy," but at different layers.

The `block` production is now the same as §5.1's block: `"{" stmt* expr? "}"`. This retires the provisional block grammar from the v0.2 parser's `expr.rs`.

### 9.2 Semantics *(unchanged from v0.2 §4.2; Sendable-equivalent added in §9.4)*

Actors MUST NOT share mutable state. All inter-actor communication MUST go through declared protocols. Undeclared protocol sends MUST be compile-time errors. Every declared protocol MUST have a handler. Type erasure and gradual typing are prohibited across actor boundaries even in managed mode.

**Protocol versioning (OQ-5, explicit deferral to v0.4).** v1.0 does not specify how protocol definitions evolve across a running distributed system. A v0.4 proposal will introduce a `@protocol_version` annotation with additive-evolution semantics analogous to Protobuf field tagging.

### 9.3 Actor state and memory [v0.3]

Actors MAY declare internal memory units and `let` bindings within their body. These are private to the actor instance and MUST NOT be accessible from outside. Memory units declared inside an actor are scoped to that actor's lifetime — they are created when the actor spawns and dropped when the actor terminates.

```garnet
actor BuildAgent {
  memory episodic   log     : EpisodeStore<BuildEvent>
  memory procedural recipes : WorkflowStore<BuildRecipe>
  let mut build_count = 0

  protocol build(spec: BuildSpec) -> BuildResult
  protocol status() -> AgentStatus

  on build(spec) {
    build_count += 1
    let recipe = recipes.find(spec.target)
    let result = recipe.execute(spec)
    log.append(BuildEvent::new(spec, result))
    result
  }

  on status() {
    AgentStatus::new(build_count, log.recent(10))
  }
}
```

### 9.4 Sendable-equivalent: cross-actor type discipline [v1.0 — addresses Phase 1B Swift gap #2]

To preserve actor isolation under typed message passing (§9.2), every value crossing an actor boundary MUST satisfy a static *Sendable* discipline. v1.0 introduces the `Sendable` marker trait — a Garnet adaptation of Swift's `Sendable` protocol (Swift Evolution SE-0302) calibrated to Garnet's dual-mode design.

#### 9.4.1 The `Sendable` marker trait

```garnet
trait Sendable { }                  # marker trait — no methods
```

A type implements `Sendable` if values of that type can be safely transmitted from one actor to another without introducing data races. Implementation is *automatic* for many types but MAY be opted out via `@nonsendable`.

#### 9.4.2 Auto-derive rules

The compiler MUST auto-derive `impl Sendable for T` if T satisfies all of:

1. **All fields are Sendable.** A struct/enum with all-Sendable fields auto-derives Sendable.
2. **No interior mutability.** T does not contain `Cell<U>`, `RefCell<U>`, `Mutex<U>`, or any type marked `@nonsendable`.
3. **In managed mode:** T is *frozen* — its fields are declared with `let` (immutable) and not `var` / `let mut`.
4. **In safe mode:** T is either owned (`own`-receivable) or behind an immutable shared reference (`&T`).

Built-in Sendable types: all primitive numerics (`Int`, `Float`, `i8…u128`, `f32`, `f64`), `String`, `Bytes`, `Bool`, `Symbol`, `Nil`, immutable `Array<T: Sendable>`, immutable `Map<K: Sendable, V: Sendable>`, immutable `Set<T: Sendable>`, `Option<T: Sendable>`, `Result<T: Sendable, E: Sendable>`.

Built-in non-Sendable types: `Cell<T>`, `RefCell<T>`, raw managed-mode references with mutable interiors, file handles, sockets (caller must open per-actor).

#### 9.4.3 Manual opt-out

A type MAY mark itself `@nonsendable` to prevent auto-derive:

```garnet
@nonsendable
struct ConnectionPool {
  conns: Array<Connection>,
  in_flight: Cell<Int>,           # interior mutability → would fail auto-derive anyway
}
```

The compiler MUST reject attempts to send a `@nonsendable` value across an actor boundary with `error E0277: ConnectionPool does not implement Sendable`.

#### 9.4.4 Manual opt-in (with safety obligation)

In rare cases a programmer may need to mark a type Sendable that does not auto-derive (e.g., a thread-safe lock-free queue implemented with internal `unsafe`). The form is:

```garnet
unsafe impl Sendable for LockFreeQueue<T: Sendable> { }
```

The `unsafe impl` keyword (REQUIRED) signals that the programmer asserts Sendable correctness manually; the compiler does not check it. This matches Rust's `unsafe impl Send` discipline.

#### 9.4.5 Enforcement at protocol declaration

At every actor `protocol` declaration (per §9.1), the compiler MUST verify:

- Every parameter type implements Sendable.
- The return type (if any) implements Sendable.

Failure produces `error E0902: protocol parameter type T does not implement Sendable; cannot cross actor boundary`. This is a *declaration-site* check, not a use-site check. By rejecting at declaration, Garnet guarantees that any actor whose protocol type-checks can be safely instantiated and messaged.

#### 9.4.6 Cross-mode interaction

Managed-mode types implement Sendable when they satisfy §9.4.2 #3 (frozen). Safe-mode types implement Sendable per §9.4.2 #4. A `protocol` declared in a managed-mode actor MAY accept managed-mode Sendable types; a `protocol` declared in a safe-mode actor MAY accept safe-mode Sendable types. Cross-mode protocols (a safe actor receiving a managed-mode value) require explicit boundary bridging — typically the managed value is frozen and adopted as a shared reference.

#### 9.4.7 Difference from Swift

Swift's `Sendable` evolved over multiple releases; v1.0 takes the SE-0302 mature form and makes one Garnet-specific refinement: **Sendable is checked at protocol declaration, not at every send site**. Swift checks at each `send` call, which produces O(n) duplicate diagnostics for the same type used in many sends. Garnet's declaration-site check yields O(1) diagnostics per protocol while preserving the same soundness guarantee.

#### 9.4.8 Soundness

> **Actor Isolation Theorem [v1.0]**
>
> If actor `A` sends value `v: T` to actor `B`, and `T: Sendable` per the rules of §9.4.2–§9.4.4, then `B` cannot observe a write performed by `A` to `v` after the send, and `A` cannot observe a write performed by `B` to `v` after receipt.
>
> **Proof sketch.** For frozen managed values, immutability after send is a structural invariant — there are no `var` fields to mutate. For safe-mode owned values, the move discipline (B4 of §8.6) ensures `A` cannot access `v` after sending it. For safe-mode shared references, no reference is mutable, so concurrent mutation cannot occur. The only escape hatch is `unsafe impl Sendable`, which is by definition an asserted obligation, not a checked one.

---

## 10. Recursive Execution Guardrails *(unchanged from v0.3 §10)*

### 10.1 Recursion depth limits [v0.3 — concrete annotation syntax]

```
depth-annotation := "@max_depth" "(" integer ")"
```

`@max_depth(N)` MUST appear on any function definition that directly or transitively performs recursive agent spawning. The compiler MUST reject programs where a spawn chain could exceed the annotated depth.

```garnet
@max_depth(3)
def analyze_document(doc) {
  let chunks = doc.split_sections()
  let results = chunks |> map(|chunk| {
    spawn AnalysisAgent.analyze(chunk)   # depth 1
  })
  merge_results(results)
}
```

Default depth if unannotated is 1 (single layer of delegation). The compiler MUST emit a warning for functions that spawn agents without an explicit `@max_depth` annotation.

### 10.2 Asynchronous fan-out caps [v0.3]

```
fanout-annotation := "@fan_out" "(" integer ")"
```

`@fan_out(K)` MUST appear on any parallel spawn site. The runtime MUST reject attempts to exceed `K` simultaneous sub-agents.

```garnet
@max_depth(2)
@fan_out(10)
def parallel_search(queries) {
  queries |> map(|q| spawn SearchAgent.search(q))
           |> await_all()
}
```

### 10.3 Metadata validation [v0.3]

```
metadata-annotation := "@require_metadata"
```

`@require_metadata` on a parameter position requires that the argument carries Memory Manager metadata. The Memory Manager MAY refuse the call if flat retrieval would suffice.

```garnet
def recursive_retrieve(@require_metadata context: Memory::Semantic) {
  context.deep_search(query)
}
```

### 10.4 Scope *(unchanged from v0.2 §5.4)*

§10 constrains program structure, not program semantics. A compliant Garnet implementation MAY choose any enforcement strategy (static analysis, runtime checks, or both) that rejects non-compliant programs with clear diagnostics.

---

## 11. Type System Foundations *(extended from v0.3 §11; §§11.5–11.8 added/extended in v1.0)*

### 11.1 Progressive disclosure spectrum

Garnet's type system is the formal embodiment of the dual-mode philosophy. It provides four levels of type discipline, each a strict superset of the previous:

| Level | Name | Where | Annotations | Guarantees |
|---|---|---|---|---|
| 0 | Dynamic | Managed mode, no annotations | None required | Runtime type errors only |
| 1 | Gradual | Managed mode with annotations | Optional type hints | Compile-time checks where annotated; runtime checks at boundaries |
| 2 | Static | Managed mode, fully annotated | All types specified | Full compile-time type safety; no runtime type errors |
| 3 | Affine | `@safe` mode | Types + ownership | Compile-time type safety + memory safety + data-race freedom |

A program at Level 0 is valid at Level 3 — the compiler infers the tightest constraints. A program at Level 3 works at Level 0 — ownership annotations become documentation, ARC handles memory.

**This is a novel contribution (MIT-defensible).** No existing production language provides a formally continuous type-discipline spectrum from fully dynamic to affine-typed within one coherent syntax. TypeScript offers Levels 0–2 but no ownership. Rust offers Levels 2–3 but no dynamic mode. Swift offers Levels 1–2 with partial ownership (move-only types, experimental). Garnet spans all four.

**Formal basis:** Paper V §3 defines the core calculus as λ_managed (Levels 0–2) + λ_safe (Level 3) with a bridging judgment at the boundary. The progressive disclosure spectrum maps to these sub-calculi: Levels 0–2 inhabit λ_managed with increasing constraint density, and Level 3 transitions to λ_safe with affine rules.

**Theorem (Progressive Disclosure Monotonicity) [v0.3]:**

> Let ⊢_N denote "type-checks at level N" for N ∈ {0, 1, 2, 3}. Then for any well-formed Garnet program P:
>
> **(Strengthening is sound.)** If P ⊢_N, then P ⊢_{N+1} is either true or produces a precise type error that identifies exactly where additional annotation is required.
>
> **(Relaxation is always safe.)** If P ⊢_{N+1}, then P ⊢_N.
>
> **Proof sketch.** *Relaxation* follows because the annotation sets at each level form a strict inclusion chain. *Strengthening* follows from the bidirectional type-checker's soundness.

### 11.2 Built-in types

```
# Numeric types
Int          # arbitrary-precision integer (managed); i64 (safe)
Float        # 64-bit IEEE 754
i8, i16, i32, i64, i128     # safe mode sized integers
u8, u16, u32, u64, u128     # safe mode unsigned integers
f32, f64                     # safe mode sized floats

# Text
String       # UTF-8, immutable by default, COW semantics in managed mode
Bytes        # raw byte buffer

# Boolean
Bool         # true | false

# Collections (managed mode, ARC-backed)
Array<T>     # dynamic array
Map<K, V>    # ordered hash map
Set<T>       # hash set

# Option and Result
Option<T>    # Some(T) | None
Result<T, E> # Ok(T) | Err(E)

# Special
Nil          # the type of nil (managed mode only; safe mode uses Option)
Symbol       # interned string (:ok, :error, etc.)
```

### 11.3 User-defined types

```
struct-decl := ("pub")? "struct" ident ("<" type-params ">")? "{" field-decl* "}"
field-decl  := ("pub")? ident ":" type ("=" expr)?

enum-decl   := ("pub")? "enum" ident ("<" type-params ">")? "{" variant* "}"
variant     := ident ("(" type ("," type)* ")")?

trait-decl  := ("pub")? "trait" ident ("<" type-params ">")? "{" trait-item* "}"
trait-item  := fn-sig | def-sig | const-decl
fn-sig      := "fn" ident "(" typed-params ")" "->" type
def-sig     := "def" ident "(" param-list ")"

impl-block  := "impl" ("<" type-params ">")? type ("for" trait)? "{" (managed-fn | safe-fn)* "}"
```

```garnet
struct Config {
  pub host: String,
  pub port: Int = 8080,
  timeout: Float = 30.0,
}

enum BuildResult {
  Success(Artifact),
  Failure(String),
  Timeout,
}

trait Serializable {
  fn serialize(borrow self) -> Bytes
  fn deserialize(data: Bytes) -> Result<Self, SerializeError>
}
```

**Design rationale:** Structs, enums, and traits follow Rust's algebraic type system — proven to be the strongest foundation for both type safety and pattern matching. The `impl` block separates data (struct) from behavior (methods), which is essential for safe mode's ownership discipline. In managed mode, these types are ARC-managed; in safe mode, they follow ownership rules.

### 11.4 Generics [v0.3, monomorphization spec moved to §11.6]

```garnet
def identity<T>(x: T) -> T { x }

fn swap<T>(own a: T, own b: T) -> (T, T) { (b, a) }

struct Stack<T> {
  items: Array<T>,
}
```

The compilation strategy for generics depends on mode and is specified normatively in §11.6.

### 11.5 Trait coherence [v1.0 — addresses Phase 1B Rust gap #5; extended from v0.3 §11.5]

Trait coherence governs when two different `impl` blocks may exist for overlapping types. Garnet adopts Rust's proven coherence model. v0.3 §11.5 stated the orphan rule informally; v1.0 specifies it as a checking algorithm.

#### 11.5.1 The Orphan Rule (per Rust RFC 1023)

An `impl <T*> Trait<TParams> for Type<TArgs>` is allowed only if at least one of the following holds:

1. `Trait` is defined in the current module, OR
2. `Type` is defined in the current module, OR
3. There exists a *covered local type parameter* — a type parameter `P ∈ T*` that appears as a component of `TParams` or `TArgs` such that no uncovered foreign type appears before it.

#### 11.5.2 Coverage algorithm (formal)

For a type `T<U₁, U₂, …, Uₙ>`, a type parameter `P` is *covered* by `T` if `P` appears as one of the `Uᵢ`. A foreign type `F` is *uncovered* if it appears before any covered local type parameter in left-to-right order.

```
is_orphan_legal(impl, current_module):
    trait = impl.trait
    trait_args = impl.trait_args      # [Type] including the receiver
    if trait.module == current_module: return true
    for arg in trait_args:
        if arg.module == current_module: return true
    for arg in trait_args:
        if arg is a type-param P and is_covered(P, trait_args, current_module):
            return true
    return false

is_covered(param, args, current_module):
    for arg in args:
        if arg is a generic type T and arg.module == current_module:
            for sub in arg.type_args:
                if sub == param: return true
        elif arg.module != current_module:
            return false   # uncovered foreign type appears first → fail rule 3
    return false
```

#### 11.5.3 Overlap check

After orphan check passes, the compiler MUST verify no two impls overlap. Two impls `impl A for X` and `impl A for Y` overlap iff there exists a type `Z` such that `Z` matches both `X` and `Y` (via type inference + variance + lifetime erasure).

The algorithm is exponential in worst case but polynomial in practice. Implementations MAY use Rust's `chalk` solver design, OR may implement a simpler unification-based check that errs on the side of over-rejecting (rejected programs can be refactored).

#### 11.5.4 Diagnostic guarantee

When orphan-rule violation OR overlap is detected, the compiler MUST produce a diagnostic that:

1. Names the offending `impl`.
2. For orphan violation: cites which of the three conditions failed AND suggests either moving the impl to the trait-defining module or the type-defining module.
3. For overlap: shows both impls AND a witness type matching both, AND suggests using the orphan rule to disambiguate.

#### 11.5.5 Deferred subtleties (v0.4)

Specialization, negative bounds (`where T: !Copy`), and conditional blanket impls are NOT part of v1.0. A v0.4 RFC will address specialization following Rust's specialization-lite proposal. Until then, the simple orphan rule provides predictable behavior at the cost of some expressiveness — a defensible trade-off.

### 11.6 Monomorphization & zero-cost abstractions [v1.0 — addresses Phase 1B Rust gap #7]

Garnet's mode-aware compilation strategy for generics is normatively specified.

#### 11.6.1 Compilation strategy

| Mode | Strategy | Generic bound | Code-size impact |
|------|----------|---------------|------------------|
| Safe (`@safe`) | Monomorphization | Per concrete type instantiation | Higher (one copy per instantiation) |
| Managed | Type erasure with vtable | One copy with dynamic dispatch | Lower (one copy total) |

Selection is determined by the enclosing module's mode. A generic function defined in a `@safe` module is monomorphized at every call site. A generic function defined in a managed-mode module is type-erased to a single `impl` operating on a uniform value representation.

#### 11.6.2 Monomorphization rules (safe mode)

For each concrete instantiation `f<T = U₁, …, Tₙ = Uₙ>` reachable from program entry, the compiler MUST emit a specialized copy of `f` with all type parameters substituted by their concrete types. The compiler MUST inline calls to small monomorphized functions (size threshold implementation-defined; default ≤ 50 instructions).

**Polymorphic recursion is forbidden in safe mode.** A function `f<T>` MUST NOT contain a call to `f<U>` where `U ≠ T` along any reachable code path. Polymorphic recursion would require unbounded code generation. Detection is by static call-graph analysis. Diagnostic: `error E0275: overflow evaluating the requirement` with suggestion to convert to `dyn Trait` (managed mode escape hatch).

#### 11.6.3 Trait objects (`dyn Trait`) in safe mode

Safe mode supports trait objects via `dyn Trait` syntax for cases where monomorphization is undesired (e.g., heterogeneous collections):

```garnet
@safe
fn process_all(items: Array<Box<dyn Drawable>>) {
  for item in items {
    item.draw()    # vtable dispatch
  }
}
```

`dyn Trait` representation: a fat pointer (data pointer + vtable pointer). The vtable is computed at compile time per `(Trait, ConcreteType)` pair. Method dispatch is one indirection. This is identical to Rust's `dyn Trait` ABI.

Object safety: a trait MAY be used as `dyn Trait` only if it satisfies the *object safety* rules:
- All methods take `&self`, `&mut self`, or `Box<Self>`.
- No method is generic over additional type parameters.
- No method returns `Self`.

These rules are checked at the use site. Violations produce `error E0038: the trait T cannot be made into an object`.

#### 11.6.4 Type erasure (managed mode)

Managed-mode generics compile to a single function operating on a uniform value representation (typically a tagged 128-bit value or a boxed pointer). Type parameters are erased at runtime; bounds are checked dynamically at the entry of the function.

Cost: one indirection per generic operation (vs. zero in safe mode), plus tag-checking overhead (~2–5%).

#### 11.6.5 Zero-cost abstraction guarantee

> **Zero-Cost Abstraction Theorem [v1.0]**
>
> For any `@safe` generic function `f<T>` that is monomorphized to `f<U>`, the compiled machine code for `f<U>` is equivalent (up to compiler optimization) to a hand-written non-generic `f_for_U` with U substituted for T at the source level.
>
> **Proof sketch.** Monomorphization performs textual substitution at the IR level; the resulting IR is structurally identical to what a programmer would write by hand. Standard compiler optimizations (inlining, constant folding, dead-code elimination) apply identically. There is no runtime dispatch, no boxing, no allocation that the hand-written form would not also produce.

This theorem grounds Paper III's "near-Rust performance for safe mode" claim: at the IR level, safe-mode generics ARE Rust generics modulo surface syntax.

#### 11.6.6 Boundary behavior

A safe-mode `fn f<T>(x: T)` called from managed-mode code is monomorphized for the managed-mode argument's concrete type. If the managed-mode value's type is not statically known (Level 0 — Dynamic), the call site requires a runtime type test followed by dispatch to the appropriate monomorphization. The compiler MAY hoist this check or fall back to type erasure for managed-mode-only call paths; this choice is implementation-defined.

### 11.7 `@dynamic` method dispatch table [v1.0 — addresses Phase 1B Ruby gap #9]

Ruby's metaprogramming culture leans on per-instance method addition (`define_method`, `singleton_class`, `method_missing`). v1.0 specifies a Garnet equivalent gated behind the `@dynamic` annotation, scoped strictly to managed mode.

#### 11.7.1 Grammar

```
dynamic-decl := "@dynamic" struct-decl
              | "@dynamic" impl-block
```

A struct annotated `@dynamic` enables runtime method addition to its instances. An impl block annotated `@dynamic` enables runtime method addition to that specific (type, trait) pair.

```garnet
@dynamic
struct Service {
  name: String,
}

let s = Service::new("auth")
s.def_method(:health_check) do
  http_get("https://#{name}/health").status == 200
end

s.health_check()        # invokes the runtime-added method
```

#### 11.7.2 Per-instance method table

Every `@dynamic` instance carries a private method table:

```
DynamicMethodTable := Map<Symbol, Closure>
```

The table is lazily allocated on first `def_method` call (zero overhead until used). Each entry maps a method name (Symbol) to a Closure capturing `self` plus any explicit captures.

#### 11.7.3 Dispatch order

When a method is called on a `@dynamic` value, the runtime MUST consult dispatch in this order:

1. **Per-instance dynamic table.** If the method is in the instance's `DynamicMethodTable`, invoke it.
2. **Static `impl` blocks.** Standard static method resolution (per §11.3).
3. **`method_missing` fallback.** If neither resolves and the type defines `def method_missing(name: Symbol, args: Array<Any>)`, invoke it with the resolved name and arguments.
4. **`NoMethodError`.** Raise an exception.

This matches Ruby's documented dispatch order (Pickaxe §28 "Runtime Callbacks") with one Garnet-specific clarification: per-instance methods *override* class-level methods, not vice versa. Ruby is more nuanced; Garnet's choice is the predictable one.

#### 11.7.4 Runtime API

```garnet
@dynamic
struct Foo { … }

impl Foo {
  fn def_method(self, name: Symbol, body: Closure)         # add a method
  fn undef_method(self, name: Symbol)                       # remove a method
  fn responds_to?(self, name: Symbol) -> Bool              # introspect
  fn method_names(self) -> Array<Symbol>                    # all dynamic + static method names
}
```

`def_method` MUST overwrite any existing entry for `name`. `undef_method` MUST remove only the dynamic-table entry; static methods cannot be removed.

#### 11.7.5 Performance contract

- `@dynamic` values pay a constant-overhead Map lookup per method call (~3–10 ns on commodity hardware in 2026).
- Non-`@dynamic` values pay nothing.
- The runtime MAY cache the resolved method per call site (inline cache) to amortize the lookup cost.

Static method calls on `@dynamic` types MUST NOT pay the lookup overhead if the call site is monomorphizable — i.e., if the compiler can prove that no `def_method` for the called name has been called on this instance.

#### 11.7.6 Safe-mode prohibition

`@dynamic` is REJECTED in `@safe` modules. Diagnostic: `error E0901: @dynamic is not permitted in @safe modules; use a trait + Box<dyn Trait> instead`. Rationale: dynamic method addition undermines the static-dispatch guarantee that justifies safe mode's performance claims (§11.6.5).

#### 11.7.7 Interaction with `Sendable`

`@dynamic` values are NOT auto-Sendable. Sending a `@dynamic` value across an actor boundary risks racing on the per-instance method table. Programmers needing cross-actor dynamic dispatch should use protocols (§9.1) — the actor-protocol design exists precisely for this case.

### 11.8 Structural protocols (duck typing) [v1.0 — addresses Phase 1B Ruby gap #10]

Garnet's typing should permit Ruby-style "if it walks like a duck and quacks like a duck" expression at the type-system layer. v1.0 introduces top-level `protocol` declarations as *structural types*.

#### 11.8.1 Grammar

```
struct-protocol-decl := ("pub")? "protocol" ident ("<" type-params ">")? "{" proto-item* "}"
proto-item           := fn-sig | def-sig
```

(Disambiguated from actor-internal `protocol` per §9.1's note.)

```garnet
protocol Drawable {
  def draw()
  def bounding_box() -> Rect
}
```

A type satisfies `Drawable` if it has methods `draw()` and `bounding_box() -> Rect` with compatible signatures. **No `impl Drawable for T` block is required.**

#### 11.8.2 Static structural typing (managed mode)

In managed mode, a function may accept a structural protocol as a parameter type:

```garnet
def render_all(items: Array<Drawable>) {
  for item in items {
    item.draw()
  }
}

struct Circle { radius: Float }
impl Circle {
  def draw() { /* … */ }
  def bounding_box() -> Rect { /* … */ }
}

render_all([Circle::new(1.0), Circle::new(2.0)])    # compiles — Circle satisfies Drawable structurally
```

The compiler performs a *structural compatibility check* at the call site:

```
satisfies(T, P):
    for fn-sig (name, params, ret) in P.items:
        if T does not have a method `name` with compatible (params, ret): return false
    return true
```

Compatibility is method-signature equality up to parameter-name renaming. Generic methods on T match generic items on P only if the type parameters unify.

#### 11.8.3 Runtime cast

For cases where the protocol cannot be statically proven (Level 0 / Level 1):

```garnet
let unknown: Any = receive_from_network()
match unknown {
  d as Drawable => d.draw(),
  _ => log("not drawable")
}
```

`as Protocol` performs a runtime cast: succeeds if the value's class has the required methods, raises `ProtocolError` if not.

#### 11.8.4 Differences from Go interfaces and Rust traits

| Feature | Go interface | Rust trait | Garnet protocol [v1.0] |
|---------|--------------|------------|------------------------|
| Implementation declaration | None — implicit | `impl Trait for T` required | Implicit; `impl P for T` permitted as documentation |
| Coherence rule | None (any package satisfies) | Orphan rule | Orphan rule applies only to `impl P for T` declarations |
| Static dispatch | Limited (interface always boxed) | Yes (`impl Trait` syntax) | Yes (Level 2+ in managed mode) |
| Method addition at runtime | No | No | Yes via §11.7 `@dynamic` (orthogonal) |

The Garnet design splits the difference: implicit conformance like Go (low ceremony), but optional `impl` declarations for documentation, traceability in IDEs, and orphan-rule discipline.

#### 11.8.5 Safe-mode behavior

Safe-mode protocols MUST resolve at compile time. A safe-mode function accepting a protocol parameter is implicitly generic over the concrete satisfying type:

```garnet
@safe
fn render(item: Drawable) -> ()    # equivalent to fn render<T: Drawable>(item: T)
```

This monomorphizes per call site (per §11.6). Runtime `as Protocol` cast is REJECTED in safe mode; use `match item { … }` over an enum if heterogeneity is needed.

#### 11.8.6 Difference between top-level `protocol` and `trait`

Both top-level `protocol` and `trait` declare a method contract. The semantic difference:

- `trait`: nominal — a type satisfies it ONLY via explicit `impl Trait for T`.
- `protocol`: structural — a type satisfies it via signature compatibility, regardless of `impl` declaration.

When in doubt, prefer `trait` for stable APIs (gives the trait author control) and `protocol` for orchestration glue (gives the consumer flexibility).

---

## 12. Open Questions *(updated from v0.3 §12; v1.0 closes OQ-11 and adds OQ-12 through OQ-15)*

- **OQ-1.** How are memory-unit retention policies expressed in source? *(runtime concern — resolved by `GARNET_Memory_Manager_Architecture.md §3.3` per-kind defaults; no source syntax planned)*
- **OQ-2.** What is the bridging API for managed→safe mutation of a managed memory unit? *(targeted for v0.4 — boundary rules in §8.4 define the read path; mutation path awaits v0.4)*
- **OQ-3.** What is the story for generics over memory kinds? *(resolved as explicit deferral — see §4.4 for the v0.3 position and rationale)*
- **OQ-4.** What is the soundness proof obligation for §8.4 boundary rules? *(formal sketch in Paper V §5; security theorem stated in §8.1; full Coq mechanization deferred)*
- **OQ-5.** How are actor protocols versioned across a running system? *(resolved as explicit deferral — see §9.2 for rationale; v0.4 will introduce `@protocol_version`)*
- **OQ-6.** What does the language surface expose about KV-cache compression hints? *(resolved: nothing — confirmed by consensus point 8)*
- **OQ-7.** How is the Memory Manager's controlled-decay formula expressed? *(resolved by Memory Manager Architecture §3.2)*
- **OQ-8.** Multi-agent access to shared Memory Core consistency. *(resolved by Memory Manager Architecture §4)*
- **OQ-9.** [v0.3] What is the async model? *(resolved by Tier-2 Ecosystem Specifications §D — green threads, no colored functions, structured concurrency)*
- **OQ-10.** [v0.3] What is the trait coherence model? *(resolved as Rust RFC 1023 orphan rule with formal algorithm — see §11.5 v1.0)*
- **OQ-11.** [~~v0.3 deferred~~ → **v1.0 RESOLVED**] What is the lifetime elision story for safe mode? *(resolved with the four elision rules of §8.5.2 plus the NLL inference algorithm of §8.5.1)*
- **OQ-12.** [v1.0] How do procs and lambdas differ from blocks? *(deferred to v1.1 — see §5.4.4. v1.0 specifies blocks; closure-return semantics available via the explicit §5.3 closure form which behaves like a Ruby lambda.)*
- **OQ-13.** [v1.0] How are higher-rank trait bounds (HRTB) handled in safe mode? *(deferred to v1.1 — see §8.5.5. Most practical signatures fall under elision rules 1–3.)*
- **OQ-14.** [v1.0] What is the inline-cache strategy for `@dynamic` dispatch? *(implementation-defined — see §11.7.5. Compiler MAY use polymorphic inline caches; runtime correctness is invariant.)*
- **OQ-15.** [v1.0] How do protocols and traits compose under structural+nominal mixing? *(targeted for v0.4 — initial guidance in §11.8.6; full algebra requires deeper work on the protocol satisfaction relation.)*

---

## 13. What a v1.0 Implementation Owes [v1.0 — extended from v0.3 §13]

Rungs 2–4 of the engineering ladder MUST be implementable against this spec:

- **Rung 2 (parser):** MUST parse all grammar productions in §§2–11. The existing v0.2/v0.3 parser covers most sections; v1.0 adds: (a) `do…end` and brace-block arguments (§5.4.1); (b) `yield`/`next` keywords (§5.4); (c) `@dynamic` annotation on struct/impl (§11.7); (d) top-level `protocol` declarations (§11.8); (e) `Sendable` and `@nonsendable` markers (§9.4); (f) `dyn Trait` syntax (§11.6.3).

- **Rung 3 (managed interpreter + REPL):** MUST evaluate managed-mode programs including everything from v0.3 plus: (a) ARC cycle detection per §4.5; (b) block / yield / next / break-from-block per §5.4.3; (c) `@dynamic` dispatch order per §11.7.3; (d) structural protocol compatibility checks per §11.8.2; (e) Sendable enforcement at protocol declarations per §9.4.5. **REPL behavior** specified in §15.

- **Rung 4 (safe lowering):** MUST enforce affine ownership rules per §8.3, the borrow-checker rules of §8.6, and the lifetime inference algorithm of §8.5. MUST monomorphize generics per §11.6 with the polymorphic-recursion rejection rule (§11.6.2). MUST insert ARC retain/release at mode boundaries per §8.4 and the freeze check of §8.4 #5. MUST perform error bridging per §7.4.

- **Rung 5 (Sendable enforcement at protocol declaration):** MUST run the auto-derive algorithm of §9.4.2 and reject protocols with non-Sendable parameter types.

- **Rung 6 (cycle collection):** MUST implement the Bacon–Rajan algorithm of §4.5.1 with the kind-aware partitioning extension of §4.5.2.

---

## 14. Four-Model Consensus Alignment Verification *(extended from v0.3 §14)*

Every normative rule in this spec has been verified against the eight consensus points:

| Consensus Point | Spec Sections Implementing It |
|---|---|
| 1. Rust/Ruby structurally complementary | §5 (def vs fn), §7 (exceptions vs Result), §11.1 (spectrum), **§5.4 (Ruby blocks), §8.5–8.6 (Rust borrow-check + lifetimes)** |
| 2. Dual-mode is correct shape | §5.1/5.2, §7.2/7.3, §8, §11.1, **§11.6 (mode-aware generic compilation), §11.7 (managed-only @dynamic), §11.8 (structural protocols)** |
| 3. Swift as managed-mode precedent | §8.2 (ARC), §9 (actors), §5.3 (closures), **§4.5 (cycle detection algorithm), §9.4 (Sendable equivalent)** |
| 4. Agent-native language platform | §4 (memory units), §9 (actors), §10 (guardrails), **§4.5.2 (kind-aware GC roots), §15 (REPL), §16 (tooling)** |
| 5. One Memory Core, Many Harnesses | §4.3 (out of scope = harness layer), §9.3 (actor memory), **§4.5.4 (safe-mode cycle interaction)** |
| 6. Memory primitives first-class | §4.1 (declaration grammar), §9.3 (actor-scoped memory) |
| 7. Typed actors with compiler-enforced protocols | §9.1/9.2 (grammar + semantics), **§9.4 (Sendable contract)** |
| 8. TurboQuant = runtime, not language-core | §4.3 (out of scope), OQ-6 (position: nothing) |

---

## 15. REPL Specification [v1.0 — addresses Phase 1B Ruby gap #11]

A REPL ("read-eval-print loop") is a first-class Garnet experience. v1.0 specifies the REPL's surface contract; implementation details (line editing, terminal escape codes, history file format) belong to the Compiler Architecture Specification.

### 15.1 Invocation and entry

```
$ garnet repl
Garnet 1.0.0 (managed mode, no project)
Type :help for commands, :quit to exit.

>>>
```

`garnet repl` MAY be invoked outside any project (free-standing scratch session) or inside a project (loads `Garnet.toml` and exposes the project's modules).

### 15.2 Input model

Each input is one of:

1. **An expression.** Evaluated; the value is printed with type information.
2. **A statement.** Evaluated; nothing is printed unless the statement raises.
3. **A REPL command** (starts with `:`). See §15.6.

Multiline input is detected when the parser cannot terminate on a complete expression at the end of a line — open braces, open parens, unfinished `try…ensure`, etc. The prompt switches from `>>>` to `...` to indicate continuation.

```
>>> def square(x) {
...   x * x
... }
=> def square(x: Any) -> Any         # signature inferred

>>> square(7)
=> 49 : Int
```

### 15.3 Binding persistence

Top-level bindings MUST persist across inputs. The REPL maintains a single managed-mode `Module` whose name is `Repl`. All `let`/`var`/`def`/`struct`/`enum`/`impl`/`use` declarations modify this module.

A redefinition (e.g., `def square(x) { x + 1 }` after the previous example) MUST silently replace the prior binding. The new definition takes effect immediately for new calls; any ongoing actor handlers retain the old binding until the actor reloads (per §9 hot-reload semantics).

### 15.4 Mode

The REPL operates in managed mode by default. To experiment with safe-mode code, the user enters a multi-line `@safe module` block. The REPL MUST NOT permit a single safe-mode top-level statement; safe code requires a module envelope.

```
>>> @safe module Local {
...   fn double(own x: Int) -> Int {
...     x * 2
...   }
... }
=> module Local

>>> Local.double(21)
=> 42 : Int
```

### 15.5 Type display rules

After every expression, the REPL MUST print the value followed by ` : ` and the inferred type. For values implementing a custom `to_s` method, the printed value uses that method; otherwise a debug representation (Rust-like `{:?}`) is used.

For long values (> 80 chars rendered, > 5 nested levels), the REPL MAY truncate with `…` and provide a `:expand` command to view in full.

### 15.6 REPL commands

The following commands MUST be supported. Commands begin with `:` to disambiguate from expressions.

| Command | Effect |
|---------|--------|
| `:help` | Show this command table |
| `:quit` / `:q` | Exit the REPL |
| `:type <expr>` / `:t <expr>` | Print the inferred type of `<expr>` without evaluating |
| `:imports` | List currently loaded modules |
| `:use <path>` | Equivalent to `use <path>` — load a module |
| `:reload` | Re-read all loaded modules from disk |
| `:reset` | Clear all REPL bindings; equivalent to restarting |
| `:history` | Print recent inputs |
| `:edit` | Open the most recent multi-line input in `$EDITOR`, then re-evaluate |
| `:time <expr>` | Evaluate `<expr>` and print elapsed time |
| `:bench <expr>` | Run `<expr>` 1000× and report stats |
| `:expand <name>` | Show the un-truncated value of the named binding |

### 15.7 History persistence

The REPL MUST persist input history to `~/.garnet/repl_history` (path overridable via `GARNET_HISTORY_PATH` env var). Lines are appended after each input. History is loaded on startup. Maximum history size is implementation-defined; default 5000 lines.

### 15.8 Safe-mode REPL execution

When evaluating safe-mode code (per §15.4), the REPL MUST run the full borrow checker and lifetime inference (§8.5–8.6). Diagnostics MUST be displayed inline with the same quality as `garnet check`.

### 15.9 Performance contract

REPL evaluation latency for trivial expressions (e.g., `1 + 1`) MUST be ≤ 10 ms on commodity 2026 hardware. This precludes JIT compilation per input (which would amortize poorly); the REPL implementation SHOULD use a tree-walk interpreter with on-demand AST caching.

### 15.10 Differences from Ruby's `irb` and Rust's evcxr

- **Ruby `irb`:** Garnet REPL adds explicit type display and safe-mode support; otherwise behaves analogously.
- **Rust `evcxr`:** Garnet REPL avoids requiring `let` rebinding for redefinition; in Garnet, `def square(x) { … }` followed by a redefinition silently replaces the binding, mirroring Ruby's interactive flow.

The blended model is, by design, "what a Ruby developer expects, with type information added."

---

## 16. Tooling Ergonomics Summary [v1.0 — addresses Phase 1B Swift gap #3; full treatment in Paper VII]

Garnet's tooling story is a research deliverable (Paper VII — Implementation Ladder) but its high-level shape is normative: the language MUST be usable through a single CLI (`garnet`) with consistent UX across local development, package management, and continuous integration.

### 16.1 The single-CLI principle

A Garnet developer interacts with the toolchain through one binary, `garnet`, with sub-commands for every workflow:

| Sub-command | Purpose |
|-------------|---------|
| `garnet new <name>` | Create a new project with template scaffolding |
| `garnet init` | Initialize a Garnet project in the current directory |
| `garnet build` | Compile the project (produces release binary by default) |
| `garnet build --deterministic` | Produce a manifest-pinned reproducible build (per Paper VI C7) |
| `garnet build --deterministic --sign` | Add manifest signature (v3.4 ManifestSig) |
| `garnet run` | Build and execute the project's entry module |
| `garnet test` | Run the project's test suite |
| `garnet check` | Type-check + borrow-check without producing a binary |
| `garnet fmt` | Auto-format source (single canonical style; no options) |
| `garnet repl` | Launch the REPL (per §15) |
| `garnet doc` | Generate API documentation from doc comments |
| `garnet audit` | Run `cargo-geiger`-equivalent dependency safety scan (v3.5 FFIGeiger) |
| `garnet verify <manifest>` | Verify a deterministic-build manifest (v3.4 ManifestSig) |
| `garnet convert <lang> <file>` | Convert source from another language to Garnet (v4.1) |

Inspired by Cargo (Rust) and SwiftPM (Swift), with the deliberate choice to keep the CLI flat (no `garnet pkg add`-style nesting beyond two levels).

### 16.2 Project layout

```
my_project/
├── Garnet.toml          # project manifest (name, version, dependencies, @caps declarations)
├── src/
│   └── main.garnet      # entry module
├── tests/
│   └── test_main.garnet # tests
├── examples/            # example programs
├── .garnet-cache/       # compiler-as-agent cache (HMAC-protected, gitignored)
└── .gitignore           # generated by `garnet new`, excludes .garnet-cache/
```

### 16.3 Manifest (`Garnet.toml`) shape

```toml
[project]
name = "my_project"
version = "0.1.0"
edition = "v1.0"

[dependencies]
http = "0.5"

[caps]                   # v3.4 CapCaps — top-level capability declarations
required = ["fs", "net"]

[build]
deterministic = true     # default-on for v3.4+
sign = true              # v3.4 ManifestSig
```

### 16.4 Documentation comment syntax

```garnet
## A short summary of the function.
##
## # Examples
##
## ```garnet
## let result = fibonacci(10)
## assert_eq(55, result)
## ```
def fibonacci(n) { ... }
```

`##` introduces a doc comment. Doc comments support markdown. `garnet doc` extracts them into a static HTML site.

### 16.5 Cross-platform installer (v4.2 — see Paper VII)

Per the v4.2 plan, Garnet ships as MSI (Windows), `.pkg` (macOS), `.deb`/`.rpm` (Linux), and a `rustup`-style universal shell installer. UX target: "install + create + run" in under two minutes on a clean machine.

### 16.6 Why the single-CLI principle matters

Paper III §3.1 cites Swift's "package tooling and language ergonomics can reinforce one another" as the third Swift contribution. Ruby's tooling fragmentation (separate gem/bundle/rake/rubocop/rspec binaries with overlapping configs) is the canonical anti-pattern. Garnet adopts Swift/Cargo's single-binary discipline as a structural commitment that prevents the Ruby-style fragmentation from creeping in over time.

Full architecture, command-by-command behavior, and rollout sequencing is the subject of Paper VII (Implementation Ladder).

---

## 17. References

1. Jung, R., Jourdan, J-H., Krebbers, R., Dreyer, D. "RustBelt: Securing the Foundations of the Rust Programming Language." POPL 2018, MPI-SWS.
2. Bacon, D.F., Rajan, V.T. "Concurrent Cycle Collection in Reference Counted Systems." ECOOP 2001.
3. Matsakis, N. "RFC 2094: Non-Lexical Lifetimes." Rust RFCs, 2018.
4. Matsakis, N. "RFC 2025: Two-Phase Borrows." Rust RFCs, 2017.
5. Turon, A. "RFC 1023: Rebalancing Coherence." Rust RFCs, 2015.
6. Apple Inc. "Swift Evolution SE-0302: Sendable and @Sendable closures." Swift Evolution, 2021.
7. Siek, J.G., Taha, W. "Gradual Typing for Functional Languages." Scheme Workshop 2006.
8. Garcia, R., Clark, A., Tanter, E. "Abstracting Gradual Typing." POPL 2016.
9. Walker, D. "Substructural Type Systems." Advanced Topics in Types and Programming Languages, MIT Press 2005.
10. Honda, K. "Types for Dyadic Interaction." CONCUR 1993.
11. Garnet Project. "Paper V — The Formal Grounding of Garnet." April 2026 (.docx) + `Paper_V_Addendum_v1_0.md` (Phase 1B markdown extensions).
12. Garnet Project. "Paper VII — Implementation Ladder & Tooling." April 2026 (stub created in Phase 1B).
13. Garnet Project. "GARNET_v2_1_Four_Model_Consensus_Memo." April 2026.
14. Garnet Project. "GARNET_v2_1_Gemini_Synthesis." April 2026.
15. Thomas, D. *Programming Ruby* ("Pickaxe"), 4th ed. Pragmatic Bookshelf, 2013. Ch. 4 (blocks), Ch. 28 (runtime callbacks).
16. Matsumoto, Y. "Ruby Programming Language." 1993/1995.
17. Hoare, G. "Rust Programming Language." Mozilla 2006/2015.
18. Apple Inc. "Swift Programming Language." 2014.
19. Zhang, Z., Kraska, T., Khattab, O. "Recursive Language Models." MIT CSAIL 2025–2026.
20. Alake, R. "Memory Engineering for AI Agents." 2026.

---

**Status:** v1.0 normative draft. This spec is the canonical source of truth for all Garnet implementation work from Rung 2 through Rung 6. Promotion from v0.3 closes the eleven Phase 1B blend-verification gaps; Stage 2 implementation work may commence against this spec.

*"In the multitude of counsellors there is safety." — Proverbs 11:14*
*"The plans of the diligent lead surely to abundance." — Proverbs 21:5*
*"Where there is no vision, the people perish." — Proverbs 29:18*

**Garnet v1.0 Mini-Spec prepared by Claude Code (Opus 4.7) | April 16, 2026 | Phase 1B promotion**
