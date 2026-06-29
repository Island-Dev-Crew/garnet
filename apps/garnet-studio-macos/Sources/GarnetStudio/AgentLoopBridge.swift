// M5 — Agent-Loop Console (pure parser + projection).
//
// Renders an EXISTING `garnet agent-loop --record-dir` dossier as a four-gate
// pipeline (check → diff-caps → run → seal). Ported from the Windows shell's
// reader (apps/garnet-studio/src-tauri/src/commands.rs) so both Studios show the
// same verdict. Every verdict is the CLI's own — read verbatim from
// `decision.md` and the artifacts, NEVER recomputed. The console never runs the
// agent loop; it only reads a directory. Human approval, the widening (diff-caps)
// gate, and seal provenance stay visibly separate. Acceptance is "on capability +
// depth evidence" only — never a claim of full boundedness or safety.

import Foundation

public enum AgentLoopGate: String, Equatable, Sendable, CaseIterable {
    case check
    case diffCaps = "diff-caps"
    case run
    case seal

    public var order: Int {
        switch self {
        case .check: return 0
        case .diffCaps: return 1
        case .run: return 2
        case .seal: return 3
        }
    }

    public var label: String {
        switch self {
        case .check: return "check"
        case .diffCaps: return "diff-caps"
        case .run: return "run · enforced kernel"
        case .seal: return "seal"
        }
    }
}

public enum GateStatus: String, Equatable, Sendable {
    case pass
    case reject
    case notReached

    public var glyph: String {
        switch self {
        case .pass: return "✓"
        case .reject: return "✕"
        case .notReached: return "·"
        }
    }

    public var label: String {
        switch self {
        case .pass: return "pass"
        case .reject: return "reject"
        case .notReached: return "not reached"
        }
    }
}

public enum AgentLoopOutcome: String, Equatable, Sendable {
    case accepted
    case rejected
}

public struct AgentLoopGateRow: Equatable, Sendable {
    public let gate: AgentLoopGate
    public let status: GateStatus
    public let detail: String
}

public struct ManifestFunction: Codable, Equatable, Sendable {
    public let name: String
    public let caps: [String]
}

public struct CapabilityManifest: Equatable, Sendable {
    public let schema: String
    public let aggregate: [String]
    public let functions: [ManifestFunction]
    public let wildcard: Bool
}

extension CapabilityManifest: Codable {
    enum CodingKeys: String, CodingKey { case schema, aggregate, functions, wildcard }
    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        schema = (try? c.decode(String.self, forKey: .schema)) ?? ""
        aggregate = (try? c.decode([String].self, forKey: .aggregate)) ?? []
        functions = (try? c.decode([ManifestFunction].self, forKey: .functions)) ?? []
        wildcard = (try? c.decode(Bool.self, forKey: .wildcard)) ?? false
    }
}

/// Autonomous-acceptance provenance — rendered as its OWN panel, kept visibly
/// separate from human approval (there is none) and the widening gate.
public struct SealAttestation: Equatable, Sendable {
    public let agent: String
    public let autonomous: String
    public let decision: String
    public let gateVersion: String
    public let model: String
    public let tool: String
}

public struct TransparencyLogEntry: Equatable, Sendable {
    public let index: Int
    public let program: String
    public let caps: [String]
    public let capsBlake3: String
    public let prevBlake3: String
}

extension TransparencyLogEntry: Codable {
    enum CodingKeys: String, CodingKey {
        case index, program, caps
        case capsBlake3 = "caps_blake3"
        case prevBlake3 = "prev_blake3"
    }
    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        index = (try? c.decode(Int.self, forKey: .index)) ?? 0
        program = (try? c.decode(String.self, forKey: .program)) ?? ""
        caps = (try? c.decode([String].self, forKey: .caps)) ?? []
        capsBlake3 = (try? c.decode(String.self, forKey: .capsBlake3)) ?? ""
        prevBlake3 = (try? c.decode(String.self, forKey: .prevBlake3)) ?? ""
    }
}

/// The raw text of the record-dir artifacts. Kept separate from disk I/O so the
/// parser is a pure function, unit-testable without a directory.
public struct RecordDirFiles: Equatable, Sendable {
    public let decisionMd: String?
    public let diffCapsTxt: String?
    public let capabilityManifestJson: String?
    public let sealJson: String?
    public let transparencyLogJsonl: String?
    public let runTrapTxt: String?

    public init(
        decisionMd: String?, diffCapsTxt: String?, capabilityManifestJson: String?,
        sealJson: String?, transparencyLogJsonl: String?, runTrapTxt: String?
    ) {
        self.decisionMd = decisionMd
        self.diffCapsTxt = diffCapsTxt
        self.capabilityManifestJson = capabilityManifestJson
        self.sealJson = sealJson
        self.transparencyLogJsonl = transparencyLogJsonl
        self.runTrapTxt = runTrapTxt
    }
}

/// The whole dossier read from a `--record-dir`.
public struct AgentLoopDossier: Equatable, Sendable {
    public let ran: Bool
    public let recordDir: String
    public let outcome: AgentLoopOutcome
    public let rejectedAt: AgentLoopGate?
    public let gates: [AgentLoopGateRow]
    public let decisionMd: String
    public let diffCapsText: String
    public let capabilityManifest: CapabilityManifest?
    public let sealAuthorship: String
    public let sealAttestation: SealAttestation?
    public let transparencyLog: [TransparencyLogEntry]
    public let error: String

    static func error(_ recordDir: String, _ message: String) -> AgentLoopDossier {
        AgentLoopDossier(
            ran: false, recordDir: recordDir, outcome: .rejected, rejectedAt: nil, gates: [],
            decisionMd: "", diffCapsText: "", capabilityManifest: nil, sealAuthorship: "",
            sealAttestation: nil, transparencyLog: [], error: message)
    }

    /// Pure parser over the record-dir artifact contents — no disk I/O, so the
    /// verdict mapping and gate derivation are unit-testable directly.
    public static func parse(recordDir: String, files: RecordDirFiles) -> AgentLoopDossier {
        guard let decisionMd = files.decisionMd else {
            return .error(
                recordDir,
                "no decision.md in the directory — not an agent-loop --record-dir dossier.")
        }
        let (outcome, rejectedAt) = parseOutcome(decisionMd)
        let diffCapsText = files.diffCapsTxt ?? ""
        let manifest = files.capabilityManifestJson.flatMap(decodeManifest)
        let (authorship, attestation) = parseSeal(files.sealJson)
        let log = parseTransparencyLog(files.transparencyLogJsonl)
        let gates = buildGates(
            outcome: outcome, rejectedAt: rejectedAt, diffCapsText: diffCapsText,
            runTrap: files.runTrapTxt, sealPresent: attestation != nil)

        return AgentLoopDossier(
            ran: true, recordDir: recordDir, outcome: outcome, rejectedAt: rejectedAt,
            gates: gates, decisionMd: decisionMd, diffCapsText: diffCapsText,
            capabilityManifest: manifest, sealAuthorship: authorship,
            sealAttestation: attestation, transparencyLog: log, error: "")
    }

    /// The verdict is READ from `decision.md`'s first line — never recomputed. The
    /// three rejection reasons map to the gate that refused.
    static func parseOutcome(_ decisionMd: String) -> (AgentLoopOutcome, AgentLoopGate?) {
        let head = (decisionMd.split(whereSeparator: \.isNewline).first.map(String.init) ?? "")
            .lowercased()
        if head.contains("accepted") { return (.accepted, nil) }
        if head.contains("widening") { return (.rejected, .diffCaps) }
        if head.contains("trap") || head.contains("ceiling") { return (.rejected, .run) }
        if head.contains("check") { return (.rejected, .check) }
        return (.rejected, nil)
    }

    static func buildGates(
        outcome: AgentLoopOutcome, rejectedAt: AgentLoopGate?, diffCapsText: String,
        runTrap: String?, sealPresent: Bool
    ) -> [AgentLoopGateRow] {
        AgentLoopGate.allCases.map { gate in
            let status: GateStatus
            switch outcome {
            case .accepted:
                // The Seal gate may only read Pass when a seal was actually
                // parsed — never "sealed" from the heading alone.
                status = (gate == .seal && !sealPresent) ? .notReached : .pass
            case .rejected:
                if let rg = rejectedAt {
                    if gate == rg {
                        status = .reject
                    } else if gate.order < rg.order {
                        status = .pass
                    } else {
                        status = .notReached
                    }
                } else {
                    // Unknown rejection: claim no gate passed rather than overstate.
                    status = .notReached
                }
            }
            return AgentLoopGateRow(
                gate: gate, status: status,
                detail: gateDetail(gate, status, diffCapsText, runTrap))
        }
    }

    static func gateDetail(
        _ gate: AgentLoopGate, _ status: GateStatus, _ diffCapsText: String, _ runTrap: String?
    ) -> String {
        switch (gate, status) {
        case (_, .notReached): return ""
        case (.check, .pass):
            return "parsed and checked — fails closed before diff-caps, run, or seal"
        case (.check, .reject): return "proposal failed `garnet check`"
        case (.diffCaps, _): return diffCapsVerdictLine(diffCapsText)
        case (.run, .pass):
            return "ran without tripping the enforced kernel (@caps + @max_depth)"
        case (.run, .reject): return runTrapLine(runTrap)
        case (.seal, .pass):
            return "sealed — attested with autonomous-acceptance provenance (seal.json)"
        case (.seal, .reject): return "seal step errored"
        }
    }

    /// The CLI's own one-line diff-caps verdict — rendered verbatim, never recomputed.
    static func diffCapsVerdictLine(_ text: String) -> String {
        text.split(whereSeparator: \.isNewline)
            .map { $0.trimmingCharacters(in: .whitespaces) }
            .last { $0.hasPrefix("diff-caps:") } ?? ""
    }

    /// The runtime-error line from `run_trap.txt`, verbatim.
    static func runTrapLine(_ trap: String?) -> String {
        guard let trap else { return "" }
        let lines = trap.split(whereSeparator: \.isNewline).map {
            $0.trimmingCharacters(in: .whitespaces)
        }
        return lines.last { $0.contains("runtime error") } ?? lines.last { !$0.isEmpty } ?? ""
    }

    static func decodeManifest(_ json: String) -> CapabilityManifest? {
        guard let data = json.trimmingCharacters(in: .whitespacesAndNewlines).data(using: .utf8)
        else { return nil }
        return try? JSONDecoder().decode(CapabilityManifest.self, from: data)
    }

    /// seal.json carries the attestation under `predicate` (or at the root). Parsed
    /// loosely so a partial seal still surfaces the authorship it does have.
    static func parseSeal(_ json: String?) -> (String, SealAttestation?) {
        guard let json,
            let data = json.trimmingCharacters(in: .whitespacesAndNewlines).data(using: .utf8),
            let root = try? JSONSerialization.jsonObject(with: data) as? [String: Any]
        else { return ("", nil) }
        let predicate = (root["predicate"] as? [String: Any]) ?? root
        let authorship = predicate["authorship"] as? String ?? ""
        var attestation: SealAttestation?
        if let a = predicate["attestation"] as? [String: Any] {
            attestation = SealAttestation(
                agent: a["agent"] as? String ?? "",
                autonomous: a["autonomous"] as? String ?? "",
                decision: a["decision"] as? String ?? "",
                gateVersion: a["gate_version"] as? String ?? "",
                model: a["model"] as? String ?? "",
                tool: a["tool"] as? String ?? "")
        }
        return (authorship, attestation)
    }

    static func parseTransparencyLog(_ jsonl: String?) -> [TransparencyLogEntry] {
        guard let jsonl else { return [] }
        return jsonl.split(whereSeparator: \.isNewline)
            .filter { !$0.trimmingCharacters(in: .whitespaces).isEmpty }
            .compactMap { line in
                guard let data = line.data(using: .utf8) else { return nil }
                return try? JSONDecoder().decode(TransparencyLogEntry.self, from: data)
            }
    }
}
