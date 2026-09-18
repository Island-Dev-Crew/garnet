# ADR 0005 — Publish the funding goals with ranges and no total

**Status.** Accepted (2026-09-15), internal label F1. Implemented on
`docs/funding.html`.

## Context

Nine pieces of work were costed on 2026-09-15 from published figures: labor at
the US Bureau of Labor Statistics median for software developers at the low end,
senior contractor rates at the high end, plus named vendor prices where a goal
carries one. Every figure has a source; none is a quote from anyone who would do
the work.

The nine sum to a figure with a wide spread, most of it in two goals. Publishing
that sum invites two readings the numbers cannot support: that the project has a
budget, and that the sum is what Garnet needs to be finished.

## Decision

`docs/funding.html` lists the nine goals, each with its range and one sentence
saying what it would buy. It publishes **no total**, states that the numbering is
the order the goals were costed rather than a commitment to that order, and says
plainly that the ranges are estimates from published figures and not quotes.

Each goal carries its own truth fence where the underlying claim has one — that
the review action covers declared surface only, that closing the checker's blind
spots does not make the checker complete, that the execution budget moves one
fence while memory and time ceilings stay declared, and that the MCP host change
binds that host and not MCP in general. A funding page is a claim surface like
any other page, and the fences travel with the claim.

Four further items stay off the page because they depend on something that has
not happened: a bug bounty pool, a machine-checked lemma, a peer-reviewed paper,
and the standing domain and contact costs.

## Alternatives rejected

- **Publish the total.** It reads as a budget, and the spread between the low and
  high ends is wide enough that a single sum is close to meaningless.
- **Publish a single point estimate per goal.** It would have to be either the
  low or the high end presented as fact, and neither is.
- **Publish goals without numbers.** A wish list. The cost is the part that tells
  a reader what a contribution would actually change.
- **Publish a ranked roadmap with dates.** Order and dates are not decided, and
  putting them on a public page would commit the project to a sequence chosen by
  whoever funded the first goal.
