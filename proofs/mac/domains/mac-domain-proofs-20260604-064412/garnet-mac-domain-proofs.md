# Garnet Mac Domain Proofs

- Status: `passed`
- Platform: `macos arm64`
- Domains: `6/6`
- Commands recorded: `15`
- Cross-OS role: `S107 Mac-Codex row for S109 consolidation`

| Domain | Status | Sealed | Verdict |
| --- | --- | --- | --- |
| `data_pipeline_net_egress` | `passed` | `false` | capability widening refused at diff-caps; no seal |
| `supply_chain_proc_escalation` | `passed` | `false` | declared subprocess authority addition refused by diff-caps; no seal |
| `config_processor_depth_trap` | `passed` | `false` | capability-clean proposal refused by enforced @max_depth trap; no seal |
| `accept_provenance_dossier` | `passed` | `true` | accepted on capability + depth evidence; four trust artifacts plus decision emitted |
| `pr_review_collapse` | `passed` | `false` | diff-caps hard-fails the authority-widening merge gate |
| `mcp_tool_authority_creep` | `passed` | `false` | `mcp-caps` reports high-authority tool declarations; this is a report, not an enforcement trap |

## Honest Scope

- Mac row only; Windows/Linux completion waits for their committed rows.
- Negative proofs have no seal by design.
- `mcp-caps` is static report evidence, not MCP-host enforcement.
- No seccomp, OS-sandbox, Wasmtime fuel, production, or v1.0 claim is made.
