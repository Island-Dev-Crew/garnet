// M3 — render bridge (Velocity Editor / `garnet check --format json`).
//
// The CLI is the single source of truth: this decodes `garnet check --format
// json` (schema: a `diagnostics` array + a `summary` of counts, see
// garnet-cli/src/diagnostics.rs) and projects it to a view-ready card WITHOUT
// recomputing severities or the ok/clean verdict. The View (M3
// `VelocityEditorSection`) only lays out the pure `VelocityCard`; all honesty
// rules live here so they are unit-testable (`swift test`), the CI-safe pattern
// established by M0b's `DiffCapsCard`.

import Foundation

/// A `(start, len)` byte span carried by parse diagnostics (check diagnostics are
/// message-only today, so `span` is frequently null).
public struct VelocitySpan: Codable, Equatable, Sendable {
    public let start: Int
    public let len: Int

    public init(start: Int, len: Int) {
        self.start = start
        self.len = len
    }
}

/// One structured diagnostic from `garnet check --format json`.
public struct VelocityDiagnostic: Codable, Equatable, Sendable {
    public let severity: String  // "error" | "warning" | "info"
    public let code: String
    public let message: String
    public let span: VelocitySpan?

    public init(severity: String, code: String, message: String, span: VelocitySpan?) {
        self.severity = severity
        self.code = code
        self.message = message
        self.span = span
    }
}

/// The `summary` block. The CLI always emits it (even for an empty/ok run), so its
/// presence is the "the CLI authored this" signal — we never invent a summary.
public struct VelocitySummary: Equatable, Sendable {
    public let errors: Int
    public let warnings: Int
    public let infos: Int
    public let ok: Bool

    public init(errors: Int, warnings: Int, infos: Int, ok: Bool) {
        self.errors = errors
        self.warnings = warnings
        self.infos = infos
        self.ok = ok
    }
}

extension VelocitySummary: Codable {
    enum CodingKeys: String, CodingKey {
        case errors, warnings, infos, ok
    }

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        // Tolerant: missing counts degrade to 0; `ok` defaults to "no errors".
        let e = (try? c.decode(Int.self, forKey: .errors)) ?? 0
        errors = e
        warnings = (try? c.decode(Int.self, forKey: .warnings)) ?? 0
        infos = (try? c.decode(Int.self, forKey: .infos)) ?? 0
        ok = (try? c.decode(Bool.self, forKey: .ok)) ?? (e == 0)
    }
}

/// The decoded `garnet check --format json` payload: ordered diagnostics + the
/// summary. `summary` is required (core identity); `diagnostics` degrades to empty.
public struct VelocityCheckOutput: Equatable, Sendable {
    public let diagnostics: [VelocityDiagnostic]
    public let summary: VelocitySummary

    public init(diagnostics: [VelocityDiagnostic], summary: VelocitySummary) {
        self.diagnostics = diagnostics
        self.summary = summary
    }
}

extension VelocityCheckOutput: Codable {
    enum CodingKeys: String, CodingKey {
        case diagnostics, summary
    }

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        // Core identity: a real check output always carries `summary`.
        summary = try c.decode(VelocitySummary.self, forKey: .summary)
        diagnostics = (try? c.decode([VelocityDiagnostic].self, forKey: .diagnostics)) ?? []
    }
}

/// One check invocation's outcome: the decoded output (when the CLI produced JSON)
/// plus the surrounding run metadata.
public struct VelocityReport: Equatable, Sendable {
    public let ran: Bool
    public let output: VelocityCheckOutput?
    public let exitCode: Int
    public let stderr: String

    public init(ran: Bool, output: VelocityCheckOutput?, exitCode: Int, stderr: String) {
        self.ran = ran
        self.output = output
        self.exitCode = exitCode
        self.stderr = stderr
    }

    /// Decode a report from raw `garnet check --format json` stdout + run context.
    /// Non-JSON / a missing summary yields `output == nil` (the error card), never
    /// a fabricated clean result.
    public static func decode(stdout: Data, exitCode: Int, stderr: String) -> VelocityReport {
        let output = try? JSONDecoder().decode(VelocityCheckOutput.self, from: stdout)
        return VelocityReport(
            ran: output != nil, output: output, exitCode: exitCode, stderr: stderr)
    }
}

/// The view-ready projection of a `VelocityReport`. `VelocityCard.render` is the
/// pure function M3's SwiftUI view lays out; all honesty rules live here.
public struct VelocityCard: Equatable, Sendable {
    public enum Tone: String, Sendable { case ok, warn, fail }

    /// One diagnostic row, presentation-ready (location pre-formatted).
    public struct Row: Equatable, Sendable {
        public let severity: String
        public let code: String
        public let message: String
        public let location: String?
    }

    public let isError: Bool
    public let errorText: String
    public let tone: Tone
    public let headline: String
    public let clean: Bool
    public let rows: [Row]

    public static func render(_ report: VelocityReport) -> VelocityCard {
        guard report.ran, let out = report.output else {
            return VelocityCard(
                isError: true,
                errorText: report.stderr.isEmpty ? "no output" : report.stderr,
                tone: .fail,
                headline: "check produced no diagnostics JSON (exit \(report.exitCode))",
                clean: false, rows: [])
        }
        let s = out.summary
        // The counts/severities are the CLI's — never recomputed here.
        let clean = out.diagnostics.isEmpty && s.errors == 0 && s.warnings == 0 && s.infos == 0
        let tone: Tone = s.errors > 0 ? .fail : (s.warnings > 0 ? .warn : .ok)
        let headline =
            clean
            ? "Clean — no diagnostics"
            : "\(s.errors) error\(s.errors == 1 ? "" : "s"), "
                + "\(s.warnings) warning\(s.warnings == 1 ? "" : "s"), \(s.infos) info"
        let rows = out.diagnostics.map { d -> Row in
            let location = d.span.map { "byte \($0.start)–\($0.start + $0.len)" }
            return Row(severity: d.severity, code: d.code, message: d.message, location: location)
        }
        return VelocityCard(
            isError: false, errorText: "", tone: tone, headline: headline, clean: clean, rows: rows)
    }
}
