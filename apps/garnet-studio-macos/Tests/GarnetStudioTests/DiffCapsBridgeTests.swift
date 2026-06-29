import XCTest

@testable import GarnetStudio

/// M0b — render-bridge tests for the diff-caps surface. These cover the decoder
/// (tolerant Codable) and the pure projection (`DiffCapsCard.render`) — the logic
/// M2's SwiftUI view lays out. The CLI verdict is rendered VERBATIM; the band is
/// never recomputed here.
final class DiffCapsBridgeTests: XCTestCase {

    private func report(_ json: String, exit: Int = 0) -> DiffCapsReport {
        DiffCapsReport.decode(
            stdout: Data(json.utf8), exitCode: exit, stderr: "", evidencePath: nil)
    }

    func testCleanVerdictIsBand5AndNotExpanded() {
        let card = DiffCapsCard.render(
            report(
                #"{"schema":"garnet.diff-caps.machine/1","verdict":"OK","authority_expanded":false,"capability_band":"5/5","exit_code":0,"aggregate_gained":[],"aggregate_removed":[],"wildcard_introduced":false,"functions_added":[],"functions_removed":[],"functions_caps_expanded":[],"scope":"x->y"}"#
            ))
        XCTAssertFalse(card.isError)
        XCTAssertEqual(card.tone, .ok)
        XCTAssertTrue(card.clean)
        XCTAssertTrue(card.noChanges)
        XCTAssertEqual(card.headline, "No declared authority gained — band 5/5")
        XCTAssertEqual(card.scope, "x->y")
    }

    func testAuthorityExpandedRendersTheCliBandVerbatim() {
        let card = DiffCapsCard.render(
            report(
                #"{"schema":"garnet.diff-caps.machine/1","verdict":"AUTHORITY EXPANDED","authority_expanded":true,"capability_band":"2/5","exit_code":1,"aggregate_gained":["fs"],"aggregate_removed":[],"wildcard_introduced":false,"functions_added":[],"functions_removed":[],"functions_caps_expanded":[{"name":"main","gained":["fs"]}],"scope":"base->prop"}"#
            ))
        XCTAssertFalse(card.clean)
        XCTAssertEqual(card.tone, .fail)
        XCTAssertEqual(card.headline, "Authority expanded — band 2/5, review required")
        XCTAssertFalse(card.noChanges)
        // The CLI's verdict string + band are rendered verbatim, never recomputed.
        XCTAssertTrue(card.headline.contains("2/5"))
        XCTAssertTrue(card.sections.contains(where: { $0.title == "Capabilities gained" && $0.items == ["fs"] }))
        XCTAssertTrue(
            card.sections.contains(where: {
                $0.title == "Functions that gained authority" && $0.items == ["main → fs"]
            }))
    }

    func testFunctionOnlyChangeNeverReadsAsCleanEmptyState() {
        // band 5/5, no aggregate change, but a function gained authority: must NOT
        // claim "no changes" (the load-bearing honesty rule from the TS renderer).
        let card = DiffCapsCard.render(
            report(
                #"{"schema":"garnet.diff-caps.machine/1","verdict":"REVIEW","authority_expanded":false,"capability_band":"5/5","exit_code":0,"aggregate_gained":[],"aggregate_removed":[],"wildcard_introduced":false,"functions_added":[],"functions_removed":[],"functions_caps_expanded":[{"name":"helper","gained":["net"]}],"scope":"s"}"#
            ))
        XCTAssertFalse(card.noChanges, "a function-only change must not read as 'no changes'")
    }

    func testWildcardIntroducedIsSurfaced() {
        let card = DiffCapsCard.render(
            report(
                #"{"schema":"garnet.diff-caps.machine/1","verdict":"AUTHORITY EXPANDED","authority_expanded":true,"capability_band":"1/5","exit_code":1,"aggregate_gained":["*"],"aggregate_removed":[],"wildcard_introduced":true,"functions_added":[],"functions_removed":[],"functions_caps_expanded":[],"scope":"s"}"#
            ))
        XCTAssertTrue(card.wildcardIntroduced)
        XCTAssertFalse(card.noChanges)
    }

    func testMissingListsDegradeToEmptyNotDecodeFailure() {
        // Tolerant decode: a verdict missing the array fields still decodes (lists
        // default to empty) as long as the core identity fields are present.
        let card = DiffCapsCard.render(
            report(
                #"{"schema":"garnet.diff-caps.machine/1","verdict":"OK","capability_band":"5/5"}"#))
        XCTAssertFalse(card.isError, "missing lists must degrade, not fail the decode")
        XCTAssertTrue(card.clean)
        XCTAssertTrue(card.noChanges)
    }

    func testUnparseableOutputYieldsErrorCardNotAFabricatedVerdict() {
        let card = DiffCapsCard.render(report("not json at all", exit: 2))
        XCTAssertTrue(card.isError)
        XCTAssertEqual(card.tone, .fail)
        XCTAssertTrue(card.headline.contains("exit 2"))
        XCTAssertFalse(card.clean, "an error must never read as clean")
    }

    func testMissingCoreIdentityFieldsAreAnErrorCard() {
        // No `verdict`/`capability_band` -> not a verdict the CLI authored -> error.
        let card = DiffCapsCard.render(report(#"{"schema":"garnet.diff-caps.machine/1"}"#))
        XCTAssertTrue(card.isError)
    }

    // ── M2: the command runner's JSON extraction (the stdout/stderr-merge gotcha) ──

    func testExtractJSONObjectStripsMergedStderrNoise() {
        let mixed =
            "note: this source has 1 prior failure(s) recorded in .garnet-cache/episodes.log\n"
            + #"{"schema":"garnet.diff-caps.machine/1","verdict":"AUTHORITY EXPANDED","authority_expanded":true,"capability_band":"2/5","aggregate_gained":["fs"]}"#
            + "\n"
        let data = DiffCapsCommand.extractJSONObject(from: mixed)
        XCTAssertNotNil(data, "must slice the JSON object out of merged stdout+stderr")
        let card = DiffCapsCard.render(
            DiffCapsReport.decode(stdout: data!, exitCode: 1, stderr: mixed))
        XCTAssertFalse(card.isError, "the verdict must decode despite the stderr note prefix")
        XCTAssertEqual(card.headline, "Authority expanded — band 2/5, review required")
    }

    func testExtractJSONObjectReturnsNilWhenThereIsNoObject() {
        XCTAssertNil(DiffCapsCommand.extractJSONObject(from: "note: no json here\n"))
    }
}
