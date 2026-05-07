# Garnet Examples

This directory has two deliberately different example surfaces:

1. **Canonical MVP smokes:** `mvp_01_*` through `mvp_10_*`.
   These are the current app-level dogfood corpus. Every file must parse,
   check, and run on current `main`.
2. **Real-world design drafts:** `multi_agent_builder.garnet`,
   `agentic_log_analyzer.garnet`, and `safe_io_layer.garnet`.
   These are parser-scale reference programs that intentionally exercise
   actor/runtime/stdlib shapes that are still ahead of the interpreter.

## Canonical MVP Smokes

Run the full MVP corpus from the repository root:

```bash
for file in examples/mvp_*.garnet; do
  garnet parse "$file"
  garnet check "$file"
  garnet run "$file"
done
```

These files are intentionally compact. They are not the original long design
drafts. They are the CI-enforced proof that Garnet can execute ten distinct
application-shaped workflows today:

| File | Workflow proved today |
|---|---|
| `mvp_01_os_simulator.garnet` | cooperative scheduler simulation |
| `mvp_02_relational_db.garnet` | in-memory row filtering/query score |
| `mvp_03_compiler_bootstrap.garnet` | miniature expression evaluator |
| `mvp_04_numerical_solver.garnet` | iterative convergence solver |
| `mvp_05_web_app.garnet` | route dispatch logic |
| `mvp_06_multi_agent.garnet` | deterministic researcher/synthesizer/reviewer pipeline |
| `mvp_07_game_server.garnet` | game tick simulation |
| `mvp_08_distributed_kv.garnet` | vector-clock merge scoring |
| `mvp_09_graph_db.garnet` | graph traversal score |
| `mvp_10_terminal_ui.garnet` | terminal widget layout score |

The larger historical MVP drafts are archived at
[`archive/examples/mvp-design-drafts/`](../archive/examples/mvp-design-drafts).
They should not be cited as current runtime proof unless they are reintroduced
under CI with parse/check/run coverage.

## Real-World Design Drafts

These three programs are substantive Garnet syntax examples, but they are not
claimed as fully runnable application demos on current `main`. Treat them as
parser-scale reference programs: useful for syntax coverage and design review,
not as proof that the interpreter supports every runtime feature used in the
examples.

| File | LOC | What it demonstrates |
|---|---|---|
| [`multi_agent_builder.garnet`](multi_agent_builder.garnet) | ~210 | Three-actor build pipeline (Planner / Compiler / Tester) with `episodic` memory recording every task outcome. Exercises `actor`, `protocol`, `on`, `spawn`, `memory episodic`, enum patterns with payloads, multi-line `if`/`elsif` branches. |
| [`agentic_log_analyzer.garnet`](agentic_log_analyzer.garnet) | ~225 | Log-stream analyzer using `semantic` memory (vector index of patterns), `episodic` memory (incident log), pattern-match with guards, recursion bounded by `@max_depth(8)` + `@fan_out(64)`, the pipeline operator `|>` for functional data flow. |
| [`safe_io_layer.garnet`](safe_io_layer.garnet) | ~200 | A `@safe`-mode IO API surface returning `Result<T, E>` consumed by managed-mode orchestration via `try`/`?`/`rescue`. Demonstrates Paper VI Contribution 5 (cross-boundary error bridging) with realistic shapes: stat / read / write / delete / retry. |

## Checking the design drafts

From the repository root:

```bash
garnet parse examples/multi_agent_builder.garnet
garnet parse examples/agentic_log_analyzer.garnet
garnet parse examples/safe_io_layer.garnet
```

Runtime coverage lives in the dedicated crate test suites and the CLI
template smoke path (`garnet new --template cli`, `garnet test`, `garnet run`).
The larger design drafts intentionally stay ahead of the interpreter in a few
places, especially actor-runtime integration and richer stdlib method calls.

## Why these specifically

The v3.1 hostile audit flagged that the largest example was 141 LOC and
failed to demonstrate "spans multiple languages/domains" or "agentic
workflows" claims. Each of these programs is at least 200 LOC of dense
Garnet-shaped source. The working evidence for current release behavior is
the parser/checker/interpreter/CLI test ladder; these files are design-scale
examples that should graduate into runnable smoke programs as the v0.5.x CST,
LSP, and Memory Core Tier 1 work lands.
