# Record Succession — Independent Review Request 01

Status: REVIEW REQUEST DRAFT

This request covers `mission/record-succession`, a linear fleet-fork lane based
on `efd4f6bae8b3afaba74594e57944b2548142aeae`. It does not authorize a pull
request, approval, merge, token action, or any change to
`mission/wv6-reaccept`.

## Candidate labels

- `base_commit`: `efd4f6bae8b3afaba74594e57944b2548142aeae`.
- `reviewed_head`: UNVERIFIED HYPOTHESIS — resolve from the transported fork
  ref `refs/heads/mission/record-succession` before review.
- `reviewed_tree`: UNVERIFIED HYPOTHESIS — derive from the exact transported
  `reviewed_head`; do not trust this request as a candidate binding.
- `touched_paths`: the gate's own classifier returns exactly:
  - `scripts/garnet_trust_kernel_review_status.py`
  - `scripts/garnet_workflow_yaml_requirements.txt`
  - `scripts/test_garnet_trust_kernel_review_status.py`
- `content_digest`: UNVERIFIED HYPOTHESIS — derive with the gate's own digest
  function from those exact `touched_paths` at `reviewed_head`.

`AGENTS.md` and this request are disclosed non-trust documentation changes;
the same classifier returns false for both paths.

## Proposed law for independent review

Multiple structured review records in one candidate range are lawful only
when their introduction commits and `reviewed_head` values form the same strict
linear ancestry. The uniquely tip-most record alone binds the full-range
`touched_paths`, `content_digest`, and authenticated GitHub review transport.
Every predecessor remains canonical, structurally valid, and append-only;
modification, deletion, forked ancestry, duplicate heads, or a later record
that backdates `reviewed_head` is RED.

## Mechanical pre-reads

- The `agent documentation contracts` checkout uses
  `${{ github.event.pull_request.head.sha || github.sha }}` with
  `fetch-depth: 0` and `persist-credentials: false`; pull-request evaluation is
  from the exact head, not the base or synthetic merge ref.
- A disposable pre-implementation `git merge-tree --write-tree` probe of the
  record-validation region against `efac4cb17b48b830c5e30e5ab08ad4d55111d2d0`
  exited 0 and produced synthetic tree
  `f41d7e5513a811c8f815ce147c456d9f13eab4c5` with the candidate topology code
  preserved. UNVERIFIED HYPOTHESIS — repeat against the transported
  `reviewed_head` before ruling on the final tree.
- `is_trust_kernel("scripts/garnet_workflow_yaml_requirements.txt")` returns
  `True`, so the pin file is included in `touched_paths`.

## Implementation surface

- `scripts/garnet_trust_kernel_review_status.py`: canonical-load every record,
  independently derive each introduction commit, require aligned strict
  ancestry, select one tip-most record, and pass only that record into the
  existing full-range and authenticated-transport verifier.
- `scripts/test_garnet_trust_kernel_review_status.py`: adds pure ordering tests
  plus repository fixtures for both halves of the law.
- `scripts/garnet_workflow_yaml_requirements.txt`: adds only the measured
  cp312 Linux x86_64 PyYAML 6.0.3 wheel hash
  `ba1cc08a7ccde2d2ec775841541641e4548226580ab850948cbfda66a1befcdc`.
- `AGENTS.md`: records the durable procedural invariant required by the root
  documentation contract.

## RED-before-GREEN evidence

- Exact `base_commit` baseline: 110 fixtures, 110 GREEN, 0 failures.
- Initial ordering RED: three pure fixtures failed with
  `AttributeError: module 'garnet_trust_kernel_review_status' has no attribute
  '_select_linear_record_path'`.
- Predecessor-shape RED: the malformed-schema predecessor fixture observed
  `status.ok == True` before shape-only validation was added.
- Final trust-gate suite: 120 fixtures, 120 GREEN, 0 failures on Python 3.12;
  the 110 pre-existing fixture verdicts remain unchanged.
- Required RED fixtures: forked `reviewed_head` ancestry; modified predecessor;
  deleted predecessor; tip-most record missing full-range `touched_paths`;
  later record bound to a non-tip `reviewed_head`.
- Required GREEN fixture: predecessor at an ancestor `reviewed_head` binding a
  trust-path subset, followed by a successor binding the full candidate range.
- Additional fail-closed fixtures: duplicate/equal heads and a malformed
  predecessor record.
- Agent documentation contracts: `agent-contracts: ok (24 contracts)` and
  6/6 unit fixtures GREEN.
- Workflow YAML policy: 4/4 unit fixtures GREEN; `--gate` exit 0 under Python
  3.12 with exact PyYAML 6.0.3.

The Git-heavy runs used a cached, network-disabled local container because the
Mac host had no spare process slots. No dependency was pulled, no remote was
mutated, and the exact source files were mounted read-only for verification.

## Air review request

Please independently resolve `reviewed_head` and `reviewed_tree` from the fork,
recompute `touched_paths` and `content_digest` with the gate's own functions,
repeat the merge-tree check against #521's exact head, and run the complete
fixture suite. Inspect both the RED and GREEN halves; a GREEN-only reproduction
is insufficient.

UNVERIFIED HYPOTHESIS — the lane has zero blocking findings if the independent
tree, classification, digest, merge-tree, and fixture results agree. The Air's
claim supplies the independent verdict; this request does not self-grade.

The canonical record draft remains off-tree until authenticated pull-request
metadata and the Air verdict exist. It must never be committed with invented
`pull_request_number`, `pull_request_id`, `review_state`, or `verdict` values.
