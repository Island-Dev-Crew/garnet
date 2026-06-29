import XCTest

@testable import GarnetStudio

/// M4 — tests for the Enforced / Declared Legend catalog + pure projection. The
/// enforced-vs-declared boundary is a load-bearing honesty surface: these pin
/// that EXACTLY @caps + @max_depth are enforced, the named-deferred fences stay
/// declared, the OS sandbox stays deferred, and an enforced row reads "confirmed
/// live" ONLY when the live static-gate probe reproduced the expected code.
final class EnforcementLegendBridgeTests: XCTestCase {

    private func reportWith(codes: [String], ran: Bool = true) -> VelocityReport {
        let items = codes.map {
            #"{"severity":"error","code":"\#($0)","message":"m","span":null}"#
        }.joined(separator: ",")
        let json =
            #"{"diagnostics":[\#(items)],"summary":{"errors":\#(codes.count),"warnings":0,"infos":0,"ok":false}}"#
        return ran
            ? VelocityReport.decode(stdout: Data(json.utf8), exitCode: 1, stderr: "")
            : VelocityReport(ran: false, output: nil, exitCode: -1, stderr: "no cli")
    }

    func testCatalogPinsTheEnforcedDeclaredDeferredBoundary() {
        let fences = EnforcementCatalog.fences()
        let enforced = Set(fences.filter { $0.status == .enforced }.map(\.name))
        let declared = Set(fences.filter { $0.status == .declared }.map(\.name))
        let deferred = Set(fences.filter { $0.status == .deferred }.map(\.name))

        XCTAssertEqual(enforced, ["@caps", "@max_depth"], "exactly @caps + @max_depth are enforced")
        XCTAssertEqual(declared, ["@bounded", "@mailbox", "memory", "time"])
        XCTAssertEqual(deferred, ["OS sandbox (macOS / Windows)"])
        // seccomp is Linux-only; the deferred row must say so.
        let os = fences.first { $0.name.contains("OS sandbox") }
        XCTAssertEqual(os?.backends, "Linux seccomp only")
    }

    func testProbeConfirmsOnlyWhenExpectedCodeReproduced() {
        let p = EnforcementProbe.from(
            fence: "@caps", expectedCode: "check.caps_coverage",
            report: reportWith(codes: ["check.caps_coverage"]))
        XCTAssertTrue(p.confirmed)
        XCTAssertTrue(p.ran)
    }

    func testProbeUnconfirmedWhenRanButCodeAbsent() {
        let p = EnforcementProbe.from(
            fence: "@caps", expectedCode: "check.caps_coverage",
            report: reportWith(codes: ["check.boundary_note"]))
        XCTAssertFalse(p.confirmed, "a different code must not confirm the gate")
        XCTAssertTrue(p.ran)
        XCTAssertEqual(p.observedCodes, ["check.boundary_note"])
    }

    func testProbeInconclusiveWhenCheckDidNotRun() {
        let p = EnforcementProbe.from(
            fence: "@max_depth", expectedCode: "check.annotation_error",
            report: reportWith(codes: [], ran: false))
        XCTAssertFalse(p.confirmed)
        XCTAssertFalse(p.ran)
    }

    func testRenderOrdersEnforcedThenDeclaredThenDeferred() {
        let card = EnforcementLegendCard.render(
            EnforcementLegend(fences: EnforcementCatalog.fences(), probes: [], cliAvailable: true))
        let statuses = card.rows.map(\.status)
        let firstDeclared = statuses.firstIndex(of: .declared) ?? .max
        let firstDeferred = statuses.firstIndex(of: .deferred) ?? .max
        let lastEnforced = statuses.lastIndex(of: .enforced) ?? -1
        XCTAssertLessThan(lastEnforced, firstDeclared, "enforced rows come before declared")
        XCTAssertLessThan(firstDeclared, firstDeferred, "declared rows come before deferred")
    }

    func testConfirmedGateLineOnlyWhenProbeReproduced() {
        let confirmedProbe = EnforcementProbe.from(
            fence: "@caps", expectedCode: "check.caps_coverage",
            report: reportWith(codes: ["check.caps_coverage"]))
        let card = EnforcementLegendCard.render(
            EnforcementLegend(
                fences: EnforcementCatalog.fences(), probes: [confirmedProbe], cliAvailable: true))
        let caps = card.rows.first { $0.name == "@caps" }
        XCTAssertEqual(caps?.gateState, .confirmed)
        XCTAssertTrue(caps?.gateLine.contains("confirmed live") ?? false)
        // A declared fence never carries a gate line.
        let bounded = card.rows.first { $0.name == "@bounded" }
        XCTAssertEqual(bounded?.gateState, .notApplicable)
        XCTAssertTrue(bounded?.gateLine.isEmpty ?? false)
    }

    func testNoCliYieldsInconclusiveProbesNotFakedGreen() {
        let legend = EnforcementLegendCommand.build(cli: nil, timeoutSeconds: 5)
        XCTAssertFalse(legend.cliAvailable)
        XCTAssertFalse(legend.probes.isEmpty, "enforced fences still get an (inconclusive) probe")
        XCTAssertTrue(legend.probes.allSatisfy { !$0.confirmed && !$0.ran })
        let card = EnforcementLegendCard.render(legend)
        let caps = card.rows.first { $0.name == "@caps" }
        XCTAssertEqual(caps?.gateState, .notProbed)
        XCTAssertTrue(caps?.gateLine.contains("not probed") ?? false)
    }
}
