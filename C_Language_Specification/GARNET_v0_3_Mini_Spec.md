# Garnet v0.3 Mini-Spec
**Supersedes:** v0.2 Mini-Spec Stub (April 12, 2026)
**Status:** Normative draft — canonical specification for Rungs 2–4 of the engineering ladder
**Date:** April 16, 2026
**Author:** Claude Code (Opus 4.6) at the direction of Jon — Island Development Crew
**Anchor:** *"Where there is no vision, the people perish." — Proverbs 29:18*

> This document is a specification, not a stub. All v0.2 content is preserved and extended. Normative terms follow RFC 2119 (MUST, SHOULD, MAY). Where v0.3 refines or replaces v0.2 wording, a `[v0.3]` marker appears.

---

## 1. Scope

Garnet v0.3 specifies the complete surface syntax and normative semantics for:

1. Module declarations and imports
2. Memory unit declarations *(carried from v0.2 §2)*
3. Function definitions in both managed and safe modes
4. Expression and statement grammar including control flow
5. Error handling model (dual-mode)
6. `@safe` mode boundaries and crossing rules *(extended from v0.2 §3)*
7. Typed message protocols and actor declarations *(carried from v0.2 §4)*
8. Recursive execution guardrails with concrete annotation syntax *(extended from v0.2 §5)*
9. Type system foundations (progressive disclosure spectrum)
10. Pattern matching

**Out of v0.3 scope:** codegen strategy, optimization passes, standard library contents, package manager protocol, REPL implementation details, interop ABI layout. These belong to the Compiler Architecture Specification and Standard Library Outline, not the language spec.

---

## 2. Lexical Grammar

### 2.1 Character set and encoding

Garnet source files MUST be valid UTF-8. The compiler MUST reject files containing invalid UTF-8 sequences before lexing begins. Line endings MUST be normalized to `\n` (LF); `\r\n` (CRLF) is accepted and silently converted.

### 2.2 Keywords

The following identifiers are reserved and MUST NOT be used as user-defined names:

**Mode and structure:**
`module` `use` `@safe` `@dynamic` `pub` `end`

**Declarations:**
`def` `fn` `let` `var` `const` `type` `trait` `impl` `struct` `enum`

**Memory and actors:**
`memory` `working` `episodic` `semantic` `procedural` `actor` `protocol` `on` `spawn` `send`

**Control flow:**
`if` `elsif` `else` `while` `for` `in` `loop` `break` `continue` `return` `match` `when`

**Error handling:**
`try` `rescue` `ensure` `raise` `Result` `Ok` `Err`

**Ownership (safe mode):**
`own` `borrow` `mut` `ref` `move`

**Recursion guardrails:**
`@max_depth` `@fan_out` `@require_metadata`

**Literals and values:**
`true` `false` `nil` `self` `super`

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

## 4. Memory Units *(carried from v0.2 §2, unchanged)*

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

### 4.4 Generics over memory kinds [v0.3 — explicit deferral, addresses OQ-3]

Mini-Spec v0.3 does NOT support parameterizing a function or type over a memory kind. A declaration such as `def foo<M: MemoryKind>(x: M) { ... }` is rejected by the parser. This decision is deliberate for three reasons:

1. **Kind-polymorphism would require effect-typing** to soundly track which allocator runs at each call site, which adds significant complexity to λ_managed (Paper V §3).
2. **No compelling use case yet.** The four memory kinds have different semantic profiles (short-lived, chronological, factual, behavioral) that rarely compose usefully.
3. **Future work.** A v0.4 proposal may introduce bounded kind-polymorphism with effect tracking; the deferral preserves our options.

Until then, programmers use concrete kinds per declaration. This is consistent with the v0.3 philosophy: prefer explicit over clever.

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
# Managed code calling safe code — bridging is automatic
def process_file(path) {
  # Storage.read_block returns Result<Block, StorageError>
  # In managed mode, Err becomes an exception automatically
  let block = Storage.read_block(open(path), 0)  # may raise SafeModeError
  block.data
}
```

**Design rationale (MIT-defensible):** The error-bridging mechanism is formally justified by the mode-boundary calculus in Paper V §4. The managed-to-safe direction (unwrap + raise) preserves the managed-mode invariant that errors are exceptions. The safe-to-managed direction (wrap in try) preserves the safe-mode invariant that errors are values. Neither direction loses information — the original error is preserved inside the wrapper type. This is analogous to how Swift's Objective-C interop bridges NSError to Swift errors, but generalized to a formal dual-mode boundary. The four-model consensus point 2 (dual-mode is correct) implies that each mode's error model should be native to that mode, not a compromise — and the bridging mechanism makes this possible.

---

## 8. Mode Boundaries *(extended from v0.2 §3)*

### 8.1 Formal grounding *(unchanged from v0.2 §3.1)*

Garnet's `@safe` mode is grounded in **affine type theory** (resources used at most once), formally verified by **RustBelt (Jung et al., POPL 2018, MPI-SWS)** using the **Iris framework** in Coq. See Paper V §2-§6 for the full formal treatment.

**Security Theorem (safe-mode soundness) [v0.3]:**

> For any `@safe` module M that type-checks under the Garnet v0.3 rules, the following properties hold:
>
> 1. **No use-after-free.** Dropped values are not subsequently accessible.
> 2. **No double-free.** Each owned value is dropped exactly once along every execution path.
> 3. **No data races.** Concurrent access to shared memory follows aliasing-XOR-mutation strictly.
> 4. **No memory unsafety** within the well-typed fragment (excluding explicit `extern "C"` boundaries, which are programmer-asserted unsafe by definition).
>
> **Proof sketch.** The λ_safe calculus (Paper V §3) inherits progress and preservation from RustBelt (Jung et al., POPL 2018). Garnet's module-granularity (rather than crate-granularity) refinement does not invalidate the core argument: the Iris separation-logic model of λ_safe is a sub-model of RustBelt's crate model, so any property proven for RustBelt lifts. Formal mechanization in Coq is planned for a future Paper V revision (18–30 person-months per the v2.4 handoff estimate).

This theorem statement elevates the earlier prose to a load-bearing normative claim that MIT or PLDI reviewers can scrutinize precisely.

### 8.2 Managed mode *(unchanged from v0.2 §3.2)*

Managed mode MUST provide automatic reference counting with cycle detection. Values MAY be mutated without ownership discipline. Managed mode is the default.

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
2. Aliasing-XOR-mutation for all references
3. Lexical or non-lexical lifetime inference
4. No implicit allocation on hot paths

### 8.4 Boundary rules *(unchanged from v0.2 §3.4, plus error bridging from §7.4)*

1. Managed callers invoking `@safe` functions MUST satisfy ownership preconditions at call sites
2. `@safe` returns to managed mode MUST return either an owned value (adopted into ARC) or a borrowed reference with a statically proven lifetime
3. Managed memory units MAY be read from safe mode under shared borrow, but MUST NOT be mutated except through a formal bridging API *(reserved for v0.4)*
4. [v0.3] Error bridging follows §7.4 rules automatically at every mode-crossing call site

---

## 9. Typed Message Protocols *(carried from v0.2 §4, block grammar now defined)*

### 9.1 Grammar [v0.3 — block now formally defined]

```
actor-decl    := ("pub")? "actor" ident "{" actor-item* "}"
actor-item    := protocol-decl | handler-decl | memory-decl | let-decl
protocol-decl := "protocol" ident "(" typed-params ")" ("->" type)?
handler-decl  := "on" ident "(" param-list ")" block
```

The `block` production is now the same as §5.1's block: `"{" stmt* expr? "}"`. This retires the provisional block grammar from the v0.2 parser's `expr.rs`.

### 9.2 Semantics *(unchanged from v0.2 §4.2)*

Actors MUST NOT share mutable state. All inter-actor communication MUST go through declared protocols. Undeclared protocol sends MUST be compile-time errors. Every declared protocol MUST have a handler. Type erasure and gradual typing are prohibited across actor boundaries even in managed mode.

**Protocol versioning (OQ-5, explicit deferral to v0.4).** The v0.3 Mini-Spec does not specify how protocol definitions evolve across a running distributed system (e.g., hot-reloading a protocol that added a new field, or retiring a deprecated message type). A v0.4 proposal will introduce a `@protocol_version` annotation with additive-evolution semantics analogous to Protobuf field tagging. Until then, protocol changes require a coordinated restart of all actors sharing a protocol. This limitation is acceptable for v0.3 because (a) within a single process, all actors are compiled together; (b) across processes, users must version their own APIs per standard distributed-systems discipline.

The rationale for deferral: protocol versioning intersects distribution, hot-reload (Paper VI Contribution 6), and session typing (see Memory Manager Architecture §4.5). Designing all three together in v0.4 is preferable to partial v0.3 solutions.

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

---

## 10. Recursive Execution Guardrails *(extended from v0.2 §5)*

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

### 10.2 Asynchronous fan-out caps [v0.3 — concrete annotation syntax]

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

### 10.3 Metadata validation [v0.3 — concrete annotation syntax]

```
metadata-annotation := "@require_metadata"
```

`@require_metadata` on a parameter position requires that the argument carries Memory Manager metadata. The Memory Manager MAY refuse the call if flat retrieval would suffice.

```garnet
def recursive_retrieve(@require_metadata context: Memory::Semantic) {
  # Memory Manager validates that recursive retrieval is justified
  context.deep_search(query)
}
```

### 10.4 Scope *(unchanged from v0.2 §5.4)*

§10 constrains program structure, not program semantics. A compliant Garnet implementation MAY choose any enforcement strategy (static analysis, runtime checks, or both) that rejects non-compliant programs with clear diagnostics.

---

## 11. Type System Foundations [v0.3]

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
> **Proof sketch.** *Relaxation* follows because the annotation sets at each level form a strict inclusion chain: `Annotations(Level 0) ⊂ Annotations(Level 1) ⊂ Annotations(Level 2) ⊂ Annotations(Level 3)`. Removing annotations cannot introduce new type errors because the type checker at a lower level treats unannotated positions as `Dyn` (managed) or generates inference obligations (safe). *Strengthening* follows from the bidirectional type-checker's soundness: adding an annotation that agrees with the inferred type is a no-op; adding one that disagrees produces a specific, localized error. The boundary between λ_managed (Levels 0–2) and λ_safe (Level 3) is the single non-trivial transition, and its soundness is given by Paper V's bridging judgment theorem.

This theorem elevates Paper VI Contribution 2 from a claim to a theorem with a named proof obligation, addressing the MIT reviewer question "how do you know Level N programs remain valid at Level N±1?"

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

**Design rationale:** Structs, enums, and traits follow Rust's algebraic type system — proven to be the strongest foundation for both type safety and pattern matching. The `impl` block separates data (struct) from behavior (methods), which is essential for safe mode's ownership discipline. In managed mode, these types are ARC-managed; in safe mode, they follow ownership rules. The four-model consensus point 7 (typed actors with compiler-enforced protocols) requires a strong type system for protocol message types; structs and enums provide this.

### 11.4 Generics

```garnet
def identity<T>(x: T) -> T { x }

fn swap<T>(own a: T, own b: T) -> (T, T) { (b, a) }

struct Stack<T> {
  items: Array<T>,
}
```

In managed mode, generics are erased at runtime (like Java/TypeScript). In safe mode, generics are monomorphized (like Rust) for zero-cost abstraction. The compiler automatically selects the strategy based on the enclosing module's mode.

### 11.5 Trait Coherence [v0.3 — addresses OQ-10]

Trait coherence governs when two different `impl` blocks may exist for overlapping types. Garnet adopts Rust's proven coherence model as the v0.3 baseline:

**The Orphan Rule (per Rust RFC 1023).** An `impl <T> Trait for Type` is allowed only if at least one of the following holds:

1. `Trait` is defined in the current module, OR
2. `Type` is defined in the current module, OR
3. A local type parameter appears before any type-parameter covariant in `Type` (the "uncovered type parameter" rule).

**Rationale.** The orphan rule prevents the "hashmap problem": if two unrelated crates both `impl Display for Vec<Foo>`, the compiler cannot pick one without breaking some consumer. The rule guarantees at most one canonical implementation per (trait, type) pair per compilation graph. Rust took ten years to converge on this rule; adopting it directly saves Garnet that time.

**Deferred subtleties (targeted for v0.4).** Specialization (multiple impls for overlapping types with a priority order), negative bounds (`where T: !Copy`), and blanket impls with conditional specialization are NOT part of v0.3. A v0.4 RFC will address specialization following Rust's specialization-lite proposal. Until then, the simple orphan rule provides predictable behavior at the cost of some expressiveness — a defensible v0.3 trade-off per the "prefer explicit over clever" philosophy.

**Diagnostic guarantee.** When an orphan-rule violation is detected, the compiler MUST produce a diagnostic that (a) names the offending `impl`, (b) cites which of the three conditions failed, and (c) suggests either moving the `impl` to the crate defining the trait or the crate defining the type. This matches Rust's diagnostic quality for the same error class.

---

## 12. Open Questions *(updated from v0.2 §6)*

- **OQ-1.** How are memory-unit retention policies expressed in source? *(runtime concern — resolved by `GARNET_Memory_Manager_Architecture.md §3.3` per-kind defaults; no source syntax planned)*
- **OQ-2.** What is the bridging API for managed→safe mutation of a managed memory unit? *(targeted for v0.4 — boundary rules in §8.4 define the read path; mutation path awaits v0.4)*
- **OQ-3.** What is the story for generics over memory kinds? *(resolved as explicit deferral — see §4.4 for the v0.3 position and rationale)*
- **OQ-4.** What is the soundness proof obligation for §8.4 boundary rules? *(formal sketch in Paper V §5; security theorem stated in §8.1 above; full Coq mechanization deferred)*
- **OQ-5.** How are actor protocols versioned across a running system? *(resolved as explicit deferral — see §9.2 for rationale; v0.4 will introduce `@protocol_version`)*
- **OQ-6.** What does the language surface expose about KV-cache compression hints? *(resolved: nothing — confirmed by consensus point 8)*
- **OQ-7.** How is the Memory Manager's controlled-decay formula expressed? *(resolved by `GARNET_Memory_Manager_Architecture.md §3.2` — R+R+I formulation with per-kind defaults; empirical calibration deferred to Rung 5)*
- **OQ-8.** Multi-agent access to shared Memory Core consistency. *(resolved by `GARNET_Memory_Manager_Architecture.md §4` — three access modes: exclusive, shared_read, session; session-type surface syntax targeted for v0.4)*
- **OQ-9.** [v0.3] What is the async model? *(resolved by `GARNET_Tier2_Ecosystem_Specifications.md §D` — green threads, no colored functions, structured concurrency; runtime mechanism in Compiler Architecture Spec §11.2)*
- **OQ-10.** [v0.3] What is the trait coherence model? *(resolved as Rust RFC 1023 orphan rule — see §11.5; specialization deferred to v0.4)*
- **OQ-11.** [v0.3] What is the lifetime elision story for safe mode? *(targeted for v0.4; Paper V §4 provides the formal basis via non-lexical lifetimes per Rust RFC 2094)*

---

## 13. What a v0.3 Implementation Owes [v0.3]

Rungs 2–4 of the engineering ladder MUST be implementable against this spec:

- **Rung 2 (parser):** MUST parse all grammar productions in §§2–10. The existing v0.2 parser covers §4 (memory) and §9 (actors); v0.3 extends it with §3 (modules), §5 (functions), §6 (control flow + pattern matching), §7 (error handling), §10 (guardrail annotations), and §11 (type declarations).

- **Rung 3 (managed interpreter + REPL):** MUST evaluate managed-mode programs including: function definitions and calls, variable bindings, control flow, string interpolation, pattern matching, try/rescue error handling, memory unit declarations, actor spawn and message passing. MUST provide a REPL that evaluates expressions interactively. Performance target: comparable to Ruby 3.4 interpreter (not YJIT).

- **Rung 4 (safe lowering):** MUST enforce affine ownership rules per §8.3, reject programs violating aliasing-XOR-mutation, insert ARC retain/release at mode boundaries per §8.4, and perform error bridging per §7.4. Code generation target: LLVM IR or Cranelift, with performance within 2x of equivalent Rust for compute-bound safe-mode code.

---

## 14. Four-Model Consensus Alignment Verification

Every normative rule in this spec has been verified against the eight consensus points:

| Consensus Point | Spec Sections Implementing It |
|---|---|
| 1. Rust/Ruby structurally complementary | §5 (def vs fn), §7 (exceptions vs Result), §11.1 (spectrum) |
| 2. Dual-mode is correct shape | §5.1/5.2, §7.2/7.3, §8, §11.1 |
| 3. Swift as managed-mode precedent | §8.2 (ARC), §9 (actors), §5.3 (closures) |
| 4. Agent-native language platform | §4 (memory units), §9 (actors), §10 (guardrails) |
| 5. One Memory Core, Many Harnesses | §4.3 (out of scope = harness layer), §9.3 (actor memory) |
| 6. Memory primitives first-class | §4.1 (declaration grammar), §9.3 (actor-scoped memory) |
| 7. Typed actors with compiler-enforced protocols | §9.1/9.2 (grammar + semantics) |
| 8. TurboQuant = runtime, not language-core | §4.3 (out of scope), OQ-6 (position: nothing) |

---

## References

1. Jung, R., Jourdan, J-H., Krebbers, R., Dreyer, D. "RustBelt: Securing the Foundations of the Rust Programming Language." POPL 2018, MPI-SWS.
2. Siek, J.G., Taha, W. "Gradual Typing for Functional Languages." Scheme Workshop 2006.
3. Garcia, R., Clark, A., Tanter, E. "Abstracting Gradual Typing." POPL 2016.
4. Walker, D. "Substructural Type Systems." Advanced Topics in Types and Programming Languages, MIT Press 2005.
5. Honda, K. "Types for Dyadic Interaction." CONCUR 1993.
6. Garnet Project. "Paper V — The Formal Grounding of Garnet." April 2026.
7. Garnet Project. "GARNET_v2_1_Four_Model_Consensus_Memo." April 2026.
8. Garnet Project. "GARNET_v2_1_Gemini_Synthesis." April 2026.
9. Matsumoto, Y. "Ruby Programming Language." 1993/1995.
10. Hoare, G. "Rust Programming Language." Mozilla 2006/2015.
11. Apple Inc. "Swift Programming Language." 2014.
12. Zhang, Z., Kraska, T., Khattab, O. "Recursive Language Models." MIT CSAIL 2025–2026.
13. Alake, R. "Memory Engineering for AI Agents." 2026.

---

**Status:** v0.3 normative draft. This spec is the canonical source of truth for all Garnet implementation work from Rung 2 forward.

*"In the multitude of counsellors there is safety." — Proverbs 11:14*
*"The plans of the diligent lead surely to abundance." — Proverbs 21:5*
*"Where there is no vision, the people perish." — Proverbs 29:18*

**Garnet v0.3 Mini-Spec prepared by Claude Code (Opus 4.6) | April 16, 2026**
