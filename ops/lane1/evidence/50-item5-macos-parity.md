# Lane 1 Item 5 — macOS parity evidence

Date: 2026-07-17

Local evidence host: macOS 26.5 (arm64), Python 3.14.5

State: local macOS PASS; Linux and Windows runtime evidence pending the pull-request matrix

## Preserved RED and repair

The pre-repair required-context contract suite produced 30 failures because
macOS exposed a temporary root lexically as `/var/...` while `/var` resolves to
`/private/var`. The fixtures now resolve their own temporary test root before
the strict ancestor walk. Production remains fail-closed: a caller-supplied
leaf, ancestor symlink, or reparse point is still rejected.

The Unicode trap now configures `core.precomposeunicode=false`, inserts both NFC
and NFD spellings through raw Git index operations, proves both byte-distinct
paths are present, and then requires the policy to reject their normalized
collision. It does not depend on the host filesystem preserving both spellings.

## Fresh macOS GREEN

```text
$ /private/tmp/garnet-l1-policy-venv-20260717/bin/python -I scripts/test_garnet_required_context_contract.py
Ran 16 tests
OK

$ python3 scripts/test_garnet_workflow_file_policy.py
Ran 10 tests in 1.733s
OK

$ /private/tmp/garnet-l1-policy-venv-20260717/bin/python -I scripts/test_garnet_workflow_yaml_policy.py
Ran 4 tests in 2.424s
OK

$ /private/tmp/garnet-l1-policy-venv-20260717/bin/python -I scripts/test_garnet_workflow_schema_policy.py
Ran 6 tests in 0.694s
OK
```

No local test was skipped. Targeted read-only review also reproduced the old
unresolved `/var` false red, proved the resolved `/private/var` fixture loads,
proved a deliberate ancestor symlink remains RED, and proved the NFC/NFD
collision remains RED.

The integrated parity runner then executed the frozen exact trust-content head
`b9f8bc91dd0660f0988e711fea31a535c0aae8f5` from a detached clean checkout and
emitted `/private/tmp/garnet-governance-policy-macOS-b9f8bc9.json`:

```text
head_exact: true
working_tree_clean: true
parity_exact: true
ok: true
tests: 36/36 (16 + 10 + 4 + 6)
skipped/failures/errors: 0/0/0
parity_sha256: cf3631ea6afd3443d040500c5c453c07c7609c5994975bbff74e6d9f608c8cd6
evidence_sha256: d88e841b163837b13c9a2b3ca39e3b0e65f15cd81fb92329cdac81e792a50483
```

The parity digest is deliberately head-independent and binds exact ordered test
IDs/counts/outcomes across operating systems. The separate evidence digest
binds that parity result to this exact head and macOS. Linux and Windows must
emit the same parity digest with their own exact-head evidence digests.

## Independent review and remaining boundary

An independent read-only reviewer approved the Item 5 implementation on the
current bytes and found no Linux/Windows portability regression in the raw-Git
Unicode boundary. This is not Linux or Windows runtime evidence. The identical
four-command CI step in `.github/workflows/ci.yml` must still report PASS on
Ubuntu, Windows, and macOS; any platform divergence or unexpected skip keeps
Item 5 open.

The system Python lacks exact PyYAML 6.0.3 metadata. YAML-dependent local gates
used the lane's disposable pinned virtual environment. This degradation is
recorded rather than treated as a skipped gate.
