// M4 — Enforced / Declared Legend (power-only section).
//
// Lays out the pure `EnforcementLegendCard` projection (catalog + live static-gate
// probes; all logic unit-tested in EnforcementLegendBridgeTests). No logic here,
// so it stays out of swift-test. The enforced-vs-declared boundary is rendered
// verbatim from the catalog; "confirmed live" appears only when the probe
// reproduced this run.

import SwiftUI

struct EnforcementLegendSection: View {
    let cliPath: String?
    let commandTimeoutSecs: Int

    @State private var card: EnforcementLegendCard?
    @State private var running = false

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text("Enforced / Declared Legend").font(.title2).bold()
            Text(
                "Which fences the runtime actually enforces, which are only declared, and which are platform-deferred. Status comes from the catalog and a live `garnet check` probe — not hand-written. @caps and @max_depth show enforced only where the trap evidence holds; seccomp is Linux-only. Garnet is a research-grade prototype, not production / 1.0."
            )
            .font(.callout).foregroundStyle(.secondary)

            HStack(spacing: 10) {
                Button(running ? "Probing…" : "Probe live") { build() }
                    .buttonStyle(.borderedProminent)
                    .disabled(running)
                    .help("Re-run the live static-gate probes through `garnet check`.")
                if cliPath == nil {
                    Label("No Garnet CLI — probes inconclusive", systemImage: "exclamationmark.triangle")
                        .foregroundStyle(.orange)
                }
            }

            if let card {
                if !card.cliAvailable {
                    Text(
                        "No Garnet CLI found — the enforced rows show their claim, but the live static-gate probe did not run this session."
                    )
                    .font(.caption).foregroundStyle(.orange)
                }
                ScrollView {
                    VStack(alignment: .leading, spacing: 12) {
                        ForEach(Array(card.rows.enumerated()), id: \.offset) { _, row in
                            LegendRowView(row: row)
                        }
                    }
                }
            }
            Spacer()
        }
        .padding()
        .onAppear { if card == nil { build() } }
    }

    private func build() {
        running = true
        let cli = cliPath
        let timeout = commandTimeoutSecs
        Task {
            let legend = await Task.detached {
                EnforcementLegendCommand.build(cli: cli, timeoutSeconds: timeout)
            }.value
            card = EnforcementLegendCard.render(legend)
            running = false
        }
    }
}

/// Lays out one legend row (pure projection). Presentation only.
private struct LegendRowView: View {
    let row: EnforcementLegendCard.Row

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(spacing: 8) {
                Text(row.statusLabel.uppercased())
                    .font(.caption2).bold()
                    .padding(.horizontal, 7).padding(.vertical, 2)
                    .background(badgeColor.opacity(0.18))
                    .foregroundStyle(badgeColor)
                    .clipShape(Capsule())
                Text(row.name).font(.system(.body, design: .monospaced)).bold()
                Text(row.backends).font(.caption).foregroundStyle(.secondary)
            }
            Text(row.basis).font(.callout).foregroundStyle(.secondary)
            if !row.runtimeAttestedBy.isEmpty {
                Text("Runtime trap: attested — \(row.runtimeAttestedBy) (not re-run by this probe).")
                    .font(.caption2).foregroundStyle(.secondary)
            }
            if !row.gateLine.isEmpty {
                Text(row.gateLine)
                    .font(.caption)
                    .foregroundStyle(gateColor)
            }
            Divider()
        }
    }

    private var badgeColor: Color {
        switch row.status {
        case .enforced: return .green
        case .declared: return .orange
        case .deferred: return .secondary
        }
    }

    private var gateColor: Color {
        switch row.gateState {
        case .confirmed: return .green
        case .unconfirmed: return .orange
        case .notProbed, .notApplicable: return .secondary
        }
    }
}
