# Garnet Standard Library — Layer Policy & `@stability` Semantics

| Field | Value |
|---|---|
| **Document** | Stdlib Layer Policy (normative) |
| **Status** | v0.7 draft — codifies the five-layer model + `@stability` semantics |
| **Slice** | S17 (win-opus) |
| **Companion spec** | `GARNET_v1_0_Mini_Spec.md` §11 (stdlib), §12 (capabilities) |
| **Enforcement** | `garnet-stdlib/src/registry.rs` (primitive metadata) + `garnet-check-v0.3` (`@stability` warnings, `@caps` coverage) |
| **Audience** | stdlib authors, `@garnet-lang/*` package authors (S18), reviewers |

This document is the spec that Layer-2 package authors (S18, `@garnet-lang/*`)
code against. It defines *where* a piece of functionality lives (which layer),
*how stable* its API contract is (`@stability`), and *how* functionality
graduates between layers. It is deliberately small and defensible: the goal is
a rule a sharp reviewer can apply mechanically, not a taxonomy that needs a
committee.

> **Calibrated-honesty note (v0.7).** Primitive stability is enforced today as
> **registry metadata** read by the compiler (`garnet-check-v0.3` warns at call
> sites into non-`stable` primitives). Source-level `@stability(...)`/`@uses(...)`/
> `@migration(...)` annotations on **user-defined** functions are **pending a
> parser handoff** (the annotation parser currently rejects unknown annotation
> names). Until that lands, "the compiler enforces `@stability`" is true for
> **primitives**, and **advisory-pending** for user functions — labeled here, not
> buried. See `F_Project_Management/AGENT_COORDINATION_LEDGER.md` → Handoff
> Requests (win-opus → mac-opus).

---

## §1 The five-layer model

Garnet's library surface is partitioned into five layers. The layer determines
the distribution channel, the capability ceiling, and the stability expectation.

| Layer | Namespace | Distribution | Capability posture | Who can publish |
|---|---|---|---|---|
| **0** | `core::` | Always available; compiled into every program; no import needed | **No `@caps` ever** — pure, total computation over in-memory values | Garnet maintainers only |
| **1** | `std::` | Bundled with the `garnet` binary | Capability-gated *or* pure; conservative, slow-moving API | Garnet maintainers only |
| **2** | `@garnet-lang/*` | Official packages via the registry; versioned independently of the compiler | Capability-gated; declares its own `@caps` | Garnet org, against this policy |
| **3** | `community/*` | Registry-endorsed; the curated first 10–20 | Capability-gated; reviewed | Vetted community authors |
| **4** | `*` | Anyone publishes to the registry | Capability-gated; unreviewed | Anyone |

**Layer 0 (`core::`)** is the computational floor: iterators, `Option`/`Result`
combinators, comparison, and math. These functions cannot touch the OS — they
are total functions over values already in memory — so they never carry a
capability and never need one. They are always in scope.

**Layer 1 (`std::`)** is the bundled library. It is the *conservative* layer:
APIs change slowly and only with a stability tier to back the change. Layer 1
contains two kinds of thing:
- **capability-gated** I/O (`std::env`, `std::process`, time, fs, net) — these
  carry `@caps(...)`; and
- **pure-but-library** functionality that implements an external specification
  (`std::json` → RFC 8259, `std::base64` → RFC 4648, `std::regex`, crypto) —
  zero caps, but versioned as a library because the *spec* it tracks can evolve.

**Layers 2–4** are the package ecosystem. Nothing in Layers 2–4 is bundled with
the v0.7 binary; they resolve through the registry (S13) as `@garnet-lang/*`,
`community/*`, or unqualified third-party names. They exist so that fast-moving
or high-authority functionality (an LLM client, an HTTP client) is *not* welded
into the compiler's release cadence.

### v0.7 concrete layer assignment

The registry (`garnet-stdlib/src/registry.rs`) tags every primitive with its
`Layer`. The v0.7 assignment:

| Module | Layer | Caps | Rationale |
|---|---|---|---|
| `str::*` | 0 `core` | none | language-intrinsic string compute; no external spec |
| `array::*` | 0 `core` | none | language-intrinsic collection compute |
| `core::iter::*` | 0 `core` | none | iterator combinators (higher-order, value-level) |
| `core::result::*` | 0 `core` | none | `Result` combinators |
| `core::option::*` | 0 `core` | none | `Option` combinators |
| `core::cmp::*` | 0 `core` | none | ordering / min / max / clamp |
| `core::math::*` | 0 `core` | none | total numeric functions |
| `time::*` | 1 `std` | `time` | wall/monotonic clock — OS authority |
| `fs::*` | 1 `std` | `fs` | filesystem — OS authority |
| `net::*` | 1 `std` | `net` | sockets — OS authority |
| `crypto::*` | 1 `std` | none | pure, but tracks external algorithm specs (SHA-256, BLAKE3) |
| `std::json::*` | 1 `std` | none | pure; tracks RFC 8259 |
| `std::base64::*` | 1 `std` | none | pure; tracks RFC 4648 |
| `std::regex::*` | 1 `std` | none | pure; library surface |
| `std::env::*` | 1 `std` | `env` | reads/writes process environment — OS authority |
| `std::process::*` | 1 `std` | `proc` | spawns processes — OS authority |
| `std::uuid::*` | 1 `std` | `time` (v4/v7), none (v5) | v4/v7 read the clock; v5 is a pure name hash |
| `std::log::*` | 1 `std` | none (format), `fs` (file sinks, deferred) | formatting is pure; file sinks need `fs` |

Note the registry preserves the existing bare module names (`str`, `array`,
`time`, `fs`, `net`, `crypto`) for backward compatibility — renaming them to
`core::`/`std::` prefixes would break existing programs and the interpreter's
prelude bindings. The **layer is explicit metadata**, not inferred from the
name, precisely so the surface name and the policy layer can evolve
independently.

---

## §2 Promotion criteria (`@garnet-lang/*` → `std::`)

A Layer-2 package graduates into Layer 1 (`std::`, bundled) only when **all** of:

1. **Two minor releases at `@stability(stable)`** with no breaking change between
   them. (Stability has to be demonstrated over time, not asserted at v0.1.0.)
2. **≥ 80% test coverage** as measured by the standard reporter.
3. **A documented use case** in the official `examples/`.
4. **A maintainer vote of confidence** — a single human gate during the v0.x
   series. (This becomes a quorum gate post-1.0.)

Promotion is deliberately conservative: pulling something into `std::` is a
promise to carry it for the rest of the major version (see §4). The default is
to *stay* a package. Most functionality should never need to be in `std::`.

---

## §3 Deprecation policy

Functionality leaves the stdlib on a fixed, announced schedule:

1. **Mark** the item `@stability(deprecated)` and attach a `@migration("…")`
   hint naming the replacement.
2. **Warn on use** for **two minor releases**. The warning is non-fatal and
   carries the migration hint.
3. **Remove** only at the **next major** version. No deprecated item is removed
   within a major series.

This guarantees any program that compiles today keeps compiling — with warnings
— until a major bump it can see coming.

---

## §4 Stability semantics

`@stability(<tier>)` declares the API-contract promise for a function or
primitive. Four tiers:

| Tier | Meaning | Breaking changes allowed? |
|---|---|---|
| `stable` | API contract held for the entire major version | **No**, until the next major |
| `experimental` | API may change between **minor** versions | **Yes**, between minors |
| `frozen` | No further changes; will eventually be deprecated | **No**, never (it just won't grow) |
| `deprecated` | Scheduled for removal per §3 | **No** new uses; existing uses warn |

### Enforcement (v0.7)

The compiler reads each **primitive's** tier from the registry and emits
diagnostics at call sites (`garnet-check-v0.3`):

| Callee tier | Diagnostic at the call site | Severity | Fails build? |
|---|---|---|---|
| `stable` | (none) | — | No |
| `experimental` | "calls experimental primitive `X`; API may change between minor releases" | **warning** | No |
| `deprecated` | "calls deprecated primitive `X`" + migration hint if present | **warning** | No |
| `frozen` | "calls frozen primitive `X`; it is supported but will not grow" | **info** | No |

All four are **non-fatal** (exit code 0). `@stability` is a warning-level
contract in v0.7 by design, for backward compatibility; **error-level
enforcement is v0.8 work.** A missing tier defaults to "unannotated" — the
v0.7 stdlib annotates ≥ 95% of primitives explicitly, and the
`garnet_stdlib_layer_gate.py` lane tracks that percentage.

### Opt-in (pending parser handoff)

The full contract lets a caller *opt in* to an `experimental` dependency with
`@uses(experimental)` on the calling function, which suppresses the warning at
that site (the author has acknowledged the volatility). `@uses(...)` and
user-function `@stability(...)`/`@migration(...)` require new annotation syntax
the parser does not yet accept; they are **pending the win-opus → mac-opus
handoff** and land in a follow-up. Until then, the experimental/deprecated
warnings are advisory and un-suppressable, which is the safe default.

---

## §5 First-order principle (the defensible reviewer answer)

> **Capability surface + spec volatility = layer assignment.**

Two independent axes decide a layer:

- **Capability surface** — how much OS authority the functionality needs (its
  blast radius if misused).
- **Spec volatility** — how fast the contract it implements changes.

Worked examples:

- **JSON** — zero caps, tracks RFC 8259 (effectively frozen since 2017). Low
  blast radius, low volatility → bundled, `std::json` (**Layer 1**).
- **An LLM client** — needs `@caps(net)`, and provider APIs change quarterly.
  High blast radius, high volatility → `@garnet-lang/llm` (**Layer 2**),
  versioned independently of the compiler so a provider change is a package
  bump, not a compiler release.
- **`map`/`fold`** — zero caps, the contract is mathematics. No blast radius, no
  volatility → `core::iter` (**Layer 0**), always in scope.

The principle is what keeps the boundary honest: a thing does **not** belong in
`std::` just because it is useful or pure (JSON is both, but it earns Layer 1
because it is a *library* tracking an external spec); and a thing does **not**
get bundled just because it is popular (an LLM client is popular, but its
volatility and `net` authority keep it in Layer 2). Different blast radii and
different volatilities get different layers, and therefore different upgrade and
trust expectations.

---

## §6 References

- `GARNET_v1_0_Mini_Spec.md` §11 (standard library), §12 (capability model).
- `garnet-stdlib/src/registry.rs` — the primitive table (module, name, arity,
  required caps, **stability tier**, **layer**).
- `garnet-check-v0.3/src/stability.rs` — registry-driven `@stability` call-site
  warnings.
- `scripts/garnet_stdlib_layer_gate.py` — primitives-by-layer + `@stability`
  coverage gate (the `stdlib_layer_policy` readiness lane).
- `PRD_C_S17_STDLIB_LAYERS.md` — the S17 product requirements this document
  satisfies.
