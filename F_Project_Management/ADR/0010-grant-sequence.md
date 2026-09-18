# ADR 0010 — Apply to no grant until the authority-review action ships

**Status.** Accepted (2026-09-15), internal label F11.

## Context

Ten funding programs were checked on 2026-09-15 against what Garnet can show
today. The result was consistent: the programs that would fit a project like
this one score on adoption or on criticality, and Garnet has neither yet. It is
a research-grade prototype at v0.x with a small repository footprint, and at the
time of the check GitHub's own license detection did not resolve the
dual-license to a recognized identifier.

Two windows were open in the near term. Both fit poorly, and an application
that fits poorly costs reviewer goodwill as well as time.

## Decision

Apply to nothing in September 2026.

The first application is to the **GitHub Secure Open Source Fund**, which is
open on a rolling basis, and it is submitted after two things are true: the
authority-sized review action has shipped, and the repository's license is
detected as MIT OR Apache-2.0. Both are inputs the program scores on, and both
are within the project's control. That award offsets the installer-signature
work and part of the security review.

Programs assessed and not pursued now:

- **Alpha-Omega** asks an applicant to argue that the project is critical
  infrastructure. Garnet is not, and saying so would be the argument.
- **NLnet** expects a European dimension the project does not have.
- **Sovereign Tech Fund** states it does not finance prototype development, and
  Garnet describes itself as a prototype.
- **FLOSS/fund** excludes projects with minimal usage.
- **OpenSSF** has no open route for an outside project.
- **OSTIF** is not a grant. It is the route for sourcing the outside security
  review once part of its cost is in hand.

US federal small-business programs were reviewed at the entity level rather than
as Garnet funding channels, and the decision on them sits outside this
repository.

A grant application waits on the same thing a sponsor does: something that can
be shown rather than described.

## Alternatives rejected

- **Apply broadly now.** Nine of the ten programs score on criteria Garnet does
  not meet. A rejection is a closed door for a cycle, and a poor application
  costs the next one.
- **Apply to Alpha-Omega in the October window.** It requires a criticality
  claim that would be false.
- **Apply to the GitHub fund immediately.** Traction is the criterion, and the
  cheapest way to improve it is to ship the goal that gives other people
  something to install.
- **Make a fundable claim about enforcement to strengthen an application.** It
  is the failure mode this project is organized against, and a security program
  is the worst possible place to make an unverified claim.
