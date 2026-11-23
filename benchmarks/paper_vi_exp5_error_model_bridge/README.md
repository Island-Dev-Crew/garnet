# Paper VI Experiment 5 — Error-Model Bridging Harness

Deterministic, provider-free harness verifying exception handling in managed `def`,
typed `Result` flows in safe `fn`, `def <-> fn` boundary calls, and uncatchable
enforcement trap propagation across the boundary.

```bash
python3 -I benchmarks/paper_vi_exp5_error_model_bridge/run_benchmark.py --binary target/debug/garnet
```
