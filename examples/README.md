# Garnet — Real-World Example Drafts

These three programs are substantive Garnet syntax examples, but they are
not claimed as fully runnable application demos on current `main`. Treat them
as parser-scale reference programs: useful for syntax coverage and design
review, not as proof that the interpreter supports every runtime feature used
in the examples.

| File | LOC | What it demonstrates |
|---|---|---|
| [`multi_agent_builder.garnet`](multi_agent_builder.garnet) | ~210 | Three-actor build pipeline (Planner / Compiler / Tester) with `episodic` memory recording every task outcome. Exercises `actor`, `protocol`, `on`, `spawn`, `memory episodic`, enum patterns with payloads, multi-line `if`/`elsif` branches. |
| [`agentic_log_analyzer.garnet`](agentic_log_analyzer.garnet) | ~225 | Log-stream analyzer using `semantic` memory (vector index of patterns), `episodic` memory (incident log), pattern-match with guards, recursion bounded by `@max_depth(8)` + `@fan_out(64)`, the pipeline operator `|>` for functional data flow. |
| [`safe_io_layer.garnet`](safe_io_layer.garnet) | ~200 | A `@safe`-mode IO API surface returning `Result<T, E>` consumed by managed-mode orchestration via `try`/`?`/`rescue`. Demonstrates Paper VI Contribution 5 (cross-boundary error bridging) with realistic shapes: stat / read / write / delete / retry. |

## Checking them

From `E_Engineering_Artifacts/`:

```bash
garnet parse examples/multi_agent_builder.garnet
garnet parse examples/agentic_log_analyzer.garnet
garnet parse examples/safe_io_layer.garnet
```

Runtime coverage lives in the dedicated crate test suites and the CLI
template smoke path (`garnet new --template cli`, `garnet test`, `garnet run`).
The larger example drafts intentionally stay ahead of the interpreter in a few
places, especially actor-runtime integration and richer stdlib method calls.

## Why these specifically

The v3.1 hostile audit flagged that the largest example was 141 LOC and
failed to demonstrate "spans multiple languages/domains" or "agentic
workflows" claims. Each of these programs is at least 200 LOC of dense
Garnet-shaped source. The working evidence for current release behavior is
the parser/checker/interpreter/CLI test ladder; these files are design-scale
examples that should graduate into runnable smoke programs as the v0.5.x CST,
LSP, and Memory Core Tier 1 work lands.
