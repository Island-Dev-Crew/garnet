# ADR 0014 — An actor promises nothing when it dies

**Status.** Accepted (2026-09-17). Documentation change landed with the T6-docs gap-0 PR (see CHANGELOG Unreleased).

## Context

Garnet has in-process actors that spawn, receive, answer and drop messages when
their queue is full. It has no supervision tree, no restart strategy and no
failure propagation contract. The `@mailbox` annotation is parsed and
range-checked and sets no capacity. Erlang and its descendants own supervision,
and a partial imitation would invite the comparison Garnet would lose.

## Decision

Garnet states plainly that an actor promises nothing when it dies: no restart,
no supervisor, no delivery guarantee for messages in flight. The two claims
Garnet does make about actors are compile-checked protocols and signature-gated
hot reload, and both must be provable by a test from the shipped binary before
they appear on a public page.

## Alternatives rejected

- **Build a supervision tree.** It is a large piece of work whose best version
  already exists elsewhere, and it does not feed the evidence kernel.
- **Say nothing about failure.** Silence reads as a guarantee to anyone who has
  used an actor system before.
