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

### Mechanical default-branch enforcement

The public `main` branch is governed by the active ruleset contract in
`.github/rulesets/garnet-main.json`, with its companion repository settings in
`.github/rulesets/repository-settings.json`. This first slice is a declarative
mirror. Once the prepared validators land, candidate CI validates those files
statically and a separate admin-authenticated human ceremony compares them with
the live GitHub API. The pre-activation contract requires a PR, current-base
success from 31 GitHub-Actions checks, resolved review threads, linear history,
and protection from deletion and force-push; it grants no bypass actor and
disables auto-merge. It also pins the Actions control plane to read-only default
workflow tokens and forbids Actions from approving pull-request reviews. The
32nd context is a separate activation delta described below.

Because the organization currently has one member, the approval count and
required code-owner review are intentionally zero: GitHub does not allow an
author to approve their own PR, so either setting would deadlock every change.
The required ledger includes always-present Mac Studio, Windows Studio,
agentic-dogfood, web/PWA, and pull-request parser-fuzz checks; those workflows
do not use path filters that could leave a required check pending or optional.
The final merge remains an explicit human action. The checked-in ruleset README
pre-registers the mechanical upgrade to one approval, code-owner review, and
last-push approval after a second accountable maintainer completes a
shadow-review cycle.

The 32nd context, `Base-controlled trust policy`, is activated only after its
`pull_request_target` workflow has first landed on `main`; it evaluates the
candidate as inert data using policy code from the base checkout. Until the
second-reviewer profile, exact CODEOWNERS matrix, and accepted write access are
all mechanically present, the prepared job and machine-status model report
reviewer identity as unconfigured and unverified. A name in JSON is not access evidence.
The ordered, multi-PR bootstrap is normative in `.github/rulesets/README.md`.
Once bootstrapped, the gate also requires that its job context be unique across
strict-canonical candidate workflows and that GitHub report the protected
default-branch workflow as active. GitHub ordinary status checks still bind a job name and App,
not a workflow ID or event; workflow-ID binding therefore remains a documented
platform limitation unless an organization-level required-workflow rule is
available and activated.

Live governance readback is authoritative only with an admin-authenticated API
read. A public ruleset response may omit bypass information and must never be
reported as no-bypass proof. Before each governance merge, a human runs the
authenticated gate, inspects the ruleset ID, no-bypass result, repository merge
settings, Actions token policy, and collaborator permissions, then records the
result with the reviewed commit.

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
