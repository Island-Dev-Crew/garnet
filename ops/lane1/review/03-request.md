# Lane 1 · Phase 0 — Review Request 03 (ceremony-ruled reasons #2/#10 cure)

- Date: 2026-07-27 (UTC ~15:00Z)
- Implementer: Claude Code Fable 5 — same machine, same fresh `autocrlf=false` clone
- Independent reviewer sought: Codex GPT-5.6 Sol — verdict on the cured candidate
- Supersedes: candidate `48fc752` (request 02)

## Authorized scope extension — stated plainly

This round touches two **pre-existing** (landed-main-era) claims that verdict 01
did not name. Request 02 flagged them for ruling rather than curing beyond
scope; the **ceremony seat granted the extension and ruled** — reason #2 as
BLOCKER, reason #10 as a NOTE conditional on reading-unit adjacency. This is an
authorized scope extension granted by the ceremony seat, **not scope creep**.

## Cured frozen candidate

| field | value |
|-------|-------|
| **cured head** | `4f5ebb83d5772598e37a658ed4dce78208ea86fa` |
| **cured tree** | `46b289097fa281036e8b00e5e10a3e9a6b87a175` |
| parent | `afc00c6` — linear successor; no amend, no rebase, no Update branch |
| **product_content_sha256** | `9f2bbe761b0cd6762190d2e14d795f4a02e203cc3ede53fc6c4326af5cc6c925` |
| product path count | 1576 (+1 vs `48fc752`: `02-request.md` now sits below the candidate) |

## Ruling application

- **Reason #2 (BLOCKER — cured).** "the compiler proves the claim or fails the
  build" was unbounded universal enforcement. Bounded in the page's existing
  idiom: "Garnet's acceptance is a deterministic trap: **where the trap is
  proven — the bounded, pinned Gate‑1 surface below —** the compiler proves the
  claim or fails the build." The "Acceptance the model can't fake" heading and
  probabilistic-vs-deterministic framing are kept, per the ruling.
- **Reason #10 (NOTE — adjacency test FAILED, so tightened).** The ruling
  allowed leaving it byte-identical only if the "that is the design; what is
  proven today is narrower" bound sits **inside the same reading unit**. It does
  not: that bound lives in Thesis II (Senior Judgment), a different section, and
  each `.reason` card is a self-contained reading unit a reader can encounter
  directly (e.g. via the `#point-10` xref). Cured: "the compiler **is built to
  hold it** for every junior and every agent thereafter **(proven today on the
  bounded pinned Gate‑1 surface below)**."

## Constraints re-verified at `4f5ebb8`

```
garnet_capability_scope_status:  ok=true  enforced_claim_count=2  hashes_match=true  (--gate exit 0)
pinned <b>enforced:</b> lines:   byte-identical to d7430c2 (cmp exit 0, 2 lines)
test anchors:                    test_entry_authority 1 hit, scope_shadowing_parity 1 hit
canonical snippets:              untouched (gate-verified)
launch denominators printed:     0 hits (66.7/50.0/83.3/62.5/37.5)
test_garnet_capability_scope_status: OK
git diff --check:                clean
cure scope:                      docs/why.html only — 2 insertions, 2 deletions
```

## Stop

Implementer STOPS for the verdict. The Windows WV-6 acceptance runs at the head
this verdict approves. Record law untouched (zero records added).

## Appendix — exact cure diff (`git show 4f5ebb8 -- docs/why.html`)

```diff
diff --git a/docs/why.html b/docs/why.html
index a743c3d..cd11986 100644
--- a/docs/why.html
+++ b/docs/why.html
@@ -486,7 +486,7 @@
     <hr class="hr-gem">
     <div class="reasons">
       <div class="reason" id="point-1"><div class="rn">1</div><div><p class="rh">Construction beats convention</p><p class="rb">A skill can be skipped, mis&#8209;prompted, or forgotten; a type cannot. When authority is a property of the artifact, the gap between "told to be careful" and "structurally unable to overstep" closes — and nothing outside the code can close it.</p></div></div>
-      <div class="reason" id="point-2"><div class="rn">2</div><div><p class="rh">Acceptance the model can't fake</p><p class="rb">An LLM&#8209;as&#8209;judge produces a <b>probabilistic</b> verdict from the same class of system that wrote the code. Garnet's acceptance is a deterministic trap — the compiler proves the claim or fails the build. Verification that doesn't depend on a model's judgment is rare, and getting rarer.</p></div></div>
+      <div class="reason" id="point-2"><div class="rn">2</div><div><p class="rh">Acceptance the model can't fake</p><p class="rb">An LLM&#8209;as&#8209;judge produces a <b>probabilistic</b> verdict from the same class of system that wrote the code. Garnet's acceptance is a deterministic trap: where the trap is proven — the bounded, pinned Gate&#8209;1 surface below — the compiler proves the claim or fails the build. Verification that doesn't depend on a model's judgment is rare, and getting rarer.</p></div></div>
       <div class="reason" id="point-3"><div class="rn">3</div><div><p class="rh">The guarantee travels with the artifact</p><p class="rb">Your operating model, your CI, your reviewer's diligence live in the <em>environment</em> that produced the code. Ship it elsewhere and those evaporate. Garnet's seal moves <b>with</b> the binary — the seal attests what the core proves — which matters the moment agent code crosses boundaries. It now does, constantly.</p></div></div>
       <div class="reason" id="point-4"><div class="rn">4</div><div><p class="rh">The connector explosion is the threat, not its refutation</p><p class="rb">MCP now runs in ~80% of observed cloud environments; 38% of scanned servers have <b>no authentication</b>; a CVSS&#8209;9.4 CVE proved unauthenticated RCE; supply&#8209;chain attacks ship malware under <b>valid signatures</b>. All authority&#8209;management failures. More agents × more connectors = a larger ungoverned&#8209;authority surface — the problem the tooling growth is making worse.</p></div></div>
       <div class="reason" id="point-5"><div class="rn">5</div><div><p class="rh">Diff&#8209;caps keeps a human in control at agent volume</p><p class="rb">When one senior can't read everything ten agents ship, review has to move <em>into</em> the artifact: accept a huge PR by reading one thing — <b>did its authority envelope change?</b> No skill provides that; it's a property of the type system.</p></div></div>
@@ -494,7 +494,7 @@
       <div class="reason" id="point-7"><div class="rn">7</div><div><p class="rh">Garnet is the substrate under the tooling, not a rival to it</p><p class="rb">An MCP tool server <em>written in Garnet</em> declares its authority in its types, so the host verifies the envelope before granting it. "The only standard library where every function's authority is declared and verified" isn't competing with the connector economy — it's the trust layer that economy is missing.</p></div></div>
       <div class="reason" id="point-8"><div class="rn">8</div><div><p class="rh">Regulation is writing Garnet's spec — with no reference implementation</p><p class="rb">The EU Cyber Resilience Act's reporting obligations land this September; FDA change&#8209;control plans, automotive OTA, and avionics all encode the same test — <em>a change is allowed iff it stays inside a pre&#8209;approved envelope</em>, done by humans reading documents today. The planned <code>garnet build --evidence</code> flag is designed to turn a consultant engagement into a compiler flag.</p></div></div>
       <div class="reason" id="point-9"><div class="rn">9</div><div><p class="rh">Attenuating delegation across agent swarms</p><p class="rb">When A delegates to B to C, authority should only <em>shrink</em>. Skills don't compose permissions; connectors don't attenuate; prompts can't enforce it. A capability lattice in the type system can make delegation <b>provably</b> narrow authority — load&#8209;bearing as multi&#8209;agent topologies become the default.</p></div></div>
-      <div class="reason" id="point-10"><div class="rn">10</div><div><p class="rh">It amortizes senior judgment instead of re&#8209;spending it</p><p class="rb">The crisis the AI&#8209;first model creates is that juniors and agents can't be trusted to ship unreviewed. Garnet doesn't replace senior judgment — one senior defines the envelope <b>once</b>, and the compiler enforces it on every junior and every agent thereafter. Process must be re&#8209;applied every time; a type applies itself.</p></div></div>
+      <div class="reason" id="point-10"><div class="rn">10</div><div><p class="rh">It amortizes senior judgment instead of re&#8209;spending it</p><p class="rb">The crisis the AI&#8209;first model creates is that juniors and agents can't be trusted to ship unreviewed. Garnet doesn't replace senior judgment — one senior defines the envelope <b>once</b>, and the compiler is built to hold it for every junior and every agent thereafter (proven today on the bounded pinned Gate&#8209;1 surface below). Process must be re&#8209;applied every time; a type applies itself.</p></div></div>
     </div>
   </section>
 
```
