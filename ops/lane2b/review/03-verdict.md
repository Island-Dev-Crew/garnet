# Lane 2B Review Verdict 03 — F1 cure, Shelf close, WV-6

reviewer: Claude Fable 5 (chat seat, Jon-relay mode per protocol; commit this
  file verbatim with this attribution)
reviewed_head: d386b882904d32160acbbcf02235b95002daa48a
reviewed_tree: 8f86737e0df646431a7429e37dbcee0a56af0667   request: 03-request.md
verdict: APPROVE-WITH-BLOCKERS

verified_identity: head/tree/diffstat reproduced exactly (53 files,
  +4060/−9 vs cede73c); tip 2a08653 post-head commits touch ops/lane2b/** only.
protected_bindings: garnet-cli/src/bin/garnet.rs blob 27835ca3… byte-identical
  to verdict-02 authorization; Shelf reporter blob 83a5354d… matches its bound
  digest exactly. Bindings honored.
f1_cure: VERIFIED GENUINE — sealed prelude_hash independently recomputed as
  blake3(PRELUDE_VERSION + "\n" + canonical LF prelude.rs bytes) = df4f1648… =
  sealed value, on this machine, without the implementer's toolchain. The
  .gitattributes pins are additive and target exactly the seal-input mechanism.
  New native trap prelude_build_input_is_canonical_lf makes any future CRLF
  build fail loudly. Durable cure, both directions.
weakening: none — zero assertions removed across c333db5f..head.
differential: full python battery (x86_64 Linux) 126/134 at head; 7 failures
  are the known pre-existing environmental set; ONE new-vs-main failure — see F1.

findings:
F1 (BLOCKER): scripts/test_garnet_wv_acceptance_status.py
  test_current_repository_is_pending_and_gates_red passes on main, FAILS at
  head: asserts WV-6 state == "pending"; repository now truthfully reports
  "accepted". The truth-surface test guarding the old state was not updated in
  the same series as the lawful flip. Cure: update that test to assert the new
  state (accepted + its gate expectations) in the same branch — this is a
  truth-surface pairing, not a weakening; say so in the commit. Also correct
  the packet/evidence "exact parity with main" claim, which this falsifies
  (claims match traps).
F2 (NOTE): reporter two-run byte-determinism and native cargo legs were not
  re-executed on this machine (no toolchain); committed cross-checkout
  evidence + the byte-level seal recomputation above cover the core. The next
  Air sweep should double-run the reporter from two fresh checkouts.

not_verified: cargo/native execution legs (machine unsuitable — chat seat).
next: cure F1, refresh evidence, push, commit 04-request.md. On its APPROVE
  the lane may proceed to PR under the standing ceremony.
