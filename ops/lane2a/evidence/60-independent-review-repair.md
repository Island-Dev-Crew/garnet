# Lane 2A independent review repair

Reviewed candidate: `93caedea5b01254184e2406349b64a9099c1f23e`

Reviewed tree: `8ee56395f308f5bc24b814a29f744596809a430c`

Review mode: two independent, read-only final-diff passes; neither trusted the lane prose.

## Findings

1. **HIGH - stale browser proof replay.** The reporter checked the shape of
   `git.tested_commit` and `git.tested_tree` but did not bind the proof to the
   browser inputs. A proof from ancestor `743b564` still promoted the later
   reviewed tree.
2. **MEDIUM - ambient Playwright resolution.** The harness imported a concrete
   file under local `node_modules` without proving that the runtime came from
   the direct, integrity-locked Studio dependency and documented install flow.

The first finding was independently reproduced by both reviewers. One reviewer
mutated the commit/tree labels to other valid hexadecimal values and observed
`browser_proof_valid=True`; the other confirmed the committed proof and reviewed
HEAD had different trees while the gate stayed green.

## RED

After the repaired validator and corruption tests were added, the old proof
failed exactly as required:

- `python3 -I scripts/test_garnet_wasm_readiness.py`: RED because the old proof
  lacked `runtime_inputs` and locked toolchain identity.
- `python3 scripts/garnet_wasm_readiness.py --gate`: exit 1 with
  `browser_proof_valid: false`, `browser_ready: false`.

## Repair

Commit `2f1d17775b6db1d5807fc6c2a85eb853cca74449`:

- replaces ancestry as proof authority with
  `garnet.w-play.runtime-inputs/1`, an exact digest over ten tracked files;
- includes the page, adapter, examples, icon, package bytes/provenance, smoke
  harness, Studio package manifest, and Studio lockfile;
- resolves direct `@playwright/test` only through the Studio package context,
  verifies the installed version against the lock, and rejects resolution
  outside the Studio `npm ci` tree;
- records the dependency spec, locked version/integrity, manifest hashes, and
  `npm ci --ignore-scripts` reproduction command;
- adds stale-digest, changed-file, locked-dependency, and tracked-input tests.

## Fresh GREEN

- Clean Chromium proof: PASS in 2,637 ms.
- Runtime-input digest:
  `e21134ae261f064ccb9db42a1d4150b2375fbb8c9146539be6439f3c56f75f70`.
- Browser requests: six committed, zero external, zero untracked.
- `python3 -I scripts/test_garnet_playground_browser_contract.py`: 6/6.
- `python3 -I scripts/test_garnet_playground_browser_proof.py`: 3/3.
- `python3 -I scripts/test_garnet_wasm_readiness.py`: 13/13.
- `python3 scripts/garnet_wasm_readiness.py --gate`: browser package, proof,
  and readiness true; blockers empty.

This repair does not relabel S114, WV-4, or WV-5 and does not advance the
canonical launch denominators. Final repaired-tree rereview and remote required
checks remain separate gates.
