# RFC-0001: Cross-language capability-manifest standard

- **Status:** Draft
- **Author(s):** Island Development Crew (Jon Isaac)
- **Created:** 2026-05-31
- **Tracking PR:** (this RFC's PR)

## Summary

Standardize Garnet's **capability-manifest format** — the machine-readable
authority surface a program declares (`@caps` → manifest → diff-caps → seal →
transparency log) — as a **cross-language standard**, and offer it to a neutral
standards body (**OWASP** or the **Linux Foundation**). This RFC is **intent and a
draft**: no external body has adopted it.

## Motivation

SBOM standards (SPDX, CycloneDX) inventory dependencies but **do not model
capability/permission grants**. Garnet already produces a first-class, signed,
diffable capability manifest (S35–S38) and an append-only transparency log (S68);
the format is language-agnostic. The trajectory research identifies "a
cross-language capability-manifest standard (the thing SBOMs don't model)" as
unclaimed standards territory worth seeding and donating.

## Design

- **Canonical schema seed.**
  `C_Language_Specification/GARNET_CAPABILITY_MANIFEST_STANDARD.md` defines the
  current draft profile (`capability-manifest/v1`): a deterministic producer
  record, aggregate authority set, and sorted entry list
  `{kind, name, capabilities[], source_span}`. `source_span` is `null` in the
  Garnet seed until CST/span precision is stable. The transparency-log companion
  remains documented in `C_Language_Specification/GARNET_CAPABILITY_TRANSPARENCY.md`.
- **Reference implementation.** Garnet's `garnet caps --standard-profile`,
  `garnet diff-caps`, `garnet caps-log`, and `garnet mcp-caps` are the reference
  implementation seed. `garnet caps <path>` remains the S36 Garnet-native
  envelope for backward compatibility.
- **Donation path.** Package the schema + reference implementation + test vectors
  as an OWASP project proposal or an LF sandbox project. Governance of the donated
  standard would move to the receiving body.

## Compatibility & editions

The standard is additive and out-of-band (a manifest artifact), so it introduces
no language edition change. Capability semantics remain edition-invariant.

## Honest scope (do not soften)

- This RFC **proposes** standardization and **records the intent** to donate. It
  is **not** an accepted standard and **no** external body (OWASP/LF) has reviewed
  or adopted anything. "Reference implementation" means Garnet's existing tools,
  not a multi-language ecosystem.
- Acceptance of this RFC authorizes packaging/outreach work; it does not create a
  standard, a foundation relationship, or any production/1.0 claim.

## Alternatives considered

- **Extend CycloneDX/SPDX** instead of a new schema — viable long-term; the
  capability surface maps to CycloneDX "Services" partially, but neither SBOM
  format models per-function capability grants natively. A small dedicated schema
  that *also* embeds into in-toto/SBOM is the lower-friction seed.

## Unresolved questions

- OWASP vs LF as the receiving body.
- Schema embedding into in-toto predicates vs standalone.
- Test-vector corpus scope for cross-language conformance.
