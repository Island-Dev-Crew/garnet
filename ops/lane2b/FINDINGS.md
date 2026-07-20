# Lane 2B findings register

Current at: 2026-07-20T22:45:03Z

Lane 2B is complete. The following cross-cutting findings are preserved and
assigned to Lane 0 repair #3; none reopens the accepted bounded Shelf/MCP path.

| ID | Finding | Source | Status | Required Lane 0 repair #3 outcome |
| --- | --- | --- | --- | --- |
| U-23 | W-PLAY proof/provenance artifacts do not all inherit byte-exact `.gitattributes` protection. | PR #515 evidence `ops/lane1/evidence/85-u23-wplay-byte-fidelity-windows11-arm64-2026-07-17.md` | OPEN / DEFERRED | Extend additive byte fencing to the affected proof/provenance paths and prove object-store/worktree byte equality on Windows and Unix. |
| U-24 | Windows-reserved device names in ref path segments can make an otherwise valid repository uncheckoutable on Windows. | PR #515 governance findings; upstream `aux` ref renamed, fleet fork retains cleanup inputs | OPEN / DEFERRED | Add a fail-closed ref-name policy for `aux`, `con`, `nul`, `prn`, `com*`, and `lpt*` path segments and complete fork cleanup without concealing the original evidence. |
| U-25 | Newline byte discipline is not globally uniform: an inventory found committed CRLF files, and Verdict 05 proved the Shelf reporter's text-mode stdout terminator is CRLF on Windows and LF on POSIX. | PR #515 CRLF inventory plus `ops/lane2b/review/05-verdict.md` F1 NOTE | OPEN / DEFERRED | Decide and enforce the byte-exactness policy, retain `core.autocrlf=false` lane-boot discipline where required, and use binary stdout only if cross-platform console-stream bytes become a claimed contract. The sanctioned WV-6 artifact writer is already binary and stable. |
| U-26 | Three modules pin overlapping required-context fingerprints without one sanctioned atomic writer. | PR #515 governance findings | OPEN / DEFERRED | Provide one sanctioned regenerator that updates all required fingerprint bindings together and fails closed on partial updates. |
| U-27 | The dogfood validator consumes the PR-body snapshot from the triggering `pull_request` event, so a body-only correction does not refresh the existing run. | PR #514 dogfood RED and body-only cure | OPEN / DEFERRED | Trigger on `pull_request.edited` or fetch and bind the live PR body; retain strict dogfood evidence parsing. |

## Lane-local resolutions

- Verdict 05 is APPROVE at reviewed head `927ad221d33668d458499a26f49d96ed4586563d` / tree `4d9374991bf265b78a0108e0bb62c317a43b8028`.
- PR #514 is squash-merged at `41d6ced858684ac67683d32315920bd50a52976e` / tree `e3c914b881ae59ca96d8950190729665e45808db`.
- A fresh Windows main-only clone reports Shelf accepted and WV-6 accepted 5/5 with no findings and no pull refs.
- #514 landed-marker registration is deliberately deferred to the first post-#515 truth reconciliation.
