# GARNET v4.2 — Developer Comprehension Study Protocol

**Companion to:** `Paper_III_Garnet_Synthesis_v2_1.md` §7 ("managed mode feels Ruby-like"), `DX Comparative Paper` §12 ("The Developer's Experience"), `DX Comparative Paper` §19 ("What We Measure vs. What We Argue")
**Phase:** v4.2 Phase 0 (pre-MIT DX rigor)
**Date:** 2026-04-17 (pre-registered, pre-data-collection)
**Author:** Claude Code (Opus 4.7) — Stage 6 boot
**Status:** Normative pre-registered protocol. Execution may run post-submission; the pre-registration is the rigor signal.
**Discipline:** Mirrors `Paper_VI_Empirical_Validation_Protocol.md` (Phase 1C) — the same hypothesis-procedure-criterion-harness-risk structure applied to a design claim rather than a correctness claim.
**Anchor:** *"Prove all things; hold fast that which is good." — 1 Thessalonians 5:21*

---

## Purpose

Paper III §7 and the DX Comparative Paper §12 state a design claim: *Garnet's managed mode feels Ruby-like in ergonomics, and Garnet's safe mode feels Rust-like in rigor.* This claim has been defended by argument — side-by-side code comparisons, phenomenological prose about the developer's experience, and appeals to what a trained reader recognizes when reading the language.

An argument alone is a legitimate category of evidence for a design claim (see DX Paper §19 — "What We Measure vs. What We Argue"). But the rigor discipline Garnet already demonstrated in Paper VI — pre-registering falsifiable criteria and honestly downgrading the contribution when measurement refutes the prediction — extends naturally to the "feels like" claim as well.

This protocol commits Garnet, before any data is collected, to a measurable operationalization of "managed mode feels Ruby-like":

**Operational definition.** A developer strong in Ruby, reading Garnet managed-mode code for the first time, answers code-comprehension questions at an accuracy within 10 percentage points of the same developer's accuracy on equivalent Ruby code.

If the measured data refutes this operationalization, Paper III §7's stronger claim gets honestly downgraded — exactly as Paper VI Contributions 3 and 4 were downgraded in the v4.0 Phase 4A execution report when their measured outcomes missed their pre-registered thresholds.

Pre-commitment matters. **This protocol is written before recruitment, before task authoring, before any data collection.** Anything added to "rescue" a failed hypothesis after data lands is a post-hoc rationalization, not science.

---

## Scope and non-scope

### In scope

- A single controlled comparison: comprehension accuracy + time-to-correct-answer on short code-reading tasks, across three languages, with each developer serving as their own control.
- Pre-registration of the statistical threshold that distinguishes "managed mode feels Ruby-like" (supported) from "managed mode does not feel Ruby-like" (refuted) from "inconclusive" (underpowered).
- A measurement harness specified precisely enough that a different team could replicate the study and reach a comparable conclusion.

### Explicitly out of scope

- **Code authoring (generation) comparisons.** Paper VI Contribution 1 pre-registers LLM-generation comparisons (pass@1 study); a separate protocol can cover human-authoring velocity in a future phase. This study is reading-only.
- **Long-form ergonomics claims.** "Developers feel joy writing Garnet after three months of daily use" is a longitudinal claim requiring a different study design. This protocol is a short-form comprehension measurement only.
- **Safe-mode-feels-Rust-like.** Paper VII's engineering ladder carries Rust-expertise developer feedback through the FFI + Cargo-style tooling chapters; a parallel protocol for the safe-mode-feels-Rust-like claim can mirror this one. This protocol is scoped to the managed-mode-feels-Ruby-like claim to keep the sample budget tractable.

---

## Participants

### Recruitment criteria

- **N = 5 developers.** Small sample deliberately; the cost of each recruited participant is high, and the effect size we are testing is large (10 percentage points of comprehension accuracy is not a subtle effect). With N = 5 and within-subject design (each developer serves as their own control), detectable effects at α = 0.05 are bounded by the paired-sample variance, not the between-subject variance. Power analysis below.
- **Experience requirement.** Each participant has ≥3 years of professional or substantial open-source experience in either Rust or Ruby (their "native language" for this study). Mixed-expertise developers count toward whichever language they self-identify as stronger.
- **Recruitment balance.** 3 Ruby-strong developers + 2 Rust-strong developers, or 2 Ruby-strong + 3 Rust-strong — the protocol commits in advance to recruiting either split and declares results separately by native language. Both splits are acceptable; the split is documented in the execution report, not optimized after recruitment.
- **Garnet experience.** All participants must be Garnet-naive (have not authored more than 100 LOC of Garnet and have not read Paper III or the DX Comparative Paper in full). 20 minutes of pre-task Garnet familiarization is provided: the Mini-Spec §1 "Hello, Function" through §4 "Data with Behavior".

### Incentive

- $100 gift card per participant, paid on study completion regardless of outcome. This is standard practice for HCI comprehension studies and avoids any incentive to perform in a particular direction.

### Consent and ethics

- Each participant reads and signs a single-page consent form stating: the study measures code comprehension across three languages, takes ~60 minutes, is recorded (screen only, no audio/video of the participant), and is used to inform a research paper at MIT. Participants may withdraw at any time; withdrawn data is deleted.
- No personally-identifying information is retained; participants are referred to by participant ID (P01–P05) in the execution report.

---

## Task corpus

### Task count and shape

- **6 code-comprehension tasks, each rendered in all three languages (Ruby, Rust, Garnet managed-mode).** 18 total stimuli.
- Each task presents a short (20–60 LOC) code snippet and asks the participant to answer ONE multiple-choice question about the code's behavior. The question has exactly one correct answer and three plausible distractors. Multiple-choice is chosen over free-response to eliminate grading subjectivity and keep the measurement apparatus fully mechanical.
- Tasks are counterbalanced (Latin square): each participant sees each task in only one language, and across the 5 participants every (task, language) pair is covered at least once.

### Difficulty calibration

- Tasks span two difficulty tiers: 3 tasks at "basic" (understanding a single function's return value given inputs) and 3 tasks at "intermediate" (understanding a data transformation across 2–3 functions with one branch).
- Tasks are authored by Claude (pre-registration draft) and reviewed by a Ruby-senior and a Rust-senior engineer (external to this project) for "same difficulty in all three languages" before recruitment begins. Any task flagged as language-biased by either reviewer is rewritten or dropped.

### Task topics (pre-committed)

| # | Topic                              | Tier         | What it tests                          |
|---|------------------------------------|--------------|----------------------------------------|
| 1 | String-split into key/value pairs  | basic        | Return value of a parser given input   |
| 2 | Filter + map over a list           | basic        | Identify which elements survive        |
| 3 | Struct with one method             | basic        | Method return for specific instance    |
| 4 | Error propagation across 2 funcs   | intermediate | Which path a failing call takes        |
| 5 | Recursive tree traversal           | intermediate | Depth-first visit order                |
| 6 | Small actor/receiver message pass  | intermediate | Final state after two messages         |

Every task is written in each language using the IDIOMATIC style that a senior developer in that language would accept in a code review — NOT a transliteration style. For Garnet managed-mode, the idiomatic style matches the examples in the DX Comparative Paper §§2–7.

### Stimulus randomization

- Task order is randomized per participant using a fixed seed (derived from participant ID) so it is reproducible. Language-per-task assignment follows the Latin square (pre-computed).
- No participant sees the same TASK twice in different languages — only different tasks in different languages. This avoids the confound of remembering the answer from a prior language.

---

## Measurements

### Primary measure: comprehension accuracy

- **Accuracy** = fraction of multiple-choice answers correct, per (participant, language). Each participant contributes 6 total answers distributed across 3 languages (2 per language by Latin square construction).
- Primary inferential question: *For Ruby-strong developers, is `accuracy(Garnet_managed) ≥ accuracy(Ruby) − 0.10`?*

### Secondary measure: time-to-correct-answer

- **Time-to-correct-answer** = wall-clock seconds from task stimulus appearing to correct multiple-choice selection submitted. Incorrect answers have no time measurement recorded (treated as missing; accuracy measure captures the correctness outcome).
- Secondary inferential question: *Is `median_time(Garnet_managed) ≤ 1.25 × median_time(Ruby)` for Ruby-strong developers on basic-tier tasks?*

### Self-reported tertiary measure: subjective-familiarity rating

- After each task, the participant rates "how familiar did this code feel?" on a 5-point Likert (1 = completely foreign, 5 = totally familiar).
- Tertiary inferential question: *Is the mean subjective-familiarity rating of Garnet managed-mode code from Ruby-strong developers ≥ 3.5 (the "somewhat familiar" threshold)?*

---

## Hypothesis (H)

### H-primary

**Accuracy equivalence (managed mode vs. Ruby for Ruby-strong developers).**

For Ruby-strong participants, mean comprehension accuracy on Garnet managed-mode tasks is within 10 percentage points of their accuracy on Ruby tasks:

> `H: mean(accuracy_Garnet_managed_for_Ruby_strong) ≥ mean(accuracy_Ruby_for_Ruby_strong) − 0.10`

A one-sided 90% confidence interval on the difference `accuracy_Ruby − accuracy_Garnet_managed` constructed via paired bootstrap (10,000 resamples) must have its upper bound ≤ 0.10.

### H-secondary

**Time overhead bounded (managed mode vs. Ruby for Ruby-strong developers, basic-tier only).**

> `H-secondary: median(time_Garnet_managed_basic_Ruby_strong) ≤ 1.25 × median(time_Ruby_basic_Ruby_strong)`

A 25% time-overhead ceiling, acknowledging that reading an unfamiliar-yet-familiar language is slightly slower than reading one's native language. This is measured only on basic-tier tasks, where the task complexity is low enough that syntactic friction dominates conceptual load.

### H-tertiary

**Subjective familiarity above neutral.**

> `H-tertiary: mean(familiarity_rating_Garnet_managed_Ruby_strong) ≥ 3.5`

Above 3.5 on the 5-point scale corresponds to "somewhat familiar" and above. This is the softest of the three; the primary and secondary measures are the load-bearing evidence.

---

## Pass / fail criterion (C)

### For H-primary

- **Supported.** The 90% upper confidence bound on `(Ruby_acc − Garnet_acc)` for Ruby-strong participants is ≤ 0.10.
- **Refuted.** The 90% upper confidence bound is > 0.10 AND the point estimate exceeds 0.10 (i.e., Garnet is directionally worse AND the gap is significantly larger than the 10-pp threshold).
- **Inconclusive.** The confidence interval straddles 0.10 (point estimate near the threshold; the sample is too small to distinguish "within 10 pp" from "beyond 10 pp" reliably).

### For H-secondary

- **Supported.** Median Garnet-managed-basic time ≤ 1.25 × median Ruby-basic time.
- **Refuted.** Median Garnet-managed-basic time > 1.50 × median Ruby-basic time (a 50%+ overhead is directionally too slow regardless of power).
- **Inconclusive.** Between 1.25× and 1.50×.

### For H-tertiary

- **Supported.** Mean familiarity ≥ 3.5.
- **Refuted.** Mean familiarity < 3.0 (below neutral).
- **Inconclusive.** Between 3.0 and 3.5.

### Aggregate verdict

- **Fully supported.** All three primary/secondary/tertiary hypotheses supported.
- **Partially supported.** Primary supported; secondary or tertiary inconclusive.
- **Primary refuted.** H-primary refuted regardless of secondary/tertiary. In this case Paper III §7's claim that "managed mode feels Ruby-like" must be downgraded to "managed mode is legible to Ruby developers but requires [N percentage points] additional cognitive effort on comprehension tasks; the 'feels Ruby-like' claim as stated in prior revisions is not supported by the N=5 Phase-0 measurement." The corresponding sentences in the DX Comparative Paper §§12, 14 must be updated in lockstep.

---

## Protocol (P)

### Pre-study phase (per participant)

1. **Consent form signed.** (~5 min)
2. **Self-assessment completed.** Participant self-rates expertise (3+ years, 5+ years, 10+ years) in Ruby, Rust, Garnet, and names their primary language over the past 12 months. (~3 min)
3. **Garnet familiarization.** Participant reads Mini-Spec §§1–4 silently (no interaction with the authors). A soft cap of 20 minutes. Screen recording active throughout. (~20 min)

### Study phase (per participant)

4. **Task delivery.** Tasks appear one at a time in a simple web harness. Each task shows:
   - The code snippet (20–60 LOC) in monospace font with syntax highlighting appropriate to the language.
   - The multiple-choice question below with exactly 4 options.
   - A "Submit" button that records the answer and timestamp.
   - No "back" button — answers are final once submitted (this prevents the participant from revising answers after later tasks give them more context).
5. **Inter-task rest.** 15-second forced break between tasks, during which the participant cannot see the next stimulus. (6 tasks × 60s avg + 5 breaks × 15s = ~7 min of active task time)
6. **Post-task familiarity rating.** After each task's Submit, the 5-point Likert rating appears briefly (~10 seconds per task; does not count toward time-to-correct-answer).

### Post-study phase (per participant)

7. **Free-response debrief (optional, not analyzed statistically).** Participant may leave a 1–3-sentence comment on any task. Qualitative data; used to surface outlier tasks for post-hoc exclusion, NOT to retune analysis. If a task is excluded, the exclusion rule is pre-committed: if ≥2 of 5 participants flag the same task as ambiguous or miscalibrated, that task is dropped from analysis for ALL participants. Otherwise no exclusion.

### Schedule

- Target run: single-session, ~60 minutes wall-clock per participant, via Zoom screenshare with the harness running locally on the participant's machine. All 5 participants are run within a 10-day window to minimize time-drift in the language landscape.

### Data retention

- Raw recording + answer CSV stored in a private repository accessible only to the study authors. Aggregate CSV (participant_id, task_id, language, accuracy, time_ms, familiarity) published in the execution report. Raw screen recordings destroyed 90 days after publication per the consent form's data-minimization clause.

---

## Measurement harness (M)

### Code layout

```
benchmarks/
  v4_2_developer_comprehension/
    tasks/
      01_split_kv/
        code_ruby.rb             # stimulus
        code_rust.rs             # stimulus
        code_garnet.garnet       # stimulus
        question.yaml            # text + 4 options + correct_index
        rubric.md                # pre-registered difficulty + correct answer
      02_filter_map/
      03_struct_method/
      04_error_prop/
      05_tree_traversal/
      06_actor_messages/
    harness/
      serve.py                   # local web server delivering tasks in random order
      latin_square.py            # computes per-participant language-per-task assignment
      record.py                  # logs answers + timings to a local JSONL file
    analyze/
      bootstrap_ci.py            # paired bootstrap for H-primary CI
      timing_analysis.py         # median + ratio for H-secondary
      likert_analysis.py         # mean + CI for H-tertiary
      generate_report.py         # produces execution report
```

### Output

- Per-participant JSONL file: one line per task, with fields `{participant_id, task_id, lang, correct, time_ms, familiarity, timestamp}`.
- Aggregate CSV: all participants concatenated.
- Execution report: `F_Project_Management/GARNET_v4_2_Developer_Comprehension_Study_EXECUTION.md` — written after data lands, with all three hypotheses' verdicts (supported / refuted / inconclusive) and any honest downgrades of upstream claims.

### Reproducibility

- All code, task stimuli, and the Latin-square assignment algorithm are published in the Garnet monorepo. A different team running this harness on their own 5 participants can compare their result to ours.

---

## Expected risk (R)

### If H-primary refuted

Paper III §7 and DX Comparative Paper §§12, 14 are updated in lockstep to state honestly:

> "Garnet's managed mode is legible to Ruby developers but comprehension accuracy on short tasks (N=5, Phase-0 study) is [X] percentage points below native Ruby. The 'feels Ruby-like' claim as presented in prior revisions is not supported by the measurement; the weaker claim 'managed mode is legible to Ruby developers with a measurable but bounded comprehension gap' is supported."

This is the same discipline v4.0 Paper VI revisions applied to Contributions 3 and 4 — the claim is downgraded to match the measured outcome. No rescue, no post-hoc threshold adjustment.

### If H-primary supported but H-secondary refuted

A specific honest addition to Paper III §7: "Managed mode reaches native-comparable comprehension accuracy but with [X]% time overhead on basic tasks. This is consistent with the claim that managed-mode Garnet is 'familiar-but-not-identical' to Ruby — the reader reaches the right answer but takes slightly longer to do so."

### If all three inconclusive

Paper III §7 retains its current wording AND the DX Comparative Paper §19 ("What We Measure vs. What We Argue") gains a line: "The Phase-0 N=5 comprehension study produced an inconclusive result; a larger follow-up (N≥20) is required to distinguish supported from refuted on the 10-pp threshold."

### If any task is flagged as miscalibrated by ≥2 participants

That task is dropped from analysis per the pre-committed exclusion rule. The execution report documents the exclusion, the flagging participants' comments (anonymized), and the result computed on the reduced task set. The full-task-set result is also reported for transparency; the reduced-set result is the primary evidence.

### Power-analysis caveat (pre-committed)

N=5 is small. The within-subject design helps, but with 2 tasks per language per participant, the per-language effective N is 10 observations per language (5 participants × 2 tasks). Detecting a 10-pp accuracy difference at 90% confidence requires a per-observation variance σ ≤ 0.13 on the accuracy-per-task metric (which is Bernoulli-bounded at σ_max = 0.5, so this is a strong assumption).

**Concretely: if the true gap is exactly 10 pp, this study will likely return "inconclusive" — not "refuted". The study cannot conclusively refute the H-primary claim at N=5.** This is acknowledged in advance. The study's pre-commitment value is in the DIRECTION of its finding, not the statistical power — a point estimate near zero plus a tight CI would be strong soft evidence for H-primary; a point estimate near 20 pp plus a tight CI would be strong soft evidence for refutation. An effect size measurably larger than 10 pp WILL register even at N=5.

A follow-up N≥20 study is a natural post-MIT future-work item if Phase-0 returns inconclusive or partially supported; the pre-registration ensures both Phase-0 and the follow-up measure against the same pre-committed threshold, rather than a threshold reverse-engineered from the data.

---

## Relation to other Garnet rigor artifacts

- **Paper VI Empirical Validation Protocol (Phase 1C).** This study adopts the same hypothesis-procedure-criterion-harness-risk structure. Paper VI measured correctness properties (pass@1, expressiveness ratio, hot-reload latency, etc.); this protocol extends the same discipline to a developer-experience property.
- **DX Comparative Paper §19 "What We Measure vs. What We Argue".** This protocol is the concrete companion to the §19 claim that design properties are not unmeasurable — they are measurable with a different form of evidence, and the evidence form has to match the property's category. Comprehension accuracy IS a measurable proxy for "legibility"; it is not a proxy for "joy" (which remains in the argued category).
- **v3.3 Slop Reverification.** That audit found 5 cases where green tests didn't actually prove the claim they were cited against. This protocol avoids that failure mode by pre-committing the operational definition: an accuracy within 10 pp is NOT a proxy for "feels Ruby-like" in the full phenomenological sense — it is a proxy for "a Ruby developer's comprehension is not substantially worse in Garnet than in Ruby". The paper-level language is updated (or held) based on exactly what the measurement supports.

---

## Deliverables at study completion

1. `F_Project_Management/GARNET_v4_2_Developer_Comprehension_Study_EXECUTION.md` — the execution report, parallel to `GARNET_v4_0_PAPER_VI_EXECUTION.md`. Contains:
   - Participant demographics (anonymized to P01–P05 + native language tag).
   - Per-language accuracy, time, and familiarity tables.
   - Paired-bootstrap confidence intervals for H-primary.
   - Verdicts: supported / refuted / inconclusive for each hypothesis.
   - A diff summary of any Paper III / DX Paper changes made as a result.
2. Updated `Paper_III_Garnet_Synthesis_v2_1.md` §7 (if claim downgrade triggered).
3. Updated `GARNET_v4_2_DX_Comparative_Paper.docx` §§12, 14 (if claim downgrade triggered).
4. Raw aggregate CSV published alongside the execution report.
5. `benchmarks/v4_2_developer_comprehension/` monorepo harness checked in.

---

## Ship discipline

**This protocol ships at v4.2 regardless of whether the study itself runs pre-submission.** The pre-registration IS the rigor signal — it commits Garnet to a specific, falsifiable operationalization of a claim currently defended only by argument, and it commits the project to updating the upstream claim if the measurement refutes the prediction. That commitment is visible to MIT reviewers as soon as this file is in the corpus; the execution report can follow on a slower cadence (Summer 2026 plausible target) without blocking the v4.2 submission.

Reviewers who want to execute the protocol themselves have everything they need: the recruitment criteria, the task topics, the measurement harness specification, the pass/fail thresholds, and the downgrade discipline if the data refutes the claim. A hostile reviewer running the same study on 5 different participants and getting a different result than our eventual execution report would be exactly the kind of independent replication the pre-registration invites.

---

*"Test all things; hold fast that which is good." — 1 Thessalonians 5:21*

*Written by Claude Opus 4.7 at the Stage 6 boot — 2026-04-17. Pre-registered before any data collection.*
