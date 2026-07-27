# Lane 1 · Phase 0 — Review Request 04 (verdict-02 F1/F2 cure)

- Date: 2026-07-27 (UTC ~16:00Z)
- Implementer: Claude Code Fable 5 — same machine, same fresh `autocrlf=false` clone
- Independent reviewer sought: Codex GPT-5.6 Sol — verdict on the cured candidate
- Supersedes: candidate `4f5ebb8` per verdict 02 (`ops/lane1/review/02-verdict.md`, APPROVE-WITH-BLOCKERS)

## Cured frozen candidate

| field | value |
|-------|-------|
| **cured head** | `72ae0246fb448ce33d689b1b80eb783497a7f215` |
| **cured tree** | `3c98ba05eb756377049325942842164f5d98910b` |
| parent | `14a5e45` (verdict 02) — linear successor; no amend, no rebase, no Update branch |
| **product_content_sha256** | `99c3f2701f0a19b25f2f56e5cca8f59e9f719c8dd03b1bc5f14401cebeb0c3ab` |
| product path count | 1578 (+2 vs `4f5ebb8`: `03-request.md` and `02-verdict.md` now sit below the candidate) |

Transport note, stated for the record: verdict 02 landed on the fork while the
cure was being assembled locally. The first local cure commit (`d01d043`,
parented on `533f8a9`) was **never pushed and never referenced in any request**;
it was discarded and the cure rebuilt as one commit atop `14a5e45`. No published
history was rewritten; no review artifact was orphaned.

## F1 cure — `--evidence` sweep counts (mandated report)

| state | literal mentions | qualified planned/roadmap | unqualified |
|-------|-----------------|---------------------------|-------------|
| before (at `4f5ebb8`) | 4 (lines 391, 398, 495, 544) | 3 | **1** (line 544, "the `--evidence` layer") |
| after (at `72ae024`) | 4 | **4** | **0** |

Line 544 now reads "the **planned** `--evidence` layer". No mention was added or
removed.

## F2 cure — every build/seal attribution named to the true surface

Source facts re-confirmed by the implementer before editing (read-only):
`build.rs` has **0** `check_module`/`garnet_check` references; `check.rs` calls
`garnet_check::check_module` (lines 70, 169); `seal.rs` imports only
`capability_surface`. The verdict's fixture probe (check rc=1 / build rc=0 /
seal rc=0 on `caps_violation.garnet`) is taken as controlling.

- **Reason #2 (line 489):** now "`garnet check` proves the claim or fails,
  deterministically; that proof lives on the check path today, not yet the
  build path."
- **Reason #3 (line 490):** now "today it attests the declared capability
  surface without re-running the checker; verification lives at `garnet check`,
  and binding seal to a passing check is named, unshipped work."
- **Layer map (line 449):** now "checked by `garnet check` — … not yet wired
  into `garnet build`."
- **Evidence closer (line 399):** "no evidence without a build that proves it"
  → "no evidence without a **check** that proves it."

No `garnet-cli/src` byte changed (trust-kernel; out of lane scope).

## U-32 — REGISTERED (product finding, highest priority in the register)

> garnet build and garnet seal accept a fixture that garnet check rejects.
> Enforcement is not on the build path; seal attests an unverified surface.
> Garnet's central claim — proof or the build fails — is today true of check
> only. Cure: route the checker into build and gate seal on a passing check,
> with a committed trap fixture proving all three commands agree. Full
> ceremony; own lane; must land before Lane 4 frozen candidate and before any
> public claim of build-time enforcement is restored.

U-32 goes verbatim into the PR findings block at slice 6 alongside U-23..U-31.

## Open units beyond the ceremony-scoped cure — awaiting seat ruling

Verdict 02's whole-page adversarial read enumerates reading units beyond the
cure the ceremony seat authorized for this round (F1 + build/seal attribution
+ U-32). They are **open, not silently accepted**:

| unit | verdict lines | claim character |
|------|---------------|-----------------|
| hero title/dek/SVG captions | 236-237, 275, 278 | "Enforcement by construction", "A type cannot", "undeclared = inexpressible", "the property is not [porous]" |
| The Turn section + callout | 289, 297-301 | authority "provable" to host; structural impossibility; "typed, diffable, sealed property" |
| Compiler thesis-line | 317 | "whole classes of failure don't compile" |
| Reasons #1, #6, #7, #9 (+ #10 residual) | 488, 493-494, 496-497 | structural-overstep impossibility; "only expressible thing"; host envelope verification; provable attenuation |
| Domain cards ("Where it bites") | 508-517 | ten present-tense integration/enforcement claims |
| Boundary li #1 residual | 544 | "the swarm-delegation primitive" fragment |

Options the seat can rule between: (a) a further authorized scope extension to
bound these in place (largest honest rewrite of the page's rhetorical spine),
(b) a page-level claim-status banner bounding all unpinned prose, or (c)
acceptance that the boundary section + theses scope note bound them — which
verdict 02 explicitly rejected as a reading. The implementer does not choose.

## S-WS-1 — acknowledged, not touched

`02-request.md` carries trailing whitespace on three lines. It is an add-once
review artifact whose non-modification the reviewer verifies; it will not be
edited, in line with the advisory's own instruction.

## Constraints re-verified at `72ae024`

```
garnet_capability_scope_status:  ok=true  enforced_claim_count=2  hashes_match=true  (--gate exit 0)
pinned <b>enforced:</b> lines:   byte-identical to d7430c2 (cmp exit 0)
anchors:                         test_entry_authority 1, scope_shadowing_parity 1; snippets untouched
launch denominators printed:     0 hits
build/seal attribution sweep:    none remains (incl. 'a build that proves' phrasing)
test_garnet_capability_scope_status: OK
git diff --check (this commit):  clean
cure scope:                      docs/why.html only — 5 insertions, 5 deletions
```

## Stop

Implementer STOPS for the verdict. **No head is approved for the NUC**; the
Windows WV-6 acceptance runs only at a head a verdict approves. Record law
untouched (zero records added). U-31/F5 continues to block slice 5.

## Appendix — exact cure diff (`git show 72ae024 -- docs/why.html`)

```diff
diff --git a/docs/why.html b/docs/why.html
index cd11986..2806062 100644
--- a/docs/why.html
+++ b/docs/why.html
@@ -396,7 +396,7 @@
         <div class="thesis-body-inner"><div class="thesis-body-content">
           <p>Aerospace, defense, healthcare — the buyers who matter don't accept "an AI reviewed it and said it was fine." A comment thread is not an artifact. A chat log is not a control. When the auditor asks how you <em>know</em>, they mean something you can hand over.</p>
           <p>The planned <code>garnet build --evidence</code> flag — no such build flag exists in the shipping CLI today — is designed to emit a <span class="gold">Verifiable Evidence Bundle</span> with the build: what this code is capable of, what it is incapable of, and the capability declarations those facts derive from. What is a consultant engagement today — weeks of humans reading documents — would become a compiler flag that runs on every build.</p>
-          <p>That is the roadmap, not the present: evidence stops being a deliverable you assemble for the audit and becomes exhaust the toolchain produces anyway. Until that flag ships, the runnable surface is <code>garnet caps</code>, <code>garnet diff-caps</code>, and <code>garnet seal</code>. No authority without evidence — and no evidence without a build that proves it.</p>
+          <p>That is the roadmap, not the present: evidence stops being a deliverable you assemble for the audit and becomes exhaust the toolchain produces anyway. Until that flag ships, the runnable surface is <code>garnet caps</code>, <code>garnet diff-caps</code>, and <code>garnet seal</code>. No authority without evidence — and no evidence without a check that proves it.</p>
           <div class="xrefs">
             <span class="xrefs-label">Reads with</span>
             <a class="xref" href="#point-8">№ 8 — Regulation is writing the spec</a>
@@ -446,7 +446,7 @@
         </div>
         <div class="layer-row garnet-row">
           <span class="layer-q">What was it ever allowed to do?</span>
-          <span class="layer-a"><strong>Authority.</strong> Garnet. Capability declarations checked at build — deterministic enforcement proven today on the bounded surface below — delegation designed to attenuate, and an evidence flag on the roadmap. Structural, not sampled.</span>
+          <span class="layer-a"><strong>Authority.</strong> Garnet. Capability declarations checked by <code>garnet check</code> — deterministic enforcement proven today on the bounded surface below, not yet wired into <code>garnet build</code> — delegation designed to attenuate, and an evidence flag on the roadmap. Structural, not sampled.</span>
         </div>
       </div>
       <p class="layermap-note">Tests are empirical — they check behaviors someone thought to check. Capabilities are structural — they are built to bound behaviors nobody thought of. A swarm can pass every test in the suite while holding authority it never needed; no green checkmark speaks to that. Which is why <code>garnet check</code> runs <em>inside</em> your CI, whoever's runners it's on: the faster the pipeline, the more often the declared envelope is checked — and, once the evidence flag ships, the more often the evidence would regenerate.</p>
@@ -486,8 +486,8 @@
     <hr class="hr-gem">
     <div class="reasons">
       <div class="reason" id="point-1"><div class="rn">1</div><div><p class="rh">Construction beats convention</p><p class="rb">A skill can be skipped, mis&#8209;prompted, or forgotten; a type cannot. When authority is a property of the artifact, the gap between "told to be careful" and "structurally unable to overstep" closes — and nothing outside the code can close it.</p></div></div>
-      <div class="reason" id="point-2"><div class="rn">2</div><div><p class="rh">Acceptance the model can't fake</p><p class="rb">An LLM&#8209;as&#8209;judge produces a <b>probabilistic</b> verdict from the same class of system that wrote the code. Garnet's acceptance is a deterministic trap: where the trap is proven — the bounded, pinned Gate&#8209;1 surface below — the compiler proves the claim or fails the build. Verification that doesn't depend on a model's judgment is rare, and getting rarer.</p></div></div>
-      <div class="reason" id="point-3"><div class="rn">3</div><div><p class="rh">The guarantee travels with the artifact</p><p class="rb">Your operating model, your CI, your reviewer's diligence live in the <em>environment</em> that produced the code. Ship it elsewhere and those evaporate. Garnet's seal moves <b>with</b> the binary — the seal attests what the core proves — which matters the moment agent code crosses boundaries. It now does, constantly.</p></div></div>
+      <div class="reason" id="point-2"><div class="rn">2</div><div><p class="rh">Acceptance the model can't fake</p><p class="rb">An LLM&#8209;as&#8209;judge produces a <b>probabilistic</b> verdict from the same class of system that wrote the code. Garnet's acceptance is a deterministic trap: where the trap is proven — the bounded, pinned Gate&#8209;1 surface below — <code>garnet check</code> proves the claim or fails, deterministically; that proof lives on the check path today, not yet the build path. Verification that doesn't depend on a model's judgment is rare, and getting rarer.</p></div></div>
+      <div class="reason" id="point-3"><div class="rn">3</div><div><p class="rh">The guarantee travels with the artifact</p><p class="rb">Your operating model, your CI, your reviewer's diligence live in the <em>environment</em> that produced the code. Ship it elsewhere and those evaporate. Garnet's seal moves <b>with</b> the binary — today it attests the declared capability surface without re&#8209;running the checker; verification lives at <code>garnet check</code>, and binding seal to a passing check is named, unshipped work — which matters the moment agent code crosses boundaries. It now does, constantly.</p></div></div>
       <div class="reason" id="point-4"><div class="rn">4</div><div><p class="rh">The connector explosion is the threat, not its refutation</p><p class="rb">MCP now runs in ~80% of observed cloud environments; 38% of scanned servers have <b>no authentication</b>; a CVSS&#8209;9.4 CVE proved unauthenticated RCE; supply&#8209;chain attacks ship malware under <b>valid signatures</b>. All authority&#8209;management failures. More agents × more connectors = a larger ungoverned&#8209;authority surface — the problem the tooling growth is making worse.</p></div></div>
       <div class="reason" id="point-5"><div class="rn">5</div><div><p class="rh">Diff&#8209;caps keeps a human in control at agent volume</p><p class="rb">When one senior can't read everything ten agents ship, review has to move <em>into</em> the artifact: accept a huge PR by reading one thing — <b>did its authority envelope change?</b> No skill provides that; it's a property of the type system.</p></div></div>
       <div class="reason" id="point-6"><div class="rn">6</div><div><p class="rh">Incumbent fluency ships incumbent footguns</p><p class="rb">An agent writes fluent Python <em>and</em> Python's ambient&#8209;authority footguns, because the language predates capability thinking and bolts safety on coarsely, after the fact. A language built for capability from the type system up makes the safe thing the <b>only expressible</b> thing.</p></div></div>
@@ -541,7 +541,7 @@
     <div class="bound">
       <h3>Three honest boundaries</h3>
       <ol>
-        <li><b>This is a narrower, sharper need than a grand vision.</b> Garnet is the trust substrate and enforcement layer for agent&#8209;authored and agent&#8209;granted authority — the thing beneath MCP, the <code>--evidence</code> layer, the swarm&#8209;delegation primitive — not a general&#8209;purpose better&#8209;Python humans adopt for pleasure.</li>
+        <li><b>This is a narrower, sharper need than a grand vision.</b> Garnet is the trust substrate and enforcement layer for agent&#8209;authored and agent&#8209;granted authority — the thing beneath MCP, the planned <code>--evidence</code> layer, the swarm&#8209;delegation primitive — not a general&#8209;purpose better&#8209;Python humans adopt for pleasure.</li>
         <li><b>Need is not adoption.</b> These reasons argue Garnet <em>should</em> exist. Whether it reaches the people who need it is a separate, unsolved bet that rides on a playground you can touch in thirty seconds, a real library shelf, and a community that doesn't exist yet. The idea can be right and still not make it.</li>
         <li>
           <b>The foundation claim is now bounded by a closed Gate 1, not an open&#8209;ended promise.</b> Gate 1 is closed on the documented canonical-macOS + sealed-Windows boundary. One required Windows closure row was truth gate fail-closed + <code>verify examples</code> (<a href="https://github.com/Island-Dev-Crew/garnet/pull/409">canonical macOS #409</a>; <a href="https://github.com/Island-Dev-Crew/garnet/pull/468">sealed Windows #468</a>).
```
