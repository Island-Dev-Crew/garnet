# Garnet — Real-World Example Programs

These three programs are **not toys**. Each is a substantive Garnet program
exercising the dual-mode + memory + actor + pattern-match story end-to-end,
sized to demonstrate that the v0.3 language scales beyond `hello.garnet`.

| File | LOC | What it demonstrates |
|---|---|---|
| [`multi_agent_builder.garnet`](multi_agent_builder.garnet) | ~210 | Three-actor build pipeline (Planner / Compiler / Tester) with `episodic` memory recording every task outcome. Exercises `actor`, `protocol`, `on`, `spawn`, `memory episodic`, enum patterns with payloads, multi-line `if`/`elsif` branches. |
| [`agentic_log_analyzer.garnet`](agentic_log_analyzer.garnet) | ~225 | Log-stream analyzer using `semantic` memory (vector index of patterns), `episodic` memory (incident log), pattern-match with guards, recursion bounded by `@max_depth(8)` + `@fan_out(64)`, the pipeline operator `|>` for functional data flow. |
| [`safe_io_layer.garnet`](safe_io_layer.garnet) | ~200 | A `@safe`-mode IO API surface returning `Result<T, E>` consumed by managed-mode orchestration via `try`/`?`/`rescue`. Demonstrates Paper VI Contribution 5 (cross-boundary error bridging) with realistic shapes: stat / read / write / delete / retry. |

## Running them

From `E_Engineering_Artifacts/`:

```bash
garnet parse examples/multi_agent_builder.garnet
garnet check examples/safe_io_layer.garnet
garnet run   examples/safe_io_layer.garnet
```

The `multi_agent_builder` program uses actor-runtime threads — for a clean
demo, prefer `parse` / `check`; the actor runtime exercise lives in the
dedicated `garnet-actor-runtime/tests/` suite.

## Why these specifically

The v3.1 hostile audit flagged that the largest example was 141 LOC and
failed to demonstrate "spans multiple languages/domains" or "agentic
workflows" claims. Each of these programs is at least 200 LOC of dense
Garnet that would be tedious — not impossible — to write in any other
language with comparable safety guarantees. They are the working evidence
behind the Paper VI claim that Garnet is a viable agent-native platform.
