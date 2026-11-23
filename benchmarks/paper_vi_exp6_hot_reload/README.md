# Paper VI Experiment 6 — Signed Hot-Reload & Fingerprint Stability Harness

Deterministic, provider-free harness exercising Garnet's signed hot-reload examples
(`examples/mvp_11_signed_hotreload.garnet`, `examples/mvp_11_signed_hotreload_mismatch.garnet`)
and verifying deterministic `seal` digest stability across repeated runs.

```bash
python3 -I benchmarks/paper_vi_exp6_hot_reload/run_benchmark.py --binary target/debug/garnet
```
