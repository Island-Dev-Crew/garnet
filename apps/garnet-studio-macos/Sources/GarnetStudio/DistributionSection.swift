// M7 — Distribution Reporter (power-only section).
//
// An honest macOS packaging/notarization status surface. Lays out the pure
// `DistributionReport` projection (catalog + live filesystem probe; logic
// unit-tested in DistributionBridgeTests). The headline never claims the app is
// distribution-ready while it is unsigned + un-notarized.

import SwiftUI

struct DistributionSection: View {
    @State private var repoRoot = ""
    @State private var report: DistributionReport?

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text("Distribution Reporter").font(.title2).bold()
            Text(
                "macOS packaging and notarization status. The Garnet Studio .app is a research-grade prototype: it is unsigned and un-notarized, so it is not Gatekeeper-distributable — it is for local run only. Enter a Garnet checkout path to verify the packaging scripts on disk."
            )
            .font(.callout).foregroundStyle(.secondary)

            HStack(spacing: 8) {
                TextField("optional: path to a Garnet checkout (to probe scripts)", text: $repoRoot)
                    .textFieldStyle(.roundedBorder)
                Button("Report") { rebuild() }
            }

            if let report {
                Text(report.headline).font(.callout).bold()
                ScrollView {
                    VStack(alignment: .leading, spacing: 10) {
                        ForEach(Array(report.items.enumerated()), id: \.offset) { _, item in
                            DistributionRowView(item: item)
                        }
                    }
                }
            }
            Spacer()
        }
        .padding()
        .onAppear { if report == nil { rebuild() } }
    }

    private func rebuild() {
        let root = repoRoot
        Task {
            let built = await Task.detached { DistributionCommand.report(repoRoot: root) }.value
            report = built
        }
    }
}

private struct DistributionRowView: View {
    let item: DistributionItem

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack(spacing: 8) {
                Text(item.status.label.uppercased())
                    .font(.caption2).bold()
                    .padding(.horizontal, 7).padding(.vertical, 2)
                    .background(badgeColor.opacity(0.18))
                    .foregroundStyle(badgeColor)
                    .clipShape(Capsule())
                Text(item.name).font(.body).bold()
            }
            Text(item.detail).font(.callout).foregroundStyle(.secondary)
            Divider()
        }
    }

    private var badgeColor: Color {
        switch item.status {
        case .ready: return .green
        case .deferred: return .orange
        case .absent: return .red
        case .unverified: return .secondary
        }
    }
}
