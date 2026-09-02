# CRA page anchors — cross-family confirmation record (2026-09-02)

- Branch `mission/cra-anchors-2026-09-02` · base main `fbd64bc514b573a6735b3525f4f3172be14a27d0` · content tip reviewed `9aa4893e89dedb6ce601cdbf332a1231d100b543`
- Implementing seat: Claude Fable 5.1 · Reviewing seat: Codex (codex-cli 0.147.0 via the local wrapper, cross-family, read-only, detached worktree at the tip; L-15 satisfied)
- Path class: `docs/cra-article-14.html` is not a rolling-gate trust-kernel trigger (gate `ok: true`, `touched_paths []`); this markdown record is the review artifact. The verdict below is transported verbatim, except that one retired word in the reviewing seat's prose is elided and marked; the finding content is unchanged.
- Verdict: **CONFIRM**, no findings, bound to `9aa4893e…`; this record commit is the records-class head move the verdict anticipates.

## Reviewing seat output (verbatim)

```text
```console
$ git rev-parse HEAD
9aa4893e89dedb6ce601cdbf332a1231d100b543

$ git status --porcelain
```

No output; worktree clean.

```console
$ git rev-parse fbd64bc514b573a6735b3525f4f3172be14a27d0
fbd64bc514b573a6735b3525f4f3172be14a27d0

$ git diff --name-status fbd64bc514b573a6735b3525f4f3172be14a27d0..HEAD
M	docs/cra-article-14.html

$ git diff --stat fbd64bc514b573a6735b3525f4f3172be14a27d0..HEAD
 docs/cra-article-14.html | 2 +-
 1 file changed, 1 insertion(+), 1 deletion(-)

$ git diff --numstat fbd64bc514b573a6735b3525f4f3172be14a27d0..HEAD
1	1	docs/cra-article-14.html

$ git log --oneline fbd64bc514b573a6735b3525f4f3172be14a27d0..HEAD
9aa4893 docs(cra): render the U-32 and U-33 citations as anchors
```

```console
$ git diff --unified=0 fbd64bc514b573a6735b3525f4f3172be14a27d0..HEAD -- docs/cra-article-14.html
diff --git a/docs/cra-article-14.html b/docs/cra-article-14.html
index 51cfa31..b79d183 100644
--- a/docs/cra-article-14.html
+++ b/docs/cra-article-14.html
@@ -95 +95 @@
-    <p>That position follows the existing register and ceremony rail ([U-32](https://github.com/Island-Dev-Crew/garnet/blob/main/ops/lane1/review/04-request.md), [U-33](https://github.com/Island-Dev-Crew/garnet/blob/main/ops/lane1/review/04-request-addendum-u33.md)). It does not upgrade either finding or claim that the open work has shipped.</p>
+    <p>That position follows the existing register and ceremony rail (<a href="https://github.com/Island-Dev-Crew/garnet/blob/main/ops/lane1/review/04-request.md">U-32</a>, <a href="https://github.com/Island-Dev-Crew/garnet/blob/main/ops/lane1/review/04-request-addendum-u33.md">U-33</a>). It does not upgrade either finding or claim that the open work has shipped.</p>
```

```console
$ python3 -c 'import difflib,re,subprocess; p="docs/cra-article-14.html"; b="fbd64bc514b573a6735b3525f4f3172be14a27d0"; old=subprocess.check_output(["git","show",f"{b}:{p}"],text=True).splitlines()[94]; new=subprocess.check_output(["git","show",f"HEAD:{p}"],text=True).splitlines()[94]; old_s=re.sub("<[^>]+>","",old); new_s=re.sub("<[^>]+>","",new); print("OLD_HTML_STRIPPED:",old_s); print("NEW_HTML_STRIPPED:",new_s); print("VISIBLE_DIFF:"); print("\n".join(difflib.unified_diff([old_s],[new_s],fromfile="old",tofile="new",lineterm=""))); md=lambda s: re.sub(r"\[([^]]+)\]\([^)]+\)",r"\1",s); print("OLD_VISIBLE_NORMALIZED:",md(old_s)); print("NEW_VISIBLE_NORMALIZED:",md(new_s)); print("VISIBLE_TEXT_EQUAL:",md(old_s)==md(new_s))'
OLD_HTML_STRIPPED:     That position follows the existing register and ceremony rail ([U-32](https://github.com/Island-Dev-Crew/garnet/blob/main/ops/lane1/review/04-request.md), [U-33](https://github.com/Island-Dev-Crew/garnet/blob/main/ops/lane1/review/04-request-addendum-u33.md)). It does not upgrade either finding or claim that the open work has shipped.
NEW_HTML_STRIPPED:     That position follows the existing register and ceremony rail (U-32, U-33). It does not upgrade either finding or claim that the open work has shipped.
VISIBLE_DIFF:
--- old
+++ new
@@ -1 +1 @@
-    That position follows the existing register and ceremony rail ([U-32](https://github.com/Island-Dev-Crew/garnet/blob/main/ops/lane1/review/04-request.md), [U-33](https://github.com/Island-Dev-Crew/garnet/blob/main/ops/lane1/review/04-request-addendum-u33.md)). It does not upgrade either finding or claim that the open work has shipped.
+    That position follows the existing register and ceremony rail (U-32, U-33). It does not upgrade either finding or claim that the open work has shipped.
OLD_VISIBLE_NORMALIZED:     That position follows the existing register and ceremony rail (U-32, U-33). It does not upgrade either finding or claim that the open work has shipped.
NEW_VISIBLE_NORMALIZED:     That position follows the existing register and ceremony rail (U-32, U-33). It does not upgrade either finding or claim that the open work has shipped.
VISIBLE_TEXT_EQUAL: True
```

```console
$ curl -s -o /dev/null -w '%{http_code}\n' -L https://github.com/Island-Dev-Crew/garnet/blob/main/ops/lane1/review/04-request.md
200

$ curl -s -o /dev/null -w '%{http_code}\n' -L https://github.com/Island-Dev-Crew/garnet/blob/main/ops/lane1/review/04-request-addendum-u33.md
200
```

```console
$ python3 - <<'PY'
from html.parser import HTMLParser
from pathlib import Path

class StrictWalk(HTMLParser):
    void = {"area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source", "track", "wbr"}

    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.stack = []
        self.start_tags = 0
        self.end_tags = 0
        self.mismatches = []

    def handle_starttag(self, tag, attrs):
        self.start_tags += 1
        if tag not in self.void:
            self.stack.append((tag, self.getpos()))

    def handle_startendtag(self, tag, attrs):
        self.start_tags += 1

    def handle_endtag(self, tag):
        self.end_tags += 1
        if not self.stack or self.stack[-1][0] != tag:
            self.mismatches.append((tag, self.getpos(), self.stack[-1] if self.stack else None))
            return
        self.stack.pop()

parser = StrictWalk()
parser.feed(Path("docs/cra-article-14.html").read_text(encoding="utf-8"))
parser.close()
if parser.mismatches or parser.stack:
    print(f"html.parser: FAIL (mismatches={parser.mismatches}, unclosed={parser.stack})")
    raise SystemExit(1)
print(f"html.parser: ok (start_tags={parser.start_tags}, end_tags={parser.end_tags}, mismatches=0, unclosed=0)")
PY
html.parser: ok (start_tags=86, end_tags=81, mismatches=0, unclosed=0)
```

```console
$ grep -inE '[retired word elided by the transporting seat]' docs/cra-article-14.html
$ printf 'grep_exit_status=%s\n' "$?"
grep_exit_status=1
```

```console
$ perl -e '$SIG{ALRM}=sub{print STDERR "TIMEOUT after 300 seconds\n"; exit 124}; alarm 300; exec @ARGV or die "exec failed: $!\n"' python3 -I scripts/garnet_trust_kernel_review_status.py --base fbd64bc514b573a6735b3525f4f3172be14a27d0 --head HEAD --format json
{
  "schema": "garnet.trust_kernel_review/v2",
  "ok": true,
  "discovery_ok": true,
  "discovery_source": "git",
  "base_commit": "fbd64bc514b573a6735b3525f4f3172be14a27d0",
  "head_commit": "9aa4893e89dedb6ce601cdbf332a1231d100b543",
  "trust_kernel_touched": false,
  "touched_paths": [],
  "review_record_present": false,
  "review_record_path": null,
  "reviewer": null,
  "reviewer_id": null,
  "reviewer_login": null,
  "reviewed_head": null,
  "reviewed_tree": null,
  "content_digest": null,
  "changed_count": 1,
  "problems": [],
  "review_companion_present": false,
  "companion_paths": [],
  "review_trailer_present": false,
  "trust_kernel_prefixes": [
    "garnet-check-v0.3/src/",
    "garnet-interp-v0.3/src/",
    "garnet-vm/src/",
    "garnet-stdlib/src/",
    "garnet-wasm/src/",
    ".github/actions/",
    ".github/rulesets/",
    ".github/workflows/",
    "scripts/garnet_",
    "scripts/test_garnet_",
    "F_Project_Management/W_TRUST/landed/"
  ],
  "trust_kernel_files": [
    ".github/CODEOWNERS",
    "Cargo.lock",
    "garnet-cli/Cargo.toml",
    "garnet-cli/src/bound_source.rs",
    "garnet-cli/src/cmd/add.rs",
    "garnet-cli/src/cmd/mod.rs",
    "garnet-cli/src/cmd/run.rs",
    "garnet-cli/src/cmd/test.rs",
    "garnet-cli/src/cmd/eval.rs",
    "garnet-cli/src/cmd/doctest.rs",
    "garnet-cli/src/bin/garnet.rs",
    "garnet-cli/src/lib.rs",
    "scripts/garnet_launch_readiness_status.py",
    "scripts/garnet_caps_enforcement_status.py",
    "scripts/garnet_capability_scope_status.py",
    "scripts/garnet_bounded_enforcement_status.py",
    "scripts/garnet_red_team_status.py",
    "docs/why.html",
    "C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md",
    "F_Project_Management/W_TRUST/LANDED_REVIEW_MARKERS.json"
  ]
}
```

```console
$ git rev-parse HEAD
9aa4893e89dedb6ce601cdbf332a1231d100b543

$ git status --porcelain
```

No output; final worktree remains clean.

VERDICT: CONFIRM — bound to 9aa4893e89dedb6ce601cdbf332a1231d100b543

1. None.



```
