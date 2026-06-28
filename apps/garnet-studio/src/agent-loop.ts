// Pure renderer for the Agent-Loop Console. Side-effect- and DOM-free so it can
// be unit-tested directly. It renders an EXISTING `agent-loop --record-dir`
// dossier as a four-gate pipeline (check → diff-caps → run → seal). Every verdict
// is the CLI's own (read from decision.md / the artifacts) — nothing is
// recomputed here. Human approval, the widening gate, and seal provenance are
// kept in visibly separate sections.

export type AgentLoopGate = "check" | "diff-caps" | "run" | "seal";
export type GateStatus = "pass" | "reject" | "not-reached";
export type AgentLoopOutcome = "accepted" | "rejected";

export interface AgentLoopGateRow {
  gate: AgentLoopGate;
  status: GateStatus;
  detail: string;
}

export interface ManifestFunction {
  name: string;
  caps: string[];
}

export interface CapabilityManifest {
  schema: string;
  aggregate: string[];
  functions: ManifestFunction[];
  wildcard: boolean;
}

export interface SealAttestation {
  agent: string;
  autonomous: string;
  decision: string;
  gate_version: string;
  model: string;
  tool: string;
}

export interface TransparencyLogEntry {
  index: number;
  program: string;
  caps: string[];
  caps_blake3: string;
  prev_blake3: string;
}

export interface AgentLoopDossier {
  ran: boolean;
  record_dir: string;
  outcome: AgentLoopOutcome;
  rejected_at: AgentLoopGate | null;
  gates: AgentLoopGateRow[];
  decision_md: string;
  diff_caps_text: string;
  capability_manifest: CapabilityManifest | null;
  seal_authorship: string;
  seal_attestation: SealAttestation | null;
  transparency_log: TransparencyLogEntry[];
  error: string;
}

function esc(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

const GATE_LABEL: Record<AgentLoopGate, string> = {
  check: "check",
  "diff-caps": "diff-caps",
  run: "run · enforced kernel",
  seal: "seal",
};

const STATUS_GLYPH: Record<GateStatus, string> = {
  pass: "✓",
  reject: "✕",
  "not-reached": "·",
};

const STATUS_LABEL: Record<GateStatus, string> = {
  pass: "pass",
  reject: "reject",
  "not-reached": "not reached",
};

function gateStepHtml(row: AgentLoopGateRow): string {
  const detail = row.detail
    ? `<span class="gate-detail">${esc(row.detail)}</span>`
    : `<span class="gate-detail quiet">—</span>`;
  return `<li class="gate-step ${esc(row.status)}">
    <span class="gate-glyph">${STATUS_GLYPH[row.status]}</span>
    <div class="gate-body">
      <span class="gate-name"><code>${esc(GATE_LABEL[row.gate])}</code><span class="gate-status">${esc(
        STATUS_LABEL[row.status],
      )}</span></span>
      ${detail}
    </div>
  </li>`;
}

function manifestHtml(manifest: CapabilityManifest | null): string {
  if (!manifest) return "";
  const agg = manifest.aggregate.length
    ? manifest.aggregate.map((c) => `<code class="cap">${esc(c)}</code>`).join(" ")
    : "<span class='quiet'>none</span>";
  const fns = manifest.functions
    .map(
      (f) =>
        `<li><code>${esc(f.name)}</code> → ${
          f.caps.length ? f.caps.map((c) => `<code class="cap">${esc(c)}</code>`).join(" ") : "<span class='quiet'>none</span>"
        }</li>`,
    )
    .join("");
  const wildcard = manifest.wildcard
    ? `<p class="al-warn">wildcard capability present (<code>@caps(*)</code>)</p>`
    : "";
  return `<section class="al-card">
    <h3>Capability manifest <span class="al-schema">${esc(manifest.schema)}</span></h3>
    <p>Aggregate: ${agg}</p>
    <ul class="al-fns">${fns}</ul>
    ${wildcard}
  </section>`;
}

function sealHtml(authorship: string, seal: SealAttestation | null, accepted: boolean): string {
  if (!seal) {
    // Distinguish the two no-seal states honestly: a genuine rejection (the
    // negative proof) vs. an accepted dossier whose seal.json could not be read.
    const copy = accepted
      ? "seal.json missing or unparseable — the acceptance provenance could not be read from this directory."
      : "No seal — the proposal was not accepted, so nothing was attested. (The negative proof.)";
    return `<section class="al-card seal">
      <h3>Seal provenance</h3>
      <p class="quiet">${esc(copy)}</p>
    </section>`;
  }
  // Provenance is autonomous-acceptance attestation, NOT a human approval — kept
  // explicitly separate per the dossier's "keep approval, widening, and seal
  // provenance visibly separate" requirement.
  const row = (k: string, v: string) =>
    `<div class="al-kv"><span class="al-k">${esc(k)}</span><code class="al-v">${esc(v || "—")}</code></div>`;
  return `<section class="al-card seal">
    <h3>Seal provenance <span class="al-quiet">autonomous acceptance — not a human approval</span></h3>
    ${row("authored-by", authorship)}
    ${row("agent", seal.agent)}
    ${row("model", seal.model)}
    ${row("autonomous", seal.autonomous)}
    ${row("decision", seal.decision)}
    ${row("gate-version", seal.gate_version)}
    ${row("tool", seal.tool)}
  </section>`;
}

function logHtml(entries: TransparencyLogEntry[]): string {
  if (!entries.length) return "";
  const rows = entries
    .map(
      (e) =>
        `<li><span class="al-idx">#${e.index}</span> <code>${esc(e.program)}</code> caps ${
          e.caps.length ? e.caps.map((c) => `<code class="cap">${esc(c)}</code>`).join(" ") : "<span class='quiet'>none</span>"
        } <span class="al-hash">${esc(e.caps_blake3.slice(0, 12))}</span> ← <span class="al-hash">${esc(
          e.prev_blake3.slice(0, 12),
        )}</span></li>`,
    )
    .join("");
  return `<section class="al-card">
    <h3>Transparency log <span class="al-quiet">caps-log chain</span></h3>
    <ul class="al-log">${rows}</ul>
  </section>`;
}

/** Build the Agent-Loop Console HTML for a dossier. `now` is an optional stamp. */
export function agentLoopConsoleHtml(dossier: AgentLoopDossier, now = ""): string {
  if (!dossier.ran) {
    return `<div class="diagnostic-item error"><div class="diagnostic-content"><span class="diagnostic-code">no dossier loaded</span><span class="diagnostic-message">${esc(
      dossier.error || "could not read the record directory",
    )}</span></div></div>`;
  }

  const accepted = dossier.outcome === "accepted";
  const headline = accepted
    ? `<span class="al-verdict accepted">ACCEPTED</span><span class="al-verdict-note">on capability + depth evidence</span>`
    : `<span class="al-verdict rejected">REJECTED</span><span class="al-verdict-note">at the <code>${esc(
        dossier.rejected_at ?? "?",
      )}</code> gate</span>`;

  const stamp = now ? `<time class="al-stamp">${esc(now)}</time>` : "";
  const pipeline = `<ol class="gate-pipeline">${dossier.gates.map(gateStepHtml).join("")}</ol>`;

  // Authority-gate drill-down: the diff-caps verdict + manifest, kept distinct
  // from the seal provenance below.
  const diffCaps = dossier.diff_caps_text
    ? `<section class="al-card"><h3>Authority gate <span class="al-quiet">diff-caps</span></h3><pre class="al-pre">${esc(
        dossier.diff_caps_text.trim(),
      )}</pre></section>`
    : "";

  const decision = `<section class="al-card"><h3>Decision <span class="al-quiet">decision.md, verbatim</span></h3><pre class="al-pre">${esc(
    dossier.decision_md.trim(),
  )}</pre></section>`;

  return `<section class="agent-loop-console">
    ${stamp}
    <header class="al-headline">${headline}</header>
    <p class="al-intro">The verdict is read verbatim from <code>decision.md</code>; the gate notes are Studio summaries, and the decision and diff-caps panes below are the CLI's text verbatim. <code>@caps</code> and <code>@max_depth</code> are the enforced ceilings; acceptance is "on capability + depth evidence" only — never a claim of full boundedness or safety.</p>
    ${pipeline}
    ${diffCaps}
    ${manifestHtml(dossier.capability_manifest)}
    ${sealHtml(dossier.seal_authorship, dossier.seal_attestation, accepted)}
    ${logHtml(dossier.transparency_log)}
    ${decision}
  </section>`;
}
