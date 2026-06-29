import XCTest

@testable import GarnetStudio

/// M6 — tests for the (descoped) Bootstrap generator. The macOS Studio only
/// GENERATES scripts; these pin the plan derivation and — load-bearing — that
/// every generated script is allowlist-clean (no sudo / no curl|sh / no rm -rf /
/// no profile edit), so a future edit cannot smuggle a privileged command in.
final class BootstrapBridgeTests: XCTestCase {

    func testPlanWithCliIsPreflightOnly() {
        let plan = BootstrapPlan.from(cliPath: "/usr/local/bin/garnet")
        XCTAssertTrue(plan.cliFound)
        XCTAssertEqual(plan.steps, [.preflight])
    }

    func testPlanWithoutCliIncludesBuildAndConfigure() {
        let plan = BootstrapPlan.from(cliPath: nil)
        XCTAssertFalse(plan.cliFound)
        XCTAssertEqual(plan.steps, [.preflight, .buildCli, .configureEnv])
    }

    func testEveryGeneratedScriptIsAllowlistClean() {
        let plan = BootstrapPlan.from(cliPath: nil)
        for script in BootstrapGenerator.scripts(for: plan) {
            XCTAssertEqual(
                BootstrapGenerator.violations(in: script.contents), [],
                "\(script.name) must contain no forbidden token")
        }
    }

    func testForbiddenTokenIsActuallyDetected() {
        // The guard must not be a no-op: a privileged command is caught.
        let bad = "#!/usr/bin/env bash\nsudo rm -rf /\n"
        let violations = BootstrapGenerator.violations(in: bad)
        XCTAssertTrue(violations.contains("sudo"))
        XCTAssertTrue(violations.contains("rm -rf"))
    }

    func testScriptsAreBashAndNonEmpty() {
        for step in BootstrapStep.allCases {
            let body = step.scriptContents()
            XCTAssertTrue(body.hasPrefix("#!/usr/bin/env bash"), "\(step) must be a bash script")
            XCTAssertFalse(body.isEmpty)
        }
    }

    func testConfigureEnvOnlyPrintsThePathLineNeverEditsAProfile() {
        let body = BootstrapStep.configureEnv.scriptContents()
        XCTAssertTrue(body.contains("export PATH"), "must show the PATH line to add")
        XCTAssertFalse(body.contains(">> "), "must not append to any profile file")
        XCTAssertFalse(body.contains("~/.zshrc\""), "must not write the profile itself")
    }

    func testBuildCliRequiresAnExplicitCheckoutArgument() {
        let body = BootstrapStep.buildCli.scriptContents()
        XCTAssertTrue(body.contains("cargo build --release"))
        XCTAssertTrue(body.contains("${1:-"), "the checkout path must come from an argument")
        XCTAssertEqual(BootstrapGenerator.violations(in: body), [])
    }
}
