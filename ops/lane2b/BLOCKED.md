# Lane 2B blocked checkpoint

- Implementer: Codex GPT-5.6 Sol
- Reviewer: Claude Fable 5 (chat seat, Jon-relay mode)
- Authenticated carrier / ceremony seat: Jon
- Branch: `mission/l2b-sealed-shelf-mcp`
- Launch: HOLD; lawful Band 3
- Pull-request refs fetched: none

## Verdict 03 outcome

- Verdict committed verbatim at `ops/lane2b/review/03-verdict.md`.
- Authorized truth-surface RED recorded before cure.
- Protected test binding: commit `115b1cd`, tree `b24c5d9`, blob `d10c665f`.
- Focused WV tests: 6/6.
- Full Python differential: base and lane both 928/17F/8E/3S; zero delta.
- WV-6: accepted 5/5 with 5 artifacts.
- WV-7: truthfully pending 0/5.

## Blocking condition discovered by the fresh gate

The deterministic Shelf reporter is red on the paired tree:

```text
Minimum Shelf gate FAILED: product bytes changed after the recorded runtime candidate
```

Its broad diff from branch candidate `a6f0da2` includes the authorized protected
test. Excluding the test would be an unauthorized path exclusion; rebinding the
candidate would change the independently bound reporter and would still depend
on branch history that a squash merge discards. The frozen WV-6 manifest has the
same durability issue with branch candidate `e2820ce`.

## Exact resume

1. Read immutable `ops/lane2b/review/04-verdict.md` and verify its reviewed
   head/tree and exact authorized paths before acting.
2. If authorized, RED-record and implement only the prescribed content-bound,
   squash-durable Shelf/WV provenance repair. Never fetch `refs/pull/*`.
3. Run the reporter twice from fresh LF and Windows checkouts. Claude Fable 5
   must independently repeat the two-checkout reporter run on the Air.
4. Require Shelf accepted, WV-6 accepted, WV-7 pending, focused WV tests 6/6,
   exact Python zero delta, trust gate clean, and Rust/fmt/clippy green.
5. Only an explicit final APPROVE permits Jon to open the PR. Merge remains
   human-only, and any post-merge rebinding ceremony must be named in advance.

Evidence: `ops/lane2b/evidence/15-verdict03-f1-green-and-reporter-stop.txt`.
