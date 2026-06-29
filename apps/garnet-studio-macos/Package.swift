// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "GarnetStudio",
    platforms: [
        .macOS(.v14),
    ],
    products: [
        .executable(name: "GarnetStudio", targets: ["GarnetStudio"]),
    ],
    targets: [
        // All UI and logic live in this LIBRARY target. The test target
        // `@testable import`s the library — never the executable — so the app's
        // entry point is not linked into (and cannot run inside) the XCTest host.
        .target(
            name: "GarnetStudioKit",
            resources: [
                .copy("Resources/garnet-logo.png"),
            ]
        ),
        // Thin executable: just `main.swift`, which calls `runGarnetStudio()`.
        .executableTarget(
            name: "GarnetStudio",
            dependencies: ["GarnetStudioKit"]
        ),
        .testTarget(
            name: "GarnetStudioTests",
            dependencies: ["GarnetStudioKit"]
        ),
    ]
)
