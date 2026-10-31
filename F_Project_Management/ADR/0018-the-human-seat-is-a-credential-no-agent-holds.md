# ADR 0018 — The human seat is a credential no agent holds

**Status.** Accepted (2026-10-04). Not implemented: the signing key and the
reporter that checks signed rulings come in the next train.

## Context

Integrity rules 1 and 2 put the merge of a gate change and the ruling on a
widening of declared authority in the maintainer's hands. The trust-kernel review
reporter checks that a reviewer's identity differs from the author's and the
committer's. It cannot check that a person holds that identity. Merges are
attributed to one account, and a suffix in the squashed title is the only mark
of the maintainer's own merges.

## Decision

- The maintainer holds a merge-and-ruling signing key that no agent environment
  holds. Agents keep their own identities.
- Each rule-1 merge and each rule-2 ruling becomes a signed record, and a reporter
  checks it. The reporter must fail on an unsigned record, a record signed by an
  agent identity, and a signature over different content. It is a gate change,
  so it is human-merge-only.
- Until that reporter lands, public copy calls human-merge-only a convention, and
  no demonstration claims a human ruling that a stranger can check.

## Alternatives rejected

- **Keep the title-suffix convention and say so.** It is cheap, and it leaves the
  credential route open.
- **A hardware key on the merge account only.** It reduces exposure, and it leaves
  no record a stranger can check.
- **Wait for a third-party red team to raise it.** The gap is already known, so
  waiting only defers the fix.
