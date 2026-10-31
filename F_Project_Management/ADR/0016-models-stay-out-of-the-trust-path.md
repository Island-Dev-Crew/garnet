# ADR 0016 — Models stay out of the trust path; a model call inside a program is accepted, not implemented

**Status.** Accepted (2026-10-04). Not implemented: nothing of the model call is
built until every condition below holds and an RFC for the capability kind passes.

## Context

Garnet's acceptance verdicts are recomputable from bytes. `garnet check`,
`garnet diff-caps` and `garnet verify` give the same answer on the same input,
and the agent loop composes them without reimplementing a gate. Seals record model
fields as self-declared. A model already takes part as the author of a proposal
that these tools then check.

Two bets sit side by side. `FOUNDER-STORY.md` ("Why This Matters") describes a
language for building agent systems, and agent systems call models. The tools
above describe a substrate for accepting code that agents wrote. No Garnet
program can call a model today: `net::tcp_connect` closes its stream at once, and
the strict network policy denies loopback. `garnet-suggest-llm` is staging that
nothing links (D-42), and its only transport is the test `RecordingTransport`.

## Decision

- No model output reaches check, diff-caps, seal, verify, the runtime gates, the
  baselines they compare against, merges or rulings. A model may raise an item to
  human review or set the priority of an item already in review. It never lowers a
  review, grants authority or replaces a proof.
- Models stay in Garnet as authors through the agent loop, as clients of the MCP
  host, as subjects of benchmarks, and as raise-only reviewers.
- A model call made by a Garnet program is accepted as a direction and is not
  implemented. If it is built, it gets a capability kind of its own and is never
  folded into `net`. It reaches only a hash-pinned server out of process over
  loopback, and `garnet test` and the agent loop's run stage replay recorded
  answers only, so no verdict depends on a live answer.
- It is built only when all of these hold:
  - a pilot comparing Garnet with a Python baseline favors continuing the
    language;
  - a count of requests made after the public demonstrations shows demand for
    in-language model calls;
  - an RFC for the kind passes, and the maintainer rules on it under integrity
    rule 2;
  - a test shows that check, diff-caps, verify and the agent loop's run stage
    produce the same bytes with the network off.
- `garnet-suggest-llm` is removed under D-42, in the same PR as a test that no
  crate in `garnet-cli`'s dependency closure links an HTTP client, an ML runtime or
  that crate.

## Alternatives rejected

- **A model as judge of a verdict.** Even a model that could only add refusals
  would let an unchanged input fail, which breaks the promise that anyone with the
  toolchain recomputes the evidence.
- **Declare Garnet a substrate only and retire the model call.** This settles the
  founder story's bet before any evidence exists for or against it.
- **Keep `garnet-suggest-llm` unlinked until its review date, or link it on an
  advisory channel.** An unlinked crate keeps an unused surface in the workspace,
  and an advisory link needs a binary-level test that it cannot reach a check
  result or an exit code. Neither earns its cost before the call is built.
