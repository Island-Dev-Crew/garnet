// Pure renderer for the Diff-Caps Review Gate verdict card. Kept side-effect-free
// and DOM-free so it can be unit-tested directly. It renders the CLI's
// `garnet.diff-caps.machine/1` verdict VERBATIM and never recomputes the band or
// the verdict — the CLI is the single source of truth.

export interface DiffCapsFnExpansion {
  name: string;
  gained: string[];
}

export interface DiffCapsVerdict {
  schema: string;
  verdict: string;
  authority_expanded: boolean;
  capability_band: string;
  exit_code: number;
  aggregate_gained: string[];
  aggregate_removed: string[];
  wildcard_introduced: boolean;
  functions_added: string[];
  functions_removed: string[];
  functions_caps_expanded: DiffCapsFnExpansion[];
  scope: string;
}

export interface DiffCapsReport {
  ran: boolean;
  verdict: DiffCapsVerdict | null;
  exit_code: number;
  stderr: string;
  evidence_path: string | null;
  command: string[];
}

function esc(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function list(title: string, items: string[]): string {
  if (!items.length) return "";
  return `<h4>${esc(title)}</h4><ul class="diff-caps-list">${items
    .map((c) => `<li><code>${esc(c)}</code></li>`)
    .join("")}</ul>`;
}

/**
 * Build the verdict-card HTML for a DiffCapsReport. Returns the error card when
 * the gate could not produce a verdict (exit 2 / truncated / unparseable JSON) —
 * it never invents a verdict the CLI did not author.
 */
export function diffCapsCardHtml(report: DiffCapsReport, now = ""): string {
  if (!report.ran || !report.verdict) {
    return `
      <article class="result fail">
        <header>
          <span>diff-caps produced no verdict (exit ${report.exit_code})</span>
          <time>${esc(now)}</time>
        </header>
        <pre>${esc(report.stderr || "no output")}</pre>
      </article>
    `;
  }

  const v = report.verdict;
  // The band/verdict are the CLI's — rendered verbatim, never recomputed here.
  const clean = v.capability_band === "5/5" && !v.authority_expanded;
  const tone = clean ? "ok" : "fail";
  // diff-caps FLAGS for review (exit 1) — it does not refuse a merge; the actual
  // merge-block is a separate CI integrity rule. Never assert enforcement here.
  const headline = clean
    ? "No declared authority gained — band 5/5"
    : `Authority expanded — band ${esc(v.capability_band)}, review required`;

  const wildcard = v.wildcard_introduced
    ? `<p class="diff-caps-wildcard">⚠ <code>@caps(*)</code> wildcard introduced — unbounded declared authority.</p>`
    : "";

  const fns = v.functions_caps_expanded.length
    ? `<h4>Functions that gained authority</h4><ul class="diff-caps-list">${v.functions_caps_expanded
        .map((f) => `<li><code>${esc(f.name)}</code> → ${esc(f.gained.join(", "))}</li>`)
        .join("")}</ul>`
    : "";

  // The empty-state may only claim "no changes" when ALL six diff dimensions are
  // empty — a function-only change must never read as clean.
  const nothing =
    v.aggregate_gained.length === 0 &&
    v.aggregate_removed.length === 0 &&
    v.functions_added.length === 0 &&
    v.functions_removed.length === 0 &&
    v.functions_caps_expanded.length === 0 &&
    !v.wildcard_introduced
      ? `<p class="diff-caps-none">No declared capability changes.</p>`
      : "";

  return `
    <article class="diff-caps-card ${tone}">
      <header class="diff-caps-head">
        <span class="dot ${tone}"></span>
        <strong>${esc(v.verdict)} — ${headline}</strong>
      </header>
      <div class="diff-caps-body">
        ${wildcard}
        ${list("Capabilities gained", v.aggregate_gained)}
        ${list("Capabilities removed", v.aggregate_removed)}
        ${list("Functions added", v.functions_added)}
        ${list("Functions removed", v.functions_removed)}
        ${fns}
        ${nothing}
      </div>
      <footer class="diff-caps-scope">${esc(v.scope)}</footer>
      ${report.evidence_path ? `<footer class="diff-caps-evidence">evidence: ${esc(report.evidence_path)}</footer>` : ""}
    </article>
  `;
}
