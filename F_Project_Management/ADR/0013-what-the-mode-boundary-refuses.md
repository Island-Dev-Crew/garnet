# ADR 0013 — What the mode boundary refuses, and what it records

**Status.** Accepted (2026-09-17). Not implemented.

## Context

What the checker calls a boundary check counts calls: the counter increments on
every call and method expression and compares no modes. The audit log whose
documentation promises a reviewer can read one file and enumerate every crossing
is constructed only inside its own tests. Borrow checking runs only in safe
mode, so the same ownership mistake errors from a safe caller and passes
silently from a managed one.

## Decision

A managed-to-safe crossing is an error inside a module, a warning across module
boundaries, and is recorded either way in an audit artifact that the seal
covers. The recorded crossing names the caller, the callee, the direction and
the ownership disposition, so a reviewer can recompute it rather than trust it.

## Alternatives rejected

- **Error everywhere.** It would break existing programs that cross the boundary
  deliberately, before the language has a way to express the intent.
- **Warn everywhere.** A warning inside one module is a suggestion, and the
  boundary is the language's central claim.
- **Keep the counter.** A count wearing the name of a check is worse than no
  check, because the name is what readers believe.
