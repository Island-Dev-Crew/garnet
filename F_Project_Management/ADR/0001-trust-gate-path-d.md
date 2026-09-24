# ADR 0001 — The trust gate is a required human approval, not a self-checking workflow

**Status.** Accepted (2026-09-15), internally labelled *path D*. Not implemented:
no phase of it has been applied to the repository or its settings.

## Context

Trust-kernel changes merge under a written rule set: a pull request may not
modify the gate it merges under, a capability-surface widening must fail the
gate, every autonomous merge records agent, model and gate version, and the
release tag stays the maintainer's. Today those rules are carried by review
convention and by a human-merge-only path list, not by the hosting platform.

The Base-controlled composite workflow was the attempt to carry them
mechanically. It has been red on effectively every run since it landed, because
it runs on `pull_request_target` when a pull request opens or updates and
nothing re-runs it after an approval. It has never changed the outcome of a
merge. Making it a required check in that state would
stop every pull request, including correct ones. A check that is always red
teaches reviewers to ignore it, which is worse than no check.

## Decision

The human approval is the gate. Platform settings make that approval the key to
a barrier that is otherwise closed, and no automated check ever stands in for
it. The work is sequenced in rounds, each with its own stop point:

- **R0** — a required-reviewer rule on the gate paths, together with last-push
  approval, so a push after an approval invalidates it. Prepared as a pull
  request; the settings are applied by the maintainer.
- **R2** — retire the Base-controlled composite, and land a push-time classifier
  in shadow mode. It is advisory, it never executes pull-request code, and it
  runs for at least ten pull requests before anything depends on it.
- **R3–R4** — move the evaluator into a dedicated GitHub App scoped to this
  repository with Checks and Statuses write and no Administration permission,
  then bind it as a required context.
- **Team growth** — a second approver becomes required as soon as there is a
  second eligible person.

Three assumptions behind this are unverified against the live platform and are
to be tested on a throwaway repository before R0: required reviewers on the
current plan, last-push approval with zero required approvals, and same-name
check spoofing. The written doctrine that conflicts with this design —
the "renewed by close and reopen" text and the one-rerun exception — is amended
in the same round that makes the change, not afterwards.

Until a round lands, the gate is described as convention, never as enforced.

## Alternatives rejected

- **Make the existing composite a required check.** It is red on nearly every
  run for reasons unrelated to the change under review, so requiring it would
  freeze the repository while proving nothing.
- **Repair the composite in place and keep it as the gate.** It would still be a
  workflow defined in the branch it is judging. Repair does not remove the
  property that makes it the wrong instrument.
- **Add more required contexts without a dedicated App.** Any workflow-defined
  context can be redefined by the pull request that must pass it. A separate
  identity outside the branch is the part that carries the guarantee.
- **Keep review convention alone and document it more loudly.** That is the
  status quo, and it leaves the strongest rule in the project depending on one
  person remembering it at merge time.
