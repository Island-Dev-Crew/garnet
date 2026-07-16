#!/usr/bin/env node
/**
 * render_sotu.mjs — Mission Control State of the Union renderer.
 *
 * Reads state.json (+ journal.md if present) from its own directory and writes
 * state-of-the-union.html next to them. Dependency-free, Node >= 18.
 *
 * Usage:  node ops/mission/render-sotu.mjs
 * The HTML is GENERATED — never hand-edit it. Change state.json and re-render.
 *
 * The renderer degrades gracefully on most missing fields and warns on stderr
 * when it defaults something important. A nonempty mainline checkpoint is the
 * exception: it must be internally consistent or rendering fails closed.
 */
import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const statePath = join(here, "state.json");
const journalPath = join(here, "journal.md");
const outPath = join(here, "state-of-the-union.html");

if (!existsSync(statePath)) {
  console.error(`No state.json found at ${statePath}`);
  process.exit(1);
}

let state;
try {
  // Strip a UTF-8 BOM if present — PowerShell 5.1's `-Encoding utf8` writes one,
  // and JSON.parse rejects it. Windows sessions hit this constantly.
  state = JSON.parse(readFileSync(statePath, "utf8").replace(/^﻿/, ""));
} catch (e) {
  console.error(`state.json is not valid JSON: ${e.message}`);
  process.exit(1);
}

// ---------- helpers ----------
const esc = (s) =>
  String(s ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");

// Only http(s) URLs become links; anything else renders as text.
const safeUrl = (u) => (/^https?:\/\//i.test(String(u ?? "")) ? esc(u) : null);

const warn = (msg) => console.error(`[render-sotu] warning: ${msg}`);

const m = state.mission ?? {};
const phases = Array.isArray(state.phases) ? state.phases : [];
const metrics = Array.isArray(state.metrics) ? state.metrics : [];
const risks = Array.isArray(state.risks) ? state.risks : [];
const prLog = Array.isArray(state.prLog) ? state.prLog : [];
const resume = state.resume ?? {};
const policies = state.policies ?? {};
const mainlineCheckpoint = state.mainlineCheckpoint ?? {};
const archivedRange = mainlineCheckpoint.archivedFirstParentRange ?? {};
const archivedPrOrder = Array.isArray(archivedRange.firstParentPullRequestMergeOrder)
  ? archivedRange.firstParentPullRequestMergeOrder
  : [];
const archivedPrToSha = archivedRange.pullRequestToMainSha ?? {};
const successorArchiveMerge = mainlineCheckpoint.successorArchiveMerge ?? {};

function checkpointConsistencyErrors() {
  if (!mainlineCheckpoint || typeof mainlineCheckpoint !== "object" || Array.isArray(mainlineCheckpoint)) {
    return ["mainlineCheckpoint must be an object"];
  }
  if (!Object.keys(mainlineCheckpoint).length) return [];

  const errors = [];
  const shaPattern = /^[0-9a-f]{40}$/;
  const mappingIsObject = archivedPrToSha && typeof archivedPrToSha === "object" && !Array.isArray(archivedPrToSha);
  if (!Array.isArray(archivedRange.firstParentPullRequestMergeOrder) || !archivedPrOrder.length) {
    errors.push("archived first-parent merge order must be a nonempty array");
  }
  if (!mappingIsObject) errors.push("archived PR-to-main SHA mapping must be an object");
  if (archivedRange.firstParentPullRequestCommitCount !== archivedPrOrder.length) {
    errors.push("archived declared count does not match merge-order length");
  }
  if (new Set(archivedPrOrder).size !== archivedPrOrder.length) {
    errors.push("archived merge order contains duplicate PRs");
  }

  const orderKeys = new Set(archivedPrOrder.map(String));
  if (mappingIsObject) {
    for (const pr of archivedPrOrder) {
      const sha = archivedPrToSha[String(pr)];
      if (!shaPattern.test(String(sha ?? ""))) {
        errors.push(`archived PR #${pr} has a missing or invalid full main SHA`);
      }
    }
    for (const key of Object.keys(archivedPrToSha)) {
      if (!orderKeys.has(key)) errors.push(`archived SHA mapping has extra PR #${key}`);
    }
  }

  const successorPr = successorArchiveMerge.pullRequest;
  if (!Number.isInteger(successorPr)) errors.push("successor archive PR must be an integer");
  if (!shaPattern.test(String(successorArchiveMerge.mainSha ?? ""))) {
    errors.push("successor archive merge has a missing or invalid full main SHA");
  }
  if (successorArchiveMerge.immediatelyFollowsArchivedRange !== true) {
    errors.push("successor archive merge must immediately follow the archived range");
  }

  const fullOrder = mainlineCheckpoint.checkpointFirstParentPullRequestMergeOrder;
  const expectedFullOrder = [...archivedPrOrder, successorPr];
  if (!Array.isArray(fullOrder)) {
    errors.push("full checkpoint merge order must be an array");
  } else {
    if (new Set(fullOrder).size !== fullOrder.length) {
      errors.push("full checkpoint merge order contains duplicate PRs");
    }
    if (fullOrder.length !== expectedFullOrder.length || fullOrder.some((pr, i) => pr !== expectedFullOrder[i])) {
      errors.push("full checkpoint merge order does not equal archived order plus successor");
    }
    if (mainlineCheckpoint.checkpointPullRequestCommitCount !== fullOrder.length) {
      errors.push("full checkpoint declared count does not match merge-order length");
    }
  }
  return errors;
}

const checkpointErrors = checkpointConsistencyErrors();
if (checkpointErrors.length) {
  console.error("[render-sotu] mainlineCheckpoint invalid:");
  for (const error of checkpointErrors) console.error(`  - ${error}`);
  process.exit(1);
}

const repoRoot = resolve(here, "..", "..");
const pythonExecutable = process.platform === "win32" ? "python" : "python3";
const truthFreezeGate = spawnSync(
  pythonExecutable,
  [
    "-I",
    join(repoRoot, "scripts/garnet_lane0_truth_freeze_status.py"),
    "--repo-root",
    repoRoot,
    "--state",
    statePath,
    "--plan",
    join(repoRoot, "ops/lane0/plan.lock.json"),
    "--gate",
  ],
  { cwd: repoRoot, encoding: "utf8" },
);
if (truthFreezeGate.error || truthFreezeGate.status !== 0) {
  console.error("[render-sotu] Lane 0 truth-freeze gate failed; refusing to emit HTML.");
  if (truthFreezeGate.error) console.error(`  ${truthFreezeGate.error.message}`);
  if (truthFreezeGate.stdout?.trim()) console.error(truthFreezeGate.stdout.trim());
  if (truthFreezeGate.stderr?.trim()) console.error(truthFreezeGate.stderr.trim());
  process.exit(1);
}

const checkpointRows = [
  ...archivedPrOrder.map((pr) => ({ pr, sha: archivedPrToSha[String(pr)], scope: "archived range" })),
  ...(successorArchiveMerge.pullRequest != null
    ? [{ pr: successorArchiveMerge.pullRequest, sha: successorArchiveMerge.mainSha, scope: "successor archive merge" }]
    : []),
];

for (const p of phases) {
  if (!p.status) warn(`phase ${p.id ?? "?"} has no status — defaulting to "pending"`);
}

const allTasks = phases.flatMap((p) => p.tasks ?? []);
const doneTasks = allTasks.filter((t) => t.status === "done").length;
const allGates = phases.flatMap((p) => p.gates ?? []);
const passedGates = allGates.filter((g) => g.status === "passed").length;
const pct = allTasks.length ? Math.round((doneTasks / allTasks.length) * 100) : 0;

const phaseTone = { pending: "", in_progress: "amber", blocked: "red", done: "green" };
const gateTone = { pending: "", passed: "green", failed: "red", waived: "amber" };
const taskMark = { pending: "&#9675;", in_progress: "&#9689;", blocked: "&#9888;", done: "&#9679;" };
const sevTone = { high: "red", critical: "red", medium: "amber", low: "blue" };

function journalTail(n = 12) {
  if (!existsSync(journalPath)) return [];
  const entries = readFileSync(journalPath, "utf8")
    .split(/^## /m)
    .filter((s) => s.trim())
    .map((s) => "## " + s.trim());
  return entries.slice(-n).reverse();
}

// ---------- resume prompt generation ----------
function coldStartChecklist() {
  return [
    `Read ops/mission/state.json FIRST — it is the source of truth, not this prompt.`,
    `Verify reality matches state: git log --oneline -5, and re-run the active phase's gate commands.`,
    `Follow the loop: pick next unblocked task -> branch ${policies.branchPrefix ?? "mission/"}<phase>-<slug> -> implement -> run gates -> PR -> merge per policy '${policies.merge ?? "review"}' -> update state.json -> node ops/mission/render-sotu.mjs -> commit state+journal+html.`,
    `Never hand-edit state-of-the-union.html. Never merge without fresh gate evidence in ops/mission/evidence/.`,
    `End of session: update resume.nextActions for a stranger, append journal entry, render, commit.`,
  ];
}

// Sections are null when skipped; empty strings are intentional blank-line spacers.
function joinPrompt(lines) {
  return lines.filter((l) => l !== null).join("\n");
}

function conventionLines() {
  const conv = Array.isArray(resume.conventions) ? resume.conventions : [];
  if (!conv.length) return null;
  return `Mission conventions:\n${conv.map((c) => `  - ${c}`).join("\n")}`;
}

function resumePromptFor(phase) {
  const gatesTodo = (phase.gates ?? [])
    .filter((g) => g.status !== "passed")
    .map((g) => `  - [${g.status ?? "pending"}] ${g.title ?? g.id ?? "?"}  ->  ${g.command ?? "(no command recorded)"}`);
  const tasksTodo = (phase.tasks ?? [])
    .filter((t) => t.status !== "done")
    .map((t) => `  - [${t.status ?? "pending"}] ${t.id ?? "?"}: ${t.title ?? ""}${t.notes ? ` (${t.notes})` : ""}`);
  const isActive = resume.activePhase === phase.id;
  const next = (isActive ? resume.nextActions ?? [] : []).map((a) => `  - ${a}`);
  const blockers = (isActive ? resume.blockers ?? [] : []).map((b) => `  - ${b}`);

  return joinPrompt([
    `You are resuming the "${m.name ?? "mission"}" mission at phase ${phase.id ?? "?"}: ${phase.title ?? ""}.`,
    ``,
    `Mission: ${m.tagline ?? ""}`,
    `North star: ${m.northStar ?? ""}`,
    `Repo: ${m.repo ?? "?"} (local: ${m.repoPath ?? "?"}), default branch: ${m.defaultBranch ?? "main"}.`,
    `Governing plan: ${m.planDoc ?? "?"}`,
    ``,
    `Phase goal: ${phase.goal ?? ""}`,
    tasksTodo.length ? `Remaining tasks:\n${tasksTodo.join("\n")}` : `All tasks in this phase are done.`,
    gatesTodo.length ? `Gates not yet passed:\n${gatesTodo.join("\n")}` : `All gates passed — close the phase and advance.`,
    next.length ? `Next actions (from last session):\n${next.join("\n")}` : null,
    blockers.length ? `Known blockers:\n${blockers.join("\n")}` : null,
    conventionLines(),
    ``,
    `Operating procedure:`,
    ...coldStartChecklist().map((c) => `  ${c}`),
  ]);
}

const globalPrompt = joinPrompt([
  `You are cold-starting the "${m.name ?? "mission"}" mission with zero prior context.`,
  ``,
  `Local repo: ${m.repoPath ?? "?"}  (${m.repo ?? "?"}, default branch ${m.defaultBranch ?? "main"})`,
  `Governing plan: ${m.planDoc ?? "?"}`,
  `Mission status: ${m.status ?? "?"} — active phase: ${resume.activePhase ?? "?"}`,
  ``,
  conventionLines(),
  conventionLines() ? `` : null,
  `Operating procedure:`,
  ...coldStartChecklist().map((c) => `  ${c}`),
]);

// ---------- component renderers ----------
const chip = (text, tone = "") => `<span class="chip ${tone}">${esc(text)}</span>`;

/**
 * Metric coloring honors metrics[].direction ("up"|"down" = which way is improvement):
 *   green = target reached; blue = moved toward target; amber = unchanged from baseline;
 *   red = moved AWAY from target (a regression must never look like progress).
 * Non-numeric values fall back to: green at target, amber otherwise.
 */
function metricTone(x) {
  const num = (v) => {
    const n = parseFloat(String(v).replace(/[^0-9.eE+-]/g, ""));
    return Number.isFinite(n) ? n : null;
  };
  const b = num(x.baseline), c = num(x.current), t = num(x.target);
  if (c === null || b === null || t === null) {
    return String(x.current) === String(x.target) ? "green" : "amber";
  }
  if (c === t || (x.direction === "down" && c < t) || (x.direction === "up" && c > t)) return "green";
  if (c === b) return "amber";
  const improving = x.direction === "down" ? c < b : x.direction === "up" ? c > b : Math.abs(c - t) < Math.abs(b - t);
  return improving ? "blue" : "red";
}

function metricsTable() {
  if (!metrics.length) return "";
  const rows = metrics
    .map(
      (x) =>
        `<tr><td><b>${esc(x.label)}</b></td><td>${esc(x.baseline)}</td><td><span class="chip ${metricTone(x)}">${esc(x.current)}</span></td><td>${esc(x.target)}</td></tr>`
    )
    .join("");
  return `<section aria-labelledby="metrics"><h2 id="metrics">Scoreboard: Baseline &rarr; Current &rarr; Target</h2>
  <div class="table-wrap"><table><thead><tr><th>Metric</th><th>Baseline</th><th>Current</th><th>Target</th></tr></thead><tbody>${rows}</tbody></table></div>
  <p class="small muted" style="margin-top:6px">green = at/past target &middot; blue = moving toward target &middot; amber = unchanged &middot; red = moving away</p></section>`;
}

function gateRows(gates) {
  return (gates ?? [])
    .map((g) => {
      const status = g.status ?? "pending";
      const notes = g.notes ? `<br><span class="small">${esc(g.notes)}</span>` : "";
      return `<tr>
      <td><span class="chip ${gateTone[status] ?? ""}">${esc(status)}</span></td>
      <td><b>${esc(g.title ?? g.id ?? "?")}</b>${notes}</td>
      <td>${g.command ? `<code>${esc(g.command)}</code>` : "&mdash;"}</td>
      <td>${g.evidence ? esc(g.evidence) : "&mdash;"}</td>
      <td>${g.lastRun ? esc(g.lastRun) : "&mdash;"}</td></tr>`;
    })
    .join("");
}

function taskList(tasks) {
  return (tasks ?? [])
    .map((t) => {
      const status = t.status ?? "pending";
      const url = safeUrl(t.pr?.url);
      const prLabel = t.pr?.number != null ? `PR #${esc(t.pr.number)}` : null;
      const pr = prLabel
        ? ` &mdash; ${url ? `<a href="${url}">${prLabel}</a>` : prLabel}${t.pr.mergedAt ? ` merged ${esc(t.pr.mergedAt)}` : ""}`
        : "";
      return `<li class="task-${esc(status)}"><span class="mark">${taskMark[status] ?? "&#9675;"}</span> <b>${esc(t.id ?? "?")}</b> ${esc(t.title ?? "")}${pr}${t.notes ? `<br><span class="muted small">${esc(t.notes)}</span>` : ""}</li>`;
    })
    .join("");
}

function phaseCards() {
  return phases
    .map((p) => {
      const status = p.status ?? "pending";
      const tasks = p.tasks ?? [];
      const done = tasks.filter((t) => t.status === "done").length;
      const ppct = tasks.length ? Math.round((done / tasks.length) * 100) : status === "done" ? 100 : 0;
      const active = resume.activePhase === p.id ? " active" : "";
      return `<article class="phase${active}">
      <header>
        <span class="chip ${phaseTone[status] ?? ""}">${esc(String(status).replace("_", " "))}</span>
        <h3>${esc(p.id ?? "?")} &middot; ${esc(p.title ?? "")}</h3>
        ${p.sessionsEstimate ? `<span class="muted small">est. sessions: ${esc(p.sessionsEstimate)}</span>` : ""}
      </header>
      <p class="muted">${esc(p.goal ?? "")}</p>
      <div class="bar"><div class="fill" style="width:${ppct}%"></div></div>
      <p class="small muted">${done}/${tasks.length} tasks &middot; ${ppct}%</p>
      ${tasks.length ? `<ul class="tasks">${taskList(tasks)}</ul>` : ""}
      ${(p.gates ?? []).length ? `<div class="table-wrap"><table><thead><tr><th>Gate</th><th>Check</th><th>Command</th><th>Evidence</th><th>Last run</th></tr></thead><tbody>${gateRows(p.gates)}</tbody></table></div>` : ""}
      <details class="resume"><summary>Resume prompt for ${esc(p.id ?? "?")} (copy into a fresh session)</summary>
        <div class="prompt-box"><button class="copy" type="button">Copy</button><pre>${esc(resumePromptFor(p))}</pre></div>
      </details>
    </article>`;
    })
    .join("");
}

function risksPanel() {
  const blockers = (resume.blockers ?? []).map((b) => `<li><span class="chip red">blocker</span> ${esc(b)}</li>`).join("");
  const riskItems = risks
    .map((r) => `<li><span class="chip ${sevTone[r.severity] ?? ""}">${esc(r.severity ?? "?")}</span> <b>${esc(r.title)}</b>${r.note ? ` &mdash; <span class="muted">${esc(r.note)}</span>` : ""}</li>`)
    .join("");
  if (!blockers && !riskItems) return "";
  return `<section aria-labelledby="risks"><h2 id="risks">Risks &amp; Blockers</h2><div class="panel"><ul class="bare">${blockers}${riskItems}</ul></div></section>`;
}

function prTimeline() {
  if (!prLog.length) return "";
  const rows = [...prLog]
    .reverse()
    .map((p) => {
      const url = safeUrl(p.url);
      return `<article class="tl-item"><time>#${esc(p.number ?? "?")}</time><p><b>${esc(p.title ?? "")}</b> &mdash; ${esc(p.phase ?? "?")}${p.mergedAt ? `, merged ${esc(p.mergedAt)}` : ""} ${url ? `&middot; <a href="${url}">view</a>` : ""}</p></article>`;
    })
    .join("");
  return `<section aria-labelledby="prs"><h2 id="prs">Mission PR Log (compact, ${prLog.length} rows)</h2>
    <p class="muted small">Operational mission log only; it is intentionally compact and is not the complete first-parent archive.</p>
    <div class="timeline">${rows}</div></section>`;
}

function mainlineCheckpointSection() {
  if (!checkpointRows.length) return "";
  const successorPr = successorArchiveMerge.pullRequest;
  const rows = checkpointRows
    .map((entry, index) => {
      const prUrl = safeUrl(`https://github.com/${m.repo}/pull/${entry.pr}`);
      const pr = prUrl ? `<a href="${prUrl}">#${esc(entry.pr)}</a>` : `#${esc(entry.pr)}`;
      return `<tr><td>${index + 1}</td><td>${pr}</td><td><code>${esc(entry.sha ?? "missing")}</code></td><td>${esc(entry.scope)}</td></tr>`;
    })
    .join("");
  const archiveCount = archivedPrOrder.length;
  return `<section aria-labelledby="checkpoint"><h2 id="checkpoint">Mainline Checkpoint (${checkpointRows.length} first-parent PR merges${successorPr != null ? `; successor #${esc(successorPr)}` : ""})</h2>
    <div class="panel">
      <p class="muted">Complete local first-parent checkpoint: ${archiveCount} PR commits after <code>${esc(archivedRange.baseExclusiveMainSha ?? "?")}</code> through <code>${esc(archivedRange.headInclusiveMainSha ?? "?")}</code>, followed by the separately identified successor archive merge. Source: ${esc(mainlineCheckpoint.source ?? "state.json")}.</p>
      <div class="table-wrap"><table><thead><tr><th>Order</th><th>PR</th><th>Full squash-main SHA</th><th>Checkpoint scope</th></tr></thead><tbody>${rows}</tbody></table></div>
    </div>
  </section>`;
}

function journalSection() {
  const tail = journalTail();
  if (!tail.length) return "";
  const items = tail
    .map((e) => {
      const [head, ...rest] = e.replace(/^## /, "").split("\n");
      return `<article class="tl-item"><time>&#9998;</time><p><b>${esc(head)}</b><br>${rest.map((l) => esc(l)).join("<br>")}</p></article>`;
    })
    .join("");
  return `<section aria-labelledby="journal"><h2 id="journal">Journal (latest first)</h2><div class="timeline">${items}</div></section>`;
}

// ---------- page ----------
const statusTone = m.status === "complete" ? "green" : m.status === "paused" ? "amber" : "blue";
const gatesChipTone = !allGates.length ? "" : passedGates === allGates.length ? "green" : "amber";
const html = `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${esc(m.name ?? "Mission")} — State of the Union</title>
<style>
:root{color-scheme:dark;--bg:#070f1c;--paper:#0d1b2f;--paper-2:#10243b;--ink:#eef6ff;--muted:#a9bdd3;--line:rgba(169,189,211,.18);--accent:#58d7b3;--blue:#77b7ff;--amber:#ffd166;--red:#ff8f8f;--good:#8ef2c0}
*{box-sizing:border-box}
body{margin:0;background:radial-gradient(circle at top left,rgba(88,215,179,.15),transparent 30rem),radial-gradient(circle at top right,rgba(119,183,255,.13),transparent 32rem),var(--bg);color:var(--ink);font-family:Inter,ui-sans-serif,system-ui,"Segoe UI",sans-serif;line-height:1.55}
main{width:min(1180px,calc(100% - 32px));margin:0 auto;padding:40px 0 60px}
h1,h2,h3{margin:0;line-height:1.12}
h1{font-size:clamp(2rem,5vw,3.8rem);letter-spacing:-.04em;max-width:900px;margin-top:10px}
h2{margin-bottom:14px;font-size:clamp(1.4rem,2.6vw,2rem)}
h3{font-size:1.02rem;color:#fff}
p{margin:0}
a{color:var(--blue)}
code{font-family:ui-monospace,Menlo,Consolas,monospace;font-size:.85em;color:#dff7ef;overflow-wrap:anywhere}
.eyebrow{color:var(--accent);font-size:.78rem;font-weight:800;letter-spacing:.14em;text-transform:uppercase}
.muted{color:var(--muted)}
.small{font-size:.85rem}
section{padding-top:34px}
.hero{padding:20px 0 26px;border-bottom:1px solid var(--line)}
.hero .tagline{max-width:760px;margin-top:14px;color:var(--muted);font-size:1.06rem}
.chip{display:inline-flex;align-items:center;border:1px solid var(--line);border-radius:999px;padding:3px 10px;margin:2px 6px 2px 0;background:rgba(255,255,255,.05);color:var(--muted);font-size:.78rem;font-weight:800}
.chip.green{color:var(--good);border-color:rgba(142,242,192,.4)}
.chip.amber{color:var(--amber);border-color:rgba(255,209,102,.4)}
.chip.red{color:var(--red);border-color:rgba(255,143,143,.4)}
.chip.blue{color:var(--blue);border-color:rgba(119,183,255,.4)}
.panel,.phase{border:1px solid var(--line);border-radius:16px;background:linear-gradient(180deg,rgba(16,36,59,.95),rgba(13,27,47,.95));box-shadow:0 18px 46px rgba(0,0,0,.25);padding:20px}
.phase{margin-top:14px}
.phase.active{border-color:rgba(88,215,179,.45)}
.phase header{display:flex;flex-wrap:wrap;align-items:center;gap:10px;margin-bottom:8px}
.bar{height:10px;border-radius:999px;background:rgba(255,255,255,.07);margin-top:12px;overflow:hidden}
.bar .fill{height:100%;border-radius:999px;background:linear-gradient(90deg,var(--accent),var(--blue))}
.bar.big{height:16px}
ul.tasks,ul.bare{list-style:none;margin:12px 0 0;padding:0}
ul.tasks li,ul.bare li{padding:8px 0;border-top:1px solid var(--line);color:var(--muted)}
ul.tasks li:first-child,ul.bare li:first-child{border-top:0}
ul.tasks .mark{margin-right:6px;color:var(--accent)}
li.task-done{opacity:.75}
li.task-blocked .mark{color:var(--red)}
table{width:100%;border-collapse:collapse;margin-top:12px;font-size:.9rem}
th,td{border:1px solid var(--line);padding:8px 10px;text-align:left;vertical-align:top}
th{background:rgba(255,255,255,.05);color:#fff;font-size:.76rem;letter-spacing:.06em;text-transform:uppercase}
td{color:var(--muted)}
td b{color:#fff}
.table-wrap{overflow-x:auto;border-radius:12px}
.timeline{display:grid;gap:10px}
.tl-item{display:grid;grid-template-columns:72px 1fr;gap:14px;border:1px solid var(--line);border-radius:12px;padding:12px 14px;background:rgba(255,255,255,.035)}
.tl-item time{color:var(--accent);font-size:.8rem;font-weight:900}
.tl-item p{color:var(--muted)}
details.resume{margin-top:14px;border:1px dashed rgba(88,215,179,.4);border-radius:12px;padding:10px 14px}
details.resume summary{cursor:pointer;color:var(--accent);font-weight:800}
.prompt-box{position:relative;margin-top:10px}
.prompt-box pre{margin:0;border:1px solid var(--line);border-radius:10px;background:#081120;color:#dff7ef;padding:14px;font-size:.82rem;white-space:pre-wrap;overflow-wrap:anywhere;max-height:420px;overflow:auto}
button.copy{position:absolute;top:8px;right:8px;border:1px solid var(--line);border-radius:8px;background:rgba(255,255,255,.08);color:var(--ink);font-weight:700;padding:4px 12px;cursor:pointer}
button.copy:hover{background:rgba(88,215,179,.2)}
.footer{margin-top:40px;border-top:1px solid var(--line);padding-top:16px;color:var(--muted);font-size:.9rem}
@media(max-width:900px){main{width:min(100% - 24px,760px)}.tl-item{grid-template-columns:56px 1fr}}
</style>
</head>
<body>
<main>
  <header class="hero">
    <p class="eyebrow">Mission Control &middot; State of the Union &middot; generated ${esc(new Date().toISOString().slice(0, 19).replace("T", " "))}Z</p>
    <h1>${esc(m.name ?? "Mission")}</h1>
    <p class="tagline">${esc(m.tagline ?? "")}</p>
    <div style="margin-top:14px">
      ${chip(`mission: ${m.status ?? "?"}`, statusTone)}
      ${chip(`active phase: ${resume.activePhase ?? "?"}`, "green")}
      ${chip(`${doneTasks}/${allTasks.length} tasks`, "blue")}
      ${chip(`${passedGates}/${allGates.length} gates passed`, gatesChipTone)}
      ${chip(`${prLog.length} compact PR log rows`)}
      ${checkpointRows.length ? chip(`${checkpointRows.length} checkpoint PR merges`, "blue") : ""}
      ${chip(`sessions: ${m.sessionCount ?? 1}`)}
      ${chip(`updated: ${m.updated ?? "?"}`)}
    </div>
    <div class="bar big" role="img" aria-label="Overall progress ${pct}%"><div class="fill" style="width:${pct}%"></div></div>
    <p class="small muted" style="margin-top:6px">Overall: ${pct}% of tasks complete &middot; repo ${esc(m.repo ?? "?")} &middot; plan: ${esc(m.planDoc ?? "?")}</p>
  </header>

  <section aria-labelledby="coldstart">
    <h2 id="coldstart">Cold-Start Prompt (any new session, zero context)</h2>
    <div class="panel">
      <div class="prompt-box"><button class="copy" type="button">Copy</button><pre>${esc(globalPrompt)}</pre></div>
    </div>
  </section>

  ${metricsTable()}
  ${risksPanel()}

  <section aria-labelledby="phases">
    <h2 id="phases">Phases</h2>
    ${phaseCards()}
  </section>

  ${mainlineCheckpointSection()}
  ${prTimeline()}
  ${journalSection()}

  <p class="footer">GENERATED by ops/mission/render-sotu.mjs from state.json — do not hand-edit.
  To update: edit state.json, run <code>node ops/mission/render-sotu.mjs</code>, commit both.</p>
</main>
<script>
document.querySelectorAll("button.copy").forEach((b) => {
  b.addEventListener("click", async () => {
    const text = b.parentElement.querySelector("pre").innerText;
    try { await navigator.clipboard.writeText(text); b.textContent = "Copied!"; }
    catch { const r = document.createRange(); r.selectNodeContents(b.parentElement.querySelector("pre")); const s = getSelection(); s.removeAllRanges(); s.addRange(r); b.textContent = "Select+Ctrl C"; }
    setTimeout(() => (b.textContent = "Copy"), 1600);
  });
});
</script>
</body>
</html>
`;

writeFileSync(outPath, html);
console.log(`Rendered ${outPath}`);
console.log(
  `  phases: ${phases.length}, tasks: ${doneTasks}/${allTasks.length}, gates passed: ${passedGates}/${allGates.length}, compact PR log rows: ${prLog.length}, checkpoint PR merges: ${checkpointRows.length}`,
);
