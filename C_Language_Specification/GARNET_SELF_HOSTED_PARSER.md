# Garnet self-hosted parser — seed (S72)

Self-hosting (Garnet's toolchain written in Garnet) is a maturity milestone on
the v0.8 validation runway. S72 ships the **seed**: a Garnet program that parses
a subset of Garnet's own surface syntax, demonstrating that the language can
begin to describe itself.

## What the seed does

`examples/self_hosted_parser_seed.garnet` parses `def name(params) { ... }`
declarations from an embedded Garnet source string and reports, per declaration:

- the **function name** (extracted between `def ` and `(`),
- the **arity** (parameter count), and
- whether it is **managed** (preceded by a `@caps(...)` annotation).

It uses only the **Stable, no-caps `str::`** primitives (`split`, `trim`,
`replace`, `contains`, `starts_with`) plus array indexing and `for`/`if` — so it
checks with **0 diagnostics** (no experimental-primitive stability warnings).

Running it:

```
$ garnet run examples/self_hosted_parser_seed.garnet
def main arity 0 caps yes
def add arity 2 caps no
def greet arity 1 caps no
parsed defs: 3 managed: 1
```

## CI proof

- The **canonical-examples** job (which builds the compiler) parses, checks, and
  runs the seed, asserting `parsed defs: 3 managed: 1`, then runs
  `garnet_self_hosted_parser_seed_status.py --gate`.
- The **agent-contracts** job (python-only, no compiler) runs the reporter's
  static well-formedness gate (`--gate --no-run`) and 5 unit tests.

## Honest scope (do not soften)

This is a **SEED toward self-hosting, NOT the production parser**
(`garnet-parser-v0.3`). It recognizes def headers + `@caps` lines from a source
string; it does **not** build a full AST, nor handle nested braces, expressions,
types, comments, generics, or the rest of the grammar. It does not replace or
bootstrap the Rust parser. Full self-hosting (porting `garnet-parser-v0.3` to
Garnet) remains roadmap work.
