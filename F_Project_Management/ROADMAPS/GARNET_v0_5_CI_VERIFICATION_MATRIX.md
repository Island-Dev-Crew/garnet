# Garnet v0.5 CI Verification Matrix

Date: 2026-05-07

| Gate | Command | Required before |
|---|---|---|
| Format | `cargo fmt --all -- --check` | every PR |
| Parser parity | `cargo test -p garnet-parser --test parse_v1_parser_parity` | Phase 1 merge |
| Conformance skeleton | `cargo test -p garnet-cli --test conformance_skeleton` | every language feature PR |
| Conformance phase gates | `cargo test -p garnet-cli --test conformance_phase_gates` | every roadmap/matrix PR |
| Dogfood output stability | `cargo test -p garnet-cli --test dogfood_readiness_examples` | every example/runtime PR |
| Canonical examples | `cargo test -p garnet-cli --test examples` and `target/debug/garnet parse/check/run examples/mvp_*.garnet` | every public-readiness PR |
| Workspace tests | `cargo test --workspace --no-fail-fast` | merge |
| Lints | `cargo clippy --workspace --all-targets -- -D warnings` | merge |
| Docs | `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps` | release |
| Security | `cargo deny --all-features check` and `cargo audit` | release |
| Security trust-boundary inventory | focused source scan for command execution, DB queries, filesystem/network authority, unsafe/eval/exec, secrets, sandbox/capability checks, and release integrity | every dogfood-readiness score |
| Release install | file-backed and network-backed installer smoke | tag/release |
