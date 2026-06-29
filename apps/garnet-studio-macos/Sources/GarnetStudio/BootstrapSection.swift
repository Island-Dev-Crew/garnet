// M6 — Bootstrap (power-only section, generate-scripts-only).
//
// Shows the generated allowlisted bash/zsh setup scripts (copyable) and writes
// them to a folder the operator picks. It never runs them, never uses sudo, and
// never edits a shell profile — generation only. All plan/script logic is pure
// (BootstrapBridge, unit-tested); this View is presentation + the folder write.

import SwiftUI
import UniformTypeIdentifiers

struct BootstrapSection: View {
    let cliPath: String?

    @State private var plan: BootstrapPlan?
    @State private var scripts: [BootstrapScript] = []
    @State private var pickingFolder = false
    @State private var status = ""

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text("Bootstrap").font(.title2).bold()
            Text(
                "Generates allowlisted bash/zsh setup scripts for you to inspect and run manually. Garnet Studio never runs them, never uses sudo, and never edits your shell profile — generation only. Running an installer from the app would need a Jon-approved AGENTS.md amendment."
            )
            .font(.callout).foregroundStyle(.secondary)

            if let plan {
                Text(plan.summary).font(.callout)
                HStack(spacing: 10) {
                    Button("Write scripts to folder…") { pickingFolder = true }
                        .buttonStyle(.borderedProminent)
                        .help("Write the generated scripts to a folder you choose. They are not made executable and are never run.")
                    if !status.isEmpty {
                        Text(status).font(.caption).foregroundStyle(.secondary)
                    }
                }

                ScrollView {
                    VStack(alignment: .leading, spacing: 12) {
                        ForEach(Array(scripts.enumerated()), id: \.offset) { _, script in
                            VStack(alignment: .leading, spacing: 4) {
                                Text(script.name)
                                    .font(.system(.subheadline, design: .monospaced)).bold()
                                Text(script.contents)
                                    .font(.system(.caption2, design: .monospaced))
                                    .textSelection(.enabled)
                                    .frame(maxWidth: .infinity, alignment: .leading)
                                    .padding(8)
                                    .background(Color(nsColor: .textBackgroundColor))
                                    .overlay(
                                        RoundedRectangle(cornerRadius: 4)
                                            .stroke(Color.secondary.opacity(0.2)))
                            }
                        }
                    }
                }
            }
            Spacer()
        }
        .padding()
        .onAppear { rebuild() }
        .fileImporter(
            isPresented: $pickingFolder, allowedContentTypes: [.folder],
            allowsMultipleSelection: false
        ) { result in
            if case .success(let urls) = result, let dir = urls.first { writeTo(dir) }
        }
    }

    private func rebuild() {
        let built = BootstrapPlan.from(cliPath: cliPath)
        plan = built
        scripts = BootstrapGenerator.scripts(for: built)
    }

    private func writeTo(_ dir: URL) {
        guard let plan else { return }
        let target = dir.appendingPathComponent("garnet-bootstrap", isDirectory: true)
        let accessed = dir.startAccessingSecurityScopedResource()
        defer { if accessed { dir.stopAccessingSecurityScopedResource() } }
        let result = BootstrapCommand.writeScripts(plan: plan, to: target)
        if let error = result.error {
            status = "error: \(error)"
        } else {
            status = "wrote \(result.written.count) script(s) to \(result.directory)"
        }
    }
}
