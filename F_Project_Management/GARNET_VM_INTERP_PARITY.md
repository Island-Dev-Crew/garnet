# Garnet VM / interpreter parity campaign (S73)

Garnet ships two execution backends: the tree-walking **interpreter**
(`garnet run --interp`, the default) and the bytecode **VM** (`garnet run --vm`).
Two backends means parity is a real correctness concern — they must not silently
diverge. S73 makes that a gated, reproducible campaign.

## What it checks

`scripts/garnet_vm_interp_parity.py` runs every `examples/*.garnet` program
through **both** backends and asserts they agree. The result today:

```
corpus: 33 examples/*.garnet programs
parity-ok (same stdout + exit code on both backends): 33/33
divergences: none
```

## Parity predicate — the deterministic surface

For each program, parity holds iff the two backends produce the **same stdout**
**and** the **same exit code**. The campaign compares **stdout + exit code only**
and ignores stderr, for two documented reasons:

1. **Cosmetic VM wrapper.** The VM prefixes runtime errors with `vm error:`
   where the interpreter does not — e.g. on `mvp_11_signed_hotreload_mismatch`
   both backends exit 1 with the same `BLAKE3 fingerprint mismatch` exception,
   but the VM's stderr string carries the extra prefix. Same semantics,
   different wrapper.
2. **Episodic-cache nondeterminism.** The compiler's episodic cache
   (`.garnet-cache/episodes.log`) emits run-to-run-varying strategy notes on
   **stderr**. Program **stdout is deterministic** across cache state, so it is
   the sound channel for semantic-parity comparison.

## CI

- **canonical-examples** job (builds the compiler): runs the differential
  campaign with `--gate` — fails on any divergence.
- **agent-contracts** job (python-only): runs the static gate (`--no-run`, corpus
  present) + 7 unit tests.

## Honest scope (do not soften)

This is **corpus-based** parity over the shipped examples, **not** a proof of
total semantic equivalence between the backends. Divergences, if any arise as the
corpus grows, are **reported, not hidden**. The stderr wrapper-prefix difference
is a known, cosmetic, non-semantic difference, documented here rather than masked.
