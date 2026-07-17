# Lane 1 Item 6 — fail-soft repair evidence

Date: 2026-07-17
Implementation commit: `ab4171f`
Independent final verdict: **APPROVE**, no blocking findings.

## Preserved RED sequence

The first implementation was rejected after three executable false greens:
malformed inline dependency data, a vendored path not bound to its dependency,
and a missing explicit test root. The first repair was rejected again after
valid TOML 1.0 declaration variants were ignored, duplicate TOML keys passed,
a source could be replaced between validation and read, and multiple CLI roots
silently selected the last one.

The final test-first repair preserved both rounds of counterexamples. Its new
RED was 5 failures in the 32-case TOML/CLI suite plus an initially compile-RED
identity-binding unit trap.

## Implemented boundary

- `toml = 0.8.23` parses the complete manifest instead of a partial line
  grammar; spaced/quoted headers, dotted keys, inline tables, and dependency
  subtables are recognized, while malformed and duplicate keys are errors.
- `same-file = 1.0.6` binds an opened source identity. The validated bytes are
  carried into vendor, helper, and test execution without reopening a mutable
  lexical path.
- Vendored dependencies are exactly `.garnet/vendor/<dependency>` and reject
  mismatches, traversal, absolute paths, non-directories, and symlink escapes.
- Missing explicit project roots and multiple positional roots return errors;
  an existing empty project remains a valid zero-test project.

Both new dependencies are exact-pinned and were checked under Rust 1.95.

## Fresh GREEN and review

```text
cargo +1.95.0 test -p garnet-cli --test fail_closed_setup --locked: 32/32
cargo test -p garnet-cli bound_source: 2/2
cargo test -p garnet-cli: PASS
cargo +1.95.0 check -p garnet-cli --all-targets --locked: PASS
cargo clippy -p garnet-cli --all-targets -- -D warnings: PASS
cargo fmt --all -- --check: PASS
check-agent-contracts.py: 24 contracts
test_check_agent_contracts.py: 6/6
```

The final independent reviewer re-ran the original and follow-up attacks,
verified that the retained handle—not a later path open—supplies source bytes,
and approved the slice without findings.
