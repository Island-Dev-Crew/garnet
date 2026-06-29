import SwiftUI

/// Public entry point for the thin `GarnetStudio` executable target.
///
/// The app's `@main` was removed when these sources moved into the
/// `GarnetStudioKit` library target. The reason is the SwiftPM anti-pattern of
/// `@testable import`-ing an *executable* target: doing so links the executable's
/// entry point into the XCTest host, so `GarnetStudioApp.init()` (which calls
/// `Foundation.exit(...)` on `--self-test`/`--smoke-test`/… and constructs the
/// SwiftUI `App`) could run inside the test process and abort the
/// `GarnetStudioTests` suite on the CI macos-latest runner (it did —
/// unreproducible on a local GUI session). Testing a *library* removes that
/// hazard entirely. The executable's `main.swift` calls this; nothing else does.
public func runGarnetStudio() {
    GarnetStudioApp.main()
}
