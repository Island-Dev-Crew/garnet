# Lane 1 Item 1 — GOV-009 local evaluator evidence

Date: 2026-07-17

Scope: local, injected-data implementation of the fresh, exact-reviewed-head,
outcome, and strict policy-equality mechanisms. No credential was discovered,
read, inherited, persisted, printed, or passed to a process. No network or live
GitHub request was made.

## Preserved RED

Before the test existed:

```text
$ python3 -I scripts/test_garnet_github_governance_gate.py
python: can't open file '.../scripts/test_garnet_github_governance_gate.py':
[Errno 2] No such file or directory
exit 2
```

After adding the adversarial test first, before adding the implementation:

```text
$ python3 -I scripts/test_garnet_github_governance_gate.py
FileNotFoundError: .../scripts/garnet_github_governance_gate.py
exit 1
```

The first implementation run retained one RED assertion: 8 tests passed and
the incomplete/empty collection diagnostic differed from the test's expected
fail-closed classification. The collector boundary was made explicit so an
empty result reports `collection is empty`; no acceptance condition was
weakened.

A later type-confusion trap then produced three deliberate RED failures:
Python's ordinary equality treated `0` as equal to `false` and `15368.0` as
equal to integer `15368`. The final evaluator uses recursive type-exact
equality for every checked-in/live ruleset and repository/Actions setting, so
those coordinated JSON type substitutions are RED.

Independent review then rejected the first GREEN with three gaps. New traps
were added before the repair and reproduced all three as a deliberate RED:

```text
$ python3 -I scripts/test_garnet_github_governance_gate.py
Ran 12 tests in 0.069s
FAILED (failures=9)
```

The false greens were a required-context name repeated on an unrelated check
suite and a coordinated fabricated context substituted through the supplied
policy, check rows, and both ruleset documents. The remaining failures proved
that `CollectionResult` accepted malformed `problems`/`rows` containers,
malformed row elements, booleans in integer fields, and page/byte/row counts
above the transport's hard bounds. No production condition was relaxed to
turn those tests green.

A follow-on policy-shape trap retained the correct digest but removed the
binding's checked-in workflow projection. It deliberately failed one of 13
tests with only the evaluator's generic fail-closed fallback; the evaluator
now validates that projection explicitly before calling the identity join.

A second independent review rejected that GREEN because the checked policy
documents were not themselves trust-anchored and the producer digest omitted
v2 semantics and immutable YAML names. Tests were added first for coordinated
checked+live branch-condition, rule, and `allow_auto_merge` drift; a nested
tuple that JSON would otherwise serialize like a list; coordinated declared
and observed semantic fabrication; and a coordinated binding/live workflow
name change:

```text
$ python3 -I scripts/test_garnet_github_governance_gate.py
Ran 17 tests in 0.094s
FAILED (failures=6)
```

All six were genuine false greens. The repair pins the full checked documents
and the complete v2 binding projection; no comparison or type check was made
advisory.

A third independent review found that the public evaluator still accepted
already-parsed checked authorities, so ordinary `json.loads` could collapse a
duplicate key before policy saw it. A raw-file trap was added first for both
`garnet-main.json` and `repository-settings.json`; both coordinated duplicate
keys reproduced the false green. The public evaluator now owns a bounded,
duplicate-rejecting regular-file loader and never accepts caller-parsed checked
authority objects.

## Fresh Item 1 GREEN

```text
$ python3 -I scripts/test_garnet_github_governance_transport.py
Ran 23 tests in 0.008s
OK

$ python3 -I scripts/test_garnet_github_link_headers.py
Ran 12 tests in 0.055s
OK

$ python3 -I scripts/test_garnet_github_governance_gate.py
Ran 18 tests in 0.121s
OK

$ /private/tmp/garnet-l1-policy-venv-20260717/bin/python -I scripts/test_garnet_required_context_contract.py
Ran 15 tests in 0.014s
OK

$ python3 -m py_compile scripts/garnet_github_governance_gate.py scripts/test_garnet_github_governance_gate.py
exit 0
```

## Integrated identity rerun after final semantic restamp

The earlier concurrent integration RED was preserved while action-pin and
base-controlled workflow edits changed the semantic projection. After the
workflow set was frozen, all 32 declarations were recomputed from the immutable
index, the active aggregate and Item 1 binding digest were restamped together,
and the integrated suites were rerun:

```text
$ /private/tmp/garnet-l1-policy-venv-20260717/bin/python -I scripts/test_garnet_required_context_contract.py
Ran 15 tests
OK

$ /private/tmp/garnet-l1-policy-venv-20260717/bin/python -I scripts/test_garnet_required_context_evaluator.py
Ran 13 tests
OK

$ /private/tmp/garnet-l1-policy-venv-20260717/bin/python -I scripts/test_garnet_workflow_identity_policy.py
Ran 6 tests
OK

$ /private/tmp/garnet-l1-policy-venv-20260717/bin/python -I scripts/test_garnet_github_governance_gate.py
Ran 18 tests
OK
```

The earlier RED remains useful regression evidence; it no longer describes the
final integrated tree. Live authenticated evidence and U-17 remain open.

## Local mechanism

`scripts/garnet_github_governance_gate.py` is pure and has no credential or
network CLI. It accepts only the existing transport's all-or-zero
`ObjectResult` and `CollectionResult` values. It type-checks the exact result,
`problems`, row-container, and row-element types and enforces the transport's
body, page, collection-byte, and collection-row maxima. It returns no bindings
on any problem.

The public entry point loads both checked authority documents itself. Each path
must remain beneath the repository root with no symlink/reparse component; the
final object must be a bounded regular file opened with no-follow semantics;
device/inode, size, and modification time must remain stable across the read;
bytes must be strict UTF-8; and JSON parsing rejects duplicate keys before the
type-exact canonical digest check. Missing, replaced, oversized, malformed, or
duplicate-key authorities are all RED.

The evaluator:

- binds `Island-Dev-Crew/garnet`, default branch `main`, and one explicit full
  lowercase reviewed SHA;
- selects the unique maximum `run_attempt` itself for every expected workflow
  at that exact SHA;
- hashes the ordered `(context, workflow, event, job, matrix)` producer
  projection and requires the exact checked-in preactivation contract digest
  `899944d4f0344e4b53cdd3cb37b1da26061f5eaab5d49d8482f8157b1ed51aaa`;
- separately pins the ordered declared semantic projection to
  `1b5eeb6bdc983c35073726494aee26bb5bc6d72384204297f367e411090b4ee1`,
  requires every observed semantic digest to equal its declaration, and pins
  the ordered full binding
  `(context, workflow, event, job, matrix, declared semantic, observed
  semantic, immutable workflow YAML name)` to
  `2c54b511d0b1509bbc33be545c8cc45bcf1fc924789d08a62658c7df1322c9bb`;
- freezes freshness to an injected second-precision UTC clock and a constant
  two-hour window, rejecting stale, future, noncanonical, or inverted times;
- requires selected workflow runs and required check runs to be
  `completed`/`success`;
- feeds only the selected cohort into the existing GOV-008 workflow/App
  identity evaluator;
- rejects duplicate workflow/run/suite/check and workflow-attempt domains,
  rejects every required-context name repeated anywhere in the complete check
  enumeration before selecting suites, and rejects missing/split
  latest-attempt contexts, wrong repository/head/event/App, partial or empty
  collections, malformed collection internals, and any transport problem;
- type-validates and canonically hashes the entire checked ruleset and
  repository-settings objects before they can serve as comparison authorities;
  the pinned SHA-256 values are respectively
  `46366962f5b11a1c150a7e76e5f2fd7d4bbfa6d1ba63d445f0f04184a6d74c6f`
  and `c4f0dd0025fb9e3edbd8a12e320da49151353e09a06038737955a4a268378e3a`;
- then compares ruleset `18936562` and every live ruleset field with the
  anchored ruleset, requires `bypass_actors: []`, and compares the selected
  repository and Actions settings with the anchored repository settings;
- rejects credential-like fields before evaluating or rendering evidence; and
- exposes only sanitized binding/status dataclasses. The status labels its
  authority `injected-offline` and the live settings/no-bypass clause
  `blocked-u17`.

## Digest update contract

Digest changes are reviewed policy changes, never runtime self-updates. The
update path is:

1. Finish all workflow/policy edits and independently review their W_TRUST
   companion.
2. Parse each checked JSON document with duplicate-key rejection. Require an
   exact `dict`/`list`/`str`/`int`/`bool`/`null` object tree, NFC strings, no
   non-printable strings, bounded depth/nodes/integers, and no tuples, floats,
   subclasses, or other Python objects.
3. Serialize the validated object as UTF-8 with
   `json.dumps(value, ensure_ascii=False, sort_keys=True,
   separators=(",", ":"), allow_nan=False)` and SHA-256 those exact bytes.
4. Evaluate the checked-in required-context contract against the immutable
   workflow projection. Hash the 31 ordered full binding tuples listed above
   with UTF-8 JSON using `ensure_ascii=False` and compact separators.
5. Update the pinned constants and their adversarial fixtures in the same
   trust-reviewed PR, then rerun transport, Link-header, governance, identity,
   required-context contract/evaluator, and compilation suites. Any mismatch
   remains RED.

The final restamp completed step 4 after the shared workflow tree was frozen.
Future workflow changes intentionally turn the integrated projection RED until
the same reviewed digest-update contract is repeated.

## Clause status

| GOV-009 clause | Current status |
|---|---|
| authenticated transport | Implemented and merged in #500; current run used no credential/live request |
| strict pagination | Implemented and merged in #502 |
| complete bounded collection | Implemented and merged in #506; the new evaluator rejects malformed problem/row containers and elements, empty results, invalid integer types, and page/byte/row counts outside transport bounds |
| fresh cohort | Local mechanism implemented and adversarially tested with injected data; live evidence pending |
| exact reviewed head | Local mechanism implemented and adversarially tested with injected data; live evidence pending |
| terminal successful outcomes | Local mechanism implemented and adversarially tested with injected data; live evidence pending |
| live settings and no-bypass readback | **BLOCKED — U-17** |

The claim therefore remains **partial**. This evidence does not promote the
three newly implemented mechanisms to authenticated live proof, and it does
not close the seventh clause. Jon must provision the dedicated
admin-authoritative token before a future explicit-token collector can produce
the live ruleset/settings/no-bypass evidence. The evaluator never reads
`GH_TOKEN`, `GITHUB_TOKEN`, `gh` configuration, or any ambient credential.

An independent read-only reviewer approved the final local/offline mechanism
after rerunning 18/18 evaluator tests, 23/23 transport tests, compilation, and
direct symlink, reparse, invalid-UTF-8, oversize, identity-substitution,
read-mutation, and ambient-credential probes. That approval explicitly does not
close U-17 or promote local injected evidence to authenticated live proof.
