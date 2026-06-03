# Garnet Studio Domain Proof Matrix

Status: `passed`
Suite: `all`
Platform: `linux x86_64`
Cases: `20/20`
Commands: `60/60`
Source included: `false`
Provider API called: `false`

## Cases

| Case | Group | Status | Parse | Check | Run |
| --- | --- | --- | --- | --- | --- |
| `mvp_01_os_simulator` | core-mvp | `passed` | `passed` | `passed` | `passed` |
| `mvp_02_relational_db` | core-mvp | `passed` | `passed` | `passed` | `passed` |
| `mvp_03_compiler_bootstrap` | core-mvp | `passed` | `passed` | `passed` | `passed` |
| `mvp_04_numerical_solver` | core-mvp | `passed` | `passed` | `passed` | `passed` |
| `mvp_05_web_app` | core-mvp | `passed` | `passed` | `passed` | `passed` |
| `mvp_06_multi_agent` | core-mvp | `passed` | `passed` | `passed` | `passed` |
| `mvp_07_game_server` | core-mvp | `passed` | `passed` | `passed` | `passed` |
| `mvp_08_distributed_kv` | core-mvp | `passed` | `passed` | `passed` | `passed` |
| `mvp_09_graph_db` | core-mvp | `passed` | `passed` | `passed` | `passed` |
| `mvp_10_terminal_ui` | core-mvp | `passed` | `passed` | `passed` | `passed` |
| `mvp_11_signed_hotreload` | trust-boundary | `passed` | `passed` | `passed` | `passed` |
| `mvp_11_signed_hotreload_mismatch` | trust-boundary | `passed` | `passed` | `passed` | `passed` |
| `agent_toolbelt_01_triage_router` | agent-toolbelt | `passed` | `passed` | `passed` | `passed` |
| `agent_toolbelt_02_capability_budget` | agent-toolbelt | `passed` | `passed` | `passed` | `passed` |
| `agent_toolbelt_03_memory_recall` | agent-toolbelt | `passed` | `passed` | `passed` | `passed` |
| `agent_toolbelt_04_release_gate` | agent-toolbelt | `passed` | `passed` | `passed` | `passed` |
| `agent_toolbelt_05_repair_planner` | agent-toolbelt | `passed` | `passed` | `passed` | `passed` |
| `multi_agent_builder` | agentic-design | `passed` | `passed` | `passed` | `passed` |
| `agentic_log_analyzer` | agentic-design | `passed` | `passed` | `passed` | `passed` |
| `safe_io_layer` | agentic-design | `passed` | `passed` | `passed` | `passed` |

## Honesty Notes

- A passed expected-failure case means Garnet rejected the unsafe path with the expected diagnostic.
- This matrix proves current CLI parse/check/run behavior for the selected examples only.
- It does not claim Windows signing, winget, Linux package completion, Windows ARM64, or provider-backed conversion.
