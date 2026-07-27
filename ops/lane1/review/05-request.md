# Lane 1 · Phase 0 — Review Request 05 (ceremony-authorized WV pin rebind)

- Date: 2026-07-27 (UTC ~17:15Z)
- Implementer: Claude Code Fable 5 — same machine, same fresh `autocrlf=false` clone
- Independent reviewer sought: Codex GPT-5.6 Sol
- Basis: verdict 04 (`a33a76b`) APPROVED exact head `72ae024`; the ceremony seat then authorized a **scoped four-constant rebind** (ad1e009a precedent) because the WV reporter's pins still bound the Lane 2B candidate, making slice 4 mechanically unreachable at any newer head.

## Candidate

| field | value |
|-------|-------|
| **rebind head** | `f1ec5699703688037c3b6320c630ff160379cf48` |
| **tree** | `b35a775dd4a33e94d861e4e6636f11933138c11f` |
| parent | `a33a76b` (verdict 04) — linear successor; no amend/rebase/Update branch |
| **product_content_sha256** | `5d3e7f727b56dbeb22d570c0b61a3b45b5e0b8df4e3b6305be896f9b5ed42b43` |
| product path count | 1581 |
| self-consistency | digest recomputed **at the committed head** == pinned EXPECTED value ✓ |

The digest differs from `72ae024`'s `99c3f270…/1578` **by design**: the three review commits above the approved content head (`04-request`, U-33 addendum, `04-verdict`) live under digest-included `ops/lane1/`. The values were **recomputed, not transcribed** — transcribing `99c3f270` from the prior round would have produced a reporter that can never accept. The rebind commit itself moves the digest by zero (both touched paths are digest-excluded), proven by the self-consistency check above.

## Semantics — ruled by the seat, verified at the source by the implementer

| constant | usage sites | semantics | action |
|----------|-------------|-----------|--------|
| `REVIEWED_TREE_PRODUCT_SHA256` (`1e669217…`) | lines 84, 309 | emitted as reportorial provenance; **never compared** | **byte-identical** |
| `REVIEWED_TREE_PATH_COUNT` (1527) | lines 84 (dataclass), 310 | same | **byte-identical** |
| `REVIEWED_HEAD` / `REVIEWED_TREE` | 381-382 (provenance args, format-validated), 553-554 (manifest template), wv 241/247 (manifest comparison) | review-boundary pins | → `72ae0246…` / `3c98ba05…` |
| `EXPECTED_PRODUCT_CONTENT_SHA256` / `EXPECTED_PRODUCT_PATH_COUNT` | 374/376 (**live-tree comparison**), 383, 311-312/555 | live content binding | → `5d3e7f72…` / `1581` (recomputed) |

Exactly **four** values changed in the reporter; zero logic; comments untouched; `git diff` appended below for byte-level verification.

## PROOF.json — agreement requirement proven, then satisfied

Per the instruction's conditional ("if and only if the reporter requires agreement"), the reporter was run **before** touching `PROOF.json`. It emitted exactly four requiring findings:

```
proof reviewedHead is not canonical
proof reviewedTree is not canonical
proof productContentSha256 is not canonical
proof productPathCount is not canonical
```

**Answer to the mandated report: yes, the update was needed — for exactly those four mirror fields** (`_validate_proof` compares each against the rebound constants). They were updated by surgical line replacement preserving all other bytes. `reviewedTreeProductSha256` / `reviewedTreePathCount` were **not flagged** (they compare against the unchanged historical constants) and are untouched, honoring "leave its reviewedTree* fields untouched" in the only reading consistent with the reporter going green.

## Post-rebind gate outputs

```
smoke_garnet_minimum_shelf:  ok=true · 5/5 checks (core-ring-tier1, mcp-raw-byte-stdio,
                             sealed-baseline, reject-without-seal, deterministic-shelf-reporter)
                             · findings=[] · --gate exit 0 · landed_main_commit=None (topic branch, correct)
garnet_wv_acceptance --wv WV-6:  partial · 5/5 checks · four findings, all expected:
                             reviewedHeadSha / reviewedTreeSha / productContentSha256 mismatch
                             (old Windows manifest still binds dcf6008f/2f8c9ad8) + live digest
                             5d3e7f72 != manifest 2f8c9ad8 — cured only by the NUC regenerating
                             evidence at the approved head. Partial here is correct, not a failure.
test_garnet_minimum_shelf_provenance:  OK
test_garnet_wv_acceptance_status:      5/6 — sole failure is the STANDING freeze red
                             (test_current_repository_tracks_wv6_acceptance_and_wv7_pending,
                             'partial' != 'accepted') charged in verdicts 02/04; the rebind adds
                             NO new failure.
```

## U-29 scope fold — registered (per the seat: fold, do not open a new lane)

> WV acceptance requires a reviewed constant-rebind in
> `scripts/smoke_garnet_minimum_shelf.py` for every new candidate, so each
> reconciliation costs a rebind round plus a native-platform run. The U-29
> redesign must remove this treadmill, not just the digest-drift symptom:
> acceptance should bind the landed commit/tree it was proven at, with drift
> reported separately, and candidate pins derived rather than hand-pinned.

## Stop

Implementer STOPS for the verdict. **The NUC reruns slice 4 at whatever head that verdict approves.** Record law untouched. U-31 (slice 5), U-32 (own lane), U-33 (Lane 3) all carry unchanged.

## Appendix — exact rebind diff (`git show f1ec569`)

```diff
diff --git a/proofs/minimum-shelf/lane2b/PROOF.json b/proofs/minimum-shelf/lane2b/PROOF.json
index 75c9a8b..17546ab 100644
--- a/proofs/minimum-shelf/lane2b/PROOF.json
+++ b/proofs/minimum-shelf/lane2b/PROOF.json
@@ -1,11 +1,11 @@
 {
   "schema": "garnet.minimum-shelf-proof/v2",
-  "reviewedHead": "dcf6008fd4291baf719dc361a82f2062ea60bfd2",
-  "reviewedTree": "f3272b9610dba756bd414cafc825fd7462d7a294",
+  "reviewedHead": "72ae0246fb448ce33d689b1b80eb783497a7f215",
+  "reviewedTree": "3c98ba05eb756377049325942842164f5d98910b",
   "reviewedTreeProductSha256": "1e6692175ea8fe2dd5b04fad4a492dc8ce48767dd07d88fd11a0847ce96749d5",
   "reviewedTreePathCount": 1527,
-  "productContentSha256": "2f8c9ad860bd9c6dbd1e005b0c82af0288dd3a736bb416a3978708c12e6fa1fd",
-  "productPathCount": 1571,
+  "productContentSha256": "5d3e7f727b56dbeb22d570c0b61a3b45b5e0b8df4e3b6305be896f9b5ed42b43",
+  "productPathCount": 1581,
   "platform": "windows",
   "implementer": "Codex GPT-5.6 Sol",
   "reviewer": "Claude Code Fable 5",
diff --git a/scripts/smoke_garnet_minimum_shelf.py b/scripts/smoke_garnet_minimum_shelf.py
index a4744bd..69376a4 100644
--- a/scripts/smoke_garnet_minimum_shelf.py
+++ b/scripts/smoke_garnet_minimum_shelf.py
@@ -21,8 +21,8 @@ WV6_ROOT = Path("proofs/windows/launch-verification/wv6-minimum-shelf")
 CROSS_CHECKOUT_EVIDENCE = Path(
     "ops/lane2b/evidence/17-content-reporter-cross-checkout.txt"
 )
-REVIEWED_HEAD = "dcf6008fd4291baf719dc361a82f2062ea60bfd2"
-REVIEWED_TREE = "f3272b9610dba756bd414cafc825fd7462d7a294"
+REVIEWED_HEAD = "72ae0246fb448ce33d689b1b80eb783497a7f215"
+REVIEWED_TREE = "3c98ba05eb756377049325942842164f5d98910b"
 REVIEWED_TREE_PRODUCT_SHA256 = (
     "1e6692175ea8fe2dd5b04fad4a492dc8ce48767dd07d88fd11a0847ce96749d5"
 )
@@ -30,9 +30,9 @@ REVIEWED_TREE_PATH_COUNT = 1527
 # Replaced after all Verdict-04-authorized product paths are staged. This
 # reporter path is itself excluded, so the final constant is not self-referential.
 EXPECTED_PRODUCT_CONTENT_SHA256 = (
-    "2f8c9ad860bd9c6dbd1e005b0c82af0288dd3a736bb416a3978708c12e6fa1fd"
+    "5d3e7f727b56dbeb22d570c0b61a3b45b5e0b8df4e3b6305be896f9b5ed42b43"
 )
-EXPECTED_PRODUCT_PATH_COUNT = 1571
+EXPECTED_PRODUCT_PATH_COUNT = 1581
 MAX_JSON_BYTES = 64 * 1024
 MAX_HEX_BYTES = 4 * 1024 * 1024
 SHA_RE = re.compile(r"^[0-9a-f]{40}$")
```
