# Garnet stdlib promotion wave (S76)

Garnet's stdlib primitives each carry an `@stability` tier (Stable / Experimental
/ Frozen / Deprecated; see the S17 layer gate). The S76 promotion wave elevates
the foundational **`core::*` layer** from Experimental to **Stable** and keeps the
**`std::*`** utilities Experimental.

## What was promoted

The whole `core::*` layer — **30 primitives** — Experimental → Stable:

- `core::iter::*` — map, filter, fold, zip, take, drop, collect, chain, enumerate
- `core::result::*` — ok, err, map, and_then, or_else, unwrap_or
- `core::option::*` — some, none, map, and_then, unwrap_or
- `core::cmp::*` — min, max, clamp, ordering
- `core::math::*` — abs, sqrt, pow, floor, ceil, round

Stdlib stability distribution moves from 27 Stable / 59 Experimental to
**57 Stable / 29 Experimental**.

## Promotion criteria (a primitive is promoted iff ALL hold)

1. **Core layer** — foundational, `RequiredCaps::none` (no host authority).
2. **Frozen semantics** — universally established (functional iterators,
   Result/Option combinators, comparisons, basic math); the API will not change
   between minor releases.
3. **Test-covered and corpus-used.**

## What was deliberately kept Experimental

The `std::*` families are **not** promoted:

- **Host-authority surfaces** — `std::env::*`, `std::process::*`.
- **Evolving-API utilities** — `std::json::*`, `std::regex::*`, `std::uuid::*`,
  `std::base64::*`, `std::log::*` — whose APIs may legitimately change (regex flag
  syntax, json options, base64 padding variants, uuid versions).

Promoting these to silence example warnings would **game the `@stability`
contract**. The warnings they emit are *correct*.

## Effect on the corpus (honest)

- `examples/novel_07_functional_core_pipeline.garnet` (core-only) now checks with
  **0 diagnostics**.
- `novel_04` / `novel_05` / `novel_06` **still emit stability warnings** — they
  use `std::*` experimental utilities (e.g. `std::base64::encode`, `std::json`,
  `std::regex`, `std::uuid`, `std::log`), and those warnings remain accurate.

## Honest scope (do not soften)

This is a **stability judgement**, not warning-suppression. Only the genuinely
foundational `core::*` layer was promoted; every `std::*` utility stays
Experimental until its API is actually settled. `scripts/garnet_stdlib_promotion_status.py
--gate` enforces that the wave stayed scoped (all `core::*` Stable **and** `std::*`
still Experimental) so a future blanket flip cannot pass silently.
