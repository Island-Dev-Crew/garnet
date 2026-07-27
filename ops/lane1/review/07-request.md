# Lane 1 · Phase 0 — Review Request 07 (U-35 cure: ops/lane1/ digest exclusion)

- Date: 2026-07-27 (UTC ~23:35Z)
- Implementer: Claude Code Opus 4.8 — same machine, same fresh `autocrlf=false` clone
- Independent reviewer sought: Codex GPT-5.6 Sol — delta review of the implemented U-35 cure
- Authorization: verdict 05 (`db6ab65`) — U-35 AUTHORIZE-WITH-CONSTRAINTS, exactly `b"ops/lane1/"`, five items, five traps with (c) corrected and (e) added
- RED recorded first: `ops/lane1/evidence/92-u35-cure-red.md` (`2f2377d`) — the three traps fail before the exclusion

## Cure candidate

| field | value |
|-------|-------|
| **cure head** | `7ad43855115103fdf2c08dddcb21cd6fd001334e` |
| **cure tree** | `ad4335a036578e6e0e1d3577614091d88a261cef` |
| parent | `2f2377d` (RED) ← `db6ab65` (verdict 05) — linear; no amend/rebase/Update branch |
| **product_content_sha256** | `e89cb2996def7eec8e419dac235d55f985734e84be6f991c565da35d46feb64f` |
| product path count | **1544** (was 1578 at the last included-set count; the 40 `ops/lane1/` paths now leave the set) |

The pin was **RECOMPUTED at the cured tree**, not transcribed. The reviewer's `494e4837…/1544` diagnostic omitted the included byte changes (the `+b"ops/lane1/"` line and the three new traps in `test_garnet_minimum_shelf_provenance.py`); those move the digest to `e89cb299…` while the count stays 1544 (existing files modified, no new included paths).

## Authorized scope — exactly five, nothing more

| # | surface | change |
|---|---------|--------|
| 1 | `scripts/garnet_content_provenance.py` | append the single line `b"ops/lane1/"` to `FROZEN_MUTABLE_PREFIXES` — literal prefix only, general predicate DENIED |
| 2 | `scripts/test_garnet_minimum_shelf_provenance.py` | +3 focused traps in the existing surface (see below) |
| 3 | `scripts/smoke_garnet_minimum_shelf.py` | ONLY `EXPECTED_PRODUCT_CONTENT_SHA256` → `e89cb299…` and `EXPECTED_PRODUCT_PATH_COUNT` → `1544` |
| 4 | `proofs/minimum-shelf/lane2b/PROOF.json` | ONLY `productContentSha256` / `productPathCount` mirrors |
| 5 | `ops/lane1/**` | RED evidence, this request, BLOCKED/journal heartbeat |

**Untouched (per authorization):** reporter logic; `REVIEWED_HEAD` (`72ae024…`); `REVIEWED_TREE` (`3c98ba05…`); `REVIEWED_TREE_PRODUCT_SHA256` (`1e669217…`); `REVIEWED_TREE_PATH_COUNT` (1527); both `reviewedTree*` PROOF fields; every other exclusion; the reporter self-path; all workflows/rulesets. Diff-stat of the cure commit: 4 files, 61 insertions, 4 deletions.

## Five traps — all required, (c) as corrected, (e) added

- **Trap (a)** — `test_included_product_change_moves_while_lane1_does_not`: an `ops/lane1/`-only change moves neither digest nor verifier; a `product.txt` blob change moves the digest AND trips `_verify_product_content`. **PASS.**
- **Trap (b)** — `test_lane1_review_artifacts_do_not_move_the_digest`: adding/modifying only `ops/lane1/` artifacts leaves digest and count byte-identical. **PASS.**
- **Trap (c)** — CORRECTED (compare the final cure candidate against a LATER review-request tip, NOT `f1ec569`): recomputed below; the review-artifact commit carrying this request must have the same digest/count as the cure head. **Reported at the bottom.**
- **Trap (d)** — STRENGTHENED — `test_frozen_mutable_prefixes_are_exactly_the_authorized_set`: asserts the tuple is exactly the three historical prefixes + `b"ops/lane1/"`, reporter self-path unchanged, `ops/lane1/…` excluded while `ops/lane3/…` is NOT (a general predicate fails this). **PASS.**
- **Trap (e)** — ADDITIONAL — recompute at the exact committed cure tree; new reporter constants and PROOF mirrors equal the recomputed pair; the old `5d3e7f72…/1581` pair FAILS; the Shelf gate is green; both historical `reviewedTree*` values byte-identical. **All PASS** (outputs below).

```
Trap tests (pre-cure: 3 FAIL, recorded in evidence/92; post-cure):
test_frozen_mutable_prefixes_are_exactly_the_authorized_set (__main__.SquashDurableContentProvenanceTests.test_frozen_mutable_prefixes_are_exactly_the_authorized_set) ... ok
test_included_product_change_moves_while_lane1_does_not (__main__.SquashDurableContentProvenanceTests.test_included_product_change_moves_while_lane1_does_not) ... ok
test_lane1_review_artifacts_do_not_move_the_digest (__main__.SquashDurableContentProvenanceTests.test_lane1_review_artifacts_do_not_move_the_digest) ... ok

Trap (e) at committed cure head 7ad4385:
  recomputed pair          e89cb299…/1544
  reporter EXPECTED == pair True
  PROOF mirrors == pair     True
  OLD 5d3e7f72/1581 FAILS   True
  historical 1e669217/1527  reporter True / proof True
  exclusion tuple exact     True
  reporter self-path intact True
  shelf --gate              exit 0 (accepted, 5/5, findings=[])
```

## Post-cure gate outputs

```
provenance suite (test_garnet_minimum_shelf_provenance):  6/6 OK
shelf reporter:  ok=true, state accepted, 5/5 checks, findings=[], e89cb299…/1544, --gate exit 0
WV-6:            partial, 5/5 checks, 4 expected stale-native-manifest findings (cured only by the NUC)
WV suite:        5/6 — sole failure is the standing partial!=accepted freeze red (charged in verdicts 02/04/05)
git diff --check (cure commit):  clean
```

## Sequencing (mandatory) — satisfied

The exclusion and the rederived pair are in the SAME commit (`7ad4385`). There is no intermediate red-reporter window: the Shelf gate is green at the cure head. The RED (`2f2377d`) precedes the cure; the request/heartbeat follow it as excluded artifacts.

## Stop

Implementer STOPS for the verdict. **No NUC head is claimed** — only a subsequent independent verdict may name one, and it runs at the head that verdict approves. Record law untouched. U-31 (slice 5), U-32 (own lane), U-33 (Lane 3) carry unchanged.

## Appendix A — exact freeze facts (filled at request-commit time)
- cure head : `7ad43855115103fdf2c08dddcb21cd6fd001334e`
- cure tree : `ad4335a036578e6e0e1d3577614091d88a261cef`
- product digest : `e89cb2996def7eec8e419dac235d55f985734e84be6f991c565da35d46feb64f`
- product path count : `1544`
- old superseded pin : `5d3e7f72…/1581` (request 05, now unusable — provenance predicate changed)

## Appendix B — exact cure diff (`git show 7ad4385`)

```diff
diff --git a/proofs/minimum-shelf/lane2b/PROOF.json b/proofs/minimum-shelf/lane2b/PROOF.json
index 17546ab..59c458b 100644
--- a/proofs/minimum-shelf/lane2b/PROOF.json
+++ b/proofs/minimum-shelf/lane2b/PROOF.json
@@ -4,8 +4,8 @@
   "reviewedTree": "3c98ba05eb756377049325942842164f5d98910b",
   "reviewedTreeProductSha256": "1e6692175ea8fe2dd5b04fad4a492dc8ce48767dd07d88fd11a0847ce96749d5",
   "reviewedTreePathCount": 1527,
-  "productContentSha256": "5d3e7f727b56dbeb22d570c0b61a3b45b5e0b8df4e3b6305be896f9b5ed42b43",
-  "productPathCount": 1581,
+  "productContentSha256": "e89cb2996def7eec8e419dac235d55f985734e84be6f991c565da35d46feb64f",
+  "productPathCount": 1544,
   "platform": "windows",
   "implementer": "Codex GPT-5.6 Sol",
   "reviewer": "Claude Code Fable 5",
diff --git a/scripts/garnet_content_provenance.py b/scripts/garnet_content_provenance.py
index dffa7f7..ed787ed 100644
--- a/scripts/garnet_content_provenance.py
+++ b/scripts/garnet_content_provenance.py
@@ -24,6 +24,7 @@ FROZEN_MUTABLE_PREFIXES = (
     b"ops/lane2b/",
     b"proofs/",
     b"F_Project_Management/W_TRUST/",
+    b"ops/lane1/",
 )
 REPORTER_PATH = b"scripts/smoke_garnet_minimum_shelf.py"
 GIT_OID_RE = re.compile(rb"^[0-9a-f]{40}$")
diff --git a/scripts/smoke_garnet_minimum_shelf.py b/scripts/smoke_garnet_minimum_shelf.py
index 69376a4..b1bf932 100644
--- a/scripts/smoke_garnet_minimum_shelf.py
+++ b/scripts/smoke_garnet_minimum_shelf.py
@@ -30,9 +30,9 @@ REVIEWED_TREE_PATH_COUNT = 1527
 # Replaced after all Verdict-04-authorized product paths are staged. This
 # reporter path is itself excluded, so the final constant is not self-referential.
 EXPECTED_PRODUCT_CONTENT_SHA256 = (
-    "5d3e7f727b56dbeb22d570c0b61a3b45b5e0b8df4e3b6305be896f9b5ed42b43"
+    "e89cb2996def7eec8e419dac235d55f985734e84be6f991c565da35d46feb64f"
 )
-EXPECTED_PRODUCT_PATH_COUNT = 1581
+EXPECTED_PRODUCT_PATH_COUNT = 1544
 MAX_JSON_BYTES = 64 * 1024
 MAX_HEX_BYTES = 4 * 1024 * 1024
 SHA_RE = re.compile(r"^[0-9a-f]{40}$")
diff --git a/scripts/test_garnet_minimum_shelf_provenance.py b/scripts/test_garnet_minimum_shelf_provenance.py
index 451c4fa..896db32 100644
--- a/scripts/test_garnet_minimum_shelf_provenance.py
+++ b/scripts/test_garnet_minimum_shelf_provenance.py
@@ -22,6 +22,7 @@ def _load(name: str, filename: str):
 
 shelf = _load("smoke_garnet_minimum_shelf", "smoke_garnet_minimum_shelf.py")
 wv = _load("garnet_wv_acceptance_status", "garnet_wv_acceptance_status.py")
+cp = _load("garnet_content_provenance", "garnet_content_provenance.py")
 
 
 class SquashDurableContentProvenanceTests(unittest.TestCase):
@@ -112,6 +113,61 @@ class SquashDurableContentProvenanceTests(unittest.TestCase):
             any("product content digest" in item for item in findings), findings
         )
 
+    # ------------------------------------------------------------------ U-35
+    # The frozen construction must treat Lane 1's own review/operational
+    # artifacts as non-product (like ops/lane2b/), so a review round cannot move
+    # the product digest and invalidate the WV pin at merge.
+
+    def test_frozen_mutable_prefixes_are_exactly_the_authorized_set(self) -> None:
+        # Trap (d): the exclusion tuple is exactly the three historical prefixes
+        # plus the single authorized Lane 1 prefix — never a general predicate.
+        self.assertEqual(
+            cp.FROZEN_MUTABLE_PREFIXES,
+            (
+                b"ops/lane2b/",
+                b"proofs/",
+                b"F_Project_Management/W_TRUST/",
+                b"ops/lane1/",
+            ),
+        )
+        self.assertEqual(cp.REPORTER_PATH, b"scripts/smoke_garnet_minimum_shelf.py")
+        # Lane 1 is excluded; a sibling lane namespace is NOT (a general
+        # ops/<lane>/ predicate would wrongly exclude ops/lane3/ and fail here).
+        self.assertTrue(cp._is_mutable(b"ops/lane1/review/07-request.md"))
+        self.assertFalse(cp._is_mutable(b"ops/lane3/note.txt"))
+
+    def test_lane1_review_artifacts_do_not_move_the_digest(self) -> None:
+        # Trap (b): adding and modifying only ops/lane1/ artifacts leaves the
+        # digest and count byte-identical.
+        before, count_before = cp.tracked_content_digest(self.root)
+        self._write("ops/lane1/review/99-later-request.md", "later review artifact\n")
+        self._write("ops/lane1/journal.md", "heartbeat line\n")
+        self._write("ops/lane1/evidence/zz.txt", "evidence\n")
+        self._git("add", ".")
+        after, count_after = cp.tracked_content_digest(self.root)
+        self.assertEqual(before, after)
+        self.assertEqual(count_before, count_after)
+
+    def test_included_product_change_moves_while_lane1_does_not(self) -> None:
+        # Trap (a): the crux pair. A product blob change moves the digest AND
+        # trips the content verifier; an ops/lane1/-only change does neither.
+        baseline, _ = cp.tracked_content_digest(self.root)
+        self._write("ops/lane1/evidence/only-lane1.txt", "lane1 only\n")
+        self._git("add", ".")
+        lane1_digest, _ = cp.tracked_content_digest(self.root)
+        self.assertEqual(baseline, lane1_digest)
+        self.assertEqual([], shelf._verify_product_content(self.root, baseline))
+        self._write("product.txt", "tampered product\n")
+        self._git("add", ".")
+        product_digest, _ = cp.tracked_content_digest(self.root)
+        self.assertNotEqual(baseline, product_digest)
+        self.assertTrue(
+            any(
+                "product content digest" in item
+                for item in shelf._verify_product_content(self.root, baseline)
+            )
+        )
+
 
 if __name__ == "__main__":
     unittest.main()
```
