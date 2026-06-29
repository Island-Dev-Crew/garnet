// M0b — render bridge (diff-caps).
//
// The CLI is the single source of truth: this decodes `garnet diff-caps
// --machine` (schema `garnet.diff-caps.machine/1`) and projects it to a
// view-ready card WITHOUT ever recomputing the band or the verdict. It mirrors
// apps/garnet-studio/src/diff-caps.ts so the native Studio renders the same
// verbatim verdict the Windows shell does. M2 lays `DiffCapsCard` out in SwiftUI;
// this file is the pure, unit-tested projection it consumes.

import Foundation

/// A function that gained declared authority in the proposal.
public struct DiffCapsFnExpansion: Codable, Equatable, Sendable {
    public let name: String
    public let gained: [String]
}

/// The CLI's machine verdict. Decoded tolerantly: a missing list degrades to
/// empty rather than failing the whole decode, but the CORE identity fields
/// (`schema`, `verdict`, `capability_band`) must be present or the decode fails
/// (we never invent a verdict the CLI did not author).
public struct DiffCapsVerdict: Equatable, Sendable {
    public let schema: String
    public let verdict: String
    public let authorityExpanded: Bool
    public let capabilityBand: String
    public let exitCode: Int
    public let aggregateGained: [String]
    public let aggregateRemoved: [String]
    public let wildcardIntroduced: Bool
    public let functionsAdded: [String]
    public let functionsRemoved: [String]
    public let functionsCapsExpanded: [DiffCapsFnExpansion]
    public let scope: String
}

extension DiffCapsVerdict: Codable {
    enum CodingKeys: String, CodingKey {
        case schema
        case verdict
        case authorityExpanded = "authority_expanded"
        case capabilityBand = "capability_band"
        case exitCode = "exit_code"
        case aggregateGained = "aggregate_gained"
        case aggregateRemoved = "aggregate_removed"
        case wildcardIntroduced = "wildcard_introduced"
        case functionsAdded = "functions_added"
        case functionsRemoved = "functions_removed"
        case functionsCapsExpanded = "functions_caps_expanded"
        case scope
    }

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        // Core identity: required.
        schema = try c.decode(String.self, forKey: .schema)
        verdict = try c.decode(String.self, forKey: .verdict)
        capabilityBand = try c.decode(String.self, forKey: .capabilityBand)
        // Tolerant: missing scalars/lists degrade to safe defaults.
        authorityExpanded = (try? c.decode(Bool.self, forKey: .authorityExpanded)) ?? false
        exitCode = (try? c.decode(Int.self, forKey: .exitCode)) ?? 0
        wildcardIntroduced = (try? c.decode(Bool.self, forKey: .wildcardIntroduced)) ?? false
        aggregateGained = (try? c.decode([String].self, forKey: .aggregateGained)) ?? []
        aggregateRemoved = (try? c.decode([String].self, forKey: .aggregateRemoved)) ?? []
        functionsAdded = (try? c.decode([String].self, forKey: .functionsAdded)) ?? []
        functionsRemoved = (try? c.decode([String].self, forKey: .functionsRemoved)) ?? []
        functionsCapsExpanded =
            (try? c.decode([DiffCapsFnExpansion].self, forKey: .functionsCapsExpanded)) ?? []
        scope = (try? c.decode(String.self, forKey: .scope)) ?? ""
    }
}

/// One diff-caps invocation's outcome: the verdict (when the CLI produced one)
/// plus the surrounding run metadata.
public struct DiffCapsReport: Equatable, Sendable {
    public let ran: Bool
    public let verdict: DiffCapsVerdict?
    public let exitCode: Int
    public let stderr: String
    public let evidencePath: String?

    public init(
        ran: Bool, verdict: DiffCapsVerdict?, exitCode: Int, stderr: String,
        evidencePath: String?
    ) {
        self.ran = ran
        self.verdict = verdict
        self.exitCode = exitCode
        self.stderr = stderr
        self.evidencePath = evidencePath
    }

    /// Decode a report from raw `garnet diff-caps --machine` stdout + run context.
    /// A non-decodable / absent verdict yields `verdict == nil` (the error card),
    /// never a fabricated clean verdict.
    public static func decode(
        stdout: Data, exitCode: Int, stderr: String, evidencePath: String? = nil
    ) -> DiffCapsReport {
        let verdict = try? JSONDecoder().decode(DiffCapsVerdict.self, from: stdout)
        return DiffCapsReport(
            ran: verdict != nil, verdict: verdict, exitCode: exitCode, stderr: stderr,
            evidencePath: evidencePath)
    }
}

/// The view-ready projection of a `DiffCapsReport`. `DiffCapsCard.render` is the
/// pure function M2's SwiftUI view lays out; all honesty rules live here so they
/// are unit-testable (`swift test`), not buried in a `View`.
public struct DiffCapsCard: Equatable, Sendable {
    public enum Tone: String, Sendable { case ok, fail }

    /// A titled list section (omitted entirely when empty, like the TS renderer).
    public struct Section: Equatable, Sendable {
        public let title: String
        public let items: [String]
    }

    public let isError: Bool
    public let errorText: String
    public let tone: Tone
    public let headline: String
    public let clean: Bool
    public let wildcardIntroduced: Bool
    public let sections: [Section]
    public let noChanges: Bool
    public let scope: String
    public let evidencePath: String?

    public static func render(_ report: DiffCapsReport) -> DiffCapsCard {
        guard report.ran, let v = report.verdict else {
            return DiffCapsCard(
                isError: true,
                errorText: report.stderr.isEmpty ? "no output" : report.stderr,
                tone: .fail,
                headline: "diff-caps produced no verdict (exit \(report.exitCode))",
                clean: false, wildcardIntroduced: false, sections: [], noChanges: false,
                scope: "", evidencePath: report.evidencePath)
        }
        // The band/verdict are the CLI's — never recomputed here.
        let clean = v.capabilityBand == "5/5" && !v.authorityExpanded
        let headline =
            clean
            ? "No declared authority gained — band 5/5"
            : "Authority expanded — band \(v.capabilityBand), review required"

        var sections: [Section] = []
        func push(_ title: String, _ items: [String]) {
            if !items.isEmpty { sections.append(Section(title: title, items: items)) }
        }
        push("Capabilities gained", v.aggregateGained)
        push("Capabilities removed", v.aggregateRemoved)
        push("Functions added", v.functionsAdded)
        push("Functions removed", v.functionsRemoved)
        if !v.functionsCapsExpanded.isEmpty {
            sections.append(
                Section(
                    title: "Functions that gained authority",
                    items: v.functionsCapsExpanded.map { "\($0.name) → \($0.gained.joined(separator: ", "))" }))
        }
        // "No changes" may be claimed ONLY when ALL six diff dimensions are empty —
        // a function-only change must never read as clean (the TS renderer's rule).
        let noChanges =
            v.aggregateGained.isEmpty && v.aggregateRemoved.isEmpty && v.functionsAdded.isEmpty
            && v.functionsRemoved.isEmpty && v.functionsCapsExpanded.isEmpty && !v.wildcardIntroduced

        return DiffCapsCard(
            isError: false, errorText: "", tone: clean ? .ok : .fail, headline: headline,
            clean: clean, wildcardIntroduced: v.wildcardIntroduced, sections: sections,
            noChanges: noChanges, scope: v.scope, evidencePath: report.evidencePath)
    }
}
