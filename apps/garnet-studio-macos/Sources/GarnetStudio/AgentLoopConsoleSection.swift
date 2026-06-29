// M5 — Agent-Loop Console (power-only section).
//
// Lays out an EXISTING `agent-loop --record-dir` dossier as a four-gate pipeline
// (check → diff-caps → run → seal) plus the CLI's own decision.md and diff-caps
// text verbatim, the capability manifest, the seal provenance, and the
// transparency-log chain. All parsing/verdict logic is in AgentLoopBridge (unit-
// tested); this View is presentation only. It never runs the agent loop.

import SwiftUI

struct AgentLoopConsoleSection: View {
    @State private var recordDir = ""
    @State private var dossier: AgentLoopDossier?
    @State private var loading = false

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text("Agent-Loop Console").font(.title2).bold()
            Text(
                "Open an existing `garnet agent-loop --record-dir` dossier. The verdict is read verbatim from decision.md; acceptance is on capability + depth evidence only — never a claim of full boundedness or safety. The console never runs the loop."
            )
            .font(.callout).foregroundStyle(.secondary)

            HStack(spacing: 8) {
                TextField("path to an agent-loop --record-dir", text: $recordDir)
                    .textFieldStyle(.roundedBorder)
                Button(loading ? "Loading…" : "Load") { load() }
                    .disabled(loading || recordDir.isEmpty)
            }

            if let dossier {
                if !dossier.ran {
                    Label(
                        dossier.error.isEmpty ? "could not read the record directory" : dossier.error,
                        systemImage: "xmark.octagon"
                    ).foregroundStyle(.red)
                } else {
                    ScrollView {
                        VStack(alignment: .leading, spacing: 14) {
                            headline(dossier)
                            gatePipeline(dossier)
                            if !dossier.diffCapsText.isEmpty {
                                verbatim("Authority gate — diff-caps", dossier.diffCapsText)
                            }
                            manifestPanel(dossier.capabilityManifest)
                            sealPanel(dossier)
                            verbatim("Decision — decision.md, verbatim", dossier.decisionMd)
                        }
                    }
                }
            }
            Spacer()
        }
        .padding()
    }

    private func load() {
        loading = true
        let dir = recordDir
        Task {
            let result = await Task.detached { AgentLoopCommand.load(recordDir: dir) }.value
            dossier = result
            loading = false
        }
    }

    private func headline(_ d: AgentLoopDossier) -> some View {
        let accepted = d.outcome == .accepted
        return HStack(spacing: 8) {
            Text(accepted ? "ACCEPTED" : "REJECTED")
                .font(.headline).bold()
                .foregroundStyle(accepted ? Color.green : Color.red)
            Text(
                accepted
                    ? "on capability + depth evidence"
                    : "at the \(d.rejectedAt?.label ?? "?") gate"
            )
            .font(.caption).foregroundStyle(.secondary)
        }
    }

    private func gatePipeline(_ d: AgentLoopDossier) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            ForEach(Array(d.gates.enumerated()), id: \.offset) { _, row in
                HStack(alignment: .top, spacing: 8) {
                    Text(row.status.glyph)
                        .foregroundStyle(glyphColor(row.status)).bold()
                        .frame(width: 16)
                    VStack(alignment: .leading, spacing: 2) {
                        HStack(spacing: 8) {
                            Text(row.gate.label).font(.system(.body, design: .monospaced)).bold()
                            Text(row.status.label).font(.caption2).foregroundStyle(.secondary)
                        }
                        if !row.detail.isEmpty {
                            Text(row.detail).font(.caption).foregroundStyle(.secondary)
                        }
                    }
                }
            }
        }
    }

    @ViewBuilder
    private func manifestPanel(_ manifest: CapabilityManifest?) -> some View {
        if let manifest {
            VStack(alignment: .leading, spacing: 4) {
                Text("Capability manifest").font(.headline)
                Text("schema: \(manifest.schema)").font(.caption2).foregroundStyle(.secondary)
                Text(
                    "aggregate: "
                        + (manifest.aggregate.isEmpty ? "none" : manifest.aggregate.joined(separator: " "))
                ).font(.system(.caption, design: .monospaced))
                ForEach(Array(manifest.functions.enumerated()), id: \.offset) { _, fn in
                    Text(
                        "\(fn.name) → "
                            + (fn.caps.isEmpty ? "none" : fn.caps.joined(separator: " "))
                    ).font(.system(.caption, design: .monospaced))
                }
                if manifest.wildcard {
                    Label("wildcard capability present (@caps(*))", systemImage: "asterisk.circle")
                        .font(.caption).foregroundStyle(.orange)
                }
            }
        }
    }

    private func sealPanel(_ d: AgentLoopDossier) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("Seal provenance").font(.headline)
            if let seal = d.sealAttestation {
                Text("autonomous acceptance — not a human approval")
                    .font(.caption2).foregroundStyle(.secondary)
                kv("authored-by", d.sealAuthorship)
                kv("agent", seal.agent)
                kv("model", seal.model)
                kv("autonomous", seal.autonomous)
                kv("decision", seal.decision)
                kv("gate-version", seal.gateVersion)
                kv("tool", seal.tool)
            } else {
                Text(
                    d.outcome == .accepted
                        ? "seal.json missing or unparseable — the acceptance provenance could not be read."
                        : "No seal — the proposal was not accepted, so nothing was attested. (The negative proof.)"
                ).font(.caption).foregroundStyle(.secondary)
            }
            if !d.transparencyLog.isEmpty {
                Text("Transparency log — caps-log chain").font(.subheadline).padding(.top, 4)
                ForEach(Array(d.transparencyLog.enumerated()), id: \.offset) { _, e in
                    Text(
                        "#\(e.index) \(e.program) caps "
                            + (e.caps.isEmpty ? "none" : e.caps.joined(separator: " "))
                            + "  \(e.capsBlake3.prefix(12)) ← \(e.prevBlake3.prefix(12))"
                    ).font(.system(.caption2, design: .monospaced)).foregroundStyle(.secondary)
                }
            }
        }
    }

    private func kv(_ key: String, _ value: String) -> some View {
        HStack(spacing: 8) {
            Text(key).font(.caption2).foregroundStyle(.secondary).frame(width: 90, alignment: .leading)
            Text(value.isEmpty ? "—" : value).font(.system(.caption2, design: .monospaced))
        }
    }

    private func verbatim(_ title: String, _ text: String) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title).font(.headline)
            Text(text.trimmingCharacters(in: .whitespacesAndNewlines))
                .font(.system(.caption, design: .monospaced))
                .textSelection(.enabled)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(8)
                .background(Color(nsColor: .textBackgroundColor))
                .overlay(RoundedRectangle(cornerRadius: 4).stroke(Color.secondary.opacity(0.2)))
        }
    }

    private func glyphColor(_ status: GateStatus) -> Color {
        switch status {
        case .pass: return .green
        case .reject: return .red
        case .notReached: return .secondary
        }
    }
}
