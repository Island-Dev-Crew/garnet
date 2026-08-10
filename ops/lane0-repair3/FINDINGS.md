# Lane 0 Repair 3 Findings Register

This register preserves the exact scope used before a cure. The machine-readable
path enumeration is `U25_SCOPE.json`; prose or a carried count must not replace
that exact-tree record.

| ID | State | Exact-tree scope | Disposition |
|---|---|---|---|
| U-25 | IMPLEMENTED — REVIEW REQUIRED | RED scope: `efd4f6bae8b3afaba74594e57944b2548142aeae` / tree `e9bce10421c1eac2a514291212b87d61a5289037`; ten CR-bearing text paths enumerated in `U25_SCOPE.json`. GREEN implementation: `be89de21684c84bb0aaae8382906864a48637d20` / tree `1faeed401996985d9b6d8412225ab706783c3355`. | Six sealed `proofs/**` paths are closed with no byte change. Three CRLF assets are LF-normalized and explicitly pinned. The Lane 2B generator rejects CR-bearing inputs/output and regenerated clean HTML. Independent review remains required. |
| U-45 | PROPOSED | This lane's touched findings must name paths at a full commit and tree before implementation. | Replace count-pinned finding scopes with exact-tree path enumerations. New checkers report violating paths and do not assert a repository-wide total. |
| U-47 | PROPOSED — DEFERRED | Exact integration head `efd4f6bae8b3afaba74594e57944b2548142aeae` / tree `e9bce10421c1eac2a514291212b87d61a5289037`: the isolated Python battery has two standing failures named in evidence 10. | Lane address: Lane 3 integration-baseline reconciliation. The adoption-surface and release-assets reds pre-exist this branch and are outside Slice 5. First recorded in Verdict 01 (`d99b1ca`) at 05:09 CDT, then inherited and confirmed by this implementer seat's Rust/Cargo 1.95.0 battery run; registration and Lane 3 routing are this seat's contribution. No cure here. |
| U-48 | PROPOSED — DEFERRED | Reviewer Air default resolves to Rust/Cargo 1.94.1; implementer Mac default resolves to Rust/Cargo 1.95.0. The same battery consequently has three additional focused-cargo failures on the Air default, all green under `+1.95.0`. | Lane address: Lane 3 procedural/gate hardening. Every battery record carries resolved toolchains; material cargo gates should pin `+1.95.0` rather than inherit a seat default. No workflow or gate command is changed here. |
| U-49 | PROPOSED — DEFERRED | At both `efd4f6b` and `5e5a24c`, `ops/lane2b/render-sotu.mjs` contains a double-encoded em dash in the mission-complete cold-start string; generated `ops/lane2b/state-of-the-union.html` carries it. | Lane address: Lane 3 ops/documentation sweep. Fix the producer and regenerate there. This slice does not edit either path. This was initially misregistered as U-46; Lane 2C owns U-46 by earlier registration and reviewed priority. |
| U-50 | PROPOSED — DEFERRED | Collision reproduced between `ops/lane2c/PROPOSED-DOCTRINE.md` at fork head `1bc64c4061250531b12d08007553d5db0f4b2d98` and this register at prior request tip `2564b17c92ce05a19bd04fba94ddabfb10b0169f`: both assigned unrelated U-46 findings. Fifteen current `mission/*` fork heads were checked; U-49 and U-50 were otherwise free. | Lane address: Lane 3 governance surface. Found by the chat seat this round. Cure with one reserved-range allocator file, or lane-namespaced provisional IDs until merge assigns a global number. Until then, a lane opening an ID first greps all open fork branches. |
| U-51 | PROPOSED — DEFERRED | At reviewed head `608bae46bb4f554c7ffd455f01ef4cbf44faee3c` / tree `7913546b516d6ef9144bdc62c02eddf9c209055f`, the same-family Codex audit seat demonstrated that `GIT_ATTR_SOURCE` does not neutralize `.git/info/attributes` or `core.attributesFile`; either can produce an exact-commit false green. The boundary also lacks a Git 2.40 minimum, so older Git can silently ignore `GIT_ATTR_SOURCE`, and committed diff-family/binary attributes remain authoritative for the scanned commit. | Lane address: a successor slice in this lane after Slice 13 or beside it. Found by the Codex audit seat, same-family. The successor must adjudicate the ambient-attribute channels, the Git version floor, and the committed-attribute boundary. This records the residuals without widening F1 or moving the reviewed checker. A scan of 460 non-main fork heads found no other U-51 assignment. |

U-25's earlier “eleven files” premise was wrong at authorship. It is superseded
by the exact-tree enumeration above; there is no eleven-file target to recover.

## Review 01 blocker and findings disposition

- **B1 — DISCLOSED; SUCCESSOR REQUIRED.** At reviewed head `5e5a24c`, the
  product pair is `cd9c080f…/1553`, not pinned `ea38d354…/1544`, and the
  `.gitattributes` hash is `b2a14050…`, not pinned `b8b22a96…`. The exact 14
  digest-included paths are enumerated in evidence 07. The F1 head moves the
  pair again to `20830394…/1555`. Shelf and WV-6 failures are expected and
  fail closed. No pin is rebound. The named successor is the post-record
  freeze/rebind followed by NUC WV-6 re-acceptance, each under its own review.
- **F1 — IMPLEMENTED; RE-REVIEW REQUIRED.** Fixture RED is commit `cbcf0a1`
  / tree `ea4f981`; `GIT_ATTR_SOURCE=<resolved-commit>` GREEN is `9d6baef` /
  tree `7df00be`. Because the checker changed after Verdict 01, the checker
  itself must be audited again.
- **F2 — CURED.** The literal `ops/.../evidence` file case now follows the
  existing evidence fence. This was a safe-direction divergence and the cure
  changed only the checker and its existing fixture suite.
- **F3 — DEFERRED AS PROPOSED U-49.** Pre-existing renderer mojibake is routed
  to Lane 3; no renderer or generated HTML change is made here.
- **F4 — RECORDED, NO BYTE CHANGE.** Three sealed paths retain inert `eol: lf`
  while `text` is unset. The CRLF `MANIFEST.sha256` must have CR stripped from
  the verification stream only before `shasum -c`; the sealed file itself is
  not normalized.
- **F5 — RECORDED, NO REWRITE.** Evidence 05/06 say `ref: HEAD`, but their
  commit/tree fields bind the actual revisions. Historical evidence remains
  unchanged.
