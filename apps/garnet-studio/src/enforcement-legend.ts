// Pure renderer for the Enforced / Declared Legend. Side-effect- and DOM-free so
// it can be unit-tested directly. The status of each fence comes from the typed
// payload `studio_enforcement_legend` builds (the catalog + live `garnet check`
// probes) — never hand-written into this markup. The "enforced — confirmed live"
// state is shown ONLY when the live static-gate probe actually reproduced; an
// unconfirmed or un-run probe never reads as a confirmation.

export type FenceStatus = "enforced" | "declared" | "deferred";

export interface EnforcementFence {
  name: string;
  status: FenceStatus;
  backends: string;
  basis: string;
  runtime_attested_by: string;
  probe_code: string;
}

export interface EnforcementProbe {
  fence: string;
  expected_code: string;
  confirmed: boolean;
  ran: boolean;
  exit_code: number;
  observed_codes: string[];
}

export interface EnforcementLegend {
  fences: EnforcementFence[];
  probes: EnforcementProbe[];
  cli_available: boolean;
}

function esc(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

const STATUS_LABEL: Record<FenceStatus, string> = {
  enforced: "Enforced",
  declared: "Declared",
  deferred: "Deferred",
};

/**
 * The static-gate line for an enforced fence: a CONFIRMED badge only when the
 * live probe reproduced the expected diagnostic this run; otherwise an honest
 * "not confirmed" / "not probed" — never a faked green.
 */
function staticGateLine(probe: EnforcementProbe | undefined): string {
  if (!probe) return "";
  if (probe.confirmed) {
    return `<p class="legend-probe confirmed">✓ Static gate confirmed live this run — <code>${esc(
      probe.expected_code,
    )}</code> reproduced (exit ${probe.exit_code}).</p>`;
  }
  if (probe.ran) {
    const saw = probe.observed_codes.length
      ? probe.observed_codes.map((c) => `<code>${esc(c)}</code>`).join(", ")
      : "no diagnostics";
    return `<p class="legend-probe unconfirmed">⚠ Static gate NOT confirmed this run — expected <code>${esc(
      probe.expected_code,
    )}</code>, saw ${saw}.</p>`;
  }
  return `<p class="legend-probe unconfirmed">Static gate not probed — no Garnet CLI to run <code>${esc(
    probe.expected_code,
  )}</code>.</p>`;
}

function fenceRowHtml(fence: EnforcementFence, probe: EnforcementProbe | undefined): string {
  const runtime = fence.runtime_attested_by
    ? `<p class="legend-runtime">Runtime trap: <strong>attested</strong> — ${esc(
        fence.runtime_attested_by,
      )} <span class="legend-quiet">(not re-run by this probe)</span>.</p>`
    : "";
  const probeLine = fence.status === "enforced" ? staticGateLine(probe) + runtime : "";
  return `<article class="legend-fence ${esc(fence.status)}">
    <header class="legend-fence-head">
      <span class="legend-badge ${esc(fence.status)}">${esc(STATUS_LABEL[fence.status])}</span>
      <code class="legend-name">${esc(fence.name)}</code>
      <span class="legend-backends">${esc(fence.backends)}</span>
    </header>
    <p class="legend-basis">${esc(fence.basis)}</p>
    ${probeLine}
  </article>`;
}

/**
 * Build the legend HTML from the typed payload. `now` is an optional timestamp
 * for the header.
 */
export function enforcementLegendHtml(legend: EnforcementLegend, now = ""): string {
  const byFence = new Map(legend.probes.map((p) => [p.fence, p]));

  const cliBanner = legend.cli_available
    ? ""
    : `<p class="legend-banner warn">No Garnet CLI found — the enforced rows show their claim, but the live static-gate probe did not run this session. Set <code>GARNET_CLI</code> or add <code>garnet</code> to PATH to re-confirm.</p>`;

  const intro = `<p class="legend-intro">Which fences the runtime actually <strong>enforces</strong>, which are only <strong>declared</strong>, and which are platform-<strong>deferred</strong>. Status is generated from the CLI/source catalog and a live <code>garnet check</code> probe — not hand-written. <code>@caps</code> and <code>@max_depth</code> are shown enforced only where the trap evidence holds.</p>`;

  const order: FenceStatus[] = ["enforced", "declared", "deferred"];
  const rows = order
    .flatMap((status) => legend.fences.filter((f) => f.status === status))
    .map((f) => fenceRowHtml(f, byFence.get(f.name)))
    .join("");

  const stamp = now ? `<time class="legend-stamp">${esc(now)}</time>` : "";
  return `<section class="enforcement-legend">${stamp}${intro}${cliBanner}<div class="legend-fences">${rows}</div></section>`;
}
