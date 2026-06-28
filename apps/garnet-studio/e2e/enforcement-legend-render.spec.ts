import { test, expect } from "@playwright/test";
import {
  enforcementLegendHtml,
  type EnforcementLegend,
  type EnforcementFence,
  type EnforcementProbe,
  type FenceStatus,
} from "../src/enforcement-legend";

// Pure-function unit tests for the legend renderer — no browser/Tauri needed
// (the e2e suite cannot drive it because invoke() rejects outside Tauri).

function fence(over: Partial<EnforcementFence> = {}): EnforcementFence {
  return {
    name: "@caps",
    status: "enforced" as FenceStatus,
    backends: "VM + interpreter",
    basis: "deny-by-default host authority",
    runtime_attested_by: "S100 require_capability trap (VM + interp); red-team",
    probe_code: "check.caps_coverage",
    ...over,
  };
}

function probe(over: Partial<EnforcementProbe> = {}): EnforcementProbe {
  return {
    fence: "@caps",
    expected_code: "check.caps_coverage",
    confirmed: true,
    ran: true,
    exit_code: 1,
    observed_codes: ["check.caps_coverage"],
    ...over,
  };
}

function legend(over: Partial<EnforcementLegend> = {}): EnforcementLegend {
  return {
    fences: [fence()],
    probes: [probe()],
    cli_available: true,
    ...over,
  };
}

test.describe("enforcementLegendHtml (pure renderer)", () => {
  test("an enforced fence with a CONFIRMED live probe shows the confirmation + the enforced badge", () => {
    const html = enforcementLegendHtml(legend());
    expect(html).toContain("legend-badge enforced");
    expect(html).toContain("Static gate confirmed live this run");
    expect(html).toContain("check.caps_coverage");
    // The runtime trap is labelled attested and explicitly NOT re-run here.
    expect(html).toContain("attested");
    expect(html).toContain("not re-run by this probe");
  });

  test("an enforced fence whose probe RAN but did not confirm is NOT shown as confirmed", () => {
    // False-green guard: a probe that ran and saw a different code must never
    // read as "confirmed".
    const html = enforcementLegendHtml(
      legend({
        probes: [probe({ confirmed: false, observed_codes: ["parse.reserved_word"] })],
      }),
    );
    expect(html).not.toContain("confirmed live this run");
    // Structural guard: the confirmed branch carries class "legend-probe
    // confirmed"; pinning the class (not just the phrase) survives copy edits.
    expect(html).not.toContain("legend-probe confirmed");
    expect(html).toContain("Static gate NOT confirmed this run");
    expect(html).toContain("parse.reserved_word");
  });

  test("an enforced fence whose probe did NOT run is inconclusive, never confirmed", () => {
    const html = enforcementLegendHtml(
      legend({
        cli_available: false,
        probes: [probe({ confirmed: false, ran: false, exit_code: -1, observed_codes: [] })],
      }),
    );
    expect(html).not.toContain("confirmed live this run");
    expect(html).not.toContain("legend-probe confirmed");
    expect(html).toContain("Static gate not probed");
    // No CLI → an honest banner, not a silent pass.
    expect(html).toContain("No Garnet CLI found");
  });

  test("a CONFIRMED probe stays confirmed amid extra diagnostic codes", () => {
    // probe_from_report confirms on `expected present among others`; the renderer
    // must still show the confirmation (and the expected code) when the run was
    // noisy — never drop the confirmation because other codes were also emitted.
    const html = enforcementLegendHtml(
      legend({
        probes: [
          probe({
            confirmed: true,
            observed_codes: ["check.caps_coverage", "check.boundary_note"],
          }),
        ],
      }),
    );
    expect(html).toContain("legend-probe confirmed");
    expect(html).toContain("Static gate confirmed live this run");
    expect(html).toContain("check.caps_coverage");
  });

  test("a declared fence shows the Declared badge and NO probe/confirmation line", () => {
    const html = enforcementLegendHtml(
      legend({
        fences: [
          fence({
            name: "@bounded",
            status: "declared",
            backends: "Wasmtime fuel only",
            runtime_attested_by: "",
            probe_code: "",
          }),
        ],
        probes: [],
      }),
    );
    expect(html).toContain("legend-badge declared");
    expect(html).not.toContain("confirmed live this run");
    expect(html).not.toContain("Runtime trap:");
  });

  test("a deferred fence shows the Deferred badge", () => {
    const html = enforcementLegendHtml(
      legend({
        fences: [
          fence({
            name: "OS sandbox (macOS / Windows)",
            status: "deferred",
            backends: "Linux seccomp only",
            runtime_attested_by: "",
            probe_code: "",
          }),
        ],
        probes: [],
      }),
    );
    expect(html).toContain("legend-badge deferred");
    expect(html).toContain("OS sandbox (macOS / Windows)");
  });

  test("rows render enforced → declared → deferred regardless of input order", () => {
    const html = enforcementLegendHtml(
      legend({
        fences: [
          fence({ name: "time", status: "declared", runtime_attested_by: "", probe_code: "" }),
          fence({ name: "@caps", status: "enforced" }),
          fence({
            name: "OS sandbox (macOS / Windows)",
            status: "deferred",
            runtime_attested_by: "",
            probe_code: "",
          }),
        ],
      }),
    );
    // Badge classes appear only in rows (never in the intro copy), so their
    // positions reflect the true row order.
    const enforcedAt = html.indexOf("legend-badge enforced");
    const declaredAt = html.indexOf("legend-badge declared");
    const deferredAt = html.indexOf("legend-badge deferred");
    expect(enforcedAt).toBeGreaterThan(-1);
    expect(enforcedAt).toBeLessThan(declaredAt);
    expect(declaredAt).toBeLessThan(deferredAt);
  });

  test("html-escapes fence content (no raw injection)", () => {
    const html = enforcementLegendHtml(
      legend({ fences: [fence({ basis: "<script>alert(1)</script>" })] }),
    );
    expect(html).not.toContain("<script>alert(1)</script>");
    expect(html).toContain("&lt;script&gt;");
  });
});
