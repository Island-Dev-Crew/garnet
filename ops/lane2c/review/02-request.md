# Lane 2C Review Request 02 - Verdict 01 B1 Evidence Cure

## Seats and exact boundary

- Implementer: OpenAI Codex, GPT-5-based model; exact submodel/version not
  exposed by the harness.
- Implementer machine: `NUCBOX_M2PRO_S`, Ubuntu WSL2 x86_64 on `/dev/sdd`
  ext4.
- Independent reviewer: Claude Code on Claude Fable 5
  (`claude-fable-5`, Anthropic), on `Pulses-MacBook-Air.local`, Darwin 25.5.0
  arm64 (Apple M5).
- Review carrier: IDC-Trust-Review only.
- Merge authority: Jon (IslandDevCrew) only.
- Branch: `mission/l2c-memory-teardown`.
- Request parent: `29cb1c700c47b48bd4f2902c94c794fd0c6a3cb7`.
- Base: `efd4f6bae8b3afaba74594e57944b2548142aeae`.
- Reviewed product head: `5cd113617acd35307bb028463833a8da2bbd6ad2`.
- Reviewed product tree: `85faad1de5a2c47cb632bedea78dfb89d209001a`.

Review the exact transported branch head containing this request and record it
in `ops/lane2c/review/02-verdict.md`. The product boundary above is unchanged;
the successor delta is restricted to `ops/lane2c/**`.

## Verdict 01 disposition

Verdict 01 remains BLOCKED on exactly B1 until this independent re-review.
Its cycle-correctness challenge resolved in the repair's favor with named
tests and invariant analysis. The accepted 18/18 Callgrind hash and count
confirmation is not in scope for re-derivation.

This request supplies only the evidence cure for:

> B1 - no leak evidence for the cheaper teardown.

## Six Memcheck captures

Command:

```sh
valgrind --tool=memcheck --leak-check=full --show-leak-kinds=all \
  --error-exitcode=99 BINARY CASE 1024
```

| Phase | Case | Definitely lost | Indirectly lost | Possibly lost | Still reachable | Capture SHA-256 |
|---|---|---:|---:|---:|---:|---|
| Before | working-clear | 0 B / 0 blocks | 0 B / 0 blocks | 0 B / 0 blocks | 544 B / 1 block | `b826c9d35e4444096b11ce8f5fa408e740abf722d7467cb5f5a579ac6715936e` |
| Before | episodic-drop | 0 B / 0 blocks | 0 B / 0 blocks | 0 B / 0 blocks | 544 B / 1 block | `2759f6dfb949983f506802b18c89e915ab935aee4b671e009aa1f1f7b00a9ccd` |
| Before | semantic-drop | 0 B / 0 blocks | 0 B / 0 blocks | 0 B / 0 blocks | 544 B / 1 block | `326b3301e3e31f93215088224c2a17168007f5d8953a3d32e12e4c2bd3e08673` |
| After | working-clear | 0 B / 0 blocks | 0 B / 0 blocks | 0 B / 0 blocks | 544 B / 1 block | `900c0f04c694cf237b376e09b86025740918610b96716733256a82ef3ebf2851` |
| After | episodic-drop | 0 B / 0 blocks | 0 B / 0 blocks | 0 B / 0 blocks | 544 B / 1 block | `80d57e8d03d0c96e6c7c3cc271010cb939c5d60aa4bf48e4db52a9b3e6b6a09b` |
| After | semantic-drop | 0 B / 0 blocks | 0 B / 0 blocks | 0 B / 0 blocks | 544 B / 1 block | `296c2e5ec7c4271b744451404ddfd802db4f0776298348cba63249892621a012` |

For every case, the before-to-after delta is zero bytes and zero blocks for
definitely lost, indirectly lost, possibly lost, and still reachable.

## Binary and environment provenance

- Base binary: original artifact reused after exact SHA-256 match:
  `4577447bdfba5163467c48fc59d6444688a094c52df7a9360ffbeaa9f3f00a72`.
- Product binary: original artifact reused after exact SHA-256 match:
  `0ca1e4e38471ba34ffe51274216a6de144910fb5d0c791a40be7a012bcdb9810`.
- Tool: Valgrind Memcheck 3.22.0.
- Filesystem: `/dev/sdd` ext4 inside Ubuntu WSL2.
- Quiet state: irrelevant to deterministic leak accounting. No quiet ritual
  was performed, no service was stopped, and no quiet window is claimed.
- Captures: `ops/lane2c/evidence/memcheck/{before,after}/`.
- Replay: `ops/lane2c/replay_memcheck.sh`.
- Hash and semantic verifier: `ops/lane2c/verify_evidence.py`.
- Byte-exactness: Valgrind emits a single trailing space on its blank
  `==pid== ` separator lines. The raw captures preserve those hashed bytes;
  whitespace checks exclude exactly the six `.memcheck.txt` files.

## Required review

1. Confirm the exact branch head, unchanged product head/tree, and that the
   successor delta from `29cb1c7` is `ops/lane2c/**` only.
2. Run:

   ```sh
   python3 -I ops/lane2c/verify_evidence.py --gate
   python3 -I scripts/garnet_lane0_closeout_status.py --gate
   python3 -I scripts/garnet_msrv_status.py --gate
   python3 -I scripts/garnet_frozen_backlog_status.py --gate
   python3 -I scripts/garnet_capability_scope_status.py --gate
   python3 -I scripts/garnet_evidence_integrity_status.py --gate
   ```

3. Verify all six Memcheck files are covered by `MANIFEST.sha256`; confirm
   their hashes, heads, cases, size, leak categories, error summaries, and
   zero before-to-after deltas agree with `measurement.json`.
4. Confirm both binaries were reused, not rebuilt, after their hashes matched
   the pre-existing harness provenance.
5. Confirm F2 is records-only:
   `ops/lane2c/DOCTRINE.md` became
   `ops/lane2c/PROPOSED-DOCTRINE.md`. The reviewer-authored Verdict 01 and
   exact-head historical records retain the former filename as historical
   truth; all current references use the proposed name.
6. Confirm F3 was not implemented and no product path moved.

## Requested verdict

Write `ops/lane2c/review/02-verdict.md` at the exact reviewed head. State both
model identities and machines. Decide whether the six captures and their zero
before-to-after leak deltas clear B1. The implementer does not author the
verdict. Any approval authorizes only Jon's later merge decision; it does not
record acceptance, merge, tag, release, or launch promotion.
