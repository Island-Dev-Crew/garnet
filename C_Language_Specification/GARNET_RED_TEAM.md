# Garnet v0.8.1 — kernel red-team (S114)

An adversarial red-team that **actively tried to defeat** the enforced trust kernel,
ran the attacks on the real binary, and recorded both what held **and the holes
found**. The original S114 (Claude fleet) found one HIGH (impl-method surface
blindness). The 2026-06-25 **independent, cross-lineage** re-verification (OpenAI
Codex) then found two more HIGHs — a load-time `let`/`const` `@caps` bypass and an
invalid-`@max_depth` seal — and the Opus final review found the first remediation
(`4994867`) closed them only on the `run`/VM/`agent-loop` lanes, leaving the **same
bypass open** on `eval`/`test`/`doctest`/`repl`/dependency-preload (a fail-open
`require_capability` at `active_frames==0`). All are now fixed: `4994867` for the
wired lanes, and **S114-FIX-2** closes the residual via deny-by-default mediation.
Finding and fixing real holes is the honest outcome the academic bar
(CMU/MIT/Rice/Berkeley) rewards; this is not a "nothing broke" claim. S114 is
independently-re-verified-with-fixes **pending Jon's acceptance**, not
self-attested-closed.

## Method

Six attackers (caps-laundering, max_depth-bypass, diff-caps-evasion, seal-forgery,
agent-loop-bypass, seccomp-policy audit), each constructing concrete malicious
programs and running the real `garnet` commands, then a **skeptical referee** that
re-classified every claim — HELD / HOLE (real enforced-ceiling break) / DECLARED-NOT-
ENFORCED (only hit a named-deferred ceiling, not a hole). The referee rejected
several attacker over-claims (recorded below).

## HOLE found + FIXED — HIGH: impl-method capability-surface blindness

`capability_surface()` (`garnet-check-v0.3/src/capability_surface.rs`) scanned only
top-level `Item::Fn` and **skipped `Item::Impl` methods**, while the interpreter
(`eval.rs`) enforces `@caps` on **any** managed `FnDef` including impl methods. So an
`@caps(fs)` impl method that reads a file was **load-bearing and enforced** (removing
the annotation traps) yet contributed **nothing** to the surface that `diff-caps`,
the seal `capability_manifest`, the caps-log, and `sandbox` consume.

**Net effect (reproduced end-to-end on both backends):** a proposal that adds real
file-read authority via an impl method passed `diff-caps` as *"no authority
expansion, band 5/5"* (exit 0), and `agent-loop` **ACCEPTED + SEALED** it with an
embedded manifest of `aggregate:[]` — defeating the enforced diff-caps widening gate
and the agent-loop acceptance gate. The identical widening at top level was correctly
caught. This was the **one clean break of an enforced ceiling.**

**Fix (`capability_surface.rs`):** recurse into `Item::Impl` methods (and nested
`Item::Module` functions). After the fix, `garnet caps` on the impl-method program
reports `aggregate:["fs"]` (`Reader::read:["fs"]`); `diff-caps` flags
`+ caps GAINED: fs`, AUTHORITY EXPANDED (exit 1); and `agent-loop` **REJECTS at
stage diff-caps, never sealing it**. Regression tests:
`impl_method_caps_are_in_the_surface`, `nested_module_fn_caps_are_in_the_surface`.
(The static `check` caps-coverage already recursed into `Item::Impl`, so it was not
blind — the defect was solely in the surface derivation.)

## HOLES recorded (LOW — within honest "stub"/mitigated scope; follow-up)

- **caps-log `--verify` accepts a forged TAIL entry.** `verify_log` only checks the
  `prev_blake3` forward link and never re-derives `caps_blake3` from `caps`; the tail
  has no successor committing to its hash, so rewriting the most-recent entry's caps
  (+ a bogus `caps_blake3`) still prints *"chain intact … (append-only)"* (exit 0).
  Non-tail tampering **is** caught (CHAIN BROKEN). The module honestly self-declares a
  *local hash-chained stub — no signed tree head, no witness* (not Rekor), and verify
  executes no host effect. Follow-up: bind `caps_blake3` to `caps` in verify and/or
  anchor the tip; soften the "append-only verified" wording.
- **in-toto seal `subject.digest` is capability-blind.** `stable_ast_repr` omits
  `@caps`, so `@caps(fs)` and `@caps(fs,net,proc)` share an identical subject digest.
  **Mitigated:** the predicate embeds a differing `source_blake3` + `capability_manifest`,
  so a cosign signature over the predicate is not fooled — only a consumer keying
  provenance off the canonical subject digest is at risk. Follow-up: encode the
  capability surface into the subject (a digest change, deferred to avoid churning
  every existing seal).

## HELD — the enforced kernel withstood

- **proc authority is double-gated** (`require_capability` + `require_entry_capability`):
  laundering `proc` through an impl method **or** a top-level helper while `main
  @caps()` TRAPs (`requires program entry @caps(proc)`) on both `--interp` and `--vm`,
  no subprocess spawned. (This is why `proc` resisted the same impl-method placement
  that broke the single-gated `fs`/`net`/`env`.)
- **`@max_depth` self-recursion** traps deterministically at depth `N+1` on both
  backends; mutual recursion through the annotated name traps; out-of-range
  `@max_depth(9999)` is rejected (`1..=64`).
- **diff-caps top-level widening** → AUTHORITY EXPANDED (exit 1); agent-loop REJECTS,
  never seals. Mis-cased/unknown cap names are conservatively treated as gains.
- **Signed-manifest reattach to different source** → `verify` FAILs `source_hash`
  mismatch (Ed25519 bound to the re-derived BLAKE3 of source+AST).

## DECLARED-NOT-ENFORCED — correctly NOT holes (referee corrections)

- An `@max_depth(N)` function delegating deep recursion to an **unannotated** helper
  is not a ceiling break — no annotated bound is violated (the agent-loop "depth
  evidence" wording is an over-claim, not a defeat; recommendation-grade).
- An interp **stack-overflow** on extreme unannotated recursion where the VM
  succeeds is a real backend robustness divergence — a hardening target, not a
  ceiling hole.
- `time`/`@bounded` fuel/memory/`@mailbox` under-declaration: `time` is honestly
  named-deferred at runtime; `check` flags top-level under-declaration; diff-caps's
  *widening* contract is not violated when the declared surface is unchanged.
- The `@caps(*)` wildcard baseline is documented and out of scope.
- **(S114-FIX-2 — was the residual HIGH, now closed.)** The no-managed-frame allow
  (`active_frames==0`) is no longer "out of scope": the independent re-verification
  showed it let `eval`/`test`/`doctest`/`repl`/dependency-preload execute load/eval-time
  host authority unenforced. The `garnet` binary is now **deny-by-default** at
  `active_frames==0` (complete mediation / fail-safe default); only library/embedder
  direct calls (no Garnet program context) keep the permissive default.

## Honest scope

The enforced ceilings in scope are: the diff-caps widening gate, the agent-loop
acceptance gate, the runtime `@caps` host-authority trap (fs/net/env/proc), the
`@max_depth` per-function-name recursion trap, the static `check` caps-coverage, and
(Linux) the applied seccomp policy. The original impl-method HIGH and the two
independent-re-verification HIGHs (load-time `let`/`const` `@caps` bypass;
invalid-`@max_depth` seal) are fixed, and the deny-by-default residual closure
(S114-FIX-2) covers the `eval`/`test`/`doctest`/`repl`/dependency-preload lanes; two
LOW stub-scoped findings remain recorded for follow-up. `@bounded` fuel, memory, time,
`@mailbox`, and macOS/Windows OS-sandbox remain named-deferred. v0.8.1 is
research-grade; no production / 1.0 claim.
