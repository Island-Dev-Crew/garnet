# ADR 0009 — The public contact door is a role address, then a form

**Status.** Accepted (2026-09-15), internal label F10. Not implemented: the
address is published as plain text, and the routing and form are not built.

## Context

Garnet has one maintainer. A personal address on a public page is a permanent
target for scrapers and advertisers, and it burns the one inbox that matters. It
is also the wrong long-term answer: a role address survives a change of person,
and a personal address does not.

Publishing an address as a `mailto:` link makes the scraping trivial and returns
unstructured text. A form returns structured fields and can carry basic abuse
controls.

## Decision

**`hello@garnet-lang.org` is the only contact address on any public Garnet
surface** — pages, `security.txt`, schema and metadata fields, README and
documentation. No personal address appears on any of them, and a human is
reached through the role address rather than instead of it.

The routing is built in one step: the domain's DNS moves to a provider with
email routing, the role address is routed to an endpoint the maintainer
operates, and a copy reaches a human inbox so nothing is lost. The address is
not described as live until a real message has been sent to it and confirmed to
arrive.

Once routing works, the funding and contact pages replace the plain-text address
with a form that posts structured fields to the same endpoint, with a honeypot
rather than a challenge that penalizes people using assistive technology. Until
the form exists the address stays on the page **as plain text**, never as a
`mailto:` link.

## Alternatives rejected

- **A personal address, published.** It exposes the only maintainer's inbox, and
  it has to be changed on every surface the day anything about the arrangement
  changes.
- **A `mailto:` link now.** It is the shape scrapers are built for, and it gives
  a sender no structure to fill in.
- **A form first, routing later.** A form that posts nowhere is a silent
  drop. The receiving end is built before the front door is opened.
- **A catch-all address.** It accepts every invented local part, which makes it a
  spam funnel and destroys any signal about where a message came from.
- **A contact form behind a CAPTCHA.** It penalizes exactly the people least
  able to work around it, for a volume of mail that does not justify it.
