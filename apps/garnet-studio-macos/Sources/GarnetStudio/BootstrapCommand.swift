// M6 — Bootstrap script writer (nonisolated; writes files only, NEVER executes).
//
// The only side effect is writing the generated scripts to a folder the operator
// picks. There is no Process spawn, no chmod +x, no execution — the operator runs
// the scripts manually after inspecting them. A script that trips the allowlist is
// refused rather than written.

import Foundation

public enum BootstrapCommand {
    public struct WriteResult: Equatable, Sendable {
        public let directory: String
        public let written: [String]
        public let error: String?
    }

    /// Write the plan's scripts into `directory` (created if needed). Refuses to
    /// write any script that violates the allowlist; never executes anything.
    public static func writeScripts(plan: BootstrapPlan, to directory: URL) -> WriteResult {
        let scripts = BootstrapGenerator.scripts(for: plan)
        for script in scripts {
            let violations = BootstrapGenerator.violations(in: script.contents)
            if !violations.isEmpty {
                return WriteResult(
                    directory: directory.path, written: [],
                    error:
                        "refusing to write \(script.name): forbidden token(s) "
                        + violations.joined(separator: ", "))
            }
        }
        let fm = FileManager.default
        do {
            try fm.createDirectory(at: directory, withIntermediateDirectories: true)
            var written: [String] = []
            for script in scripts {
                let url = directory.appendingPathComponent(script.name)
                try script.contents.write(to: url, atomically: true, encoding: .utf8)
                written.append(url.path)
            }
            return WriteResult(directory: directory.path, written: written, error: nil)
        } catch {
            return WriteResult(
                directory: directory.path, written: [], error: error.localizedDescription)
        }
    }
}
