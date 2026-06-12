// StudioChrome.swift — UI rows of the PR #391 standard, macOS-native.
//
//   Row 2  Launch experience  — SplashView holds during boot with live status;
//                               700 ms minimum, 25 s hard dismissal ceiling;
//                               honors Reduce Motion.
//   Row 3  Simple/Power modes — @AppStorage-backed (native affordance); the
//                               power-only sections stay compiled into the app
//                               (contract copy intact) and are hidden by mode,
//                               never removed from the source.
//   Row 6  Truth surface      — TruthTilesPanel renders docs/truth.json values
//                               or an explicit "truth surface unavailable"
//                               state; zero hand-written release numbers.
//   Row 9  Status bar         — version • mode • truth state • evidence root.
//
// Row 7 (hover help) is applied at each control via native `.help(...)` tags.

import SwiftUI

// MARK: - Row 2 · Splash

struct SplashView: View {
    let status: String
    let reduceMotion: Bool
    @State private var pulse = false

    var body: some View {
        ZStack {
            Color(red: 0.04, green: 0.04, blue: 0.05)
                .ignoresSafeArea()
            VStack(spacing: 18) {
                LogoView()
                    .frame(width: 96, height: 96)
                    .scaleEffect(pulse && !reduceMotion ? 1.04 : 1.0)
                    .animation(
                        reduceMotion ? nil : .easeInOut(duration: 1.1).repeatForever(autoreverses: true),
                        value: pulse
                    )
                Text("Garnet Studio")
                    .font(.system(size: 30, weight: .bold, design: .rounded))
                Text("Rust Rigor. Ruby Velocity. One Coherent Language.")
                    .font(.callout)
                    .foregroundStyle(.secondary)
                ProgressView()
                    .controlSize(.small)
                    .padding(.top, 6)
                Text(status)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .accessibilityLabel("Launch status: \(status)")
            }
        }
        .onAppear { pulse = true }
        .help("Garnet Studio is checking the local toolchain before opening the workbench.")
    }
}

/// Boot controller: runs the launch checks off the main thread and enforces
/// the 700 ms minimum / 25 s ceiling from `StudioBootSequence`.
@MainActor
final class StudioBootModel: ObservableObject {
    @Published var splashVisible = true
    @Published var statusMessage = "Starting…"
    @Published var truth: StudioTruthSummary = .unavailable(reason: "not yet loaded")
    @Published var cliLocated = false

    private let started = Date()

    func beginBoot() {
        statusMessage = "Loading settings…"
        let settings = StudioSettingsStore().load().normalized()
        _ = settings // settings are consumed by the runner bridge per spawn

        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            let cliPath = GarnetCLILocator().locate()
            let repoRoot = StudioTruthSummary.locateRepoRoot()
            let truth = StudioTruthSummary.load(repoRoot: repoRoot)
            DispatchQueue.main.async {
                guard let self else { return }
                self.cliLocated = cliPath != nil
                self.truth = truth
                self.statusMessage = cliPath != nil
                    ? "garnet CLI located — opening workbench"
                    : "garnet CLI not found yet — workbench opens with locator help"
                self.dismissAfterMinimum()
            }
        }

        // Hard ceiling: never hold the splash past the bound, even if a check hangs.
        DispatchQueue.main.asyncAfter(
            deadline: .now() + .milliseconds(StudioBootSequence.ceilingMilliseconds)
        ) { [weak self] in
            self?.splashVisible = false
        }
    }

    private func dismissAfterMinimum() {
        let elapsedMs = Int(Date().timeIntervalSince(started) * 1000)
        let waitMs = max(0, StudioBootSequence.minimumMilliseconds - elapsedMs)
        DispatchQueue.main.asyncAfter(deadline: .now() + .milliseconds(waitMs)) { [weak self] in
            self?.splashVisible = false
        }
    }
}

// MARK: - Row 6 · Truth tiles

struct TruthTilesPanel: View {
    let truth: StudioTruthSummary

    var body: some View {
        Panel(title: "Live Truth Surface") {
            switch truth {
            case .unavailable(let reason):
                Text("Truth surface unavailable — \(reason)")
                    .font(.callout)
                    .foregroundStyle(.secondary)
                    .help("Live release statistics come from docs/truth.json (the xtask truth surface). When it cannot be read, Studio says so instead of inventing numbers.")
            case .loaded(let fields):
                HStack(spacing: 12) {
                    TruthTile(label: "Version", value: fields.version ?? "—")
                        .help("Workspace version stamped by cargo xtask truth — not hand-written.")
                    TruthTile(label: "Latest tag", value: fields.latestTag ?? "—")
                        .help("Latest release tag recorded in docs/truth.json.")
                    TruthTile(label: "Primitives", value: fields.primitiveCount.map(String.init) ?? "—")
                        .help("Stdlib registry primitive count, generated from the registry itself.")
                    TruthTile(label: "Workspace tests", value: fields.workspaceTestCount.map(String.init) ?? "—")
                        .help("Workspace test count from the truth surface; an absent value renders as a dash, never a guess.")
                }
                Text("Values are read live from docs/truth.json; Studio never hand-writes release statistics.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
    }
}

struct TruthTile: View {
    let label: String
    let value: String

    var body: some View {
        VStack(spacing: 4) {
            Text(value)
                .font(.system(size: 20, weight: .bold, design: .rounded))
            Text(label)
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .frame(minWidth: 110)
        .padding(.vertical, 10)
        .padding(.horizontal, 8)
        .background(Color.white.opacity(0.06))
        .clipShape(RoundedRectangle(cornerRadius: 10))
        .accessibilityElement(children: .combine)
        .accessibilityLabel("\(label): \(value)")
    }
}

// MARK: - Row 9 · Status bar

struct StudioStatusBar: View {
    let interfaceMode: String
    let truth: StudioTruthSummary
    let cliLocated: Bool

    private var truthBadge: String {
        if case .loaded = truth { return "truth: live" }
        return "truth: unavailable"
    }

    var body: some View {
        HStack(spacing: 14) {
            Text("Garnet Studio v\(StudioVersion.release)")
                .help("Single version stamp, gated against the workspace release by the macOS shell contract test.")
            Divider().frame(height: 12)
            Text("mode: \(interfaceMode)")
                .help("Simple mode shows the core workbench; Power mode reveals the full cockpit. Toggle in the header or Settings.")
            Divider().frame(height: 12)
            Text(truthBadge)
                .help("Whether docs/truth.json was readable at launch. Unavailable is reported honestly, never papered over.")
            Divider().frame(height: 12)
            Text(cliLocated ? "garnet CLI: located" : "garnet CLI: not found")
                .help("Whether a garnet binary was found on the standard lookup paths at launch.")
            Spacer()
            Text("evidence root: ~/Desktop/dogfood")
                .help("Studio evidence readers are constrained to the Desktop dogfood root; nothing outside it is readable from the app.")
        }
        .font(.caption)
        .foregroundStyle(.secondary)
        .padding(.horizontal, 14)
        .padding(.vertical, 6)
        .background(.black.opacity(0.25))
    }
}
