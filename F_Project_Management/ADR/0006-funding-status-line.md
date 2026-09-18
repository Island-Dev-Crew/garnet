# ADR 0006 — The funding status line names who funds Garnet, not a dollar figure

**Status.** Accepted (2026-09-15), internal label F2. Implemented on
`docs/funding.html`.

## Context

The funding page needs one line at the top that a reader can trust without
reading further. The candidate wording was a dollar figure — "$0 received".

A dollar figure is a claim about every account that could have received money,
and it can only be made after each one has been checked. The categorical claim
is different: it is a statement about which relationships exist, and the
maintainer knows that directly.

## Decision

The page opens with:

> No grant, sponsor, donor or pilot partner funds Garnet today. Garnet is
> pre-revenue, and every goal below is unfunded.

"$0 received" is used only once every payment dashboard has been checked, and
then only for the period that check covers. Categorical claims about
relationships are preferred over arithmetic claims about balances, because the
first can be verified by the person writing the sentence and the second cannot.

## Alternatives rejected

- **"$0 received."** Stronger, shorter, and not yet checkable. If one dashboard
  shows a stray amount, the headline sentence of the funding page is false.
- **"We are not currently accepting donations."** True today but describes a
  policy rather than the state, and it would have to be rewritten the day a
  channel opens.
- **Leave the page silent about status.** Silence on a funding page reads as
  "funded, details withheld", which is the opposite of the case.
- **A progress bar toward a target.** There is no target, and the widget implies
  both a total (see [ADR 0005](0005-funding-goals-ranges-no-total.md)) and an
  open channel (there is none).
