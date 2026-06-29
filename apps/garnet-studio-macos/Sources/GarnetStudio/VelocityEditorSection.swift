// M3 — Velocity Editor (power-only section).
//
// A live Garnet source buffer checked through `garnet check --format json`. The
// View only lays out the pure `VelocityCard` projection (decode/projection are
// unit-tested in VelocityBridgeTests) — no logic here, so it stays out of
// swift-test. The checker's `.garnet-cache` side-effect is isolated to a
// throwaway directory by `VelocityCheckCommand` (the editor never touches the
// user's project tree).

import SwiftUI

struct VelocityEditorSection: View {
    let cliPath: String?
    let commandTimeoutSecs: Int

    @State private var source = "fn main() {\n    print(\"hello, garnet\")\n}\n"
    @State private var card: VelocityCard?
    @State private var running = false

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text("Velocity Editor").font(.title2).bold()
            Text(
                "Type Garnet source and run `garnet check --format json` on the live buffer. The checker's .garnet-cache side-effect is isolated to a throwaway directory — your project tree is never touched."
            )
            .font(.callout).foregroundStyle(.secondary)

            TextEditor(text: $source)
                .font(.system(.body, design: .monospaced))
                .frame(minHeight: 220)
                .overlay(
                    RoundedRectangle(cornerRadius: 6).stroke(Color.secondary.opacity(0.3))
                )
                .accessibilityLabel("Garnet source buffer")

            HStack(spacing: 10) {
                Button(running ? "Checking…" : "Check") { check() }
                    .buttonStyle(.borderedProminent)
                    .keyboardShortcut("r", modifiers: .command)
                    .disabled(running || cliPath == nil || source.isEmpty)
                    .help("Run `garnet check --format json` over the buffer (⌘R).")
                if cliPath == nil {
                    Label("No Garnet CLI found", systemImage: "exclamationmark.triangle")
                        .foregroundStyle(.orange)
                }
            }

            if let card { VelocityCardView(card: card) }
            Spacer()
        }
        .padding()
    }

    private func check() {
        guard let cli = cliPath else { return }
        running = true
        let s = source
        let timeout = commandTimeoutSecs
        Task {
            let report = await Task.detached {
                VelocityCheckCommand.run(cli: cli, source: s, timeoutSeconds: timeout)
            }.value
            card = VelocityCard.render(report)
            running = false
        }
    }
}

/// Lays out a `VelocityCard` (the pure M3 projection). Presentation only.
private struct VelocityCardView: View {
    let card: VelocityCard

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            if card.isError {
                Label(card.headline, systemImage: "xmark.octagon").foregroundStyle(.red).bold()
                Text(card.errorText)
                    .font(.system(.caption, design: .monospaced)).foregroundStyle(.secondary)
            } else {
                Label(card.headline, systemImage: iconName(card.tone))
                    .foregroundStyle(toneColor(card.tone)).bold()

                ForEach(Array(card.rows.enumerated()), id: \.offset) { _, row in
                    HStack(alignment: .top, spacing: 8) {
                        Text(row.severity.uppercased())
                            .font(.caption2).bold()
                            .foregroundStyle(severityColor(row.severity))
                            .frame(width: 64, alignment: .leading)
                        VStack(alignment: .leading, spacing: 2) {
                            Text(row.message).font(.system(.body, design: .monospaced))
                            HStack(spacing: 8) {
                                Text(row.code).font(.caption2).foregroundStyle(.secondary)
                                if let loc = row.location {
                                    Text(loc).font(.caption2).foregroundStyle(.secondary)
                                }
                            }
                        }
                    }
                }
                if card.clean {
                    Text("No diagnostics.").foregroundStyle(.secondary)
                }
            }
        }
        .padding(.top, 6)
    }

    private func iconName(_ tone: VelocityCard.Tone) -> String {
        switch tone {
        case .ok: return "checkmark.seal"
        case .warn: return "exclamationmark.triangle"
        case .fail: return "xmark.octagon"
        }
    }

    private func toneColor(_ tone: VelocityCard.Tone) -> Color {
        switch tone {
        case .ok: return .green
        case .warn: return .orange
        case .fail: return .red
        }
    }

    private func severityColor(_ severity: String) -> Color {
        switch severity {
        case "error": return .red
        case "warning": return .orange
        default: return .secondary
        }
    }
}
