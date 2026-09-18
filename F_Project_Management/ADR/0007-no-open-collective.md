# ADR 0007 — Open Collective is not a funding channel for Garnet

**Status.** Accepted (2026-09-15), internal label F4.

## Context

`.github/FUNDING.yml` listed four channels. Three of them did not exist: the
Open Collective and Patreon pages returned 404, and the listed Ko-fi name
resolved to no creator page. The GitHub Sponsors listing was a draft that had
never been published. The file's own comments recorded that state, on a public
file.

Open Collective was the one worth a separate decision, because it is otherwise a
good fit for a small open-source project: a fiscal host, public ledgers, and a
grant-friendly structure. Its expense policy is the blocker. A project whose
maintainer operates through a limited liability company cannot transfer donated
funds to that company, which is the arrangement Garnet is maintained under.

## Decision

Open Collective is not used. `.github/FUNDING.yml` is reduced to a single custom
URL pointing at the funding page:

```yaml
custom: ["https://garnet-lang.org/funding.html"]
```

Nothing else is listed until the channel behind it exists and resolves. GitHub
Sponsors is added the day its listing is public and approved. Patreon stays out
until there is a regular publishing schedule for it to support. A channel
appears in the file the day it works, and not one day earlier — the same rule
that governs the site's own links.

## Alternatives rejected

- **Keep Open Collective and route around the expense policy.** It would mean
  changing how the project is operated to fit a donation platform. The
  arrangement should follow the work, not the funnel.
- **Keep the four entries and fix them later.** Three of four were dead links on
  a public file, and the comments explaining that were themselves published.
  A dead payment link is a worse signal than no payment link.
- **List a channel in draft state.** A Sponsors listing that is not public
  redirects to the organization page, so the sponsor button would lead a willing
  contributor to a dead end.
- **Remove `FUNDING.yml` entirely.** The custom URL costs nothing and points
  at the one page that states the truth, which is the only thing there is to say
  right now.
