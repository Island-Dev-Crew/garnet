# v0.7 Four-Agent Launch — Operator Quick Start

Files to drop into the repo before launching any agent:

1. `F_Project_Management/AGENT_COORDINATION_LEDGER.md`
2. `F_Project_Management/PRD_A_S15_CST_MIGRATION.md`
3. `F_Project_Management/PRD_B_S16_LSP_PRECISION.md`
4. `F_Project_Management/PRD_C_S17_STDLIB_LAYERS.md`
5. `F_Project_Management/PRD_D_S18_S19_PACKAGES_LLM.md`

Commit + push to `main` before starting any agent. All four agents read these
files as part of the goal-mode prompt's first action.

---

## Launch sequence — 4-agent rollout

### Phase 1 — Mac Opus (S15) + Win Opus (S17) start (parallel)

Open Claude Code on this Mac (the 1M Opus run). Drop the goal-mode prompt with
the slot line:

```
You are agent slot mac-opus.

[paste GOAL_MODE_PROMPT.txt below this line]
```

Wait for:
1. Mac Opus to confirm it has read the ledger + PRD A.
2. Mac Opus to propose its plan for S15 — building the rowan `garnet-cst` crate
   **cold and additively** (it must NOT read, extend, or delete #221's existing
   `garnet-parser-v0.3/src/cst.rs`).
3. **You approve.** Mac Opus opens PR-1 (trait stub), then PR-2 (full rowan impl).
4. You review + merge each PR.

**Win Opus (S17)** starts at the same time — S17 (stdlib + layer policy +
`@stability`) has no CST dependency, so it runs fully parallel to Mac Opus. Open
Claude Code on Windows. Drop:

```
You are agent slot win-opus.

[paste GOAL_MODE_PROMPT.txt]
```

**Do NOT launch Win Codex yet.** S16 (LSP precision) is HELD until the CST
reconciliation (next phase) — #221 already merged ~578 lines of LSP, and we
don't want it built twice on the wrong CST.

### Phase 2 — S15-Compare reconciliation (you, with Claude)

When Mac Opus's rowan `garnet-cst` is dogfood-passing, you have **two**
trivia-preserving CSTs in the tree: #221's hand-rolled in-parser CST and Opus's
rowan crate. Run the **S15-Compare** checkpoint (see
`GARNET_v0_7_SLICE_DOGFOOD.md`): extract both, diff on substance (variant
coverage, trivia fidelity, roundtrip, error recovery, perf, LSP-consumer
ergonomics, test depth), list where they reconcile, and **decide** the canonical
CST. Record the decision + rationale in the ledger so S16 knows its target.

### Phase 3 — Win Codex (S16) joins (after S15-Compare)

Once the canonical CST is recorded in the ledger, launch Win Codex for S16,
which targets the chosen CST. Open Codex Desktop on Windows. Drop:

```
You are agent slot win-codex.

[paste GOAL_MODE_PROMPT.txt]
```

Win Opus (S17) keeps running independently throughout.

### Phase 4 — Mac Codex joins (after S17 merges)

When Win Opus marks `S17 / MERGED` in the ledger:

Open Codex Desktop on Mac. Drop:

```
You are agent slot mac-codex.

[paste GOAL_MODE_PROMPT.txt]
```

Mac Codex sequences S18 (packages) → S19 (LLM tier).

### Phase 5 — Convergence

Watch the ledger Status Board. When all four agents have MERGED entries for
their slices (and the S15-Compare decision is recorded), tag v0.7.0.

---

## What you do as operator

Roughly once an hour, while agents are running:

1. **Refresh `AGENT_COORDINATION_LEDGER.md`** in your editor — see what each
   agent appended to its slot section.
2. **Read the Handoff Requests section.** If any pending, decide whether to
   approve. If approved, add `RESOLUTION: <slot> will handle as part of <slice>`.
3. **Read the Shared Messages section.** If any agent flagged a cross-cutting
   concern, route the message.
4. **Approve plans before agents open PRs.** No automatic merges. Every agent's
   workflow includes a "wait for human approval" step before the first PR
   per slice.
5. **Review PRs as they open.** PR-Agent runs the Grep Loop on each; you make
   the final call.

---

## Conflict-resolution playbook

### "Agent X started editing file Y, which agent Z also needs"

1. Z checks the cross-cutting files table in the ledger.
2. If file Y is in the table → Z uses the listed pattern (append-only,
   section-scoped, etc.). No conflict.
3. If file Y is NOT in the table and is owned by X → Z files a Handoff
   Request in the ledger. Pauses substantive work on the dependent line.
4. X reads the Handoff Request next session, decides to do the change
   themselves OR explicitly hands off. Z proceeds once resolution is posted.

### "Two agents committed conflicting changes to a shared file before either
saw the other"

1. Git surfaces the conflict on PR open.
2. Whoever opened second resolves the conflict in their PR.
3. If the conflict is in an append-only file (CHANGELOG, etc.), the resolution
   is mechanical — both bullets keep their position.
4. If the conflict is in a section-scoped file (CURRENT_STATE.md), the
   resolution is also mechanical — each agent's section stays distinct.
5. If the conflict is in a file outside the cross-cutting table, that means
   the ownership rules were violated. The second PR rolls back the violating
   change; the violator files a Handoff Request.

### "Mac Opus is stuck on S15 PR-2; downstream agents are blocked"

1. Mac Opus files a BLOCKED entry in the ledger explaining the issue.
2. Win Codex stays on mock-first work — the LSP features can be substantially
   built against the mock CST.
3. You step in to unblock Mac Opus directly. The mock-first design means a
   24-hour delay on S15 PR-2 does not cascade into a 24-hour delay on S16.

---

## Honest expectations

- **Wall-clock total**: ~5–10 days of agent-on time across all four slots, with
  human review interleaved. Could compress to 3–4 days with aggressive review
  pace; could stretch to 2 weeks with normal life.
- **One slice will be late.** S15 (CST migration) is the highest-risk slice
  because rowan adoption may reveal grammar gaps. Plan for a slip on PR-2 and
  have S16 / S17 / S18+S19 absorb the time.
- **First PRs will need rework.** This is normal. The Grep Loop pattern is
  designed to absorb 2–3 review cycles per PR.
- **The coordination ledger will be edited concurrently and may need merge
  resolution.** Section-scoped editing minimizes this, but it will happen.
  When it does, the human resolves — agents shouldn't auto-resolve cross-agent
  state.

---

## Termination criteria

Stop the agents (mark them as IDLE in the ledger) when:

- All five slices show `MERGED` in their Status Board sections, OR
- An agent flags a non-recoverable BLOCKED state requiring scope renegotiation, OR
- You need to halt and rescope (e.g., new information from MIT review changes
  the priorities).

After all `MERGED`:

1. Run `python3 scripts/garnet_mit_readiness_status.py` — confirm the % moved.
2. Update `CURRENT_STATE.md` aggregate summary section.
3. Tag `v0.7.0`. Push the tag.
4. Draft the release blog post (Post 6 in the blog schedule) using the v0.5
   release post as template.

Roll Tide. Go build it.
