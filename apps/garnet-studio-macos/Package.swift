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
        .executableTarget(
            name: "GarnetStudio",
            resources: [
                .copy("Resources/garnet-logo.png"),
            ]
        ),
        .testTarget(
            name: "GarnetStudioTests",
            dependencies: ["GarnetStudio"]
        ),
    ]
)
