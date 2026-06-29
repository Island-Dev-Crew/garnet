import XCTest

@testable import GarnetStudio

/// M5 — tests for the Agent-Loop Console pure parser. The verdict is READ from
/// decision.md and the gate pipeline derived from it — never recomputed. These
/// pin: the accepted/rejected mapping, the rejected-at-gate cases, the seal-only-
/// when-parsed rule, verbatim decision/diff-caps text, and the artifact decoders.
final class AgentLoopBridgeTests: XCTestCase {

    private func files(
        decision: String?, diffCaps: String? = nil, manifest: String? = nil,
        seal: String? = nil, log: String? = nil, runTrap: String? = nil
    ) -> RecordDirFiles {
        RecordDirFiles(
            decisionMd: decision, diffCapsTxt: diffCaps, capabilityManifestJson: manifest,
            sealJson: seal, transparencyLogJsonl: log, runTrapTxt: runTrap)
    }

    private func status(_ d: AgentLoopDossier, _ gate: AgentLoopGate) -> GateStatus? {
        d.gates.first { $0.gate == gate }?.status
    }

    private let sealJson = """
        {"predicate":{"authorship":"Jon Isaac","attestation":{"agent":"claude","autonomous":"true","decision":"accepted","gate_version":"v1","model":"opus","tool":"garnet"}}}
        """

    func testAcceptedDossierPassesAllGatesWhenSealed() {
        let d = AgentLoopDossier.parse(
            recordDir: "/r",
            files: files(decision: "# ACCEPTED on capability + depth evidence\n", seal: sealJson))
        XCTAssertTrue(d.ran)
        XCTAssertEqual(d.outcome, .accepted)
        XCTAssertNil(d.rejectedAt)
        for gate in AgentLoopGate.allCases {
            XCTAssertEqual(status(d, gate), .pass, "\(gate) should pass on an accepted+sealed dossier")
        }
        XCTAssertEqual(d.sealAuthorship, "Jon Isaac")
        XCTAssertEqual(d.sealAttestation?.model, "opus")
        XCTAssertEqual(d.sealAttestation?.gateVersion, "v1")
    }

    func testAcceptedButNoSealMarksSealGateNotReached() {
        // The Seal gate may only read Pass when a seal was actually parsed.
        let d = AgentLoopDossier.parse(
            recordDir: "/r", files: files(decision: "# Accepted\n"))
        XCTAssertEqual(d.outcome, .accepted)
        XCTAssertEqual(status(d, .check), .pass)
        XCTAssertEqual(status(d, .seal), .notReached, "no seal.json -> seal not 'sealed' from heading")
        XCTAssertNil(d.sealAttestation)
    }

    func testRejectedAtDiffCapsWideningGate() {
        let d = AgentLoopDossier.parse(
            recordDir: "/r",
            files: files(
                decision: "# REJECTED — capability widening detected\n",
                diffCaps: "diff-caps: authority expanded (capability band 2/5)\n"))
        XCTAssertEqual(d.outcome, .rejected)
        XCTAssertEqual(d.rejectedAt, .diffCaps)
        XCTAssertEqual(status(d, .check), .pass)
        XCTAssertEqual(status(d, .diffCaps), .reject)
        XCTAssertEqual(status(d, .run), .notReached)
        XCTAssertEqual(status(d, .seal), .notReached)
        XCTAssertEqual(
            d.gates.first { $0.gate == .diffCaps }?.detail,
            "diff-caps: authority expanded (capability band 2/5)",
            "the diff-caps verdict line is rendered verbatim")
    }

    func testRejectedAtRunTrapGate() {
        let d = AgentLoopDossier.parse(
            recordDir: "/r",
            files: files(
                decision: "# REJECTED — recursion ceiling trap\n",
                runTrap: "garnet: runtime error: @max_depth(8) exceeded\n"))
        XCTAssertEqual(d.rejectedAt, .run)
        XCTAssertEqual(status(d, .diffCaps), .pass)
        XCTAssertEqual(status(d, .run), .reject)
        XCTAssertEqual(
            d.gates.first { $0.gate == .run }?.detail,
            "garnet: runtime error: @max_depth(8) exceeded")
    }

    func testRejectedAtCheckGate() {
        let d = AgentLoopDossier.parse(
            recordDir: "/r", files: files(decision: "# REJECTED at check\n"))
        XCTAssertEqual(d.rejectedAt, .check)
        XCTAssertEqual(status(d, .check), .reject)
        XCTAssertEqual(status(d, .diffCaps), .notReached)
    }

    func testMissingDecisionMdIsAnErrorDossier() {
        let d = AgentLoopDossier.parse(recordDir: "/r", files: files(decision: nil))
        XCTAssertFalse(d.ran)
        XCTAssertTrue(d.error.contains("decision.md"))
    }

    func testDecisionMdIsCarriedVerbatim() {
        let body = "# ACCEPTED\n\nScope: this attests capability + depth only, not full safety.\n"
        let d = AgentLoopDossier.parse(recordDir: "/r", files: files(decision: body, seal: sealJson))
        XCTAssertEqual(d.decisionMd, body, "decision.md is carried byte-for-byte")
    }

    func testCapabilityManifestAndLogDecode() {
        let manifest = """
            {"schema":"garnet-capability-manifest-v1","aggregate":["fs"],"functions":[{"name":"main","caps":["fs"]}],"wildcard":false}
            """
        let log = """
            {"index":0,"program":"a.garnet","caps":["fs"],"caps_blake3":"abc123","prev_blake3":"000000"}
            {"index":1,"program":"b.garnet","caps":[],"caps_blake3":"def456","prev_blake3":"abc123"}
            """
        let d = AgentLoopDossier.parse(
            recordDir: "/r",
            files: files(decision: "# Accepted\n", manifest: manifest, seal: sealJson, log: log))
        XCTAssertEqual(d.capabilityManifest?.schema, "garnet-capability-manifest-v1")
        XCTAssertEqual(d.capabilityManifest?.aggregate, ["fs"])
        XCTAssertEqual(d.capabilityManifest?.functions.first?.name, "main")
        XCTAssertEqual(d.transparencyLog.count, 2)
        XCTAssertEqual(d.transparencyLog.last?.index, 1)
        XCTAssertEqual(d.transparencyLog.first?.capsBlake3, "abc123")
    }

    func testNoSealOnRejectionIsTheNegativeProofNotAnError() {
        let d = AgentLoopDossier.parse(
            recordDir: "/r", files: files(decision: "# REJECTED — capability widening\n"))
        XCTAssertNil(d.sealAttestation, "a rejected proposal attests nothing")
        XCTAssertEqual(d.outcome, .rejected)
    }
}
