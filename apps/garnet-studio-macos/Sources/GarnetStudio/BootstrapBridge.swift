// M6 — Bootstrap (generate-scripts-only; pure plan + script generation).
//
// DESCOPED by design: the macOS Studio GENERATES allowlisted bash/zsh setup
// scripts to a folder for the OPERATOR to inspect and run manually. It never
// spawns an installer, never uses sudo, and never edits your shell profile —
// doing any of that from the app would need a Jon-approved AGENTS.md amendment.
// The generated scripts are read-only checks + a `cargo build` + a printed PATH
// line; an allowlist guard (no sudo / no curl|sh / no rm -rf) is enforced here
// and unit-tested, so a future edit cannot smuggle a privileged command in.

import Foundation

/// The three safe bootstrap steps. (The Windows shell also has an
/// install-python step via winget; on macOS dependency install is left to the
/// operator's package manager, so it is intentionally NOT generated.)
public enum BootstrapStep: String, CaseIterable, Sendable {
    case preflight
    case buildCli
    case configureEnv

    public var scriptName: String {
        switch self {
        case .preflight: return "00-preflight.sh"
        case .buildCli: return "10-build-garnet-cli.sh"
        case .configureEnv: return "20-configure-path.sh"
        }
    }

    public var summary: String {
        switch self {
        case .preflight: return "Read-only check for git / python3 / cargo / rustc."
        case .buildCli: return "Build the Garnet CLI from a checkout (cargo build --release)."
        case .configureEnv: return "Print the PATH line to add (does not edit your profile)."
        }
    }

    public func scriptContents() -> String {
        switch self {
        case .preflight:
            return """
                #!/usr/bin/env bash
                # Garnet Studio bootstrap — preflight (read-only; safe to run).
                set -euo pipefail
                echo "Garnet bootstrap preflight"
                for tool in git python3 cargo rustc; do
                  if command -v "$tool" >/dev/null 2>&1; then
                    echo "  ok   $tool -> $(command -v "$tool")"
                  else
                    echo "  MISS $tool (install it with your package manager before building)"
                  fi
                done

                """
        case .buildCli:
            return """
                #!/usr/bin/env bash
                # Garnet Studio bootstrap — build the CLI from a checkout (operator-run only).
                # Usage: 10-build-garnet-cli.sh /path/to/garnet/checkout
                set -euo pipefail
                REPO="${1:-REPLACE_WITH_GARNET_CHECKOUT}"
                if [ ! -d "$REPO" ]; then
                  echo "error: pass your Garnet checkout path as the first argument" >&2
                  exit 2
                fi
                cd "$REPO"
                cargo build --release -p garnet-cli
                echo "Built: $REPO/target/release/garnet"

                """
        case .configureEnv:
            return """
                #!/usr/bin/env bash
                # Garnet Studio bootstrap — print the PATH line to add. Does NOT edit any
                # profile; copy the line below into ~/.zshrc or ~/.bashrc yourself.
                set -euo pipefail
                BIN_DIR="${1:-REPLACE_WITH_GARNET_CHECKOUT/target/release}"
                echo "Add this to your shell profile, then restart your shell:"
                echo "  export PATH=\\"$BIN_DIR:\\$PATH\\""

                """
        }
    }
}

/// The plan: which steps are needed, derived from whether a CLI was already found.
public struct BootstrapPlan: Equatable, Sendable {
    public let cliFound: Bool
    public let cliPath: String?
    public let steps: [BootstrapStep]
    public let summary: String

    /// With a CLI already on PATH only the preflight check is offered; without one,
    /// the full build + PATH-configure sequence is generated (still operator-run).
    public static func from(cliPath: String?) -> BootstrapPlan {
        if let cliPath {
            return BootstrapPlan(
                cliFound: true, cliPath: cliPath, steps: [.preflight],
                summary:
                    "A Garnet CLI is already available — only the read-only preflight check is offered."
            )
        }
        return BootstrapPlan(
            cliFound: false, cliPath: nil,
            steps: [.preflight, .buildCli, .configureEnv],
            summary:
                "No Garnet CLI found — generates preflight, build, and PATH-configure scripts for you to run.")
    }
}

/// One generated, named script.
public struct BootstrapScript: Equatable, Sendable {
    public let name: String
    public let contents: String
}

public enum BootstrapGenerator {
    /// Tokens a generated bootstrap script may NEVER contain — the allowlist guard.
    /// These cover privilege escalation, remote-pipe execution, and destructive
    /// removals. Enforced here and unit-tested so the generator cannot drift into
    /// emitting a privileged command.
    public static let forbiddenTokens = [
        "sudo", "rm -rf", "curl", "wget", "| sh", "|sh", "| bash", "|bash",
        "chmod 777", "> /etc", "osascript", "launchctl",
    ]

    public static func scripts(for plan: BootstrapPlan) -> [BootstrapScript] {
        plan.steps.map { BootstrapScript(name: $0.scriptName, contents: $0.scriptContents()) }
    }

    /// The forbidden tokens present in a script (empty == clean).
    public static func violations(in contents: String) -> [String] {
        forbiddenTokens.filter { contents.contains($0) }
    }
}
