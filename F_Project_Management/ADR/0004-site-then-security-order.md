# ADR 0004 — Organize the site first; the third-party app review goes last

**Status.** Accepted (2026-09-15); the reporting-route timing was ruled on
2026-09-17. Partly implemented: HTTPS enforcement landed on 2026-09-15. The
rest is sequenced but not done.

## Context

Two bodies of work were ready at the same time: correcting what the public site
says, and changing the repository's security settings. Interleaving them risked
a change of setting landing while the page describing it was still wrong, and a
reporting route being advertised before the route existed.

## Decision

The work runs in this order.

1. **Organize.** Correct the false statements on the site, move internal
   material out of `docs/`, and put the missing files in place — a 404 page, a
   `.well-known/security.txt`, the governance wording, and the funding page.
2. **Switch on the reporting routes.** Private vulnerability reporting and
   security alerts go on, and HTTPS is enforced on the site.
3. **Prove the contact door.** A real message is sent to the published address
   and confirmed to arrive at the endpoint that receives it. The address is not
   called live until an inbound message has been captured.
4. **Review third-party application access last.** Reviewing an installed
   application's permissions narrows what an already-installed integration can
   do, so it runs at the end of the pass rather than in the middle of it.

`security.txt` and private vulnerability reporting move together: the file is
published in the same change that the setting is switched on, so the file never
points at a route that is off. Ruled on 2026-09-17: the file ships in this first
site change, and private vulnerability reporting is switched on the day that
change merges. It is the one security setting that does not wait for the
dedicated pass, because leaving it off would make the file misdirect a reporter. The remaining settings work — credential split,
two-factor requirements, the reviewer team, and the trust-gate rounds of
[ADR 0001](0001-trust-gate-path-d.md) — is collected into one dedicated pass
rather than spread across the build, unless an item would break something
immediately.

Step 1 is also the step that lets the rest be described accurately: a setting
changed while the page still describes the old one produces exactly the failure
this project exists to avoid.

## Alternatives rejected

- **Security settings first.** It advertises a reporting route before the file
  that points reporters at it exists, and it changes behavior while the public
  description is still stale.
- **Interleave the two.** Every interleaving produces a window in which the site
  and the settings disagree, and the windows are hard to audit afterwards.
- **Publish `security.txt` now and switch reporting on later.** The file would
  name a route that is off. A contact file that misdirects a reporter is worse
  than no file.
- **Review third-party application access first.** It is the step most likely to
  interrupt work in progress, and nothing else in the sequence depends on it.
