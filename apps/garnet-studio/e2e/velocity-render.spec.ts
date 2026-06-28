import { test, expect } from "@playwright/test";
import {
  velocityDiagnosticsHtml,
  lineForByteOffset,
  latestOnly,
  type VelocityCheckReport,
  type VelocityDiagnostic,
} from "../src/velocity";

// Pure-function unit tests for the live-check renderer — no browser needed (the
// e2e suite cannot drive it because invoke() rejects outside Tauri).

function report(over: Partial<VelocityCheckReport> = {}): VelocityCheckReport {
  return {
    ran: true,
    diagnostics: [],
    errors: 0,
    warnings: 0,
    infos: 0,
    ok: true,
    exit_code: 0,
    stderr: "",
    ...over,
  };
}

function diag(over: Partial<VelocityDiagnostic> = {}): VelocityDiagnostic {
  return {
    severity: "error",
    code: "check.caps_coverage",
    message: "needs @caps(fs)",
    span: null,
    ...over,
  };
}

test.describe("velocityDiagnosticsHtml (pure renderer)", () => {
  test("clean buffer shows the no-diagnostics state", () => {
    expect(velocityDiagnosticsHtml(report(), "let x = 1")).toContain("No diagnostics");
  });

  test("a check diagnostic (no span) renders honestly as 'whole buffer' — never a faked line", () => {
    const html = velocityDiagnosticsHtml(
      report({ diagnostics: [diag()], errors: 1, ok: false, exit_code: 1 }),
      "fn main() { read_file() }",
    );
    expect(html).toContain("check.caps_coverage");
    expect(html).toContain("diagnostic-item error");
    expect(html).toContain("whole buffer");
    expect(html).not.toMatch(/line \d+ ·/); // no span → no fabricated line:col
  });

  test("a parse diagnostic with a byte span resolves to a precise line", () => {
    const buffer = "line1\nline2\nXbad"; // byte 12 is on line 3
    const html = velocityDiagnosticsHtml(
      report({
        diagnostics: [diag({ code: "parse.reserved_word", span: { start: 12, len: 1 } })],
        errors: 1,
        ok: false,
        exit_code: 1,
      }),
      buffer,
    );
    expect(html).toContain("parse.reserved_word");
    expect(html).toContain("line 3");
    expect(html).toContain("bytes 12–13");
  });

  test("severity drives the badge and tone", () => {
    const html = velocityDiagnosticsHtml(
      report({ diagnostics: [diag({ severity: "warning", code: "check.boundary_note" })], warnings: 1, exit_code: 1 }),
      "x",
    );
    expect(html).toContain("diagnostic-item warning");
    expect(html).toContain("diagnostic-badge warning");
  });

  test("ran=false renders an honest 'did not run' error, not a diagnostic", () => {
    const html = velocityDiagnosticsHtml(
      report({ ran: false, ok: false, exit_code: -1, stderr: "Garnet CLI not found" }),
      "x",
    );
    expect(html).toContain("did not run");
    expect(html).toContain("Garnet CLI not found");
  });

  test("lineForByteOffset is byte-accurate across newlines", () => {
    const buf = "a\nbb\nccc";
    expect(lineForByteOffset(buf, 0)).toBe(1);
    expect(lineForByteOffset(buf, 2)).toBe(2); // just after the first newline
    expect(lineForByteOffset(buf, 5)).toBe(3); // just after the second newline
  });

  test("ran but not ok with NO per-item diagnostics is NOT a green pass", () => {
    // The false-green guard: errors > 0 / ok = false with an empty diagnostics
    // array must never read as "✓ ... check out".
    const html = velocityDiagnosticsHtml(
      report({ diagnostics: [], ok: false, errors: 1, exit_code: 1, stderr: "internal: 1 error" }),
      "x",
    );
    expect(html).not.toContain("No diagnostics");
    expect(html).not.toContain("check out");
    expect(html).toContain("check reported a problem");
  });
});

test.describe("latestOnly (out-of-order resolve guard)", () => {
  test("drops a stale earlier resolve and delivers only the latest", async () => {
    const delivered: string[] = [];
    let resolveFirst: (v: string) => void = () => {};
    const update = latestOnly<number, string>(
      (n) =>
        n === 1
          ? new Promise<string>((r) => {
              resolveFirst = r;
            })
          : Promise.resolve("second"),
      (r) => delivered.push(r),
    );
    const p1 = update(1); // in-flight, not yet resolved
    await update(2); // resolves "second" → seq advances
    resolveFirst("first"); // the stale earlier run resolves late → dropped
    await p1;
    expect(delivered).toEqual(["second"]);
  });
});
