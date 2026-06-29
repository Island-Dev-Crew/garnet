// M7 — Distribution Reporter probe (nonisolated; read-only filesystem checks).
//
// The only side effect is `fileExists` checks under a repo root the user provides
// — no spawn, no writes. With no repo root the probe items read `unverified`
// (never a fabricated "ready").

import Foundation

public enum DistributionCommand {
    public static func report(repoRoot: String?) -> DistributionReport {
        guard let repoRoot, !repoRoot.trimmingCharacters(in: .whitespaces).isEmpty else {
            return DistributionReport.build(repoRootProvided: false, exists: { _ in false })
        }
        let base = URL(fileURLWithPath: repoRoot, isDirectory: true)
        return DistributionReport.build(repoRootProvided: true) { relativePath in
            FileManager.default.fileExists(atPath: base.appendingPathComponent(relativePath).path)
        }
    }
}
