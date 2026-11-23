# Paper VI Experiment 4 — Kind-Aware Allocation & Mnemos Memory Harness

Deterministic, provider-free harness exercising Mnemos 4-tier memory workloads
(`examples/novel_01_capability_budgeted_memory_agent.garnet`,
`examples/novel_05_s22_stdlib_memory_pipeline.garnet`) and recording repeatable
wall-time and child max RSS telemetry with binary SHA-256 provenance.

```bash
python3 -I benchmarks/paper_vi_exp4_kind_allocation/run_benchmark.py --binary target/debug/garnet
```
