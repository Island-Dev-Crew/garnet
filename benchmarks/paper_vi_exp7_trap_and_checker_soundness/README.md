# Paper VI Soundness & Trap-Enforcement Reproducibility Harness

Deterministic, provider-free verification harness for the `v0.8.4-dev` soundness
properties landed in PR #615 (`65cb502c`):

1. **Uncatchable Enforcement Traps (`C1 / BE-F1`)**: verifies that capability
   and depth traps escape `rescue`, `ensure { return ... }`, and `rescue`-origin
   `ensure` overrides, while ordinary user exceptions remain rescuable.
2. **Static Capability Propagation (`FE-F2b` & Pipeline)**: verifies `garnet check`
   catches undeclared authority inside `match` guards, `Pattern::Literal`,
   closures, `BinOp::Pipeline` (`1 |> memory::working`), and deeply nested string
   interpolations (up to 128 nesting levels).
3. **Safe-Mode `try`/`rescue` Soundness (`FE-F4`)**: verifies `garnet check`
   rejects `try`/`rescue` inside safe `fn` across `let`-bindings, closures,
   `Pattern::Literal`, and deeply nested string interpolations.

## Usage

```bash
cargo build -p garnet-cli --locked
python3 -I benchmarks/paper_vi_exp7_trap_and_checker_soundness/run_benchmark.py \
  --binary target/debug/garnet
```

Outputs a deterministic JSON summary (with binary SHA-256 and per-probe exit
codes) and exits `0` only if every probe meets its expected soundness outcome.
