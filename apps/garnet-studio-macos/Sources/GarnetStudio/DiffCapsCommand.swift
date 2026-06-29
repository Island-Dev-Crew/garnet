// M2 — diff-caps review command runner (nonisolated, so its logic is swift-testable
// without referencing the @MainActor panel View — the CI-safe pattern from M0b).
//
// Runs `garnet diff-caps --machine <baseline> <proposal>` and decodes it via the
// M0b render bridge. The CLI is the single source of truth; the band/verdict are
// never recomputed here.

import Foundation

public enum DiffCapsCommand {
    /// The disciplined process runner merges stdout+stderr into one stream, and the
    /// CLI prints episodic-cache `note:` lines to stderr — so the machine JSON must
    /// be sliced out of the merged output before decoding. Returns the bytes from
    /// the first `{` through the last `}`, or nil when there is no JSON object.
    public static func extractJSONObject(from text: String) -> Data? {
        guard let start = text.firstIndex(of: "{"), let end = text.lastIndex(of: "}"), start <= end
        else { return nil }
        return Data(text[start...end].utf8)
    }

    /// Run diff-caps and decode the verdict. A non-JSON / error output decodes to
    /// the error card (never a fabricated clean verdict) via `DiffCapsReport.decode`.
    public static func run(
        cli: String, baseline: String, proposal: String, timeoutSeconds: Int
    ) -> DiffCapsReport {
        let result = StudioProcessRunner.run(
            executableURL: URL(fileURLWithPath: cli),
            arguments: ["diff-caps", "--machine", baseline, proposal],
            timeoutSeconds: timeoutSeconds)
        let json = extractJSONObject(from: result.output) ?? Data(result.output.utf8)
        return DiffCapsReport.decode(
            stdout: json, exitCode: Int(result.exitCode), stderr: result.output)
    }
}
