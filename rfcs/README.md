# Garnet RFCs

Substantive changes to Garnet — language surface, semantics, the capability
model, stability tiers, editions — go through an **RFC** before implementation.
The RFC makes the *reasoning* a durable, reviewable artifact (see `GOVERNANCE.md`).

## When an RFC is required

- Any change to the language grammar or semantics.
- Any change to the capability model (`@caps`, the manifest, diff-caps, seal).
- Any stability-tier promotion/demotion policy change.
- Any new edition or backwards-incompatible change.

Bug fixes, docs, tests, and small additive features do **not** need an RFC — a
PR with green CI and dogfood evidence is enough.

## Lifecycle

```
Draft  →  Discussion  →  Accepted | Rejected | Withdrawn
```

- **Draft** — authored from `0000-template.md`, numbered with the next free
  integer (`NNNN-short-title.md`).
- **Discussion** — opened as a PR; reviewed against the calibrated-honesty
  doctrine (no faked runtime enforcement / cross-platform proof / release
  readiness / 1.0 claims).
- **Accepted / Rejected / Withdrawn** — the maintainer records the decision and
  its rationale in the RFC's `Status`.

An accepted RFC authorizes implementation slices; it is not itself an
implementation, and acceptance is not a readiness or production-claim.

## Index

- `0000-template.md` — the RFC template.
- `0001-capability-manifest-standard.md` — *Draft.* Standardize Garnet's
  capability-manifest format as a cross-language standard and offer it to a
  neutral body (OWASP / Linux Foundation). **Intent + draft, not accepted.**
- `0002-integer-overflow-policy.md` — *Accepted.* Integer arithmetic is
  checked-by-default (runtime diagnostic, not silent wrap or abort), with
  explicit wrapping operations where wanted. Design ruling (W-REBUILD J5);
  implementation is a separate slice.
