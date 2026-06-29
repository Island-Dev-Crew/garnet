import GarnetStudioKit

// Thin executable: all UI and logic live in `GarnetStudioKit` so the test target
// can `@testable import` the library without hosting this entry point in the
// XCTest process. See `runGarnetStudio()` for why the App's `@main` moved here.
runGarnetStudio()
