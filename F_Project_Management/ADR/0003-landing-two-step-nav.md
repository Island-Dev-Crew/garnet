# ADR 0003 — Landing navigation: two-step section links with a static full-page link beside each heading

**Status.** Accepted (2026-09-15), internal labels N1–N6. Not implemented: no
branch carries it.

## Context

The landing page carries long sections that also exist as full pages — Why,
Install, Playground, Status. A single nav bar cannot serve both readers: someone
skimming wants the section on the page they are already on, and someone who came
to read wants the page. Sending everyone to the full page loses the skimmer;
sending everyone to the anchor hides four pages behind scrolling.

## Decision

Why, Install, Playground and Status become **two-step** links. The first press
scrolls to the section and swaps the href; the second press opens the full page.
A garnet bead marks a two-step link, and its armed state is carried in the
accessible name, not by color alone. Beside each section heading sits a plain
static link to the full page, so the second step is never the only route.

Constraints that are part of the decision:

- The markup keeps the real destination in `href` (Status keeps
  `href="status.html"`). The two-step behavior is added by a separate script,
  in `try`/`catch`, after the existing page script. Without JavaScript the links
  behave as ordinary links to the full pages.
- Code and Capabilities stay one click. Only the four section-and-page pairs are
  two-step.
- The arming cue is shown at most twice per visitor.
- Links disarm on scroll-away, Escape, `pageshow`, or a viewport width change.
- The arrival zone is at least 48 px, with a scroll-idle failsafe, and
  `scroll-padding-top` equals the sticky bar height.
- Bead space is reserved only after the script runs, and the bar tightens below
  360 px.
- The service worker is not changed.
- The wordmark becomes a home link on the landing page and on status, minispec,
  synthesis, novel and ladder.

It ships as two changes: the wordmark home links first, then the two-step
behavior.

Verified on headless Chromium at 1440 px and 390 px: a double-click and a 120 ms
tap-tap each produce exactly one navigation. **WebKit and iOS are unverified.**
No claim is made about them until a run exists.

## Alternatives rejected

- **Anchor links only.** The four full pages become unreachable from the nav,
  and Status in particular is where a reader checks a claim.
- **Full-page links only.** Every nav press leaves the page a reader is still
  skimming.
- **A dropdown per item, with both destinations.** More markup, a worse target
  on touch, and it makes two ordinary destinations feel like a menu.
- **Deciding by pointer type or viewport.** Behavior that differs by device is
  behavior a reader cannot learn, and it fails the person who resizes.
- **Changing the service worker to help the second step.** Out of proportion to
  the problem, and it risks a cached-shell bug on every page to save one fetch.
