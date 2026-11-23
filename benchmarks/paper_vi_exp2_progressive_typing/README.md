# Paper VI Experiment 2 — Progressive Type-Disclosure Harness

Deterministic, provider-free harness verifying Garnet's managed (`def`) to safe (`fn`)
progressive disclosure checks (`garnet check --format json`) across clean and violating
programs with binary SHA-256 provenance.

```bash
python3 -I benchmarks/paper_vi_exp2_progressive_typing/run_benchmark.py --binary target/debug/garnet
```
