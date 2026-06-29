// M2 — Diff-Caps Review Gate (power-only section).
//
// Renders `garnet diff-caps --machine` VERBATIM via the M0b render bridge. The
// View only lays out the pure `DiffCapsCard` projection (decode/projection are
// unit-tested in M0b; the JSON extraction in DiffCapsCommand tests) — no logic
// here, so it stays out of swift-test (the CI dual-runner dislikes @MainActor
// View tests). diff-caps FLAGS for review; it never refuses a merge.

import SwiftUI

struct DiffCapsReviewSection: View {
    let cliPath: String?
    let commandTimeoutSecs: Int

    @State private var baseline = ""
    @State private var proposal = ""
    @State private var card: DiffCapsCard?
    @State private var running = false

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text("Diff-Caps Review Gate").font(.title2).bold()
            Text(
                "Renders `garnet diff-caps --machine OLD NEW` verbatim. diff-caps FLAGS a capability widening for review — it never refuses a merge; the merge-block is a separate CI integrity rule."
            )
            .font(.callout).foregroundStyle(.secondary)

            HStack(spacing: 8) {
                TextField("baseline .garnet", text: $baseline).textFieldStyle(.roundedBorder)
                TextField("proposal .garnet", text: $proposal).textFieldStyle(.roundedBorder)
                Button(running ? "Reviewing…" : "Review") { review() }
                    .disabled(running || cliPath == nil || baseline.isEmpty || proposal.isEmpty)
            }

            if cliPath == nil {
                Label("No Garnet CLI found", systemImage: "exclamationmark.triangle")
                    .foregroundStyle(.orange)
            }
            if let card { DiffCapsCardView(card: card) }
            Spacer()
        }
        .padding()
    }

    private func review() {
        guard let cli = cliPath else { return }
        running = true
        let b = baseline
        let p = proposal
        let timeout = commandTimeoutSecs
        Task {
            let report = await Task.detached {
                DiffCapsCommand.run(
                    cli: cli, baseline: b, proposal: p, timeoutSeconds: timeout)
            }.value
            card = DiffCapsCard.render(report)
            running = false
        }
    }
}

/// Lays out a `DiffCapsCard` (the pure M0b projection). Presentation only.
private struct DiffCapsCardView: View {
    let card: DiffCapsCard

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            if card.isError {
                Label(card.headline, systemImage: "xmark.octagon").foregroundStyle(.red).bold()
                Text(card.errorText)
                    .font(.system(.caption, design: .monospaced)).foregroundStyle(.secondary)
            } else {
                Label(
                    card.headline,
                    systemImage: card.tone == .ok ? "checkmark.seal" : "exclamationmark.triangle"
                )
                .foregroundStyle(card.tone == .ok ? Color.green : Color.orange).bold()

                if card.wildcardIntroduced {
                    Label(
                        "@caps(*) wildcard introduced — unbounded declared authority.",
                        systemImage: "asterisk.circle"
                    ).foregroundStyle(.orange)
                }
                ForEach(card.sections, id: \.title) { section in
                    Text(section.title).font(.headline)
                    ForEach(section.items, id: \.self) { item in
                        Text(item).font(.system(.body, design: .monospaced))
                    }
                }
                if card.noChanges {
                    Text("No declared capability changes.").foregroundStyle(.secondary)
                }
                Text(card.scope).font(.caption).foregroundStyle(.secondary)
                if let evidence = card.evidencePath {
                    Text("evidence: \(evidence)").font(.caption2).foregroundStyle(.secondary)
                }
            }
        }
        .padding(.top, 6)
    }
}
