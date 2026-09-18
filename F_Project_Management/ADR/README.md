# Garnet ADRs — architecture and project decision records

An **ADR** records a decision that was already taken: what was chosen, why, and
what was turned down. It is a record, not a proposal. Once written it is not
rewritten to match later events — a decision that changes gets a new ADR that
supersedes the old one, and the old one's `Status` says so.

## ADR or RFC?

| | `rfcs/` | `F_Project_Management/ADR/` |
|---|---|---|
| Asks | *Should we do this?* | *What did we decide, and why?* |
| Timing | Before implementation | At or after the decision |
| Covers | Language surface, semantics, the capability model, stability tiers, editions | Gates and merge process, site and navigation, ordering of work, funding and contact policy |
| Outcome | Accepted / Rejected / Withdrawn, after a 10-day final comment period (`GOVERNANCE.md`) | Accepted, from the moment the maintainer decides |

A change to the language or the capability model needs an RFC. An ADR does not
replace one, and no ADR here grants a claim about enforcement: only a trap test
that could have failed does that.

## Numbering

`NNNN-short-title.md`, zero-padded to four digits, taking the next free integer.
Numbers are never reused, including for a withdrawn ADR. The sequence is
independent of the `rfcs/` sequence and of the `U-` / `D-` finding ids in
`F_Project_Management/W_TRUST/`.

Some ADRs cite a short label such as `F4` or `N2`. Those are the maintainer's
own decision-log ids, kept here so a future reader can line an ADR up against
the log entry it came from. They are not resolvable from this repository.

## Format

```markdown
# ADR NNNN — title

**Status.** Accepted (YYYY-MM-DD). One clause on whether it is implemented yet.

## Context
What was true when the decision was taken.

## Decision
What was chosen, in the present tense.

## Alternatives rejected
Each one, and the reason it lost.
```

Keep an ADR to a page. If it needs more, the detail belongs in a spec, a plan,
or a W_TRUST record, and the ADR links to it.

## Scope of what is recorded here

This directory is public. Decisions about prices quoted to a specific partner,
entity finances, and hardware purchases are recorded in the maintainer's private
log instead, and an ADR here carries only the part that affects people reading
or using Garnet.

## Index

| ADR | Title | Status |
|---|---|---|
| [0001](0001-trust-gate-path-d.md) | The trust gate is a required human approval, not a self-checking workflow | Accepted 2026-09-15 |
| [0002](0002-u117-cycle-cure-shape.md) | Cure the capability propagator's cycle blind spot with an iterative SCC pass | Accepted 2026-09-15 |
| [0003](0003-landing-two-step-nav.md) | Landing navigation: two-step section links with a static full-page link beside each heading | Accepted 2026-09-15 |
| [0004](0004-site-then-security-order.md) | Organize the site first; the third-party app review goes last | Accepted 2026-09-15 |
| [0005](0005-funding-goals-ranges-no-total.md) | Publish the funding goals with ranges and no total | Accepted 2026-09-15 |
| [0006](0006-funding-status-line.md) | The funding status line names who funds Garnet, not a dollar figure | Accepted 2026-09-15 |
| [0007](0007-no-open-collective.md) | Open Collective is not a funding channel for Garnet | Accepted 2026-09-15 |
| [0008](0008-no-personal-payment-handles.md) | Sponsorship is paid to the maintaining entity; no personal payment handle on a Garnet surface | Accepted 2026-09-15 |
| [0009](0009-contact-door-role-address.md) | The public contact door is a role address, then a form | Accepted 2026-09-15 |
| [0010](0010-grant-sequence.md) | Apply to no grant until the authority-review action ships | Accepted 2026-09-15 |
