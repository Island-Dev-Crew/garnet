# Lane 1 · Phase 0 — Reconciliation Unblock · Review Request 02 (F1/F2 cure)

- Date: 2026-07-27 (UTC ~14:30Z)
- Implementer: Claude Code (Fable 5 at cure time; rounds 1 slices ran under Opus 4.8) — macOS Darwin arm64 (Hughs-MacBook-Pro), same fresh `autocrlf=false` clone
- Independent reviewer sought: Codex GPT-5.6 Sol — verdict 02 on the cured candidate
- Supersedes: candidate `d7430c2` per verdict 01 (APPROVE-WITH-BLOCKERS, `ops/lane1/review/01-verdict.md`)

## Cured frozen candidate

| field | value |
|-------|-------|
| **cured head** | `48fc7529b1321ce5e54eafd9ac42523a6813f161` |
| **cured tree** | `e769ef1f17698e6a5bba51d9d69b4ee69fe56739` |
| parent | `f51c936b` (verdict 01) — linear successor; no amend, no rebase, no Update branch |
| **product_content_sha256** | `aa6a65a3f726ff49efc25ab96897de1172b8c4651da754805900b82bd75a3d43` |
| product path count | 1575 |
| prior candidate digest | `c4b3cf7c…` at `d7430c2` (1572 paths) |

Path count +3 vs the prior candidate: `01-request.md`, `BLOCKED.md`, and `01-verdict.md` now sit **below** the cured candidate and are digest-tracked (`ops/lane1/` is inside the product digest — F-3 from request 01). The Windows acceptance runs at whichever head verdict 02 approves and binds these files with it.

## Cure scope

One file: `docs/why.html` (23 insertions, 23 deletions). No gate, reporter, workflow, CLI, or trust-kernel logic touched. No CLI feature implemented (per verdict: "Do not implement a new feature under this review verdict").

### F1 — nonexistent `garnet build --evidence` shipping claim

All three literal mentions reframed as an explicitly planned/roadmap flag, none as shipping/available/demonstrable:

- **Thesis IV line:** now "— a planned flag, not in the CLI today — is designed to turn…"
- **Thesis IV body:** now "The planned … flag — no such build flag exists in the shipping CLI today — is designed to emit…", consultant-engagement sentence moved to the conditional ("would become").
- **Thesis IV close:** opens "That is the roadmap, not the present:", and names the *actually runnable* surface instead: `garnet caps`, `garnet diff-caps`, `garnet seal` (all verified present in `garnet-cli/src/lib.rs` usage/dispatch).
- **Claim tag:** "shipping flag — bundle format versioned" → plain (gray/planned) class: "planned — roadmap flag; today's runnable surface is caps / diff-caps / seal".
- **Reason #8 (pre-existing third mention):** "The planned … flag is designed to turn a consultant engagement into a compiler flag."
- **Layer map (verdict lines 450/453):** "evidence emitted with the artifact" → "an evidence flag on the roadmap"; "the evidence regenerates" → "once the evidence flag ships, … the evidence would regenerate".
- `garnet check` in the layer-map note was verified real (safe-mode checker in the CLI usage) and stays.

### F2 — enforcement/attenuation claims narrowed to the proven surface

- **Claim tags 332/357:** "enforced by the language today" → `construction` class, "partial — deterministic traps proven on the bounded surface below; wider surface declared (checker-only)" / "partial — the primitive is proven on the bounded surface below; the full envelope vocabulary is roadmap".
- **Compiler body (325):** unconditional "Code that exceeds its capabilities … fails to build" → bounded: "Where the trap is proven — the bounded, pinned Gate-1 surface in the boundary section below — … it fails, deterministically. The wider surface is declared (checker-only) today, being built to that same standard."
- **Senior Judgment (343/350):** "The compiler enforces it" → "is built to hold it"; body now "That is the design; what is proven today is narrower — the deterministic traps named in the boundary section below…"; "Review time becomes" → "then becomes".
- **Swarm (368/375-376):** "authority only shrinks / makes delegation attenuate" → "should only shrink / is designed to make"; body adds "The machine-checked proof of that lattice is in progress, not complete; until it lands, this is a design commitment, not a shipped guarantee"; "a fleet that can't misbehave" → "building toward a fleet whose authority is bounded by an envelope the toolchain checks"; claim tag gains "; not enforced today".
- **Open-Weight (424):** "no model can talk its way past a capability it was never granted" → "Today that verdict covers the bounded, pinned surface stated below — not a universal sandbox — and every widening must arrive with its own trap."
- **Layer map garnet-row (450):** "Capability types enforced at build" → "Capability declarations checked at build — deterministic enforcement proven today on the bounded surface below…".
- **Legend:** "Enforced today" removed; entries now "Partial / by construction — proof in progress" (gold), "Planned — roadmap" (gray), "World-claim" (blue), plus a scope note: *"Nothing in this section claims enforcement beyond the two pinned Gate-1 claims in the boundary section below."* Unused `.claim.enforced` and legend `.g` CSS removed.

## Re-run gate output at `48fc752`

```
garnet_capability_scope_status:  ok=true  enforced_claim_count=2  hashes_match=true  (--gate exit 0)
pinned <b>enforced:</b> lines:   byte-identical to d7430c2 (cmp exit 0, 2 lines)
test anchors + canonical snippets: untouched (gate-verified)
launch denominators printed on page: 0 hits for 66.7/50.0/83.3/62.5/37.5
'claim enforced' class remaining:  none; no 'Enforced today' label remains
test_garnet_capability_scope_status: Ran 10 tests — OK
git diff --check: clean
```

## Registered / acknowledged

- **U-31 (F4, absolute path in `08.source`):** acknowledged as registered and blocking **slice 5**, not this cure. Not attempted here, per verdict.
- **Denominator ruling accepted:** slice 5 publishes **66.7% / 50.0%** (4/6, 4/8) after the Windows WV-6 refresh, unless another gate independently closes first.
- **Windows leg:** the NUC's WV-6 acceptance runs at the head verdict 02 approves — not `d7430c2`.

## Flagged for reviewer judgment (pre-existing, not cured)

Landed-main-era reasons #2 ("the compiler proves the claim or fails the build") and #10 ("the compiler enforces it on every junior and every agent thereafter") carry present-tense enforcement rhetoric that predates this branch and passed prior review at `68317ae`. Verdict 01's F2 named only new lines, so they were left byte-identical; if verdict 02 rules they violate the same standard, they are one bounded successor edit away.

## Questions for verdict 02

1. Do the cured F1 mentions satisfy "remove the shipping/emission claim or replace it with bounded truth about an actually runnable command" — specifically the caps/diff-caps/seal substitution?
2. Does any cured sentence still assert enforcement, attenuation, or guarantee beyond the two pinned claims plus committed traps?
3. Is the +3 path-count movement (review bookkeeping baked below the successor) acceptable freeze hygiene, or should the Windows leg bind `d7430c2`-style content-only trees going forward?
4. Ruling requested on the two flagged pre-existing reasons (#2, #10).

## Stop

Implementer STOPS for verdict 02. Nothing further lands until it does; the record law is untouched (zero records added).

## Appendix — exact cure diff (`git show 48fc752 -- docs/why.html`)

```diff
diff --git a/docs/why.html b/docs/why.html
index 35ac384..a743c3d 100644
--- a/docs/why.html
+++ b/docs/why.html
@@ -175,8 +175,6 @@
   .xref:hover{color:var(--bone);border-color:var(--line-2);text-decoration:none}
   .claim{display:inline-flex;align-items:center;gap:.6em;font-family:"JetBrains Mono",monospace;font-size:10.5px;letter-spacing:.08em;text-transform:uppercase;margin-top:20px;padding:6px 12px;border-radius:9px;border:1px solid var(--line);color:var(--ash)}
   .claim::before{content:"";width:7px;height:7px;border-radius:50%;background:var(--ash)}
-  .claim.enforced{color:var(--jade-soft);border-color:rgba(52,178,123,.28)}
-  .claim.enforced::before{background:var(--jade-soft)}
   .claim.construction{color:var(--gold-soft);border-color:rgba(217,165,33,.3)}
   .claim.construction::before{background:var(--gold-soft)}
   .claim.world{color:var(--steel);border-color:rgba(91,127,181,.3)}
@@ -198,9 +196,10 @@
   .theses-legend{margin-top:22px;display:flex;flex-wrap:wrap;gap:14px;font-family:"JetBrains Mono",monospace}
   .theses-legend span{display:inline-flex;align-items:center;gap:.5em;font-size:10px;text-transform:uppercase;letter-spacing:.1em;color:var(--ash)}
   .theses-legend i{width:7px;height:7px;border-radius:50%;display:inline-block}
-  .theses-legend .g i{background:var(--jade-soft)}.theses-legend .g{color:var(--jade-soft)}
   .theses-legend .a i{background:var(--gold-soft)}.theses-legend .a{color:var(--gold-soft)}
+  .theses-legend .p i{background:var(--ash)}.theses-legend .p{color:var(--ash)}
   .theses-legend .b i{background:var(--steel)}.theses-legend .b{color:var(--steel)}
+  .theses-scope-note{margin-top:10px;font-family:"JetBrains Mono",monospace;font-size:10.5px;color:var(--ash-dim);letter-spacing:.03em}
   @media (max-width:560px){
     .thesis-toggle{grid-template-columns:auto 1fr;gap:14px;padding:18px}
     .expand-mark{grid-column:2;padding-top:6px}
@@ -322,14 +321,14 @@
       <div class="thesis-body" id="body-compiler" role="region" aria-label="The Compiler, expanded">
         <div class="thesis-body-inner"><div class="thesis-body-content">
           <p>The review industry's best tools read finished code and estimate whether it's dangerous. The strongest of them catch roughly half of real runtime bugs — after the code exists, on every pull request, forever. That's a smarter reviewer. It is still a probability.</p>
-          <p>Garnet moves the question earlier and makes it binary. What code is <em>allowed</em> to do — which files, which network, which processes, which delegated authority — is written as a <span class="gold">capability type</span>. Code that exceeds its capabilities isn't flagged for a human to weigh. It fails to build.</p>
+          <p>Garnet moves the question earlier and makes it binary. What code is <em>allowed</em> to do — which files, which network, which processes, which delegated authority — is written as a <span class="gold">capability type</span>. Where the trap is proven — the bounded, pinned Gate&#8209;1 surface in the boundary section below — code that invokes authority it never declared isn't flagged for a human to weigh: it fails, deterministically. The wider surface is declared (checker&#8209;only) today, being built to that same standard.</p>
           <p>The industry's own principle says the AI that wrote the code shouldn't be the one reviewing it. Its answer is a second model. Garnet's answer is a checker. A compiler doesn't have opinions; it has verdicts — and a verdict is the only kind of review that doesn't get tired, doesn't get noisy, and doesn't get argued past.</p>
           <div class="xrefs">
             <span class="xrefs-label">Reads with</span>
             <a class="xref" href="#point-1">№ 1 — Construction beats convention</a>
             <a class="xref" href="#point-2">№ 2 — Acceptance the model can't fake</a>
           </div>
-          <span class="claim enforced">Claim class · enforced by the language today</span>
+          <span class="claim construction">Claim class · partial — deterministic traps proven on the bounded surface below; wider surface declared (checker&#8209;only)</span>
         </div></div>
       </div>
     </article>
@@ -340,21 +339,21 @@
         <span class="numeral">II</span>
         <span>
           <span class="thesis-name">The Senior Judgment</span>
-          <span class="thesis-line">One senior defines the envelope <span class="gold">once</span>. The compiler enforces it on every junior, every agent, every commit after.</span>
+          <span class="thesis-line">One senior defines the envelope <span class="gold">once</span>. The compiler is built to hold it for every junior, every agent, every commit after.</span>
         </span>
         <span class="expand-mark" aria-hidden="true">+ open</span>
       </button>
       <div class="thesis-body" id="body-senior" role="region" aria-label="The Senior Judgment, expanded">
         <div class="thesis-body-inner"><div class="thesis-body-content">
           <p>The AI&#8209;first crisis isn't that agents write bad code. It's that juniors and agents can't be trusted to ship unreviewed — so senior engineers became full&#8209;time reviewers, and their judgment became the scarcest resource in the org. Every review tool on the market re&#8209;spends that judgment on every change. Process must be re&#8209;applied every time.</p>
-          <p>A type applies <span class="gold">once</span>. Garnet doesn't replace senior judgment — it changes what it's spent on. One senior encodes the security envelope as capability types: what this service may touch, what this agent may delegate, what this module may never do. From that moment the compiler holds the line on every contribution that follows, from a first&#8209;week junior or a thousand&#8209;agent fleet, at zero marginal senior attention.</p>
-          <p>Review time becomes taste time. Humans argue about the things worth arguing about — naming, architecture, product — because the floor is no longer up for debate.</p>
+          <p>A type applies <span class="gold">once</span>. Garnet doesn't replace senior judgment — it changes what it's spent on. One senior encodes the security envelope as capability types: what this service may touch, what this agent may delegate, what this module may never do. That is the design; what is proven today is narrower — the deterministic traps named in the boundary section below — with the full envelope vocabulary declared (checker&#8209;only) and built toward the same standard at zero marginal senior attention.</p>
+          <p>Review time then becomes taste time. Humans argue about the things worth arguing about — naming, architecture, product — because the floor is no longer up for debate.</p>
           <div class="xrefs">
             <span class="xrefs-label">Reads with</span>
             <a class="xref" href="#point-10">№ 10 — It amortizes senior judgment</a>
             <a class="xref" href="#point-5">№ 5 — Diff&#8209;caps keeps a human in control</a>
           </div>
-          <span class="claim enforced">Claim class · enforced by the language today</span>
+          <span class="claim construction">Claim class · partial — the primitive is proven on the bounded surface below; the full envelope vocabulary is roadmap</span>
         </div></div>
       </div>
     </article>
@@ -365,20 +364,20 @@
         <span class="numeral">III</span>
         <span>
           <span class="thesis-name">The Swarm</span>
-          <span class="thesis-line">When A delegates to B to C, authority only <span class="ink">shrinks</span>. The capability lattice makes delegation <span class="gold">attenuate by construction</span>.</span>
+          <span class="thesis-line">When A delegates to B to C, authority should only <span class="ink">shrink</span>. The capability lattice is designed to make delegation <span class="gold">attenuate by construction</span>.</span>
         </span>
         <span class="expand-mark" aria-hidden="true">+ open</span>
       </button>
       <div class="thesis-body" id="body-swarm" role="region" aria-label="The Swarm, expanded">
         <div class="thesis-body-inner"><div class="thesis-body-content">
           <p>Multi&#8209;agent topologies are the default now. Orchestrators spawn sub&#8209;agents that spawn sub&#8209;agents, and the newest frontier models improvise their own delegation patterns — sub&#8209;agents mutating parent state, cleanup routines inheriting kill&#8209;anything scope from the top of the tree. Charming when it's benign. Nothing granted it; nothing could have denied it.</p>
-          <p>Prompts can't enforce attenuation. Connectors don't attenuate. Skills don't compose permissions. Garnet puts a <span class="gold">capability lattice</span> in the type system: a delegated capability is always a subset of the delegator's, so authority can only narrow at every edge of the tree — by construction, whatever the model, however novel its orchestration style.</p>
-          <p>The swarm keeps its creativity. What it loses is ambient authority — the difference between hoping your fleet behaves and building a fleet that can't misbehave outside the envelope you gave it.</p>
+          <p>Prompts can't enforce attenuation. Connectors don't attenuate. Skills don't compose permissions. Garnet's design puts a <span class="gold">capability lattice</span> in the type system: a delegated capability is constrained to a subset of the delegator's, so authority narrows at every edge of the tree — whatever the model, however novel its orchestration style. The machine&#8209;checked proof of that lattice is in progress, not complete; until it lands, this is a design commitment, not a shipped guarantee.</p>
+          <p>The swarm keeps its creativity. What the design takes from it is ambient authority — the difference between hoping your fleet behaves and building toward a fleet whose authority is bounded by an envelope the toolchain checks.</p>
           <div class="xrefs">
             <span class="xrefs-label">Reads with</span>
             <a class="xref" href="#point-9">№ 9 — Attenuating delegation across swarms</a>
           </div>
-          <span class="claim construction">Claim class · by construction — machine&#8209;checked proof pipeline in progress</span>
+          <span class="claim construction">Claim class · by construction — machine&#8209;checked proof pipeline in progress; not enforced today</span>
         </div></div>
       </div>
     </article>
@@ -389,20 +388,20 @@
         <span class="numeral">IV</span>
         <span>
           <span class="thesis-name">The Evidence</span>
-          <span class="thesis-line">Regulated buyers don't want vibes. <code>garnet build --evidence</code> turns compliance into a compiler artifact.</span>
+          <span class="thesis-line">Regulated buyers don't want vibes. <code>garnet build --evidence</code> — a <span class="ink">planned</span> flag, not in the CLI today — is designed to turn compliance into a compiler artifact.</span>
         </span>
         <span class="expand-mark" aria-hidden="true">+ open</span>
       </button>
       <div class="thesis-body" id="body-evidence" role="region" aria-label="The Evidence, expanded">
         <div class="thesis-body-inner"><div class="thesis-body-content">
           <p>Aerospace, defense, healthcare — the buyers who matter don't accept "an AI reviewed it and said it was fine." A comment thread is not an artifact. A chat log is not a control. When the auditor asks how you <em>know</em>, they mean something you can hand over.</p>
-          <p><code>garnet build --evidence</code> emits a <span class="gold">Verifiable Evidence Bundle</span> with the build: what this code is capable of, what it is incapable of, and the capability declarations those facts derive from. What used to be a consultant engagement — weeks of humans reading documents — becomes a compiler flag that runs on every build.</p>
-          <p>Evidence stops being a deliverable you assemble for the audit and becomes exhaust the toolchain produces anyway. No authority without evidence — and no evidence without a build that proves it.</p>
+          <p>The planned <code>garnet build --evidence</code> flag — no such build flag exists in the shipping CLI today — is designed to emit a <span class="gold">Verifiable Evidence Bundle</span> with the build: what this code is capable of, what it is incapable of, and the capability declarations those facts derive from. What is a consultant engagement today — weeks of humans reading documents — would become a compiler flag that runs on every build.</p>
+          <p>That is the roadmap, not the present: evidence stops being a deliverable you assemble for the audit and becomes exhaust the toolchain produces anyway. Until that flag ships, the runnable surface is <code>garnet caps</code>, <code>garnet diff-caps</code>, and <code>garnet seal</code>. No authority without evidence — and no evidence without a build that proves it.</p>
           <div class="xrefs">
             <span class="xrefs-label">Reads with</span>
             <a class="xref" href="#point-8">№ 8 — Regulation is writing the spec</a>
           </div>
-          <span class="claim enforced">Claim class · shipping flag — bundle format versioned</span>
+          <span class="claim">Claim class · planned — roadmap flag; today's runnable surface is caps / diff&#8209;caps / seal</span>
         </div></div>
       </div>
     </article>
@@ -421,7 +420,7 @@
         <div class="thesis-body-inner"><div class="thesis-body-content">
           <p>For a few years, frontier labs kept a lid on dangerous capability by training their models to refuse. That era is closing. Frontier&#8209;class open&#8209;weight models now ship with little or no safety posture — they will run the offensive task, the swarm audit, the kernel exploit, for anyone, and their weights live on hardware nobody governs.</p>
           <p>You can no longer locate trust in a model's disposition. A refusal is a property of one vendor's checkpoint; your security has to hold across <em>every</em> model your org and your adversaries will ever run. And the trust engineers are now extending to agents is <em>behavioral</em> — earned per checkpoint, per month, per "it cheats less than it used to." It doesn't transfer to the next model, the fine&#8209;tune, or the open&#8209;weight drop. Every unit of earned trust converts directly into expanded autonomy — less human review, longer unattended runs — which raises, not lowers, the cost of the day that trust is wrong. So trust moves down — out of the model, into the <span class="gold">substrate</span> the model's output must pass through.</p>
-          <p>Garnet is that substrate. Enforcement in the type system is model&#8209;agnostic by design: aligned, open, fine&#8209;tuned, or adversarial, no model can talk its way past a capability it was never granted. The less you can trust the author of the code, the more the compiler is worth.</p>
+          <p>Garnet is that substrate. Enforcement in the type system is model&#8209;agnostic by design: a compiler's verdict doesn't depend on which model wrote the code or how persuasively it argues. Today that verdict covers the bounded, pinned surface stated below — not a universal sandbox — and every widening must arrive with its own trap. The less you can trust the author of the code, the more that property is worth.</p>
           <div class="xrefs">
             <span class="xrefs-label">Reads with</span>
             <a class="xref" href="#point-4">№ 4 — The connector explosion is the threat</a>
@@ -447,10 +446,10 @@
         </div>
         <div class="layer-row garnet-row">
           <span class="layer-q">What was it ever allowed to do?</span>
-          <span class="layer-a"><strong>Authority.</strong> Garnet. Capability types enforced at build, delegation that attenuates by construction, evidence emitted with the artifact. Structural, not sampled.</span>
+          <span class="layer-a"><strong>Authority.</strong> Garnet. Capability declarations checked at build — deterministic enforcement proven today on the bounded surface below — delegation designed to attenuate, and an evidence flag on the roadmap. Structural, not sampled.</span>
         </div>
       </div>
-      <p class="layermap-note">Tests are empirical — they check behaviors someone thought to check. Capabilities are structural — they bound behaviors nobody thought of. A swarm can pass every test in the suite while holding authority it never needed; no green checkmark speaks to that. Which is why <code>garnet check</code> runs <em>inside</em> your CI, whoever's runners it's on: the faster the pipeline, the more often the envelope is enforced and the evidence regenerates.</p>
+      <p class="layermap-note">Tests are empirical — they check behaviors someone thought to check. Capabilities are structural — they are built to bound behaviors nobody thought of. A swarm can pass every test in the suite while holding authority it never needed; no green checkmark speaks to that. Which is why <code>garnet check</code> runs <em>inside</em> your CI, whoever's runners it's on: the faster the pipeline, the more often the declared envelope is checked — and, once the evidence flag ships, the more often the evidence would regenerate.</p>
     </section>
 
     <!-- THE HONEST OBJECTION -->
@@ -473,10 +472,11 @@
     </article>
 
     <div class="theses-legend">
-      <span class="g"><i></i>Enforced today</span>
-      <span class="a"><i></i>By construction — proof in progress</span>
+      <span class="a"><i></i>Partial / by construction — proof in progress</span>
+      <span class="p"><i></i>Planned — roadmap</span>
       <span class="b"><i></i>World&#8209;claim</span>
     </div>
+    <p class="theses-scope-note">Nothing in this section claims enforcement beyond the two pinned Gate&#8209;1 claims in the boundary section below.</p>
   </section>
 
   <!-- THE TEN -->
@@ -492,7 +492,7 @@
       <div class="reason" id="point-5"><div class="rn">5</div><div><p class="rh">Diff&#8209;caps keeps a human in control at agent volume</p><p class="rb">When one senior can't read everything ten agents ship, review has to move <em>into</em> the artifact: accept a huge PR by reading one thing — <b>did its authority envelope change?</b> No skill provides that; it's a property of the type system.</p></div></div>
       <div class="reason" id="point-6"><div class="rn">6</div><div><p class="rh">Incumbent fluency ships incumbent footguns</p><p class="rb">An agent writes fluent Python <em>and</em> Python's ambient&#8209;authority footguns, because the language predates capability thinking and bolts safety on coarsely, after the fact. A language built for capability from the type system up makes the safe thing the <b>only expressible</b> thing.</p></div></div>
       <div class="reason" id="point-7"><div class="rn">7</div><div><p class="rh">Garnet is the substrate under the tooling, not a rival to it</p><p class="rb">An MCP tool server <em>written in Garnet</em> declares its authority in its types, so the host verifies the envelope before granting it. "The only standard library where every function's authority is declared and verified" isn't competing with the connector economy — it's the trust layer that economy is missing.</p></div></div>
-      <div class="reason" id="point-8"><div class="rn">8</div><div><p class="rh">Regulation is writing Garnet's spec — with no reference implementation</p><p class="rb">The EU Cyber Resilience Act's reporting obligations land this September; FDA change&#8209;control plans, automotive OTA, and avionics all encode the same test — <em>a change is allowed iff it stays inside a pre&#8209;approved envelope</em>, done by humans reading documents today. <code>garnet build --evidence</code> turns a consultant engagement into a compiler flag.</p></div></div>
+      <div class="reason" id="point-8"><div class="rn">8</div><div><p class="rh">Regulation is writing Garnet's spec — with no reference implementation</p><p class="rb">The EU Cyber Resilience Act's reporting obligations land this September; FDA change&#8209;control plans, automotive OTA, and avionics all encode the same test — <em>a change is allowed iff it stays inside a pre&#8209;approved envelope</em>, done by humans reading documents today. The planned <code>garnet build --evidence</code> flag is designed to turn a consultant engagement into a compiler flag.</p></div></div>
       <div class="reason" id="point-9"><div class="rn">9</div><div><p class="rh">Attenuating delegation across agent swarms</p><p class="rb">When A delegates to B to C, authority should only <em>shrink</em>. Skills don't compose permissions; connectors don't attenuate; prompts can't enforce it. A capability lattice in the type system can make delegation <b>provably</b> narrow authority — load&#8209;bearing as multi&#8209;agent topologies become the default.</p></div></div>
       <div class="reason" id="point-10"><div class="rn">10</div><div><p class="rh">It amortizes senior judgment instead of re&#8209;spending it</p><p class="rb">The crisis the AI&#8209;first model creates is that juniors and agents can't be trusted to ship unreviewed. Garnet doesn't replace senior judgment — one senior defines the envelope <b>once</b>, and the compiler enforces it on every junior and every agent thereafter. Process must be re&#8209;applied every time; a type applies itself.</p></div></div>
     </div>
```
