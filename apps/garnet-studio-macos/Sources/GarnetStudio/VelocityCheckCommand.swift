// M3 — Velocity Editor check runner (nonisolated, so its logic is swift-testable
// without referencing the @MainActor panel View — the CI-safe pattern from M0b).
//
// Runs `garnet check --format json <buffer>` over the live editor buffer and
// decodes it via the M3 render bridge. The CLI is the single source of truth; the
// severities/summary are never recomputed here.

import Foundation

public enum VelocityCheckCommand {
    /// The disciplined process runner merges stdout+stderr into one stream, and the
    /// checker prints prior-failure `note:` lines + edition warnings to stderr — so
    /// the diagnostics JSON must be sliced out of the merged output before decoding.
    /// Returns the bytes from the first `{` through the last `}`, or nil when there
    /// is no JSON object.
    public static func extractJSONObject(from text: String) -> Data? {
        guard let start = text.firstIndex(of: "{"), let end = text.lastIndex(of: "}"), start <= end
        else { return nil }
        return Data(text[start...end].utf8)
    }

    /// Run `check --format json` over `source` and decode the diagnostics.
    ///
    /// The buffer is staged in a UNIQUE throwaway directory used as the process
    /// working directory, so the checker's `.garnet-cache/episodes.log` side-effect
    /// lands in that disposable dir — never the user's project — and is removed when
    /// the run completes. Non-JSON / error output decodes to the error card (never a
    /// fabricated clean result) via `VelocityReport.decode`.
    public static func run(cli: String, source: String, timeoutSeconds: Int) -> VelocityReport {
        let fm = FileManager.default
        let dir = fm.temporaryDirectory
            .appendingPathComponent("garnet-velocity-\(UUID().uuidString)", isDirectory: true)
        defer { try? fm.removeItem(at: dir) }

        let file = dir.appendingPathComponent("buffer.garnet")
        do {
            try fm.createDirectory(at: dir, withIntermediateDirectories: true)
            try source.write(to: file, atomically: true, encoding: .utf8)
        } catch {
            return VelocityReport(
                ran: false, output: nil, exitCode: 127,
                stderr: "could not stage buffer: \(error.localizedDescription)")
        }

        let result = StudioProcessRunner.run(
            executableURL: URL(fileURLWithPath: cli),
            arguments: ["check", "--format", "json", file.path],
            workingDirectory: dir,
            timeoutSeconds: timeoutSeconds)
        let json = extractJSONObject(from: result.output) ?? Data(result.output.utf8)
        return VelocityReport.decode(
            stdout: json, exitCode: Int(result.exitCode), stderr: result.output)
    }
}
