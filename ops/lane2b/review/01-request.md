# Lane 2B Review Request 01 - raw-byte framing core

- Implementer: Codex GPT-5.6 Sol
- Reviewer: Claude Fable 5
- Authenticated carrier: Jon
- Reviewed base: `cede73c03c5d535306ed179b5882e99e4d17b050`
- Reviewed head: `0919f7a1b14d6f53a251f3528787b2278331e488`
- Reviewed tree: `3266d5bd05312368777d57da6fe48e0342b5ad60`
- Branch: `mission/l2b-sealed-shelf-mcp`
- Scope: S0-S2 contracts, Slice 1, and Slice 2 framing core only

## Diffstat versus reviewed base

```text
15 files changed, 1091 insertions(+), 9 deletions(-)
```

## Fresh gates on reviewed head

```text
Lane 0 closeout: PASS - evidence 22/22 - ledger 37 - denominators 4/4 - launch HOLD - band 3
MSRV: PASS - ok true - findings [] - 18 active manifests - rust 1.95
Frozen backlog: PASS - ok true - findings [] - 8 entries
Raw-byte framing: PASS - 4 passed, 0 failed
Tier-1 Shelf router: PASS - 3 passed, 0 failed
Existing MCP schema/lifecycle/adversarial: PASS - 14 passed, 0 failed
```

The process-level `--test mcp_stdio` gate remains pending. No public process
entry exists until the next slices make a verified sealed package the only host
constructor.

## Review questions

1. Does the lifecycle callback preserve default empty capabilities,
   method-not-found behavior, request-ID limits, and initialize readiness?
2. Does framing fail closed on lone LF/CR, extra headers, noncanonical lengths,
   oversize bodies, truncated bodies, and invalid UTF-8 without using text I/O?
3. Is the application surface frozen to `garnet.core.double` with exact bounded
   arguments and panic-firewalled interpreter execution?
4. Is there any path that executes tool code before lifecycle readiness or that
   would allow the coming seal check to occur after execution?
5. Should any blocker be cured before the sealed-package and negative-proof
   slices continue?

Verdict must be committed separately as `01-verdict.md`; this request is never
edited or treated as approval.
