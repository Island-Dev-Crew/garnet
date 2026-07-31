# Lane 2C Journal

## 2026-07-28 - implementer session

Cold booted from a fresh `origin` clone with `core.autocrlf=false`; verified
main and the stated parent lineage; all five truth-floor gates passed. The NUC
measurement ran on WSL2 ext4 after stopping OneDrive and Ubuntu cron and
verifying no concurrent build, sync, cron, or spawned agent.

The untouched base reproduced quadratic teardown in working, episodic, and
semantic stores across 256, 512, and 1,024 roots. Product commit
`0649d796ac6b78b968d868398b517974838112f3` adds exact incoming managed-ARC
edge accounting and an O(1) isolated-root candidate rejection. The same count
harness now reports linear curves. Focused cycle tests and the full memory
crate passed. The lane is parked for independent review; no verdict,
acceptance, PR merge, tag, or release action was recorded.

## 2026-07-29 - harness-placement amendment

The preceding entry's statement that the lane was parked for review is
superseded. A fresh MSRV gate found the active
`ops/lane2c/probe/Cargo.toml` as an undeclared nineteenth manifest. The
implementer stopped without changing the gate, the root workspace, or any
trust-kernel path. The stop was accepted as correct.

The superseded probe manifest had an empty `[workspace]` stanza. Cargo
therefore built it as a separate nested workspace and wrote its lockfile
outside the repository; root `Cargo.lock` remained at SHA-256
`01b8986b1cee0ef6a53ac439bd018b54fc1dca825a8f845a259ed8001e6715fa`.

The harness now ships as
`garnet-memory-v0.3/examples/lane2c_teardown_probe.rs` at product head
`5cd113617acd35307bb028463833a8da2bbd6ad2`. The active ops manifest,
transient lockfile, and probe source were removed. Proposed U-46 is recorded
in `ops/lane2c/DOCTRINE.md`: build manifests never live under `ops/**`;
harnesses live inside the crate under test; ops holds outputs, records,
verifiers, and replay scripts.

All prior profile outputs were replaced. The shipped example produced fresh
base and product curves for three cases at 256, 512, and 1,024 roots during
the quiet window `2026-07-29T08:34:09.8071444Z` through
`2026-07-29T08:35:39.5943570Z`; the ignored stress set passed 4/4 in that same
window. OneDrive, its sync helper, and Ubuntu cron were restored afterward.
Independent review remains pending; no verdict, acceptance, merge, tag, or
release action was recorded.
- 2026-07-30T16:37:58Z: INDEPENDENT REVIEW VERDICT 01 (cross-family: Claude Code on Claude Fable 5, Anthropic, `Pulses-MacBook-Air.local`; implementer is Codex GPT-5-based on the NUC) — **BLOCKED on exactly one blocker.** Correctness resolves in the fix's favor: the O(1) incoming-edge gate is decision-equivalent to the old `!live && counts>0` predicate state-by-state, previously collected cycles cannot survive (foreclosed by `root_buffer_release_collects_cycle_when_threshold_is_reached`, `allocator_edge_decrement_buffers_newly_unreachable_cycle`, `buffered_collection_scans_only_buffered_roots`), the `checked_sub().expect()` is unreachable via the public API (pairing-invariant enumeration; modes immutable; struct private), and finalization order is byte-identically produced. All request gates, 20/20 cycle tests, full memory crate, cli cache, clippy -D warnings, doc, fmt, diff --check, and the plain example run are green at product head `5cd1136` (tree `85faad1d`); Cargo.lock hash exact; zero ops/** manifests; MSRV 18-manifest set; zero refs/pull. **B1: zero memcheck evidence exists in the lane** — six `valgrind --tool=memcheck --leak-check=full` captures (working-clear/episodic-drop/semantic-drop at 1024, before `efd4f6b` and after `5cd1136`) must land under `ops/lane2c/evidence/memcheck/{before,after}/` with MANIFEST + verifier wiring on the WSL2 guest; evidence-only, no product-tree change required. QUESTION answered: the 89.8s window cannot be reproduced on this seat (no valgrind on macOS/arm64; quiet-machine law) and records endpoints only — a shortfall would narrow the quiet-state claim, NOT the load-independent Ir counts, and the record already sets wall_clock_is_claim_evidence:false. Findings: F1 Codex commit authorship (carried, Lane 1 V11 F1 ratification applies), F2 rename `DOCTRINE.md` to state it is proposed with the binding rule landing via governance Repair #3, F3 add a buffered self-cycle test. Curve evidence (18/18) was independently confirmed prior to this round and not re-derived. No measurement on the Air; no code modified; no PR/merge/acceptance action. STOP.

## 2026-07-31 - Verdict 01 B1 evidence cure

Jon confirmed the implementer lane claim at `2026-07-31T06:18:14.7128428Z`.
The fork branch was re-fetched and fast-forwarded from `e73f73d7` to
`29cb1c7`; `ops/lane2c/review/01-verdict.md` was read in full before lane
work. `origin/main` remained exact at `efd4f6b`.

Verdict 01 is preserved as BLOCKED on exactly B1. Its correctness challenge
resolved in the repair's favor with the named tests
`root_buffer_release_collects_cycle_when_threshold_is_reached`,
`allocator_edge_decrement_buffers_newly_unreachable_cycle`,
`buffered_collection_scans_only_buffered_roots`,
`releasing_the_last_root_makes_cycle_collectable`, and
`isolated_root_release_never_enters_candidate_buffer`.

The original base and product probe binaries were reused after exact SHA-256
matches to `4577447bdfba5163467c48fc59d6444688a094c52df7a9360ffbeaa9f3f00a72`
and `0ca1e4e38471ba34ffe51274216a6de144910fb5d0c791a40be7a012bcdb9810`.
Valgrind Memcheck 3.22.0 then captured working-clear, episodic-drop, and
semantic-drop at 1,024 roots for both binaries. All six report zero definitely
lost, indirectly lost, and possibly lost bytes and blocks; all six report 544
still-reachable bytes in one block. Every before-to-after byte and block delta
is zero.

Leak accounting is deterministic, so no quiet-machine ritual was performed,
no OneDrive or cron service was stopped, and no quiet window is claimed for
these captures. The product head and tree did not move. F2 is addressed by
renaming the current proposal to `ops/lane2c/PROPOSED-DOCTRINE.md`; historical
references in Request 01, Verdict 01, and the earlier journal entry retain the
filename that existed at their exact reviewed heads. F3 remains explicitly
deferred to a later product slice. Request 02 asks the independent reviewer to
evaluate B1; the implementer records no verdict, acceptance, merge, tag, or
release action.
