# S8 — Signed Hot-Reload Demo — Implementation Plan

Date: 2026-05-20
Contract: `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md` § S8
State: not-started → **planned**
Reviewer: Jon (Island Development Crew)

> S8 is one of the six v0.5.0 release gates. PR title: `S8: Signed hot-reload BLAKE3 demo`.
> Closes Paper VI Contribution 5 surface gap.

## 1. Scope (in)
- `examples/mvp_11_signed_hotreload.garnet` — success path: BLAKE3 fingerprint matches; exit 0 with `reloaded successfully` on stdout.
- `examples/mvp_11_signed_hotreload_mismatch.garnet` — mismatch path: same structure, tampered payload; raises with literal text `BLAKE3 fingerprint mismatch`; exits 1.
- New "Signed hot-reload BLAKE3 demo" lane in `scripts/garnet_mit_readiness_status.py` (verified 100% when both example files exist).
- Updated test in `scripts/test_garnet_mit_readiness_status.py` asserting the new lane + deferred `actor.reload_signed` managed syntax.
- Regenerated readiness baseline.
- `CHANGELOG.md` Added entry under [Unreleased].

## 2. Scope (out)
- **Managed-mode `actor.reload_signed` syntax** — the contract's description ("demonstrating `actor.reload_signed`") could be read as asking for managed-mode syntax exposure, but the implementation note says "the implementation worked; the demo didn't exist." The Rust runtime `actor.reload_signed` already exists and is tested in `garnet-actor-runtime/tests/reload.rs`. Exposing it to managed mode is a separate slice with its own AST + interpreter wiring. This slice keeps the demo at the program level using `crypto::blake3` and `raise`.
- **Real signed-bytes payload** — the demo uses a text-shaped marker for readability. Production payloads are arbitrary bytes.
- **Ed25519 signature verification of the payload** — the BLAKE3 fingerprint is one of two layers of signed-hot-reload integrity. The cryptographic-signature layer is validated only in the Rust runtime today.

## 3. Honest partials
- "Demo at the program level, not at the runtime API level" — the dogfood block exit codes and output strings match what an integrated managed-mode `actor.reload_signed` would produce, but the underlying mechanism is `crypto::blake3` + comparison + `raise`, not the Rust runtime call.
- "BLAKE3 hash is precomputed and baked into each fixture" — if the payload string changes, the expected hash must be regenerated. The success fixture documents the regeneration command in a comment.
- "Two examples, one rule" — adding more examples (different payload formats, multi-stage reload) is future work.

## 4. Dogfood block (per contract S8)
```bash
garnet run examples/mvp_11_signed_hotreload.garnet
# Expect: exit 0, "reloaded successfully" in stdout

garnet run examples/mvp_11_signed_hotreload_mismatch.garnet
# Expect: exit 1, "BLAKE3 fingerprint mismatch" in stderr
```

Verified locally:
- Success: `exit: 0`; stdout contains `reloaded successfully: 7c3f...`; stderr empty.
- Mismatch: `exit: 1`; stdout empty; stderr is `runtime error: exception: BLAKE3 fingerprint mismatch`.

## 5. State-machine transitions
| Transition | Evidence |
|---|---|
| not-started → planned | this file |
| planned → in-progress | draft PR `S8: Signed hot-reload BLAKE3 demo` opens |
| in-progress → review-ready | CI green; `cargo test --workspace` includes the canonical MVP examples run |
| review-ready → dogfood-passing | Jon review + CHANGELOG |
| dogfood-passing → merged | squash-merge |

## 6. Risks and mitigations
- **The demo could be read as claiming more than it does.** Mitigation: file header comments + the lane's `deferred` field + the CHANGELOG entry all explicitly say managed-mode `actor.reload_signed` syntax is NOT exposed; the demo is a program-level reproduction of the fingerprint check.
- **Hash-baking convention** — if a future contributor changes the payload string without regenerating the expected hash, the success example would silently start failing. Mitigation: the success file's comment includes the exact regeneration command.
- **Existing example numbering** — examples in `examples/` use `mvp_01_*` through `mvp_07_*` today. Using `mvp_11_*` leaves room for the other v0.5 slices' demos (S7 actor OS-thread, etc.) to land between.

## 7. What I need from Jon
None for S8.
