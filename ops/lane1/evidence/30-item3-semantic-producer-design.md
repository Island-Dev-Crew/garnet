# Lane 1 Item 3 — semantic producer fingerprints

Date: 2026-07-17
Authority: local test-first evidence on `mission/l1-governance-activation`
State: implemented and final workflow set locally restamped; cross-OS CI
evidence remains pending.

## Preserved RED

Before implementation, the real immutable workflow projector accepted all five
counterexamples: quoted `run: "true"`, quoted `run: 'true'`, step
`if: "false"`, step `if: "${{ false }}"`, and job `if: "false"`.
`test_garnet_workflow_schema_policy.py` failed five subtests. The producer
contract also had no `producer_semantic_sha256` function and the new evaluator
trap raised `AttributeError`.

## Implemented boundary

- Inventory schema v2 requires one exact lowercase SHA-256 per producer.
- The digest binds workflow-level policy and triggers, the selected job, all
  transitive dependency jobs, ordered step definitions, and the matrix member.
- A separately pinned aggregate prevents coordinated edits to the active 31
  inventory fingerprints.
- Exact vacuous true commands and false job/step conditions fail projection
  all-or-zero.
- Evaluation reports the observed fingerprint and returns no bindings on any
  mismatch.
- The final active-31 identity aggregate is
  `899944d4f0344e4b53cdd3cb37b1da26061f5eaab5d49d8482f8157b1ed51aaa`,
  its semantic aggregate is
  `e0bd4246263329f3ebee56a7f2ae7d664e898c615893a2e4b051a54676012edf`,
  and its full binding is
  `94260eb24cc922f8247d1167a8d09ad11b6e2f11e67c6c7b8baab1aaa7e50651`.
- The prepared 32-context identity aggregate is
  `505abd5474941cf5f0aa460d4474418ba93cb21b3e0faed809c1e31157e866de`,
  its semantic aggregate is
  `45dac805fe848063d2e598ce40c37358907d0a247edfe2fcd33a786cc322ff3e`,
  and its full binding is
  `74b57334dc99a76528302364531dabe0145940a34feb88275f4d29fd4d20a2f3`.
  The base-controlled producer fingerprint within that prepared state is
  `618a3bf5b61a8083baf936c33e0deec0e20b0c07692b943ff42129d496acf355`.

## Fresh local GREEN

```text
test_garnet_required_context_contract.py: 16/16
test_garnet_workflow_yaml_policy.py: 4/4
test_garnet_workflow_schema_policy.py: 6/6
test_garnet_required_context_evaluator.py: 13/13
test_garnet_workflow_identity_policy.py: 6/6
```

The inventory was re-fingerprinted only after the Item 7 base-controlled
workflow and final CI command list were frozen. No live or cross-OS promotion
is claimed by this local evidence.

An independent read-only reviewer approved the current Item 3 bytes after
rerunning all five named suites and confirming the exact vacuous command and
condition traps remain RED. The required Linux and Windows runtime results are
still pending CI.
