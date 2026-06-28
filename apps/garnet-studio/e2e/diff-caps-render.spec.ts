import { test, expect } from "@playwright/test";
import {
  diffCapsCardHtml,
  type DiffCapsReport,
  type DiffCapsVerdict,
} from "../src/diff-caps";

// Pure-function unit tests for the verdict renderer. These need no browser — the
// e2e suite cannot drive renderDiffCaps because invoke() rejects outside Tauri,
// so this exercises every rendering branch directly against the contract.

const SCOPE =
  "declared-surface-only; does not prove absence of undeclared authority; bound annotations are not part of this surface";

function verdict(over: Partial<DiffCapsVerdict> = {}): DiffCapsVerdict {
  return {
    schema: "garnet.diff-caps.machine/1",
    verdict: "no-authority-expansion",
    authority_expanded: false,
    capability_band: "5/5",
    exit_code: 0,
    aggregate_gained: [],
    aggregate_removed: [],
    wildcard_introduced: false,
    functions_added: [],
    functions_removed: [],
    functions_caps_expanded: [],
    scope: SCOPE,
    ...over,
  };
}

function report(v: DiffCapsVerdict | null, over: Partial<DiffCapsReport> = {}): DiffCapsReport {
  return {
    ran: v !== null,
    verdict: v,
    exit_code: v?.exit_code ?? 0,
    stderr: "",
    evidence_path: "C:/dogfood/diff-caps",
    command: ["garnet", "diff-caps", "--machine"],
    ...over,
  };
}

const expanded = (over: Partial<DiffCapsVerdict> = {}): DiffCapsVerdict =>
  verdict({
    verdict: "authority-expanded",
    authority_expanded: true,
    capability_band: "2/5",
    exit_code: 1,
    ...over,
  });

test.describe("diffCapsCardHtml (pure verdict renderer)", () => {
  test("clean 5/5 renders the ok tone AND the scope caveat (green is not 'safe')", () => {
    const html = diffCapsCardHtml(report(verdict()));
    expect(html).toContain("diff-caps-card ok");
    expect(html).toContain("does not prove absence of undeclared authority");
    expect(html).not.toContain("REFUSED");
  });

  test("expanded renders fail tone, the CLI band verbatim, and 'review required' — never 'REFUSED'", () => {
    const html = diffCapsCardHtml(report(expanded({ aggregate_gained: ["net"] })));
    expect(html).toContain("diff-caps-card fail");
    expect(html).toContain("2/5");
    expect(html).toContain("review required");
    // diff-caps flags for review; it does not enforce a merge block.
    expect(html).not.toContain("REFUSED");
    expect(html).toContain("<code>net</code>");
  });

  test("never recomputes the band — a 3/5 the CLI sent is shown verbatim", () => {
    const html = diffCapsCardHtml(report(expanded({ capability_band: "3/5" })));
    expect(html).toContain("3/5");
    expect(html).toContain("diff-caps-card fail");
  });

  test("wildcard introduced renders the unbounded-authority warning", () => {
    const html = diffCapsCardHtml(report(expanded({ wildcard_introduced: true })));
    expect(html).toContain("wildcard introduced");
  });

  test("a function-only change is NOT reported as 'no declared capability changes'", () => {
    const html = diffCapsCardHtml(report(expanded({ functions_added: ["new_fn"] })));
    expect(html).toContain("Functions added");
    expect(html).toContain("<code>new_fn</code>");
    expect(html).not.toContain("No declared capability changes");
  });

  test("functions_removed renders and is not swallowed by the empty-state", () => {
    const html = diffCapsCardHtml(report(verdict({ functions_removed: ["gone_fn"] })));
    expect(html).toContain("Functions removed");
    expect(html).toContain("<code>gone_fn</code>");
    expect(html).not.toContain("No declared capability changes");
  });

  test("a truly empty diff shows the no-changes state", () => {
    expect(diffCapsCardHtml(report(verdict()))).toContain("No declared capability changes");
  });

  test("no verdict (ran=false) renders an error card and invents no verdict", () => {
    const html = diffCapsCardHtml(report(null, { ran: false, exit_code: 2, stderr: "path not found" }));
    expect(html).toContain("produced no verdict (exit 2)");
    expect(html).toContain("path not found");
    expect(html).not.toContain("authority-expanded");
    expect(html).not.toContain("diff-caps-card");
  });
});
