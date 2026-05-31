# Garnet compiler-as-agent advisory — two tiers (S69)

Garnet's "compiler-as-agent" contribution (Paper VI) surfaces *suggestions*, not
just errors. It has two tiers:

## Rules tier (S10 — ACTIVE)

Deterministic, in-tree rules over the parsed AST, surfaced by
`garnet check --suggest` (and the LSP, as `INFORMATION` with the rule ID). The
shipped rules:

- `managed-fn-missing-caps` — a managed `def` with no `@caps(...)`,
- `long-parameter-list` — a function with ≥ 4 parameters,
- `empty-function-body` — a function whose body is empty.

These are the deterministic control: stable, fast, no network, no model.

## LLM tier (pending-infra)

Provider-backed suggestions — richer, context-aware advice behind the same
`Suggestion` shape. This tier is **pending-infra**: no LLM provider is wired in
this environment, and S69 does **not** call one or add a firing advisory. It is
the open leg of the Paper VI scorecard:

> "4 supported, 2 partial (downgraded honestly), 0 refuted, **1 pending-infra**"

## Paper VI Experiment 1 — prep

When the infra lands, Exp 1 measures the LLM tier against the rules-tier control:

1. Hold the rules tier fixed as the deterministic control.
2. Wire a provider-backed suggester behind the same `Suggestion` shape.
3. Measure precision/recall against a curated corpus — the idiomatic corpus
   (S57) + the 12 proof-matrix domains (S48). **No measurement is claimed here.**
4. Report results honestly; downgrade the contribution if unsupported.

`scripts/garnet_llm_suggest_readiness.py` reports this live and gates that the
rules tier stays present; the LLM tier is intentionally **not** gated.

## Honest scope (do not soften)

The LLM tier is **pending-infra** — no model is called, no provider is bundled,
and no new firing advisory ships in S69. The rules tier is the active baseline;
this slice is the readiness/experiment-prep layer, and the Paper VI scorecard
("…1 pending-infra") is surfaced verbatim, not changed.
