// Pure renderer for the Velocity Editor's live-check diagnostics. Side-effect-
// and DOM-free so it can be unit-tested directly. severity / code / message /
// span are the CLI's (garnet check --format json) — never reclassified here.

export interface DiagSpan {
  start: number;
  len: number;
}

export interface VelocityDiagnostic {
  severity: string; // "error" | "warning" | "info"
  code: string;
  message: string;
  span: DiagSpan | null;
}

export interface VelocityCheckReport {
  ran: boolean;
  diagnostics: VelocityDiagnostic[];
  errors: number;
  warnings: number;
  infos: number;
  ok: boolean;
  exit_code: number;
  stderr: string;
}

function esc(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

/**
 * 1-based line number for a byte offset, counting newline BYTES in the UTF-8
 * encoding of the buffer — byte-accurate, so multibyte source is not mislocated.
 */
export function lineForByteOffset(buffer: string, byteOffset: number): number {
  const bytes = new TextEncoder().encode(buffer);
  const end = Math.min(Math.max(byteOffset, 0), bytes.length);
  let line = 1;
  for (let i = 0; i < end; i++) {
    if (bytes[i] === 0x0a) line++;
  }
  return line;
}

/**
 * Build the diagnostics HTML for a velocity check report. `buffer` is the source
 * the report was produced from (used to resolve parse spans to a line number).
 */
export function velocityDiagnosticsHtml(report: VelocityCheckReport, buffer: string): string {
  if (!report.ran) {
    return `<div class="diagnostic-item error"><div class="diagnostic-content"><span class="diagnostic-code">check did not run (exit ${report.exit_code})</span><span class="diagnostic-message">${esc(report.stderr || "no output")}</span></div></div>`;
  }

  if (report.diagnostics.length === 0) {
    if (report.ok && report.errors === 0) {
      return `<p class="diagnostics-ok">✓ No diagnostics — the buffer parses and the declared capabilities and bounds check out.</p>`;
    }
    // The check reported a problem (not ok / errors > 0) but emitted no per-item
    // diagnostics — never render that as a clean pass.
    return `<div class="diagnostic-item error"><div class="diagnostic-content"><span class="diagnostic-code">check reported a problem (exit ${report.exit_code})</span><span class="diagnostic-message">${esc(report.stderr || `${report.errors} error(s) without per-item diagnostics`)}</span></div></div>`;
  }

  const counts = [
    report.errors
      ? `<span class="count-error">${report.errors} error${report.errors === 1 ? "" : "s"}</span>`
      : "",
    report.warnings
      ? `<span class="count-warning">${report.warnings} warning${report.warnings === 1 ? "" : "s"}</span>`
      : "",
    report.infos
      ? `<span class="count-info">${report.infos} note${report.infos === 1 ? "" : "s"}</span>`
      : "",
  ]
    .filter(Boolean)
    .join("");
  const summary = `<div class="diagnostics-summary">${counts}</div>`;

  const items = report.diagnostics
    .map((d) => {
      const sev = ["error", "warning", "info"].includes(d.severity) ? d.severity : "error";
      // Parse diagnostics carry a byte span → a precise line. Check diagnostics
      // are message-only today → honestly "whole buffer", never a faked line.
      const loc = d.span
        ? `<span class="diagnostic-loc">line ${lineForByteOffset(buffer, d.span.start)} · bytes ${d.span.start}–${d.span.start + d.span.len}</span>`
        : `<span class="diagnostic-loc">whole buffer (check diagnostics are not yet span-located)</span>`;
      return `<div class="diagnostic-item ${sev}">
        <span class="diagnostic-badge ${sev}">${esc(sev)}</span>
        <div class="diagnostic-content">
          <span class="diagnostic-code">${esc(d.code)}</span>
          <span class="diagnostic-message">${esc(d.message)}</span>
          ${loc}
        </div>
      </div>`;
    })
    .join("");

  return `${summary}<div class="diagnostics-items">${items}</div>`;
}

/**
 * A "latest wins" async runner. Returns a function you call with input; only the
 * MOST-RECENT in-flight call's result is delivered to `sink` — a stale, out-of-
 * order resolve is dropped. This is the debounce companion that stops a slow
 * earlier check from overwriting a newer one's diagnostics.
 */
export function latestOnly<I, O>(
  run: (input: I) => Promise<O>,
  sink: (result: O) => void,
): (input: I) => Promise<void> {
  let seq = 0;
  return async (input: I): Promise<void> => {
    const mine = ++seq;
    const result = await run(input);
    if (mine === seq) sink(result);
  };
}
