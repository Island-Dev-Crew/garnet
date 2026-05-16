import XCTest
@testable import GarnetStudio

final class GarnetStudioTests: XCTestCase {
    func testCliLocatorPrefersBundledExecutable() {
        let bundled = URL(fileURLWithPath: "/Applications/Garnet Studio.app/Contents/Resources/garnet")
        let locator = GarnetCLILocator(
            bundleResourceURL: bundled.deletingLastPathComponent(),
            environmentPath: "/usr/local/bin:/opt/homebrew/bin"
        )

        XCTAssertEqual(locator.candidatePaths().first, bundled.path)
    }

    func testCliLocatorFallsBackToCommonDeveloperPaths() {
        let locator = GarnetCLILocator(bundleResourceURL: nil, environmentPath: "/custom/bin:/usr/bin")

        let candidates = locator.candidatePaths()

        XCTAssertTrue(candidates.contains("/custom/bin/garnet"))
        XCTAssertTrue(candidates.contains("/usr/local/bin/garnet"))
        XCTAssertTrue(candidates.contains("/opt/homebrew/bin/garnet"))
    }

    func testSampleCatalogCoversCoreWorkbenchModes() {
        let modes = Set(GarnetSampleCatalog.samples.map(\.mode))

        XCTAssertTrue(modes.contains(.parse))
        XCTAssertTrue(modes.contains(.check))
        XCTAssertTrue(modes.contains(.run))
        XCTAssertTrue(modes.contains(.convert))
    }

    func testCommandResultClassifiesExitStatus() {
        let success = GarnetCommandResult(command: "garnet version", exitCode: 0, output: "garnet 0.4.2")
        let failure = GarnetCommandResult(command: "garnet check broken.garnet", exitCode: 1, output: "diagnostic")

        XCTAssertEqual(success.status, .success)
        XCTAssertEqual(failure.status, .failure)
    }
}
