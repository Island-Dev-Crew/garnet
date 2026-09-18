# T-L shadow lanes: a local policy observation, never merge approval

Status: opt-in prototype. No CI wiring, remote labels, signing, settings change,
or autonomous merge. This is an agent-harness policy; it changes no language
semantics and leaves existing human approval/record requirements intact.

## Interface and custody

```
python3 -I scripts/garnet_shadow_lanes.py --repo . --base <full-commit> --head <full-commit> [--garnet /absolute/trusted/garnet]
```

Both inputs are full lowercase commit IDs, with base an ancestor of head.
Committed Git blobs and trees are read with replacement objects disabled;
dirty working-tree bytes do not enter the result. Output is deterministic
`garnet.shadow-lanes/1` JSON with commit/tree IDs, changed paths, classifier
SHA-256, and `mode: shadow`, `merge_authorized: false` in every outcome.

A changed symlink, executable-mode blob or submodule is blocked. Unknown paths
require human review. Only regular Markdown prose under `docs/internals/`,
excluding agent instruction files, may receive `candidate-fast`; this means
eligible for downstream policy checks, not approved or safe. Prose can still
contain harmful instructions or false claims and needs the existing review.

A Garnet source change requires an explicit trusted local compiler binary.
The tool records its SHA-256, extracts source/config blobs into temporary
directories, invokes parse/check via `verify`, and invokes `diff-caps --machine`.
Portable case/Unicode path collisions and nonportable path components are
rejected before extraction. A root manifest sentinel prevents ambient parent
configuration from changing the check.
It does not run candidate programs, candidate tests, build scripts or workflows.
The caller owns binary selection; a hash records bytes, not origin or trust.
A malicious selected binary remains outside this tool's protection.

Missing/invalid source, incomplete diff walks, unsupported machine schemas,
duplicate JSON keys and contradictory verdicts block classification. The current
diff gates aggregate gains or a new wildcard. A per-function redistribution is
reported by the diff but is not relabeled as aggregate authority expansion.

Even when both trees check and declarations do not widen, Garnet changes are
`careful`: the current checker does not establish complete call-shape coverage
or the ADR 0013 ownership/mode boundary. The result explicitly requires human
review. An existing fs/net capability is not a license for arbitrary effects.

Exit 0 = candidate-fast observation; exit 1 = careful; exit 2 = blocked.
None is a hosting-platform approval and none may replace a required check.
This tool uses bounded subprocess timeouts and caps accepted output and source
snapshot size. It is not a sandbox for the compiler or a cryptographic notary.

## Before operational routing

A future implementation needs independently authenticated test/check evidence,
complete supported-surface coverage or conservative exclusion, exact base/head
freshness, an owner-approved policy envelope and external enforcement that the
PR cannot rewrite. Signing that envelope is separate work: this prototype emits
no key, signature or apparent signed approval. Unsupported mode-boundary cases
stay careful until the relevant ADR 0013 proof exists. Changes to the trust
kernel remain human-reviewed regardless of a capability verdict.

## Executable evidence

`python3 -I scripts/test_garnet_shadow_lanes.py` covers exact-commit custody,
changed links, unknown paths, malformed/duplicate verdicts, missing binaries,
and the never-authorize invariant. Real-compiler source journeys must accompany
review evidence; unit fixtures alone are not proof of integration.
