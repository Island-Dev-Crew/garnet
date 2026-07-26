# Lane 1 Item 7 — base-controlled old-base workflow and ceremony evidence

Date: 2026-07-17

Local evidence host: macOS 26.5 (arm64), Python 3.14.5

State: preparation PASS; live activation path implemented; execution RED
(`blocked-u17`); no activation performed

## Contract prepared

- `.github/workflows/base-controlled-trust.yml` is a `pull_request_target` workflow limited to `main`, with repository permissions limited to `contents: read` and `pull-requests: read`.
- Its external actions are pinned to full 40-character lowercase commit SHAs. It checks out the exact event base SHA with `persist-credentials: false` and fetches the exact candidate SHA into a bare Git repository.
- Candidate content is treated as data. The base-owned evaluator reads exact candidate blobs with `git ls-tree` and `git cat-file`; it never checks out, imports, or executes a candidate-owned file.
- The candidate workflow byte digest must equal the protected workflow digest from the checked-out old base. Candidate required-context inventory, ledger, and workflow semantics are evaluated by base-owned policy modules.
- Review enumeration receives only an explicit bounded stdin token. Ambient GitHub and admin credentials are removed from the evaluator environment and are never rendered. The separate admin-authoritative `GARNET_ADMIN_GITHUB_TOKEN` remains a Jon-only U-17 dependency and is not present in this workflow.
- `.github/rulesets/governance-activation-ceremony.json` is a strict canonical preparation record for ruleset `18936562`, `31 -> 32`, with `bypass_actors: []`. It records the protected workflow byte digest `33bb45ac7400ed4572f5777c1ccd277ad73316244dca413e9c890fe2da8bf4c7`.
- The ceremony remains `prepared-not-activated`. It encodes the two-PR bootstrap boundary required by `pull_request_target`: Jon first provisions the admin token and merges this bootstrap PR while the context is not required. Only after the workflow is active on `main` does Jon open a separate activation/terminus PR, perform the 31-to-32 change while that PR is open, read back the no-bypass setting, rerun the authenticated gates on its exact head, and merge that terminus PR.

## Preserved RED before implementation

The focused tests were written before their production scripts existed:

```text
$ python3 -I scripts/test_garnet_base_controlled_trust_status.py
FileNotFoundError: scripts/garnet_base_controlled_trust_status.py
exit 1

$ python3 -I scripts/test_garnet_governance_activation_ceremony.py
FileNotFoundError: scripts/garnet_governance_activation_ceremony.py
exit 1
```

The first implementation run also preserved two test failures before correction: the test loaded a second incompatible contract dataclass identity, and its static workflow indentation expectation was wrong. The tests were corrected to exercise the evaluator's own base-contract module and the actual YAML nesting; production policy was not weakened.

Integrated review later found the preparation-only activation command was
unconditionally RED and could never close after U-17 provisioning. Three
live-readback tests were added first and failed because the evaluator and
stdin-capable CLI did not exist. The repair now binds a separately supplied
admin-authoritative stdin token to the exact ruleset endpoint, requires an
exact 32-context policy with `Base-controlled trust policy` appended under the
GitHub Actions integration, and rejects bypass actors, Boolean IDs, transport
failure, ambient credentials, and any checked/live policy mismatch.

## Fresh local GREEN

```text
$ python3 -I scripts/test_garnet_base_controlled_trust_status.py
....
Ran 4 tests in 0.003s
OK

$ python3 -I scripts/test_garnet_governance_activation_ceremony.py
..........
Ran 10 tests
OK

$ python3 -m py_compile scripts/garnet_base_controlled_trust_status.py scripts/test_garnet_base_controlled_trust_status.py scripts/garnet_governance_activation_ceremony.py scripts/test_garnet_governance_activation_ceremony.py
exit 0

$ /private/tmp/garnet-l1-policy-venv-20260717/bin/python -I scripts/test_garnet_workflow_yaml_policy.py
....
Ran 4 tests in 2.309s
OK

$ /private/tmp/garnet-l1-policy-venv-20260717/bin/python -I scripts/test_garnet_workflow_schema_policy.py
......
Ran 6 tests in 0.585s
OK
```

The base-controlled suite traps the permitted `31 -> 31`, `31 -> 32`, and `32 -> 32` transitions and the forbidden `32 -> 31` downgrade; semantic-policy failure; protected-workflow byte drift; rolling-review failure; explicit-token enforcement; and the candidate-inert workflow shape. The ceremony suite traps activation drift, bypass actors, wrong ruleset/transition, token conflation, Jon-only action drift, command drift, digest drift, evidence-destination drift, unknown keys, duplicate keys, and noncanonical JSON.

## Preparation gate GREEN; activation gate RED by design

```json
{
  "activation_ok": false,
  "activation_problems": [
    "blocked-u17"
  ],
  "bypass_actors": [],
  "preparation_ok": true,
  "problems": [],
  "ruleset_id": 18936562,
  "schema": "garnet.governance-activation-ceremony/v1",
  "state": "prepared-not-activated",
  "workflow_sha256": "33bb45ac7400ed4572f5777c1ccd277ad73316244dca413e9c890fe2da8bf4c7"
}
```

`python3 -I scripts/garnet_governance_activation_ceremony.py --gate` exited
`0`. `--activation-gate` without `--github-token-stdin` exits `1` immediately;
with injected test evidence it turns GREEN only for the exact live 32-context
ruleset and empty bypass list. No real credential was read and no live GitHub
state was mutated in this run. Jon's dedicated U-17 credential and the
post-bootstrap activation remain required.

## Named dependencies and non-claims

- Item 2 owns the final authenticated rolling-review adapter behavior for fork-head identity and explicit transport wiring. Item 7 calls that base-owned adapter and fails closed if it is unavailable or RED; it does not add a workaround.
- Item 3 owns the final required-context semantic fingerprint restamp after all workflow changes land. The frozen bootstrap has final active-31 and prepared-32 identity, semantic, and full-binding pins; activation is still future state.
- Item 4 completed the repository-wide action-pin conversion: the final staged index contains 90 credited occurrences, 12 distinct actions, 13 manifest entries, and zero mutable references. Item 7's two action references and the cross-OS evidence upload are included in that green reporter result.
- The trust-kernel companion, structured independent review record, landed marker, and fresh Linux/Windows evidence remain lane-level review work. This file records local macOS evidence only and does not claim cross-OS clearance.
- The clean detached macOS parity artifact at trust-content head `b9f8bc9`
  reports 36/36 tests, zero skips/failures/errors, parity digest
  `cf3631ea6afd3443d040500c5c453c07c7609c5994975bbff74e6d9f608c8cd6`,
  and exact-head evidence digest
  `d88e841b163837b13c9a2b3ca39e3b0e65f15cd81fb92329cdac81e792a50483`.
- The system Python lacks the required PyYAML 6.0.3 package metadata; YAML-dependent checks used the lane's pinned policy virtual environment. This is a recorded local tool degradation, not a skipped gate.
- U-17 is still open. There is no admin-authoritative live no-bypass readback, no 31-to-32 activation, and no claim that Lane 1 is merge-ready.
- The bootstrap PR must land before activation. Requiring `Base-controlled trust policy` on the PR that first introduces its `pull_request_target` producer would deadlock because GitHub loads that workflow from the default branch. The arc-closure terminus therefore belongs to the separate activation PR, not this bootstrap PR.
- U-19 evidence is necessarily post-squash. The activation/terminus PR must
  derive and register this bootstrap's exact landed commit/tree marker before
  it activates 32. After the terminus squash, one bounded closeout PR must
  register the terminus marker itself. That closeout adds no GOV number and is
  the pre-declared completion exception to the governance freeze; an empty
  registry is preparation, not landed proof.
- The old-base mechanism enforces byte identity for its protected workflow and
  exact-head v2 rolling review for the candidate governance set. A separately
  typed policy-upgrade record is not implemented in this bootstrap; it is
  planned and is not part of the enforced Item 7 claim.
- The authenticated GOV-009 gate supports both pinned states: the bootstrap's
  exact 31-context policy and the terminus's exact 32-context policy with the
  base-controlled producer last. Its clean-local-HEAD binding prevents the
  post-activation rerun from substituting an older green SHA or dirty policy
  bytes.

An independent read-only reviewer approved the corrected two-PR ordering and
the earlier preparation bytes after rerunning the 4/4 base-controlled suite,
5/5 ceremony predecessor, YAML/schema policy suites, action-integrity gate,
preparation gate, and deliberately RED activation gate. The final 10-test
ceremony suite, live-readback wiring, explicit transport, endpoint projection,
and non-mutating caller environment are included in the lane's fresh integrated
exact-head review; neither review replaces the final authenticated PR review or
U-17 readback.
