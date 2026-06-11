# Garnet governance (S78)

This is the canonical governance model for Garnet. It formalizes **how changes
land** through an **RFC + edition process** rather than ad-hoc maintainer fiat —
while being honest that Garnet is, today, a single-maintainer research-grade
project.

## Who decides (honest)

Garnet is maintained by **Island Development Crew (Jon Isaac, maintainer)**. Final
decisions on language design, releases, and merges are the maintainer's,
exercised through the `Island-Dev-Crew` GitHub organization. **There is no
separate steering committee; claiming one would be fiction.** This document
describes the *process* the maintainer commits to, not an institutional body.

## How a change lands

| Change size | Path |
|---|---|
| Bug fix, docs, test, small feature | Branch in a fork → PR → green CI → maintainer review → squash-merge. |
| Language-surface / semantics / capability-model / stability-tier change | **RFC required** (see `rfcs/`) before implementation PRs. |
| Backwards-incompatible change | RFC **+** the **edition** mechanism (S32) — old editions keep parsing; capability semantics are edition-invariant. |

Every merge is CI-gated and dogfood-evidenced; the audit/diff-caps/seal trail is
the reviewer's machine-checkable record.

## The RFC process (over BDFL)

Substantive changes are proposed as **RFCs** (`rfcs/0000-template.md`), numbered
sequentially, moving through `Draft → Discussion → Accepted | Rejected |
Withdrawn`. The RFC process makes the *reasoning* a durable artifact and is the
mechanism by which decision-making broadens **as real contributors arrive — the
document changes with reality, not ahead of it.**

## Final comment period (FCP)

*Added 2026-06-11 (reassessment Directive 11 — pre-registered before the first
external contributor arrives, not after).* When an RFC's discussion converges,
the maintainer (or a future team) proposes **FCP with a disposition** (merge /
close / postpone): **10 calendar days**, announced on the RFC's tracking PR.
Any substantial new argument raised during FCP cancels it and returns the RFC
to Discussion; a clean FCP executes the disposition. Until external
contributors exist, FCP is the maintainer's self-imposed cooling-off period —
the process is real even while the parties are few, and **this section does
not move any authority**: releases, tags, and the four integrity rules remain
outside RFC scope per the rest of this document.

## Editions (compatibility evolution)

Language evolution uses **editions** (parse-time) plus runtime settings
(semantic-time), with the invariant that **capability semantics are
edition-invariant**. An edition bump is an RFC-gated event.

## Standards stewardship

Garnet seeds a **cross-language capability-manifest standard** (`@caps` surface →
manifest → transparency log; see `C_Language_Specification/GARNET_CAPABILITY_TRANSPARENCY.md`).
The intent to offer it to a neutral body (OWASP / Linux Foundation) is tracked as
**RFC-0001** — an **intent and a draft**, *not* an accepted standard. No external
body has adopted it.

## Conduct & security

See `CONTRIBUTING.md` (Code of Conduct) and `SECURITY.md` (if present) for the
conduct and vulnerability-disclosure expectations.

## Honest status (do not soften)

This is **single-maintainer governance for a research-grade prototype**. It is
enough to evaluate the project's decision path; it is **not** a claim of
institutional permanence, a foundation, or a multi-party steering body. As
contributors arrive, this document changes with the reality.
