# ADR 0012 — What a memory tier promises

**Status.** Accepted (2026-09-17). Not implemented.

## Context

The memory crate's cycle module is a fixture rather than a collector, its own
documentation says no allocator-integrated collector exists, and the store's
save and load paths are Rust-only and unreachable from a Garnet program. Any
promise beyond what runs would repeat the pattern this project exists to avoid.

## Decision

A memory tier promises scope-rooted lifetime and policy-driven eviction, and
nothing else. Garnet does not claim a garbage collector, and it does not claim
persistence from a program until a bridged save and load path ships with a
test. The public pages state the promise in those terms.

## Alternatives rejected

- **Promise automatic cycle collection.** The collector is a fixture; the claim
  would be false on the day it was published.
- **Promise persistence now.** It is the work that makes the slice large, and it
  is not needed for memory to become visible to the capability system.
