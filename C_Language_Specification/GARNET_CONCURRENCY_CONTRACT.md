# Garnet Concurrency Contract (v0.8, S41)

Status: codifies the concurrency model **as built** in `garnet-actor-runtime` and
Mini-Spec §9. This document is descriptive of the shipped runtime — it does not
introduce new semantics. Deferred items are labelled explicitly.

Garnet's concurrency model is **actors**, not async/await. (`async` is a reserved
word in a future edition only — see S32 / `garnet_parser::Edition`; there is no
`async`/`await` surface in v0.8.)

## 1. Actors

An `actor` is an isolated unit of concurrency. At spawn time each actor gets:

- **one OS thread**, and
- **one mpsc mailbox** (`garnet-actor-runtime/src/runtime.rs`).

State inside an actor is reached only through messages; there is no shared
mutable state across actors by construction.

## 2. Bounded mailboxes (no unbounded-mailbox DoS)

Mailboxes are **bounded**. The runtime uses `mpsc::sync_channel` with a default
capacity (v3.4 BoundedMail / Security Layer 2). This closes the
unbounded-mailbox DoS class: a misbehaving sender cannot make the mailbox grow
without limit.

- Default capacity applies unless overridden.
- `@mailbox(N)` overrides the per-actor capacity (`1..=1_048_576`, checker-validated).
- A full mailbox makes `tell` **block** (back-pressure); `try_tell` returns
  `SendError::Full` instead of blocking.

## 3. Messages: `ask` vs `tell`

An actor declares its message interface as **protocols**:

```garnet
actor Counter {
    protocol incr()            # tell  — fire-and-forget (no reply)
    protocol get() -> Int      # ask   — request-reply, Result-returning
    on incr() { ... }
    on get()  { ... }
}
```

- A protocol that **returns a value** is an **`ask`**: request-reply. The reply
  is **Result-returning** (`garnet-actor-runtime` 0.4.0, "Result-returning
  ask") — a failed/cancelled handler surfaces as an error **value**, not a panic
  that crosses the actor boundary.
- A protocol that **returns nothing** is a **`tell`**: fire-and-forget, subject
  to mailbox back-pressure (§2).

`garnet concurrency <file>` reports each actor's protocols classified as ask/tell
(see `garnet_check::concurrency_surface`).

## 4. Spawn and fan-out

`spawn <expr>` starts concurrent work. Unbounded fan-out is an **explosive
operation** (S40): `garnet ceilings` flags every `spawn`, and `@fan_out(K)`
declares a governing fan-out bound (else the default fan-out ceiling applies).

## 5. Resource bounds

- `@bounded(N)` (S39) declares a CPU/fuel budget for a function — the
  Wasmtime-fuel lowering target (enforcement deferred; see S39/S40 honest scope).
- `@mailbox(N)` (§2) bounds an actor's mailbox.
- `@fan_out(K)` bounds spawn fan-out.

## 6. Sendability

`@nonsendable` marks a type that must not cross an actor boundary (v3.4 Sendable
/ Mini-Spec v1.0 §9.4.3). The annotation is parsed and carried on the type; full
static enforcement of the send boundary (rejecting a `@nonsendable` value passed
in a message) requires value/type-flow tracking and is **deferred** — declared,
not yet statically enforced.

## 7. Hot reload

An actor's behaviour can be replaced at runtime, draining pending mailbox
traffic; reloads can be Ed25519-signed (`garnet-actor-runtime/src/reloadkey.rs`).
This is a runtime capability of the actor runtime, surfaced for completeness.

## Honest scope (what this contract does NOT yet claim)

- **No async/await.** Concurrency is actor-based; `async` is reserved for a
  future edition only.
- **`@nonsendable` enforcement is deferred** — the annotation is recorded; static
  rejection of cross-boundary sends needs type-flow analysis (a later slice).
- **Resource-bound ENFORCEMENT is deferred** — `@bounded`/`@fan_out`/ceilings are
  declared + reported (S39/S40); runtime fuel enforcement lowers to Wasmtime,
  which is absent in the current environment. No enforcement is faked.
- **Structured concurrency / cancellation** beyond the actor lifecycle + the
  Result-returning `ask` is not specified here; it is future work.
