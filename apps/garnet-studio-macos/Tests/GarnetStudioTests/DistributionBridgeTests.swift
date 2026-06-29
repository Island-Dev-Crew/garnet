import XCTest

@testable import GarnetStudio

/// M7 — tests for the Distribution Reporter. The load-bearing honesty: the macOS
/// .app is unsigned + un-notarized, so signing and notarization are always
/// `deferred` and the headline never claims distribution-readiness. Probe items
/// read `unverified` without a repo root — never a fabricated "ready".
final class DistributionBridgeTests: XCTestCase {

    private func item(_ report: DistributionReport, _ name: String) -> DistributionItem? {
        report.items.first { $0.name == name }
    }

    func testSigningAndNotarizationAreAlwaysDeferred() {
        let report = DistributionReport.build(repoRootProvided: true, exists: { _ in true })
        XCTAssertEqual(item(report, "Code signing (Developer ID)")?.status, .deferred)
        XCTAssertEqual(item(report, "Notarization (Apple notary)")?.status, .deferred)
        XCTAssertEqual(item(report, "Gatekeeper acceptance")?.status, .deferred)
    }

    func testHeadlineNeverClaimsDistributionReady() {
        let report = DistributionReport.build(repoRootProvided: true, exists: { _ in true })
        XCTAssertTrue(report.headline.contains("unsigned and un-notarized"))
        XCTAssertTrue(report.headline.contains("local run only"))
        XCTAssertFalse(report.headline.lowercased().contains("distribution-ready"))
    }

    func testProbeItemsAreUnverifiedWithoutRepoRoot() {
        let report = DistributionReport.build(repoRootProvided: false, exists: { _ in true })
        XCTAssertEqual(item(report, "Studio .app packager")?.status, .unverified)
        XCTAssertEqual(item(report, "DMG smoke harness")?.status, .unverified)
    }

    func testProbeItemsReadyWhenPresentAbsentWhenMissing() {
        let present = DistributionReport.build(
            repoRootProvided: true, exists: { $0 == "scripts/package_garnet_studio_macos.sh" })
        XCTAssertEqual(item(present, "Studio .app packager")?.status, .ready)
        XCTAssertEqual(item(present, "DMG smoke harness")?.status, .absent, "a missing probe reads absent")
    }

    func testCliSignedArtifactsAreReadyOutOfBand() {
        let report = DistributionReport.build(repoRootProvided: false, exists: { _ in false })
        // The CLI artifacts are not a repo-tree probe; they are ready out-of-band
        // (the v0.8.1 release), so they stay ready even with no repo root.
        XCTAssertEqual(item(report, "CLI signed artifacts")?.status, .ready)
    }

    func testReportFromCommandWithNoRepoRootIsUnverifiedProbes() {
        let report = DistributionCommand.report(repoRoot: nil)
        XCTAssertFalse(report.repoRootProvided)
        XCTAssertEqual(item(report, "Studio .app packager")?.status, .unverified)
    }
}
