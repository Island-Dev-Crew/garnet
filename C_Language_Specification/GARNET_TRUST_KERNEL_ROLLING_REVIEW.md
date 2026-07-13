# Garnet trust-kernel rolling review (rolling S114)

**Status: normative operational policy.** S114 — the independent red-team of
the enforced trust kernel — must not be a one-time ceremony. This policy makes
it a **recurring control**: material changes to the trust kernel trigger either
a scoped acceptance update or a fresh adversarial review, and "no self-grading"
is an operational gate, not a slogan.

The machine-readable trigger set and the gate live in
`scripts/garnet_trust_kernel_review_status.py` (schema
`garnet.trust_kernel_review/v1`); this document is its human contract.

## The trust kernel (trigger surface)

A change is **trust-kernel** if it touches any of:

- **Checker** — `garnet-check-v0.3/src/**` (capability surface, CapCaps
  call-graph propagator, capability set vocabulary).
- **Interpreter** — `garnet-interp-v0.3/src/**` (the `@caps` frame machinery,
  `require_capability` / `require_entry_capability`, the strict-no-frame latch,
  the per-instance strict scope, `stdlib_bridge` adapters).
- **VM** — `garnet-vm/src/**` (scope parity, the per-run program-entry frame).
- **Stdlib registry** — `garnet-stdlib/src/**` (the single source of truth for
  capability rows + `Guard` columns).
- **Wasm runner** — `garnet-wasm/src/**`.
- **CLI authority flows** — `garnet-cli/src/cmd/run.rs`, `cmd/test.rs`,
  `cmd/eval.rs`, `cmd/doctest.rs`, `src/bin/garnet.rs` (the latch site).
- **Capability reporters** — `garnet_launch_readiness_status.py`,
  `garnet_caps_enforcement_status.py`, `garnet_capability_scope_status.py`,
  `garnet_bounded_enforcement_status.py`, `garnet_red_team_status.py`.
- **Public claims + scope** — `docs/why.html` and
  `C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md`.

## The rule

A trust-kernel change must be **accompanied by a review companion** — one of:

1. **A scoped acceptance update** — a change to
   `F_Project_Management/LAUNCH/S114_ACCEPTANCE.json` re-affirming or narrowing
   the accepted scope for what changed.
2. **A fresh review / verdict artifact** — a new or updated file under
   `proofs/independent/s114/**`, `F_Project_Management/W_TRUST/**`, or
   `F_Project_Management/VALIDATION_REPORTS/**`.
3. **An explicit review trailer** — a `Trust-Kernel-Review: <reviewer / scope>`
   line in a commit message on the range, naming who reviewed. The reviewer
   named must not be the sole author of the change lane — independence is a
   named property, never self-asserted (see `CLAUDE.md` integrity rules and the
   S114 dossier's independence ledger).

Absent a companion, `garnet_trust_kernel_review_status.py --gate` reports
**REVIEW REQUIRED** and exits non-zero.

## Scope and honesty

- This gate detects **that** the trust kernel changed and **whether** a review
  companion is present. It does not, and cannot, judge whether a review was
  *adequate* — that is the reviewer's job. It prevents the silent-drift failure
  mode (a trust-spine change with no review trail at all).
- Wiring this gate into CI is a workflow change and therefore **human-merge-only**
  (`CLAUDE.md` integrity rule 1). Until Jon wires it, it is a local/manual
  control; the file set and gate exist so that wiring is a one-line follow-up.
- Independence relabels and S114 acceptance remain **Jon-only**. This policy
  organizes the recurring review; it never grants an AI lane the authority to
  bless its own trust-kernel change as independently reviewed.

## Usage

```
# Against the current branch vs origin/main:
python scripts/garnet_trust_kernel_review_status.py --gate

# Explicit changed set (CI / testing), bypassing git:
python scripts/garnet_trust_kernel_review_status.py --gate \
  --changed-file garnet-interp-v0.3/src/eval.rs \
  --changed-file F_Project_Management/LAUNCH/S114_ACCEPTANCE.json
```
