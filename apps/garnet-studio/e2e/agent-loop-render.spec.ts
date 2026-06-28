import { test, expect } from "@playwright/test";
import {
  agentLoopConsoleHtml,
  type AgentLoopDossier,
  type AgentLoopGateRow,
  type GateStatus,
} from "../src/agent-loop";

// Pure-function unit tests for the Agent-Loop Console renderer — no browser/Tauri
// needed (the e2e suite cannot drive it because invoke() rejects outside Tauri).

function gates(
  check: GateStatus,
  diffCaps: GateStatus,
  run: GateStatus,
  seal: GateStatus,
  details: Partial<Record<string, string>> = {},
): AgentLoopGateRow[] {
  return [
    { gate: "check", status: check, detail: details.check ?? "" },
    { gate: "diff-caps", status: diffCaps, detail: details["diff-caps"] ?? "" },
    { gate: "run", status: run, detail: details.run ?? "" },
    { gate: "seal", status: seal, detail: details.seal ?? "" },
  ];
}

function dossier(over: Partial<AgentLoopDossier> = {}): AgentLoopDossier {
  return {
    ran: true,
    record_dir: "/rec",
    outcome: "accepted",
    rejected_at: null,
    gates: gates("pass", "pass", "pass", "pass", {
      "diff-caps": "diff-caps: no authority expansion (capability band 5/5)",
    }),
    decision_md: "# Agent-loop decision: ACCEPTED\n\nNOT a claim of full boundedness or safety.",
    diff_caps_text: "diff-caps: no authority expansion (capability band 5/5)",
    capability_manifest: {
      schema: "garnet-capability-manifest-v1",
      aggregate: ["fs"],
      functions: [{ name: "main", caps: ["fs"] }],
      wildcard: false,
    },
    seal_authorship: "sim:scripted-agent",
    seal_attestation: {
      agent: "scripted-agent-v1",
      autonomous: "true",
      decision: "accepted-on-capability+depth-evidence",
      gate_version: "dogfood-gate-v1",
      model: "simulated",
      tool: "garnet-agent-loop",
    },
    transparency_log: [
      { index: 0, program: "accept_proposal", caps: ["fs"], caps_blake3: "cda46a92aaaa", prev_blake3: "genesis" },
    ],
    error: "",
    ...over,
  };
}

test.describe("agentLoopConsoleHtml (pure renderer)", () => {
  test("an accepted dossier renders the ACCEPTED verdict and four passing gates", () => {
    const html = agentLoopConsoleHtml(dossier());
    expect(html).toContain("al-verdict accepted");
    expect(html).toContain("ACCEPTED");
    // Four gate steps, all pass.
    expect((html.match(/gate-step pass/g) || []).length).toBe(4);
    expect(html).toContain("capability band 5/5");
    // Artifacts rendered.
    expect(html).toContain("garnet-capability-manifest-v1");
    expect(html).toContain("scripted-agent-v1");
    expect(html).toContain("accept_proposal");
    // decision.md rendered verbatim — the honest scope disclaimer survives.
    expect(html).toContain("NOT a claim of full boundedness or safety");
  });

  test("seal provenance is its own section, labelled autonomous — not a human approval", () => {
    const html = agentLoopConsoleHtml(dossier());
    expect(html).toContain("al-card seal");
    expect(html).toContain("not a human approval");
    expect(html).toContain("simulated"); // the model is honest about being scripted
  });

  test("a rejected-at-diff-caps dossier stops the pipeline and writes no seal", () => {
    const html = agentLoopConsoleHtml(
      dossier({
        outcome: "rejected",
        rejected_at: "diff-caps",
        gates: gates("pass", "reject", "not-reached", "not-reached", {
          "diff-caps": "diff-caps: AUTHORITY EXPANDED — review required (capability band 2/5)",
        }),
        diff_caps_text: "diff-caps: AUTHORITY EXPANDED — review required (capability band 2/5)",
        capability_manifest: null,
        seal_attestation: null,
        seal_authorship: "",
        transparency_log: [],
        decision_md: "# Agent-loop decision: REJECTED (capability widening)",
      }),
    );
    expect(html).toContain("al-verdict rejected");
    expect(html).toContain("AUTHORITY EXPANDED");
    expect((html.match(/gate-step reject/g) || []).length).toBe(1);
    expect((html.match(/gate-step not-reached/g) || []).length).toBe(2);
    // No seal on a rejected proposal — the negative proof is stated, not faked.
    expect(html).toContain("not accepted, so nothing was attested");
    expect(html).not.toContain("scripted-agent-v1");
  });

  test("a rejected-at-run dossier shows the enforced-kernel trap and no seal", () => {
    const html = agentLoopConsoleHtml(
      dossier({
        outcome: "rejected",
        rejected_at: "run",
        gates: gates("pass", "pass", "reject", "not-reached", {
          "diff-caps": "diff-caps: no authority expansion (capability band 5/5)",
          run: "runtime error: bounded: @max_depth(4) exceeded for `digest` (recursion depth 5)",
        }),
        seal_attestation: null,
        decision_md: "# Agent-loop decision: REJECTED (enforced-ceiling trap)",
      }),
    );
    expect(html).toContain("@max_depth(4) exceeded");
    expect((html.match(/gate-step pass/g) || []).length).toBe(2);
    expect((html.match(/gate-step reject/g) || []).length).toBe(1);
    expect((html.match(/gate-step not-reached/g) || []).length).toBe(1);
  });

  test("an accepted dossier whose seal could not be read says so, not 'not accepted'", () => {
    // The degraded-accept honesty fix: outcome accepted but seal absent must NOT
    // render the rejection's "not accepted" copy.
    const html = agentLoopConsoleHtml(
      dossier({ outcome: "accepted", seal_attestation: null, seal_authorship: "" }),
    );
    expect(html).toContain("seal.json missing or unparseable");
    expect(html).not.toContain("was not accepted");
  });

  test("a wildcard capability manifest renders the widening warning", () => {
    // The @caps(*) widening signal must render loudly when present.
    const html = agentLoopConsoleHtml(
      dossier({
        capability_manifest: {
          schema: "garnet-capability-manifest-v1",
          aggregate: ["fs"],
          functions: [],
          wildcard: true,
        },
      }),
    );
    expect(html).toContain("al-warn");
    expect(html).toContain("@caps(*)");
  });

  test("a dossier that did not load renders an honest error, not an empty pipeline", () => {
    const html = agentLoopConsoleHtml(
      dossier({ ran: false, error: "record directory not found, or it is not a directory." }),
    );
    expect(html).toContain("no dossier loaded");
    expect(html).toContain("record directory not found");
    expect(html).not.toContain("al-verdict");
    expect(html).not.toContain("gate-step");
  });

  test("gates render in pipeline order: check → diff-caps → run → seal", () => {
    const html = agentLoopConsoleHtml(dossier());
    const check = html.indexOf(">check<");
    const diffCaps = html.indexOf(">diff-caps<");
    const run = html.indexOf("run · enforced kernel");
    const seal = html.indexOf(">seal<");
    expect(check).toBeGreaterThan(-1);
    expect(check).toBeLessThan(diffCaps);
    expect(diffCaps).toBeLessThan(run);
    expect(run).toBeLessThan(seal);
  });

  test("html-escapes dossier content (no raw injection)", () => {
    const html = agentLoopConsoleHtml(dossier({ decision_md: "<script>alert(1)</script>" }));
    expect(html).not.toContain("<script>alert(1)</script>");
    expect(html).toContain("&lt;script&gt;");
  });
});
