import XCTest

@testable import GarnetStudio

/// M3 — render-bridge tests for the Velocity Editor. These cover the tolerant
/// decoder (`VelocityReport`/`VelocityCheckOutput`) and the pure projection
/// (`VelocityCard.render`) — the logic M3's SwiftUI view lays out. The CLI's
/// diagnostics/summary are rendered as-is; nothing is recomputed here.
final class VelocityBridgeTests: XCTestCase {

    private func report(_ json: String, exit: Int = 0) -> VelocityReport {
        VelocityReport.decode(stdout: Data(json.utf8), exitCode: exit, stderr: "")
    }

    func testCleanBufferRendersOkCard() {
        let card = VelocityCard.render(
            report(#"{"diagnostics":[],"summary":{"errors":0,"warnings":0,"infos":0,"ok":true}}"#))
        XCTAssertFalse(card.isError)
        XCTAssertEqual(card.tone, .ok)
        XCTAssertTrue(card.clean)
        XCTAssertEqual(card.headline, "Clean — no diagnostics")
        XCTAssertTrue(card.rows.isEmpty)
    }

    func testErrorDiagnosticsRenderFailCardWithRows() {
        let card = VelocityCard.render(
            report(
                #"{"diagnostics":[{"severity":"error","code":"check.caps_coverage","message":"f needs fs via g","span":null}],"summary":{"errors":1,"warnings":0,"infos":0,"ok":false}}"#,
                exit: 1))
        XCTAssertFalse(card.isError)
        XCTAssertEqual(card.tone, .fail)
        XCTAssertFalse(card.clean)
        XCTAssertTrue(card.headline.contains("1 error"))
        XCTAssertEqual(card.rows.count, 1)
        XCTAssertEqual(card.rows.first?.code, "check.caps_coverage")
        XCTAssertEqual(card.rows.first?.severity, "error")
        XCTAssertNil(card.rows.first?.location, "a span-less diagnostic has no location")
    }

    func testWarningOnlyRendersWarnTone() {
        let card = VelocityCard.render(
            report(
                #"{"diagnostics":[{"severity":"warning","code":"check.boundary_note","message":"b","span":null}],"summary":{"errors":0,"warnings":1,"infos":0,"ok":true}}"#))
        XCTAssertEqual(card.tone, .warn)
        XCTAssertFalse(card.clean, "a warning is not a clean buffer")
        XCTAssertTrue(card.headline.contains("1 warning"))
    }

    func testSpanRendersByteLocation() {
        let card = VelocityCard.render(
            report(
                #"{"diagnostics":[{"severity":"error","code":"parse.reserved_word","message":"async is reserved","span":{"start":13,"len":5}}],"summary":{"errors":1,"warnings":0,"infos":0,"ok":false}}"#,
                exit: 1))
        XCTAssertEqual(card.rows.first?.location, "byte 13–18")
    }

    func testMissingDiagnosticsListDegradesToEmptyNotDecodeFailure() {
        // Tolerant decode: a payload missing the `diagnostics` array still decodes
        // (list defaults to empty) as long as `summary` is present.
        let card = VelocityCard.render(report(#"{"summary":{"ok":true}}"#))
        XCTAssertFalse(card.isError, "missing diagnostics must degrade, not fail the decode")
        XCTAssertTrue(card.clean)
    }

    func testMissingSummaryIsAnErrorCard() {
        // No `summary` -> not a check output the CLI authored -> error card, never
        // a fabricated clean result.
        let card = VelocityCard.render(report(#"{"diagnostics":[]}"#))
        XCTAssertTrue(card.isError)
        XCTAssertEqual(card.tone, .fail)
    }

    func testUnparseableOutputYieldsErrorCardNotAFabricatedClean() {
        let card = VelocityCard.render(report("not json at all", exit: 2))
        XCTAssertTrue(card.isError)
        XCTAssertTrue(card.headline.contains("exit 2"))
        XCTAssertFalse(card.clean, "an error must never read as clean")
    }

    func testExtractJSONObjectStripsMergedStderrNote() {
        let mixed =
            "note: this source has 1 prior failure(s) recorded in .garnet-cache/episodes.log\n"
            + #"{"diagnostics":[],"summary":{"errors":0,"warnings":0,"infos":0,"ok":true}}"#
            + "\n"
        let data = VelocityCheckCommand.extractJSONObject(from: mixed)
        XCTAssertNotNil(data, "must slice the JSON object out of merged stdout+stderr")
        let card = VelocityCard.render(
            VelocityReport.decode(stdout: data!, exitCode: 0, stderr: mixed))
        XCTAssertFalse(card.isError, "the diagnostics must decode despite the stderr note prefix")
        XCTAssertTrue(card.clean)
    }

    func testExtractJSONObjectReturnsNilWhenThereIsNoObject() {
        XCTAssertNil(VelocityCheckCommand.extractJSONObject(from: "note: no json here\n"))
    }
}
