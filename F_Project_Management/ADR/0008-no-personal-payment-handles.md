# ADR 0008 — Sponsorship is paid to the maintaining entity; no personal payment handle on a Garnet surface

**Status.** Accepted (2026-09-15), internal label F5. The removal of the
personal link is implemented; the sponsorship listing is drafted and not
published.

## Context

The funding page and the landing page footer both linked to the maintainer's
personal tip page. That page carries a personal goal unrelated to Garnet, which
meant a reader who followed the link from a Garnet page was asked to fund
something else, under a personal name.

Separately, the GitHub Sponsors listing had been drafted but never published, so
the question of who the recipient is had not been settled in public.

## Decision

Two rules, and they apply to every Garnet surface.

1. **No personal payment handle appears on a Garnet surface.** Not in the
   footer, not on the funding page, not in `.github/FUNDING.yml`, not in a
   README. The links to the personal page are removed from `docs/funding.html`
   and the landing page footer.
2. **Sponsorship is received by Island Development Crew**, the entity that
   maintains Garnet, and the listing is published under the organization rather
   than a personal account. Tiers are drafted and are not published until the
   listing is.

Money given to Garnet is spent on the goals on the funding page. The entity's
own running costs do not come out of it, and they do not appear on that page.

The same separation applies in the other direction: personal projects and
personal costs are funded elsewhere and are not presented to a Garnet reader as
Garnet's needs.

## Alternatives rejected

- **Keep the personal link, with a note that it is personal.** A reader who
  arrives from a Garnet page reasonably reads the destination as Garnet's. The
  note does less work than the link does.
- **Keep it until the organization channel opens.** That trades an accurate page
  for an uninterrupted funnel, on a page whose entire purpose is to be accurate.
  Having no channel for a while is the correct state to publish.
- **Publish the Sponsors listing immediately to fill the gap.** The recipient
  and payout arrangements are settled with an accountant first. A listing that
  has to be corrected afterwards is worse than a listing that opens late.
- **Point the sponsor button at the personal account under the organization's
  name.** It would make the public record disagree with where the money lands,
  which is the specific thing this ADR exists to prevent.
