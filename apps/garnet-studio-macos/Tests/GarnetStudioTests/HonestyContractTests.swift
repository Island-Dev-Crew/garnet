import XCTest

@testable import GarnetStudio

/// M1 — honesty-cleanup parity. A negative contract that pins the do-not-regress
/// honesty rails carried over from the five Windows reviews. The converter writes
/// plain files to ~/Desktop/dogfood with NO OS sandbox, so no action help may
/// claim "sandbox"; and every action's help must be real copy, never empty.
final class HonestyContractTests: XCTestCase {

    /// The action titles whose help copy is the user-facing honesty surface.
    private let actions = [
        "Parse", "Check", "Run", "Run Selected", "Convert", "Assist Plan",
        "Advisory Bundle", "Advisory Review", "Advisory Handoff",
    ]

    func testNoActionHelpClaimsAnOsSandbox() {
        // The surviving overclaim ("sandboxed output" on Convert) is relabeled;
        // assert it cannot return on ANY action. (When M4 adds a deferred OS-sandbox
        // *row label*, that allowed surface lives outside actionHelp, so this stays
        // a clean negative contract over the action copy.)
        for title in actions {
            let help = GarnetStudioRootView.actionHelp(for: title)
            XCTAssertFalse(
                help.lowercased().contains("sandbox"),
                "action '\(title)' help must not claim an OS sandbox the converter does not provide: \(help)"
            )
        }
    }

    func testConvertHelpDescribesLocalOutputWithoutAnySandboxClaim() {
        let help = GarnetStudioRootView.actionHelp(for: "Convert")
        XCTAssertTrue(help.contains("Active conversion"), "Convert help must still describe active conversion")
        XCTAssertTrue(
            help.lowercased().contains("local"),
            "Convert help must describe its real local output: \(help)"
        )
        XCTAssertFalse(
            help.lowercased().contains("sandbox"),
            "Convert help must not use the word 'sandbox' at all (the overclaim): \(help)"
        )
    }

    func testEveryKnownActionHasNonEmptyHelpCopy() {
        for title in actions {
            let help = GarnetStudioRootView.actionHelp(for: title)
            XCTAssertFalse(
                help.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty,
                "action '\(title)' must have real help copy")
        }
    }

    func testAdvisoryActionsKeepTheirNoSourceLeavesHonestyRail() {
        // Advisory actions must keep claiming source stays local / omitted —
        // the "no source leaves the machine" / "omits source" rail.
        let assist = GarnetStudioRootView.actionHelp(for: "Assist Plan").lowercased()
        XCTAssertTrue(
            assist.contains("no source") || assist.contains("leaves the machine"),
            "Assist Plan must keep its no-source-leaves honesty rail")
        let bundle = GarnetStudioRootView.actionHelp(for: "Advisory Bundle").lowercased()
        XCTAssertTrue(
            bundle.contains("omits source") || bundle.contains("no source"),
            "Advisory Bundle must keep its source-omission honesty rail")
    }
}
