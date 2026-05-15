# Garnet Empirical Validation Plan

Date: 2026-05-14
Status: planned

## Goal

Create a reproducible empirical baseline before any broad performance/correctness
claims, and keep each pilot bounded by a script, dataset, and metrics contract.

## Falsifiable first milestone

- Run one archived pilot dataset through a deterministic script and publish:
  elapsed runtime, memory envelope, and a minimal correctness check output.

Acceptance command:

```sh
python3 scripts/run_empirical_pilot.py \
  --dataset data/pilots/math-mini \
  --output artifacts/empirical/pilot-001.json
```

Current target is placeholder-oriented until dataset pipeline and scripts are added.

## Scope

- Keep scope to one reproducible task family before scaling.
- Do not claim production-grade empirical validity without versioned scripts and
  archived outputs.
- Track sampling and preprocessing assumptions directly with collected metrics.

## Milestone 1: Pilot baseline

1. Add dataset packaging and hash metadata.
2. Add one deterministic pilot runner script.
3. Record one baseline output artifact and failure/mismatch logs.
4. Add a minimal CI check that validates output schema.

## Milestone 2: Comparative baselines

1. Add one control implementation (or previous run profile) for baseline comparison.
2. Add repeated runs with variance capture and confidence interval reporting.
3. Add one dashboard-friendly summary file under `data/` or `artifacts/`.

## Milestone 3: Expanded pilot family

1. Add a second independent pilot family (compiler throughput or actor runtime,
   not both at once).
2. Add publication of protocol, data location, and reproducible setup script.
3. Gate claims in `CURRENT_STATE.md` and conformance reporting to the completed
   pilot stage only.
