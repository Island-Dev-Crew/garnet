# Garnet S2 Bytecode VM Dogfood Evidence

Date: 2026-05-20
Branch: `codex/s2-bytecode-vm-scaffold`
Base commit during local dogfood: `5cc0cb1`
Machine: Apple M5, macOS 26.4.1 (25E253), arm64
Toolchain: `rustc 1.94.1`, `cargo 1.94.1`

## Slice

S2: Bytecode VM scaffold.

Goal from `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md`: instruction set,
serializer, loader, and execution for the top 10-15 opcode families, with
tree-walk fallback for the rest.

## Current Truth

- `garnet-vm/` is source-present and covered by unit/integration tests.
- The scaffold supports 15 opcode families: `Binary`, `Call`, `CallMethod`,
  `Const`, `IterInit`, `IterNext`, `Jump`, `JumpIfFalse`, `LoadGlobal`,
  `LoadLocal`, `MakeArray`, `Pop`, `Return`, `StoreLocal`, and `Unary`.
- Unsupported language forms fall back at function boundaries to the existing
  tree-walk interpreter.
- The benchmark below measures a prepared bytecode artifact against a loaded
  tree-walk interpreter on this local M5 Mac. It is PR dogfood evidence, not a
  standing benchmark campaign or production native-compiler proof.

## Dogfood Commands

```bash
cargo build -p garnet-vm --release
cargo build -p garnet-cli --bin garnet --release
cargo bench -p garnet-vm --bench parse_compile_execute > /tmp/vm-bench.txt

for f in examples/mvp_0{1,2,3,4,5}_*.garnet; do
  target/release/garnet run --vm "$f" > /tmp/vm.out
  target/release/garnet run --interp "$f" > /tmp/interp.out
  diff /tmp/vm.out /tmp/interp.out || exit 1
  printf 'matched %s\n' "$f"
done
```

## Local Output Summary

```text
$ cargo build -p garnet-vm --release
Finished `release` profile [optimized] target(s)

$ cargo build -p garnet-cli --bin garnet --release
Finished `release` profile [optimized] target(s)

$ cargo bench -p garnet-vm --bench parse_compile_execute > /tmp/vm-bench.txt
Finished `bench` profile [optimized] target(s)
```

Benchmark timing summary from `/tmp/vm-bench.txt`:

```text
parse_compile_execute/vm/mvp_01_os_simulator
                        time:   [1.4699 us 1.4742 us 1.4801 us]
parse_compile_execute/interp/mvp_01_os_simulator
                        time:   [3.6165 us 3.6240 us 3.6405 us]
parse_compile_execute/vm/mvp_02_relational_db
                        time:   [637.56 ns 640.24 ns 643.16 ns]
parse_compile_execute/interp/mvp_02_relational_db
                        time:   [1.5563 us 1.5751 us 1.5987 us]
parse_compile_execute/vm/mvp_03_compiler_bootstrap
                        time:   [282.99 ns 286.09 ns 292.66 ns]
parse_compile_execute/interp/mvp_03_compiler_bootstrap
                        time:   [503.39 ns 508.12 ns 514.97 ns]
parse_compile_execute/vm/mvp_04_numerical_solver
                        time:   [978.00 ns 992.68 ns 1.0093 us]
parse_compile_execute/interp/mvp_04_numerical_solver
                        time:   [2.2229 us 2.2486 us 2.3216 us]
parse_compile_execute/vm/mvp_05_web_app
                        time:   [1.1615 us 1.1694 us 1.1818 us]
parse_compile_execute/interp/mvp_05_web_app
                        time:   [2.2065 us 2.2179 us 2.2378 us]
```

VM/interpreter stdout diff loop:

```text
matched examples/mvp_01_os_simulator.garnet
matched examples/mvp_02_relational_db.garnet
matched examples/mvp_03_compiler_bootstrap.garnet
matched examples/mvp_04_numerical_solver.garnet
matched examples/mvp_05_web_app.garnet
```

## Status Reporter Delta

Before S2:

```text
scripts/garnet_proof_benchmark_status.py -> 3 Criterion harnesses
scripts/garnet_mit_readiness_status.py -> completion_percent 58.1, proof_empirics 40.0
```

After S2:

```text
scripts/garnet_proof_benchmark_status.py -> 4 Criterion harnesses, including vm_parse_compile_execute
scripts/garnet_mit_readiness_status.py -> completion_percent 58.1, proof_empirics 45.0
```

The broader MIT/productization percentage did not move because the objective is
averaged across open lanes; S2 makes the proof/empirics lane more granular
without closing mechanized proof, external user studies, or the benchmark
measurement campaign.

## Verification Run

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
cargo deny check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/test_garnet_mit_readiness_status.py
PYTHONDONTWRITEBYTECODE=1 python3 scripts/test_garnet_proof_benchmark_status.py
PYTHONDONTWRITEBYTECODE=1 python3 scripts/test_garnet_benchmark_no_run.py
PYTHONDONTWRITEBYTECODE=1 python3 scripts/garnet_mit_readiness_status.py --check-no-regression
PYTHONDONTWRITEBYTECODE=1 python3 scripts/garnet_conformance_matrix_check.py
python3 scripts/check-agent-contracts.py
```

## Desktop Evidence Bundle

```text
/Users/idc2.0/Desktop/garnet-s2-bytecode-vm-evidence/S2-dogfood.md
/Users/idc2.0/Desktop/garnet-s2-bytecode-vm-evidence/machine-metadata.txt
/Users/idc2.0/Desktop/garnet-s2-bytecode-vm-evidence/mit-readiness.json
/Users/idc2.0/Desktop/garnet-s2-bytecode-vm-evidence/proof-benchmark-status.json
/Users/idc2.0/Desktop/garnet-s2-bytecode-vm-evidence/vm-bench.txt
/Users/idc2.0/Desktop/garnet-s2-bytecode-vm-evidence/MANIFEST.sha256
```

Manifest verification:

```text
S2-dogfood.md: OK
machine-metadata.txt: OK
mit-readiness.json: OK
proof-benchmark-status.json: OK
vm-bench.txt: OK
```

## Not Claimed

- No production native compiler proof.
- No stable bytecode ABI.
- No full-language lowering.
- No full safe-mode lowering.
- No OS-thread actor bridge.
- No standing benchmark measurement campaign.
