# S114 Red-Team — Independent-Lane Verification Checklist

**Status:** S114 is **self-verified**, not independent. This doc scopes what an
*independent* lane must do to clear that label. **Jon assigns the lane; it
cannot be the fleet that produced S114** (this checklist's author included).

## Why S114 is only self-verified

S114 red-teamed the **enforced** trust kernel: six LLM "attacker" agents
generated attack programs and ran real `garnet` commands, then a **skeptical
referee** re-classified every claim HELD / HOLE / DECLARED-NOT-ENFORCED. One
genuine **HIGH** break was found (impl-method capability-surface blindness) and
fixed; two LOW stub-scoped holes were recorded.

The integrity gap: **the same fleet authored the attackers, the referee, the
fix, and the gate** (`scripts/garnet_red_team_status.py`,
`C_Language_Specification/GARNET_RED_TEAM.md`). Attacker = defender = judge. The
result is real and honestly recorded, but it is **self-attestation** — exactly
why the non-negotiable says *never call the S114 red-team "independent,"* and the
report carries the verbatim anchor *"not a 'nothing broke' claim."* No amount of
re-running the fleet's own scripts removes this; independence is a property of
**who** verifies, not how many times.

## What "independent" means here (acceptance preconditions)

1. **Different operator + agent/model lineage** than the S114 fleet. A fresh
   human or an agent fleet with no shared session/state with the S114 run.
2. **Fresh checkout** of the repo at a named commit; the lane records that commit.
3. **No reuse of the fleet's attack artifacts as the source of truth.** The six
   attacker scripts and the referee transcript may be *read for coverage*, but
   each finding must be **independently re-derived / re-constructed**, not
   re-executed and rubber-stamped. Re-running `garnet_red_team_status.py` (a
   static "is the fix + report present" gate) is **necessary but not
   sufficient** — it proves the artifacts exist, not that the kernel holds.

## The checklist (all must be independently confirmed)

- [ ] **HIGH fix actually traps — by fresh construction.** Independently write a
      program that exercises an impl method requiring a capability its enclosing
      `@caps` does not cover (and a nested-module variant). Confirm `garnet check`
      flags it. Do **not** rely on the in-repo regression tests
      (`impl_method_caps_are_in_the_surface`, `nested_module_fn_caps_are_in_the_surface`)
      as the proof — write your own and confirm it goes red on a reverted fix.
- [ ] **Re-derive the attack surface.** Independently enumerate attacks against
      the *enforced* surface: `@caps` host-authority (env/proc/fs/net/log),
      `@max_depth` recursion ceiling, `capability_surface` coverage, `caps-log`
      tail integrity, `seal` subject digest. Find any HOLE the fleet missed.
- [ ] **Independent referee classification.** Classify each attack HELD / HOLE /
      DECLARED-NOT-ENFORCED with your own skeptical judgment — and confirm the
      fleet's classifications were not generous (no HOLE mislabeled HELD or
      DECLARED-NOT-ENFORCED).
- [ ] **LOW holes still honest.** Confirm the two recorded LOW holes — `caps-log`
      forged-TAIL and `seal` `subject.digest` capability-blindness — remain
      correctly scoped as open/mitigated within the honest stub scope (not
      silently "fixed" without a trap, not worse than recorded).
- [ ] **Named-deferred fences are truly declared, not enforced.** Confirm
      `@bounded` (Wasmtime fuel), memory, time, `@mailbox`, and macOS/Windows
      OS-sandbox are **declared-not-enforced** — no attack makes the kernel
      *claim* to enforce one of these. Only `@caps` + `@max_depth` are enforced
      (both backends; seccomp Linux-only).
- [ ] **No overclaim survives.** The report's anchors stay literally true:
      `named-deferred`, `production / 1.0 claim` (absent), `not a "nothing broke"
      claim`. v0.8.1 stays research-grade.

## Acceptance + what changes on pass

- The independent lane records its findings in an **addendum** to
  `C_Language_Specification/GARNET_RED_TEAM.md` (operator, agent/model, commit,
  per-attack verdict) — it does **not** overwrite the self-verified record.
- **Only Jon** decides, on that evidence, to relabel the kernel red-team from
  *self-verified* to *independently verified*. The "self-verified" wording in the
  report and `[[garnet-foundation-integrity-lane]]` non-negotiables stays until
  he does.
- If the independent lane finds a new HOLE: it becomes a fresh HARDEN slice
  (trap + fix + regression), and independence re-verification restarts after.

## Out of scope for the independent lane

- Re-deriving the named-deferred enforcement (Wasmtime fuel, OS sandbox) — those
  are roadmap, not S114's claim.
- Production/1.0 certification — not on the table at v0.x.

---
*Prepared by the FOUNDATION-INTEGRITY lane as a handoff. The author is part of
the S114-producing fleet and therefore cannot be the independent verifier — this
document hands the work off, it does not perform it.*
