# S67 Plan — MCP/tool capability declarations

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S67 (v0.8.1 band).
Map: reconciled plan §165-166 — close the MCP "absence of capability attestation".
Branch: `codex/s67-mcp-caps`. Base: `origin/main` @ `29d7e52` (S66).

## Approach
Bring @caps to MCP/agent tools: a `.mcpcaps` manifest declares each tool's caps;
`garnet mcp-caps` reports the surface. Self-declared, not MCP-host enforced.

## Deliverables
- `examples/mcp/agent_toolset.mcpcaps` — `tool: cap1, cap2` lines.
- `garnet-cli/src/cmd/mcp_caps.rs` + dispatch + help: parse (no serde — repo
  hand-rolled stance), report per-tool + aggregate authority, flag high-authority
  (ffi/proc/*) + unknown caps (reuse `Capability::from_ident`). `--format md|json`
  (`garnet.mcp_caps/v1`, `enforced:false`).
- `C_Language_Specification/GARNET_MCP_CAPS.md` — model + honest "not enforced".
- `garnet-cli/tests/mcp_caps.rs` — 2 cross-OS tests.

## Dogfood
- `mcp-caps agent_toolset.mcpcaps` → aggregate [ffi,fs,net,net_internal,proc,time];
  shell flagged high-authority (ffi, proc); json enforced:false.

## Honest scope (do not soften)
- Self-DECLARED, NOT MCP-host enforced — Garnet is not an MCP host; no tool-call
  interception. Enforcing/verifying at the boundary out of scope. No new lane.

## Gates
- Rust tests + ladder (workspace 0 failed; clippy clean). Ledger: `s66 →
  merged(5)` advanced; `s67` rides with S68.
