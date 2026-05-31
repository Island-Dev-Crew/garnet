# Garnet formal-verification feasibility study (S75)

**Question.** Can Garnet offer a *provable* — not merely annotated — story for
(a) termination / resource bounds and (b) `@caps` authority soundness, over a
safe subset? The trajectory research lists this as a multi-year bet: "formal
verification of `@caps` soundness (the eBPF-verifier path) for a safe subset,
giving Garnet a provable-authority story that annotations alone can't provide."

This is a **feasibility study**. It builds no verifier and proves no theorem; it
assesses what is feasible, on what foundations, at what cost.

## Foundations already in tree

- **Static explosive-operation identification** (`garnet-check-v0.3/src/explosive.rs`,
  S40). It already takes the verifier stance verbatim: *"Static termination is
  undecidable, so every `loop` is flagged regardless of an internal `break`;
  declare `@bounded` to govern it."* The AST visitor is compiler-exhaustive.
- **The safe subset** (S74): the typed, ownership-disciplined `fn` mode + the
  fn↔def boundary audit.
- **`@bounded` + default-ceiling policy** (S39/S40): a *contract*; runtime
  enforcement lowers to a Wasmtime-fuel target (deferred — wasmtime absent).

## Feasibility by target

### (a) Bounded-loop termination — the eBPF-verifier path
**Feasible for a restricted safe subset.** eBPF's verifier proves termination by
showing an induction variable is monotonic and bounded, checking loops against a
complexity limit (~1M instructions). The same is feasible for a Garnet safe
subset: admit only loops whose bound is statically derivable; reject the rest
(the halting-problem tax — safe-but-uncheckable programs are refused). `explosive.rs`
is the precursor (it already *identifies* the unbounded sites); the increment is a
checker that *proves* the bound where one exists. **Verdict: feasible; first
provable increment.**

### (b) `@caps` authority soundness
**Not feasible for the annotation model as-is; feasible atop the S74 linear mode.**
Annotations cannot be proven sound while ambient authority, FFI, and `proc`-spawned
subprocesses can launder authority (Deno's documented escape hatch). Capability
soundness becomes provable only if authority is carried as **linear capability
values** (Austral: "capability security is a consequence of linear types") — the
optional linear/effect-typed mode S74 *proposes but does not implement*. **Verdict:
feasible only on top of the (unbuilt) linear-capability mode; not for `@caps`
annotations alone.**

### (c) Mechanized metatheory (Coq / Lean / F*)
**Feasible as a research artifact for a small core, high effort, out of near-term
scope.** A mechanized soundness proof of the safe subset's type system is the
gold standard but is a multi-month research effort with no tooling in tree today.

## Decidability boundary (honest)

The halting problem forbids verifying *arbitrary* Garnet. Any provable story must
restrict to a checkable safe subset and **reject safe-but-uncheckable programs** —
eBPF's exact tradeoff. Whole-language soundness is impossible while annotations,
ambient authority, and FFI remain; that is not a defect to hide but the reason the
provable story is scoped to a subset.

## Verdict & recommendation

1. **Pursue a verified bounded-loop checker for the safe subset** (eBPF-style) as
   the first provable increment, building on `explosive.rs`.
2. **`@caps` soundness is feasible only via the S74 linear-capability mode** —
   sequence it after that mode lands.
3. **Full formal verification of the whole language is not feasible and is not the
   goal.** The honest target is a *provable safe subset*, not a proved language.

## Honest scope (do not soften)

A **feasibility study only.** No verifier, no termination proof, no SMT or
proof-assistant integration, and no `@caps`-soundness theorem ship in this slice.
Everything in (a)–(c) is assessment, not implemented behavior. The static
identification it builds on (`explosive.rs`) is real; the verification is not.
