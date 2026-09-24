# ADR 0015 — In-process actors are the language's concurrency story

**Status.** Accepted (2026-09-17). Documentation change landed with the T6-docs gap-0 PR (see CHANGELOG Unreleased).

## Context

Two concurrency models exist in the repository. The interpreter runs in-process
actors that programs can use today. A separate threaded actor runtime crate
implements bounded channels, protocol tests and signed hot reload, and no crate
in the workspace depends on it, so nothing a user runs reaches it. Documents
have described both as the future, which leaves a reader unable to tell which
one the language is.

## Decision

The language's concurrency story is the in-process actor model that runs today.
The threaded runtime crate is staging: it stays in the workspace, its crate
documentation says plainly that nothing links it, and no public page presents it
as shipped until a release links it and a test proves it from the binary.

## Alternatives rejected

- **Adopt the threaded runtime now.** The interpreter's value type is
  reference-counted and not shareable across threads, so this is a
  representation change, not a wiring change.
- **Describe both.** Two futures in one document is how the current confusion
  started.
