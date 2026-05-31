# Garnet MCP tool-capability declarations (S67)

The documented MCP gap is the **absence of capability attestation**: an agent's
tools do not declare what authority they need, so a tool-set's aggregate
authority is invisible — exactly the blind spot the AI-PR-review-collapse wedge
(S49) warns about, one layer up. S67 brings Garnet's `@caps` model to MCP/agent
tools.

## The declaration

A `.mcpcaps` manifest names each tool's required capabilities (trivial format —
the repo's hand-rolled, no-serde stance):

```
# tool: cap1, cap2     (# comments and blank lines ignored)
filesystem: fs
fetch: net
clock: time
shell: proc, ffi
search: net_internal
```

`garnet mcp-caps <file.mcpcaps>` reports:

- the **per-tool** capability surface (sorted, deduped),
- the **aggregate authority** of the whole tool-set (the union — what the agent
  could collectively do),
- **high-authority** flags for tools declaring `ffi` / `proc` / `*` (review
  these), and
- **unknown** capabilities (names that are not a known `@caps`).

`--format json` (`schema: garnet.mcp_caps/v1`, always `"enforced": false`) makes
the surface diffable — a tool-set that *gains* `proc` or `ffi` between revisions
is as visible as a program gaining a capability (`diff-caps`, S37).

## Honest scope (do not soften)

These are **self-declared** tool capabilities, **not** runtime-enforced — Garnet
is **not** an MCP host and does **not** intercept tool calls. The value is a
reviewable, diffable, attestable declaration of a tool-set's authority surface
(the `@caps` posture: declared, not inferred). Enforcing the declaration at the
MCP boundary — and verifying a tool actually honors it — is out of scope (the
same honest line as the S65/S66 attestation declarations).
