#!/usr/bin/env python3
"""Run agent-facing Garnet dogfood probes and emit evidence artifacts.

The matrix intentionally exercises both canonical workflows and advertised
advanced examples that should pass if Garnet's agent-native story is fully
executable. Failures are captured as findings instead of being hidden behind a
green-only smoke test.
"""
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Callable

ROOT = Path(__file__).resolve().parents[1]


@dataclass(frozen=True)
class Probe:
    id: str
    domain: str
    claim: str
    command: list[str]
    expect_success: bool
    expected_stdout: tuple[str, ...] = ()
    expected_stderr: tuple[str, ...] = ()
    security_domain: str = "not-applicable"
    notes: str = ""


@dataclass
class ProbeResult:
    probe: Probe
    status: str
    exit_code: int
    duration_ms: int
    stdout_log: str
    stderr_log: str
    stdout_excerpt: str
    stderr_excerpt: str
    missing_stdout: list[str] = field(default_factory=list)
    missing_stderr: list[str] = field(default_factory=list)

    @property
    def passed(self) -> bool:
        return self.status == "passed"


def timeout_output(value: str | bytes | None) -> str:
    if value is None:
        return ""
    if isinstance(value, bytes):
        return value.decode("utf-8", errors="replace")
    return value


def run(cmd: list[str], cwd: Path, timeout: int = 120) -> subprocess.CompletedProcess[str]:
    try:
        return subprocess.run(
            cmd,
            cwd=cwd,
            text=True,
            capture_output=True,
            timeout=timeout,
            check=False,
        )
    except subprocess.TimeoutExpired as exc:
        stdout = timeout_output(exc.stdout)
        stderr = timeout_output(exc.stderr)
        stderr = f"{stderr}\nTIMEOUT after {timeout}s: {' '.join(cmd)}\n"
        return subprocess.CompletedProcess(cmd, 124, stdout, stderr)


def ensure_garnet_bin(explicit: str | None) -> Path:
    if explicit:
        path = Path(explicit).expanduser().resolve()
        if not os.access(path, os.X_OK):
            raise SystemExit(f"garnet binary is not executable: {path}")
        return path

    candidate = ROOT / "target" / "debug" / "garnet"
    build = run(["cargo", "build", "-p", "garnet-cli"], ROOT, timeout=180)
    if build.returncode != 0:
        sys.stderr.write(build.stdout)
        sys.stderr.write(build.stderr)
        raise SystemExit("failed to build garnet-cli")
    if not os.access(candidate, os.X_OK):
        raise SystemExit(f"garnet binary was not produced: {candidate}")
    return candidate


def write(path: Path, text: str) -> Path:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")
    return path


def prepare_fixtures(work: Path) -> dict[str, Path]:
    fixtures = work / "fixtures"
    paths: dict[str, Path] = {}

    paths["triage"] = write(
        fixtures / "triage_router.garnet",
        """def severity_score(sev) {
  match sev {
    "fatal" => 50,
    "error" => 30,
    "warn" => 10,
    _ => 1,
  }
}

@caps()
def main() {
  let events = ["info", "warn", "error", "fatal"]
  let mut score = 0
  for event in events {
    score += severity_score(event)
  }
  println("agent triage score:", score)
  score
}
""",
    )

    paths["policy"] = write(
        fixtures / "capability_policy.garnet",
        """def allowed(tool, risk) {
  match tool {
    "read_file" if risk < 3 => true,
    "write_file" if risk < 2 => true,
    "network" => false,
    _ => false,
  }
}

@caps()
def main() {
  let checks = [
    allowed("read_file", 1),
    allowed("write_file", 3),
    allowed("network", 1),
    allowed("unknown", 0),
  ]
  let mut score = 0
  for item in checks {
    if item { score += 1 }
  }
  println("policy allowed count:", score)
  score
}
""",
    )

    paths["safe_pure"] = write(
        fixtures / "safe_pure.garnet",
        """@safe
def inc(value) {
  value + 1
}

@caps()
def main() {
  inc(40)
}
""",
    )

    paths["safe_violation"] = write(
        fixtures / "safe_violation.garnet",
        """@safe
def bad() {
  var x = 1
  raise "oops"
  x
}
""",
    )

    paths["agent_depth_bomb"] = write(
        fixtures / "agent_depth_bomb.garnet",
        "@caps()\ndef main() {\n  " + ("(" * 300) + "1" + (")" * 300) + "\n}\n",
    )

    paths["agent_no_caps"] = write(
        fixtures / "agent_no_caps.garnet",
        """def main() {
  1
}
""",
    )

    paths["agent_safe_var_mutation"] = write(
        fixtures / "agent_safe_var_mutation.garnet",
        """@safe
def bad() {
  var x = 1
  x
}
""",
    )

    paths["malformed"] = write(fixtures / "malformed_agent.garnet", "def main( { 1 }\n")

    paths["doc_source"] = write(
        fixtures / "documented_agent.garnet",
        """/// Score an agent handoff for review priority.
def priority(risk, age) {
  risk * 10 + age
}

/// Main smoke for documentation extraction.
@caps()
def main() {
  priority(3, 7)
}
""",
    )

    paths["build_source"] = write(
        fixtures / "release_agent.garnet",
        """def priority(risk, age) {
  risk * 10 + age
}

@caps()
def main() {
  priority(3, 7)
}
""",
    )

    paths["fmt_source"] = write(
        fixtures / "formatted_agent.garnet",
        """def priority(risk, age) {
  risk * 10 + age
}

@caps()
def main() {
  priority(3, 7)
}
""",
    )
    paths["dirty_fmt_source"] = write(
        fixtures / "dirty_formatted_agent.garnet",
        "def main() {   \n  1   \n}",
    )

    paths["tamper"] = write(fixtures / "tamper.garnet", "def main() { 100 }\n")

    paths["python"] = write(
        fixtures / "route_weight.py",
        """def route_weight(path):
    if path == "/":
        return 10
    if path == "/health":
        return 20
    return 1
""",
    )
    paths["ruby"] = write(
        fixtures / "score.rb",
        """def score(x)
  if x > 3
    x * 2
  else
    x
  end
end
""",
    )
    paths["rust"] = write(
        fixtures / "score.rs",
        "fn score(x: i32) -> i32 { if x > 3 { x * 2 } else { x } }\n",
    )
    paths["go"] = write(
        fixtures / "score.go",
        "func score(x int) int { if x > 3 { return x * 2 }; return x }\n",
    )
    notarization = fixtures / "notarization-preflight"
    notarization.mkdir(parents=True, exist_ok=True)
    write(
        notarization / "checks.tsv",
        "\n".join(
            [
                "pass\tApp bundle exists\t/tmp/Garnet Studio.app\tNone.",
                "blocker\tDeveloper ID Application signature missing\tSignature=adhoc\tSign with APPLE_DEV_ID_APP and hardened runtime before notarization.",
                "blocker\tAPPLE_NOTARY_PROFILE not configured\tenvironment variable is empty\tCreate a notarytool keychain profile and export its name.",
                "warning\tDMG has no stapled notarization ticket\txcrun stapler validate failed\tExpected before notarization; must pass after notarytool submit and stapler staple.",
            ]
        )
        + "\n",
    )
    write(
        notarization / "notarization-preflight-data.env",
        "\n".join(
            [
                "app_path=/tmp/Garnet Studio.app",
                "dmg_path=/tmp/GarnetStudio.dmg",
                f"output_dir={notarization}",
                "blockers=2",
                "warnings=1",
                "strict=0",
                "copy_to_desktop=0",
            ]
        )
        + "\n",
    )
    write(notarization / "MANIFEST.sha256", "fixture  ./checks.tsv\n")
    write(notarization / "MANIFEST.verify.log", "./checks.tsv: OK\n")
    paths["notarization_bundle"] = notarization
    return paths


def build_project_template_probe(
    garnet: Path,
    work: Path,
    template: str,
    target_name: str,
    expected_run: tuple[str, ...],
    expected_test: str,
) -> ProbeResult:
    target = work / target_name
    probe = Probe(
        id=f"template-{template}-run-and-test",
        domain="project scaffolding",
        claim=f"{template} template should scaffold, run, and test without manual repair",
        command=[str(garnet), "new", "--template", template, str(target)],
        expect_success=True,
        expected_stdout=(*expected_run, expected_test),
        security_domain="filesystem",
    )
    start = time.monotonic()
    stdout_parts: list[str] = []
    stderr_parts: list[str] = []
    if target.exists():
        shutil.rmtree(target)
    first = run(probe.command, work)
    stdout_parts.append("$ " + " ".join(probe.command))
    stdout_parts.append(first.stdout)
    stderr_parts.append(first.stderr)
    exit_code = first.returncode
    if first.returncode == 0:
        main = target / "src" / "main.garnet"
        second = run([str(garnet), "run", str(main)], work)
        stdout_parts.append(f"$ garnet run {target_name}/src/main.garnet")
        stdout_parts.append(second.stdout)
        stderr_parts.append(second.stderr)
        exit_code = second.returncode
        if second.returncode == 0:
            third = run([str(garnet), "test", str(target)], work)
            stdout_parts.append(f"$ garnet test {target_name}")
            stdout_parts.append(third.stdout)
            stderr_parts.append(third.stderr)
            exit_code = third.returncode
    return classify_result(
        probe,
        exit_code,
        "\n".join(stdout_parts),
        "\n".join(stderr_parts),
        int((time.monotonic() - start) * 1000),
        work,
    )


def build_tamper_probe(garnet: Path, work: Path, source: Path) -> ProbeResult:
    probe = Probe(
        id="release-manifest-tamper-detection",
        domain="release integrity",
        claim="deterministic build manifests should reject modified source",
        command=[str(garnet), "build", "--deterministic", str(source)],
        expect_success=True,
        expected_stderr=("source_hash mismatch",),
        security_domain="release-integrity",
    )
    start = time.monotonic()
    build = run(probe.command, work)
    manifest = source.with_name(source.name + ".manifest.json")
    stdout = ["$ " + " ".join(probe.command), build.stdout]
    stderr = [build.stderr]
    exit_code = build.returncode
    if build.returncode == 0:
        source.write_text("def main() { 999 }\n", encoding="utf-8")
        verify_cmd = [str(garnet), "verify", str(source), str(manifest)]
        verify = run(verify_cmd, work)
        stdout.extend(["$ " + " ".join(verify_cmd), verify.stdout])
        stderr.append(verify.stderr)
        exit_code = 0 if verify.returncode != 0 else 1
    return classify_result(
        probe,
        exit_code,
        "\n".join(stdout),
        "\n".join(stderr),
        int((time.monotonic() - start) * 1000),
        work,
    )


def copy_release_source(work: Path, source: Path, stem: str) -> Path:
    target = work / "signed-release" / f"{stem}.garnet"
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(source.read_text(encoding="utf-8"), encoding="utf-8")
    return target


def scrub_transient_key(keyfile: Path) -> None:
    keyfile.unlink(missing_ok=True)


def build_signed_release_probe(garnet: Path, work: Path, source: Path) -> ProbeResult:
    release_source = copy_release_source(work, source, "signed-release")
    keyfile = release_source.with_name("signing.key")
    probe = Probe(
        id="release-keygen-build-verify-signature",
        domain="signed release provenance",
        claim="an agent should be able to generate a signing key, sign a deterministic manifest, and verify the signature",
        command=[str(garnet), "keygen", str(keyfile)],
        expect_success=True,
        expected_stdout=("generated Ed25519 signing keypair", "signed_by", "signature valid"),
        security_domain="release-integrity",
    )
    start = time.monotonic()
    stdout = ["$ " + " ".join(probe.command)]
    stderr: list[str] = []
    keygen = run(probe.command, work)
    stdout.append(keygen.stdout)
    stderr.append(keygen.stderr)
    exit_code = keygen.returncode
    if keygen.returncode == 0:
        build_cmd = [str(garnet), "build", "--deterministic", "--sign", str(keyfile), str(release_source)]
        build = run(build_cmd, work)
        stdout.extend(["$ " + " ".join(build_cmd), build.stdout])
        stderr.append(build.stderr)
        exit_code = build.returncode
        if build.returncode == 0:
            verify_cmd = [
                str(garnet),
                "verify",
                str(release_source),
                str(release_source.with_name(release_source.name + ".manifest.json")),
                "--signature",
            ]
            verify = run(verify_cmd, work)
            stdout.extend(["$ " + " ".join(verify_cmd), verify.stdout])
            stderr.append(verify.stderr)
            exit_code = verify.returncode
    scrub_transient_key(keyfile)
    return classify_result(
        probe,
        exit_code,
        "\n".join(stdout),
        "\n".join(stderr),
        int((time.monotonic() - start) * 1000),
        work,
    )


def build_unsigned_signature_required_probe(garnet: Path, work: Path, source: Path) -> ProbeResult:
    release_source = copy_release_source(work, source, "unsigned-release")
    probe = Probe(
        id="release-unsigned-manifest-requires-signature",
        domain="signed release provenance",
        claim="signature-required verification should reject an unsigned deterministic manifest",
        command=[str(garnet), "build", "--deterministic", str(release_source)],
        expect_success=True,
        expected_stderr=("manifest is unsigned", "--signature was required"),
        security_domain="release-integrity",
    )
    start = time.monotonic()
    build = run(probe.command, work)
    stdout = ["$ " + " ".join(probe.command), build.stdout]
    stderr = [build.stderr]
    exit_code = build.returncode
    if build.returncode == 0:
        verify_cmd = [
            str(garnet),
            "verify",
            str(release_source),
            str(release_source.with_name(release_source.name + ".manifest.json")),
            "--signature",
        ]
        verify = run(verify_cmd, work)
        stdout.extend(["$ " + " ".join(verify_cmd), verify.stdout])
        stderr.append(verify.stderr)
        exit_code = 0 if verify.returncode != 0 else 1
    return classify_result(
        probe,
        exit_code,
        "\n".join(stdout),
        "\n".join(stderr),
        int((time.monotonic() - start) * 1000),
        work,
    )


def build_signed_tamper_probe(garnet: Path, work: Path, source: Path) -> ProbeResult:
    release_source = copy_release_source(work, source, "tampered-signed-release")
    keyfile = release_source.with_name("tamper-signing.key")
    probe = Probe(
        id="release-signed-manifest-tamper-detection",
        domain="signed release provenance",
        claim="signed manifest verification should still reject source tampering before trusting signature provenance",
        command=[str(garnet), "keygen", str(keyfile)],
        expect_success=True,
        expected_stderr=("source_hash mismatch",),
        security_domain="release-integrity",
    )
    start = time.monotonic()
    stdout = ["$ " + " ".join(probe.command)]
    stderr: list[str] = []
    keygen = run(probe.command, work)
    stdout.append(keygen.stdout)
    stderr.append(keygen.stderr)
    exit_code = keygen.returncode
    if keygen.returncode == 0:
        build_cmd = [str(garnet), "build", "--deterministic", "--sign", str(keyfile), str(release_source)]
        build = run(build_cmd, work)
        stdout.extend(["$ " + " ".join(build_cmd), build.stdout])
        stderr.append(build.stderr)
        exit_code = build.returncode
        if build.returncode == 0:
            release_source.write_text("def main() { 999 }\n", encoding="utf-8")
            verify_cmd = [
                str(garnet),
                "verify",
                str(release_source),
                str(release_source.with_name(release_source.name + ".manifest.json")),
                "--signature",
            ]
            verify = run(verify_cmd, work)
            stdout.extend(["$ " + " ".join(verify_cmd), verify.stdout])
            stderr.append(verify.stderr)
            exit_code = 0 if verify.returncode != 0 else 1
    scrub_transient_key(keyfile)
    return classify_result(
        probe,
        exit_code,
        "\n".join(stdout),
        "\n".join(stderr),
        int((time.monotonic() - start) * 1000),
        work,
    )


def classify_result(
    probe: Probe,
    exit_code: int,
    stdout: str,
    stderr: str,
    duration_ms: int,
    work: Path,
) -> ProbeResult:
    exit_matches = (exit_code == 0) == probe.expect_success
    missing_stdout = [needle for needle in probe.expected_stdout if needle not in stdout]
    missing_stderr = [needle for needle in probe.expected_stderr if needle not in stderr]
    status = "passed" if exit_matches and not missing_stdout and not missing_stderr else "failed"

    logs = work / "logs"
    logs.mkdir(exist_ok=True)
    stdout_log = logs / f"{probe.id}.stdout.log"
    stderr_log = logs / f"{probe.id}.stderr.log"
    stdout_log.write_text(stdout, encoding="utf-8")
    stderr_log.write_text(stderr, encoding="utf-8")

    return ProbeResult(
        probe=probe,
        status=status,
        exit_code=exit_code,
        duration_ms=duration_ms,
        stdout_log=str(stdout_log),
        stderr_log=str(stderr_log),
        stdout_excerpt=stdout[-700:],
        stderr_excerpt=stderr[-700:],
        missing_stdout=missing_stdout,
        missing_stderr=missing_stderr,
    )


def run_probe(probe: Probe, work: Path) -> ProbeResult:
    start = time.monotonic()
    completed = run(probe.command, work)
    return classify_result(
        probe,
        completed.returncode,
        completed.stdout,
        completed.stderr,
        int((time.monotonic() - start) * 1000),
        work,
    )


def app_workbench_probes(app_executable: Path | None, garnet: Path) -> list[Probe]:
    if app_executable is not None:
        return [
            Probe(
                "app-self-test",
                "macOS app workbench",
                "packaged Garnet Studio self-test should pass without a source checkout",
                [str(app_executable), "--self-test"],
                True,
                ("GarnetStudio self-test passed",),
            ),
            Probe(
                "app-smoke-test",
                "macOS app workbench",
                "packaged Garnet Studio should run the bundled CLI across workbench samples",
                [str(app_executable), "--smoke-test"],
                True,
                ("GarnetStudio smoke passed",),
            ),
        ]
    return [
        Probe(
            "app-self-test",
            "macOS app workbench",
            "Garnet Studio self-test should pass from SwiftPM",
            ["swift", "run", "--package-path", str(ROOT / "apps" / "garnet-studio-macos"), "GarnetStudio", "--self-test"],
            True,
            ("GarnetStudio self-test passed",),
        ),
        Probe(
            "app-smoke-test",
            "macOS app workbench",
            "Garnet Studio SwiftPM smoke should run workbench samples against the matrix-built Garnet CLI",
            [
                "env",
                f"PATH={garnet.parent}:{os.environ.get('PATH', '')}",
                "swift",
                "run",
                "--package-path",
                str(ROOT / "apps" / "garnet-studio-macos"),
                "GarnetStudio",
                "--smoke-test",
            ],
            True,
            ("GarnetStudio smoke passed",),
            security_domain="filesystem",
        ),
        Probe(
            "app-xctest",
            "macOS app workbench",
            "Garnet Studio XCTest target should pass",
            ["swift", "test", "--package-path", str(ROOT / "apps" / "garnet-studio-macos")],
            True,
            ("GarnetStudioTests", "0 failures"),
        ),
    ]


def web_pwa_probes(work: Path) -> list[Probe]:
    docs_dir = ROOT / "docs"
    offline_smoke = ROOT / "scripts" / "smoke_garnet_web_pwa_offline.mjs"
    local_smoke = ROOT / "scripts" / "smoke_garnet_web_pwa.sh"
    browser_smoke = ROOT / "scripts" / "smoke_garnet_web_pwa_browser.mjs"
    if not (docs_dir / "service-worker.js").is_file() or not offline_smoke.is_file():
        return []
    probes = [
        Probe(
            "smoke-web-pwa-offline-handler",
            "web/PWA productization",
            "docs PWA service worker should serve the offline shell through executable handler evidence",
            [
                "node",
                str(offline_smoke),
                "--docs-dir",
                str(docs_dir),
                "--output",
                str(work / "web-pwa-offline-handler.json"),
            ],
            True,
            ("Garnet service worker offline behavior: passed",),
            security_domain="filesystem",
            notes="Runs when the current root carries docs/PWA assets; source checkouts and packaged apps both stage them.",
        )
    ]
    if local_smoke.is_file():
        probes.append(
            Probe(
                "smoke-web-pwa-local-readiness",
                "web/PWA productization",
                "docs PWA local smoke should validate manifest, cache inventory, offline behavior, and local HTTP fetches",
                [
                    str(local_smoke),
                    "--strict",
                    "--output-dir",
                    str(work / "web-pwa-local-readiness"),
                ],
                True,
                ("Garnet Web/PWA readiness smoke: blockers=0 warnings=0",),
                security_domain="filesystem-localhost",
                notes="Uses only the checkout docs directory, Node offline handler, and a localhost static server.",
            )
        )
    if browser_smoke.is_file():
        probes.append(
            Probe(
                "smoke-web-pwa-browser-offline",
                "web/PWA productization",
                "docs PWA browser smoke should prove service-worker control and offline navigation through Chrome DevTools",
                [
                    "node",
                    str(browser_smoke),
                    "--docs-dir",
                    str(docs_dir),
                    "--output",
                    str(work / "web-pwa-browser-offline.json"),
                ],
                True,
                ("Garnet browser PWA offline smoke: passed",),
                security_domain="filesystem-localhost-browser",
                notes="Uses local headless Chrome through the dependency-free Chrome DevTools Protocol harness.",
            )
        )
    return probes


def probe_set(
    garnet: Path,
    work: Path,
    fixtures: dict[str, Path],
    app_executable: Path | None = None,
    include_app_workbench: bool = True,
) -> list[Probe | Callable[[], ProbeResult]]:
    examples = ROOT / "examples"
    return [
        Probe(
            "run-agent-triage-router",
            "agent orchestration",
            "agent triage program should parse, execute control flow, and return a stable priority score",
            [str(garnet), "run", str(fixtures["triage"])],
            True,
            ("agent triage score:", "=> 91"),
        ),
        Probe(
            "run-capability-policy",
            "agent orchestration",
            "capability policy program should handle guarded match branches and booleans",
            [str(garnet), "run", str(fixtures["policy"])],
            True,
            ("policy allowed count:", "=> 1"),
            security_domain="authn-authz",
        ),
        Probe(
            "eval-agent-reducer",
            "agent orchestration",
            "agent can use CLI eval for quick collection reduction",
            [str(garnet), "eval", "[1, 2, 3].reduce(0, |a, b| a + b)"],
            True,
            ("6",),
        ),
        lambda: build_project_template_probe(
            garnet,
            work,
            "cli",
            "generated_cli",
            ("Hello from generated_cli!", "=> 0"),
            "2 passed; 0 failed",
        ),
        lambda: build_project_template_probe(
            garnet,
            work,
            "web-api",
            "generated_web_api",
            ("starting generated_web_api", "=> 0"),
            "1 passed; 0 failed",
        ),
        lambda: build_project_template_probe(
            garnet,
            work,
            "agent-orchestrator",
            "generated_agents",
            ("=> 25",),
            "3 passed; 0 failed",
        ),
        Probe(
            "run-canonical-multi-agent-example",
            "agent orchestration",
            "canonical multi-agent builder example should run with a stable result",
            [str(garnet), "run", str(examples / "multi_agent_builder.garnet")],
            True,
            ("clean build:", "red build:", "=> 46"),
        ),
        Probe(
            "run-agent-toolbelt-triage-router",
            "agent toolbelt examples",
            "agent-facing triage router example should run with stable prioritization evidence",
            [str(garnet), "run", str(examples / "agent_toolbelt_01_triage_router.garnet")],
            True,
            ("agent toolbelt triage score:", "=> 91"),
        ),
        Probe(
            "run-agent-toolbelt-capability-budget",
            "agent toolbelt examples",
            "agent-facing capability budget example should run with stable tool-authority evidence",
            [str(garnet), "run", str(examples / "agent_toolbelt_02_capability_budget.garnet")],
            True,
            ("agent toolbelt capability score:", "=> 61"),
            security_domain="sandbox",
        ),
        Probe(
            "run-agent-toolbelt-memory-recall",
            "agent toolbelt examples",
            "agent-facing memory recall example should run with stable retrieval-ranking evidence",
            [str(garnet), "run", str(examples / "agent_toolbelt_03_memory_recall.garnet")],
            True,
            ("agent toolbelt memory recall score:", "=> 81"),
            security_domain="privacy",
        ),
        Probe(
            "run-agent-toolbelt-release-gate",
            "agent toolbelt examples",
            "agent-facing release gate example should run with stable evidence-gate scoring",
            [str(garnet), "run", str(examples / "agent_toolbelt_04_release_gate.garnet")],
            True,
            ("agent toolbelt release gate score:", "=> 80"),
            security_domain="release-integrity",
        ),
        Probe(
            "run-agent-toolbelt-repair-planner",
            "agent toolbelt examples",
            "agent-facing repair planner example should run with stable remediation-priority evidence",
            [str(garnet), "run", str(examples / "agent_toolbelt_05_repair_planner.garnet")],
            True,
            ("agent toolbelt repair plan score:", "=> 116"),
        ),
        Probe(
            "check-malformed-agent-source",
            "agent recovery and diagnostics",
            "malformed agent source should fail with a parser diagnostic an agent can act on",
            [str(garnet), "check", str(fixtures["malformed"])],
            False,
            expected_stderr=("expected identifier in parameter name",),
        ),
        Probe(
            "check-missing-agent-source",
            "agent recovery and diagnostics",
            "missing agent source should fail loudly instead of looking like an empty successful check",
            [str(garnet), "check", str(work / "fixtures" / "missing_agent_source.garnet")],
            False,
            expected_stderr=("failed to read", "No such file or directory"),
            security_domain="filesystem",
        ),
        Probe(
            "eval-unknown-agent-symbol",
            "agent recovery and diagnostics",
            "unknown symbols in quick agent eval should produce a concrete undefined-variable diagnostic",
            [str(garnet), "eval", "unknown_symbol + 1"],
            False,
            expected_stderr=("undefined variable: unknown_symbol",),
        ),
        Probe(
            "verify-missing-release-manifest",
            "agent recovery and diagnostics",
            "manifest verification should fail loudly when the referenced manifest is absent",
            [
                str(garnet),
                "verify",
                str(fixtures["build_source"]),
                str(work / "fixtures" / "missing.manifest.json"),
            ],
            False,
            expected_stderr=("failed to read", "missing.manifest.json"),
            security_domain="release-integrity",
        ),
        Probe(
            "reject-agent-depth-budget-bomb",
            "agent adversarial boundaries",
            "agent-supplied deeply nested source should trip the parser budget instead of consuming unbounded resources",
            [str(garnet), "parse", str(fixtures["agent_depth_bomb"])],
            False,
            expected_stderr=("parse budget exceeded (depth", "budget exceeded here"),
            security_domain="sandbox",
        ),
        Probe(
            "reject-agent-main-without-caps",
            "agent adversarial boundaries",
            "agent-supplied entrypoints should declare capability boundaries instead of relying on ambient authority",
            [str(garnet), "check", str(fixtures["agent_no_caps"])],
            False,
            expected_stdout=("main` function must declare its required capabilities",),
            security_domain="authn-authz",
        ),
        Probe(
            "reject-agent-safe-var-mutation",
            "agent adversarial boundaries",
            "agent-supplied @safe code should reject legacy mutable declarations that bypass the preferred safe-mode surface",
            [str(garnet), "check", str(fixtures["agent_safe_var_mutation"])],
            False,
            expected_stdout=("safe-mode violation", "uses `var`"),
            security_domain="sandbox",
        ),
        Probe(
            "check-safe-pure",
            "safe mode and capabilities",
            "safe pure function with @caps main should pass checker",
            [str(garnet), "check", str(fixtures["safe_pure"])],
            True,
            ("0 diagnostics",),
        ),
        Probe(
            "run-safe-pure",
            "safe mode and capabilities",
            "safe pure function should run through managed main",
            [str(garnet), "run", str(fixtures["safe_pure"])],
            True,
            ("=> 41",),
        ),
        Probe(
            "check-safe-violation",
            "safe mode and capabilities",
            "safe-mode checker should reject raise/var violations in @safe code",
            [str(garnet), "check", str(fixtures["safe_violation"])],
            False,
            ("safe",),
        ),
        Probe(
            "check-advertised-safe-io-example",
            "safe mode and capabilities",
            "advertised safe_io_layer example should satisfy checker without manual repair",
            [str(garnet), "check", str(examples / "safe_io_layer.garnet")],
            True,
            ("0 diagnostics",),
            security_domain="filesystem",
        ),
        Probe(
            "run-advertised-safe-io-example",
            "safe mode and capabilities",
            "advertised safe_io_layer example should run as documented",
            [str(garnet), "run", str(examples / "safe_io_layer.garnet")],
            True,
            ("golden ok / total:", "=> 402"),
            security_domain="filesystem",
        ),
        Probe(
            "convert-python-route",
            "migration assistant",
            "Python source should convert with lineage, metrics, and migrate checklist",
            [str(garnet), "convert", "python", str(fixtures["python"])],
            True,
            ("converted", "lineage", "metrics", "migrate_todo"),
            security_domain="sandbox",
        ),
        Probe(
            "convert-ruby-score",
            "migration assistant",
            "Ruby source should convert with migration evidence",
            [str(garnet), "convert", "ruby", str(fixtures["ruby"])],
            True,
            ("converted", "migrate_todo"),
            security_domain="sandbox",
        ),
        Probe(
            "convert-rust-score",
            "migration assistant",
            "Rust source should convert cleanly for simple function shapes",
            [str(garnet), "convert", "rust", str(fixtures["rust"])],
            True,
            ("100.0% clean translation",),
            security_domain="sandbox",
        ),
        Probe(
            "convert-go-score",
            "migration assistant",
            "Go source should convert cleanly for simple function shapes",
            [str(garnet), "convert", "go", str(fixtures["go"])],
            True,
            ("100.0% clean translation",),
            security_domain="sandbox",
        ),
        Probe(
            "convert-unsupported-language",
            "migration assistant",
            "unsupported source languages should fail loudly",
            [str(garnet), "convert", "javascript", str(fixtures["python"])],
            False,
            security_domain="sandbox",
        ),
        Probe(
            "report-converter-adoption-status",
            "migration assistant",
            "converter adoption status should separate active deterministic languages from gated future lanes",
            [sys.executable, str(ROOT / "scripts" / "garnet_converter_status.py"), "--format", "json"],
            True,
            (
                "stylized-migration-assistant",
                "proposed-gated",
                "javascript",
                "requires_garnet_check",
                "planned-contract",
                "provider_required",
                "CapCaps/capability boundaries",
            ),
            security_domain="sandbox",
        ),
        Probe(
            "report-assist-context-current-truth",
            "converter intelligent assist",
            "assist context pack should exist without enabling LLM conversion",
            [sys.executable, str(ROOT / "scripts" / "garnet_assist_context_pack.py"), "--format", "json"],
            True,
            (
                "\"status\": \"active-context-pack\"",
                "\"llm_conversion_active\": false",
                "not active conversion today",
                "Rust",
                "Go",
                "JavaScript",
            ),
            security_domain="sandbox",
        ),
        Probe(
            "report-assist-context-required-gates",
            "converter intelligent assist",
            "assist context pack should preserve converter promotion gates",
            [sys.executable, str(ROOT / "scripts" / "garnet_assist_context_pack.py"), "--format", "json"],
            True,
            (
                "\"provider_required\": false",
                "lineage per emitted node",
                "@sandbox default",
                "garnet check",
                "dogfood readiness bundle",
                "human audit before unquarantine",
            ),
            security_domain="sandbox",
        ),
        Probe(
            "report-assist-context-documents",
            "converter intelligent assist",
            "assist context pack should hash the real current truth and spec corpus",
            [sys.executable, str(ROOT / "scripts" / "garnet_assist_context_pack.py"), "--format", "json"],
            True,
            (
                "CURRENT_STATE.md",
                "GARNET_v1_0_Mini_Spec.md",
                "GARNET_v0_4_2_Conformance_Matrix.md",
                "\"sha256\"",
                "\"exists\": true",
            ),
            security_domain="not-applicable",
        ),
        Probe(
            "report-mit-readiness-plan-complete",
            "MIT readiness accounting",
            "objective status should separate completed tracked slices from broader productization completion",
            [sys.executable, str(ROOT / "scripts" / "garnet_mit_readiness_status.py"), "--format", "json"],
            True,
            (
                "tracked_implementation_plan",
                "\"status\": \"verified\"",
                "not full MIT/productization completion",
            ),
            security_domain="not-applicable",
        ),
        Probe(
            "report-mit-readiness-open-productization",
            "MIT readiness accounting",
            "objective status should keep notarization, mobile, and promo-video lanes open until proven",
            [sys.executable, str(ROOT / "scripts" / "garnet_mit_readiness_status.py"), "--format", "json"],
            True,
            (
                "developer_id_notarization",
                "APPLE_DEV_ID_APP",
                "mobile_distribution",
                "promo_video",
                "\"status\": \"blocked\"",
            ),
            security_domain="release-integrity",
        ),
        Probe(
            "report-mit-readiness-assist-and-frontends",
            "MIT readiness accounting",
            "objective status should keep LLM assist and broad converter frontends planned until implemented",
            [sys.executable, str(ROOT / "scripts" / "garnet_mit_readiness_status.py"), "--format", "json"],
            True,
            (
                "llm_assist",
                "active-partial",
                "deterministic local context pack",
                "broad_converter_frontends",
                "JavaScript",
            ),
            security_domain="sandbox",
        ),
        Probe(
            "report-adoption-surface-active-truth",
            "repo/site adoption surface",
            "adoption surface should keep the public hook compelling without promoting future converter or LLM lanes",
            [sys.executable, str(ROOT / "scripts" / "garnet_adoption_surface_status.py"), "--format", "json"],
            True,
            (
                "Rust rigor, Ruby velocity, agent-native dogfood evidence",
                "\"active_converter_languages\"",
                "Rust",
                "Ruby",
                "Python",
                "Go",
                "provider-backed conversion is not active",
            ),
            security_domain="not-applicable",
        ),
        Probe(
            "report-adoption-surface-planned-frontends",
            "repo/site adoption surface",
            "adoption surface should classify broader import languages as planned gates only",
            [sys.executable, str(ROOT / "scripts" / "garnet_adoption_surface_status.py"), "--format", "json"],
            True,
            (
                "JavaScript",
                "TypeScript",
                "Swift",
                "C++",
                "Perl",
                "broad deterministic converter frontends",
            ),
            security_domain="sandbox",
        ),
        Probe(
            "report-adoption-surface-use-cases",
            "repo/site adoption surface",
            "adoption surface Markdown should expose evidence-backed use cases and repo/site copy rules",
            [sys.executable, str(ROOT / "scripts" / "garnet_adoption_surface_status.py")],
            True,
            (
                "Dual-mode programming",
                "Agent toolbelt examples",
                "Migration assistant",
                "macOS workbench",
                "Repo/Site Contract",
            ),
            security_domain="not-applicable",
        ),
        Probe(
            "report-notarization-status-blockers",
            "macOS notarization readiness",
            "notarization status reporter should summarize blockers without claiming Apple notarization",
            [
                sys.executable,
                str(ROOT / "scripts" / "garnet_studio_notarization_status.py"),
                "--bundle",
                str(fixtures["notarization_bundle"]),
            ],
            True,
            ("Overall status: **blocked**", "This is not a notarization claim.", "Developer ID Application signature missing"),
            security_domain="release-integrity",
        ),
        Probe(
            "report-notarization-status-redaction",
            "macOS notarization readiness",
            "notarization status JSON should preserve credential boundary redaction and next actions",
            [
                sys.executable,
                str(ROOT / "scripts" / "garnet_studio_notarization_status.py"),
                "--bundle",
                str(fixtures["notarization_bundle"]),
                "--format",
                "json",
            ],
            True,
            (
                '"credential_values_redacted": true',
                '"overall_status": "blocked"',
                "Create a notarytool keychain profile",
            ),
            security_domain="secrets",
        ),
        Probe(
            "report-notarization-status-missing-bundle",
            "macOS notarization readiness",
            "notarization status reporter should fail loudly when the evidence bundle is absent",
            [
                sys.executable,
                str(ROOT / "scripts" / "garnet_studio_notarization_status.py"),
                "--bundle",
                str(work / "missing-notarization-bundle"),
                "--format",
                "json",
            ],
            False,
            expected_stderr=("preflight bundle not found",),
            security_domain="release-integrity",
        ),
        Probe(
            "build-deterministic-manifest",
            "release integrity",
            "deterministic build should emit a manifest sidecar",
            [str(garnet), "build", "--deterministic", str(fixtures["build_source"])],
            True,
            ("manifest",),
            security_domain="release-integrity",
        ),
        Probe(
            "verify-deterministic-manifest",
            "release integrity",
            "fresh deterministic manifest should verify unchanged source",
            [
                str(garnet),
                "verify",
                str(fixtures["build_source"]),
                str(fixtures["build_source"].with_name(fixtures["build_source"].name + ".manifest.json")),
            ],
            True,
            ("OK",),
            security_domain="release-integrity",
        ),
        lambda: build_tamper_probe(garnet, work, fixtures["tamper"]),
        lambda: build_signed_release_probe(garnet, work, fixtures["build_source"]),
        lambda: build_unsigned_signature_required_probe(garnet, work, fixtures["build_source"]),
        lambda: build_signed_tamper_probe(garnet, work, fixtures["build_source"]),
        Probe(
            "doc-extract-documented-agent",
            "developer experience",
            "doc extractor should surface /// comments for agent-facing source",
            [str(garnet), "doc", "--stdout", str(fixtures["doc_source"])],
            True,
            ("Score an agent handoff", "Main smoke"),
        ),
        Probe(
            "fmt-check-documented-agent",
            "developer experience",
            "formatter check should accept stable fixture style",
            [str(garnet), "fmt", "--check", str(fixtures["fmt_source"])],
            True,
        ),
        Probe(
            "fmt-repair-dirty-agent",
            "developer experience",
            "formatter should repair trailing whitespace and missing terminal newline in agent source",
            [str(garnet), "fmt", str(fixtures["dirty_fmt_source"])],
            True,
            ("formatted",),
        ),
        Probe(
            "memory-signed-cache-roundtrip",
            "memory persistence integrity",
            "signed typed-cache appends should round-trip with the same key",
            [
                "cargo",
                "test",
                "--manifest-path",
                str(ROOT / "Cargo.toml"),
                "-p",
                "garnet-memory",
                "--test",
                "persistence",
                "episodic_cache_signed_append_and_load_round_trips_with_key",
                "--",
                "--nocapture",
            ],
            True,
            ("episodic_cache_signed_append_and_load_round_trips_with_key", "test result: ok"),
            security_domain="privacy",
        ),
        Probe(
            "memory-signed-cache-tamper-rejection",
            "memory persistence integrity",
            "signed typed-cache loads should reject tampered payloads before mutating live memory",
            [
                "cargo",
                "test",
                "--manifest-path",
                str(ROOT / "Cargo.toml"),
                "-p",
                "garnet-memory",
                "--test",
                "persistence",
                "episodic_cache_signed_load_rejects_tampered_payload_without_mutating_store",
                "--",
                "--nocapture",
            ],
            True,
            (
                "episodic_cache_signed_load_rejects_tampered_payload_without_mutating_store",
                "test result: ok",
            ),
            security_domain="privacy",
        ),
        Probe(
            "memory-signed-cache-foreign-key-rejection",
            "memory persistence integrity",
            "signed typed-cache loads should reject foreign keys before mutating live memory",
            [
                "cargo",
                "test",
                "--manifest-path",
                str(ROOT / "Cargo.toml"),
                "-p",
                "garnet-memory",
                "--test",
                "persistence",
                "episodic_cache_signed_load_rejects_foreign_key_without_mutating_store",
                "--",
                "--nocapture",
            ],
            True,
            (
                "episodic_cache_signed_load_rejects_foreign_key_without_mutating_store",
                "test result: ok",
            ),
            security_domain="privacy",
        ),
        *web_pwa_probes(work),
        *(app_workbench_probes(app_executable, garnet) if include_app_workbench else []),
        Probe(
            "parse-advertised-log-analyzer-memory",
            "agent memory and analysis",
            "advertised log analyzer should expose semantic, episodic, and procedural memory declarations to CLI analysis",
            [str(garnet), "parse", str(examples / "agentic_log_analyzer.garnet")],
            True,
            (
                "memory Semantic spec_index",
                "memory Episodic incidents",
                "memory Procedural playbooks",
            ),
            security_domain="privacy",
        ),
        Probe(
            "run-advertised-log-analyzer",
            "agent memory and analysis",
            "advertised agentic log analyzer should run as documented",
            [str(garnet), "run", str(examples / "agentic_log_analyzer.garnet")],
            True,
            ("ingested incidents:", "=> 43"),
            security_domain="privacy",
        ),
        Probe(
            "check-advertised-log-analyzer",
            "agent memory and analysis",
            "advertised agentic log analyzer should satisfy capability checks",
            [str(garnet), "check", str(examples / "agentic_log_analyzer.garnet")],
            True,
            ("0 diagnostics",),
            security_domain="privacy",
        ),
    ]


def score(results: list[ProbeResult]) -> dict[str, int | float]:
    failures = [result for result in results if not result.passed]
    high = sum(1 for result in failures if "advertised" in result.probe.id)
    medium = len(failures) - high
    readiness = max(0, 100 - high * 12 - medium * 5)
    return {
        "total": len(results),
        "passed": len(results) - len(failures),
        "failed": len(failures),
        "high_findings": high,
        "medium_findings": medium,
        "readiness": readiness,
    }


def domain_coverage(results: list[ProbeResult], target_probe_count: int = 3) -> list[dict[str, int | str]]:
    by_domain: dict[str, list[ProbeResult]] = {}
    for result in results:
        by_domain.setdefault(result.probe.domain, []).append(result)
    coverage: list[dict[str, int | str]] = []
    for domain, items in sorted(by_domain.items()):
        probe_count = len(items)
        passed = sum(result.passed for result in items)
        coverage.append(
            {
                "domain": domain,
                "probe_count": probe_count,
                "passed": passed,
                "target_probe_count": target_probe_count,
                "coverage_percent": min(100, round((probe_count / target_probe_count) * 100)),
                "status": "adequate" if probe_count >= target_probe_count else "needs-expansion",
            }
        )
    return coverage


def render_matrix(results: list[ProbeResult]) -> str:
    lines = [
        "# Agentic Garnet Dogfood Matrix",
        "",
        "| Domain | Probe | Status | Exit | Claim | Evidence |",
        "| --- | --- | --- | ---: | --- | --- |",
    ]
    for result in results:
        evidence = f"`{Path(result.stdout_log).name}` / `{Path(result.stderr_log).name}`"
        lines.append(
            f"| {result.probe.domain} | `{result.probe.id}` | {result.status} | "
            f"{result.exit_code} | {result.probe.claim} | {evidence} |"
        )
    return "\n".join(lines) + "\n"


def render_findings(results: list[ProbeResult]) -> str:
    failures = [result for result in results if not result.passed]
    if not failures:
        return "No failing probes recorded.\n"
    chunks: list[str] = []
    for index, result in enumerate(failures, start=1):
        severity = "high" if "advertised" in result.probe.id else "medium"
        chunks.append(
            f"## AGENTIC-{index:03d}: {result.probe.id}\n\n"
            f"- Severity: {severity}\n"
            f"- Domain: {result.probe.domain}\n"
            f"- Security domain: {result.probe.security_domain}\n"
            f"- Claim tested: {result.probe.claim}\n"
            f"- Expected: exit {'0' if result.probe.expect_success else 'nonzero'}"
            f"{' and stdout containing ' + ', '.join(result.probe.expected_stdout) if result.probe.expected_stdout else ''}\n"
            f"- Actual: exit {result.exit_code}; status {result.status}\n"
            f"- Missing stdout evidence: {result.missing_stdout or 'none'}\n"
            f"- Missing stderr evidence: {result.missing_stderr or 'none'}\n"
            f"- Stdout log: `{result.stdout_log}`\n"
            f"- Stderr log: `{result.stderr_log}`\n"
            f"- Recommendation: promote this probe into a focused regression test or adjust the advertised example/docs to match implemented semantics.\n"
        )
    return "\n".join(chunks)


def render_report(results: list[ProbeResult], metadata: dict[str, str], score_data: dict[str, int | float]) -> str:
    failures = [result for result in results if not result.passed]
    failing_ids = "\n".join(f"- `{result.probe.id}`" for result in failures) or "- None"
    coverage_rows = "\n".join(
        "| {domain} | {passed}/{probe_count} | {probe_count}/{target_probe_count} | {coverage_percent}% | {status} |".format(
            **item
        )
        for item in domain_coverage(results)
    )
    app_workbench_note = ""
    audited_surfaces = (
        "CLI, template, converter, release-integrity, signed-release provenance, "
        "documentation, safe-mode, agent-toolbelt, agent-memory, repo/site adoption surface, "
        "macOS notarization readiness, and macOS app workbench paths"
    )
    if metadata["app_workbench"] == "skipped":
        app_workbench_note = (
            "\n\nmacOS app workbench probes were skipped for this run; they remain covered by "
            "Garnet Studio local/Desktop/package/DMG gates instead of the headless CI matrix."
        )
        audited_surfaces = (
            "CLI, template, converter, release-integrity, signed-release provenance, "
            "documentation, safe-mode, agent-toolbelt, agent-memory, repo/site adoption surface, "
            "and macOS notarization readiness paths"
        )
    if failures:
        decision = (
            f"Passed {score_data['passed']}/{score_data['total']} probes. The matrix found "
            "agent-facing gaps that should remain MIT-readiness improvement items until "
            "they are fixed or explicitly documented as deferred."
            f"{app_workbench_note}"
        )
        plan = (
            "1. Promote failing advanced probes into focused CLI regression tests.\n"
            "2. Reconcile the failing examples, checker diagnostics, and interpreter behavior.\n"
            "3. Update user-facing docs so advertised workflows match implemented semantics.\n"
            "4. Rerun this matrix with `--strict` before moving the readiness claim forward.\n"
            "5. Add a Garnet Studio UI entry for \"Agentic Stress Tests\" once the CLI harness stabilizes."
        )
    else:
        decision = (
            f"Passed {score_data['passed']}/{score_data['total']} probes. The audited "
            f"{audited_surfaces} all produced the expected "
            "evidence for this run."
            f"{app_workbench_note}"
        )
        plan = (
            "1. Keep the advanced examples covered by focused regression tests.\n"
            "2. Add this matrix to CI once the runtime cost is acceptable for PR checks.\n"
            "3. Add a Garnet Studio UI entry for \"Agentic Stress Tests\" that runs the same harness.\n"
            "4. Expand separate productization lanes for signed/notarized macOS, web/PWA, mobile, and promo video artifacts.\n"
            "5. Preserve production allocator ARC, native backend, proof, and empirical claims as separate executable gates."
        )
    return f"""# Garnet Agentic Dogfood Readiness Report

## Target

- Repo: `{metadata["repo"]}`
- Head: `{metadata["head"]}`
- Branch: `{metadata["branch"]}`
- Garnet binary: `{metadata["garnet"]}`
- Artifact directory: `{metadata["artifact_dir"]}`

## Decision

Readiness score: **{score_data["readiness"]}/100**

{decision}

## Failing Probes

{failing_ids}

## Score Inputs

```json
{json.dumps(score_data, indent=2)}
```

## Domain Coverage Adequacy

The readiness score tracks whether probes pass. This table tracks whether each
domain has enough independent probes to support the user's requested 3-5 probe
coverage bar.

| Domain | Passed | Probe coverage | Coverage | Status |
| --- | ---: | ---: | ---: | --- |
{coverage_rows}

## Next Implementation Plan

{plan}
"""


def render_deck(results: list[ProbeResult], metadata: dict[str, str], score_data: dict[str, int | float]) -> str:
    domain_cards = "\n".join(
        f"<article><h3>{item['domain']}</h3><p>{item['passed']}/{item['probe_count']} probes passed · {item['status']}</p></article>"
        for item in domain_coverage(results)
    )
    finding_cards = "\n".join(
        f"<li><strong>{result.probe.id}</strong>: {result.probe.claim}</li>"
        for result in results
        if not result.passed
    ) or "<li>No failing probes in this run.</li>"
    return f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Garnet Agentic Dogfood Deck</title>
  <style>
    :root {{
      color-scheme: dark;
      --bg: #090a0f;
      --panel: #161821;
      --text: #f3f4f6;
      --muted: #a7adbb;
      --garnet: #9e2b2f;
      --amber: #d9a441;
      --line: rgba(255,255,255,.12);
    }}
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0;
      background: var(--bg);
      color: var(--text);
      font-family: -apple-system, BlinkMacSystemFont, "SF Pro Display", "Segoe UI", sans-serif;
      line-height: 1.45;
    }}
    section {{
      min-height: 100vh;
      padding: 72px;
      display: flex;
      flex-direction: column;
      justify-content: center;
      border-bottom: 1px solid var(--line);
    }}
    h1 {{ font-size: 72px; line-height: .95; margin: 0 0 24px; max-width: 1050px; }}
    h2 {{ font-size: 46px; margin: 0 0 28px; }}
    p {{ color: var(--muted); font-size: 24px; max-width: 980px; }}
    .score {{ font-size: 128px; color: var(--amber); font-weight: 800; letter-spacing: 0; }}
    .grid {{ display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 18px; }}
    article {{ background: var(--panel); border: 1px solid var(--line); border-radius: 8px; padding: 22px; }}
    article h3 {{ margin: 0 0 10px; font-size: 22px; }}
    article p {{ margin: 0; font-size: 18px; }}
    ul {{ font-size: 24px; color: var(--muted); max-width: 1100px; }}
    li {{ margin: 14px 0; }}
    code {{ color: #ffd6d8; }}
    .bar {{ height: 10px; background: var(--garnet); width: {score_data["readiness"]}%; border-radius: 999px; }}
  </style>
</head>
<body>
  <section>
    <h1>Garnet Agentic Dogfood</h1>
    <p>Advanced probes for agent orchestration, agent toolbelt examples, safe-mode boundaries, migration, repo/site adoption truth, release integrity, signed-release provenance, macOS notarization readiness, docs, web/PWA productization, app onboarding, and memory-analysis examples.</p>
    <p><code>{metadata["head"][:12]}</code> · <code>{metadata["branch"]}</code></p>
  </section>
  <section>
    <h2>Readiness Score</h2>
    <div class="score">{score_data["readiness"]}/100</div>
    <div class="bar"></div>
    <p>{score_data["passed"]}/{score_data["total"]} probes passed.</p>
  </section>
  <section>
    <h2>Coverage</h2>
    <div class="grid">{domain_cards}</div>
  </section>
  <section>
    <h2>Findings</h2>
    <ul>{finding_cards}</ul>
  </section>
  <section>
    <h2>Next</h2>
    <ul>
      <li>Keep this matrix as a reusable dogfood gate for agent-facing language claims.</li>
      <li>Surface it inside Garnet Studio as an "Agentic Stress Tests" workflow.</li>
      <li>Use the same artifact contract for future web/mobile/video productization lanes.</li>
    </ul>
  </section>
</body>
</html>
"""


def write_outputs(work: Path, results: list[ProbeResult], metadata: dict[str, str]) -> None:
    score_data = score(results)
    coverage_data = domain_coverage(results)
    data = {
        "metadata": metadata,
        "score": score_data,
        "domain_coverage": coverage_data,
        "results": [
            {
                "id": result.probe.id,
                "domain": result.probe.domain,
                "claim": result.probe.claim,
                "status": result.status,
                "exit_code": result.exit_code,
                "duration_ms": result.duration_ms,
                "security_domain": result.probe.security_domain,
                "command": result.probe.command,
                "expected_stdout": result.probe.expected_stdout,
                "expected_stderr": result.probe.expected_stderr,
                "missing_stdout": result.missing_stdout,
                "missing_stderr": result.missing_stderr,
                "stdout_log": result.stdout_log,
                "stderr_log": result.stderr_log,
                "stdout_excerpt": result.stdout_excerpt,
                "stderr_excerpt": result.stderr_excerpt,
            }
            for result in results
        ],
    }
    (work / "dogfood-readiness-data.json").write_text(json.dumps(data, indent=2), encoding="utf-8")
    (work / "dogfood-readiness-matrix.md").write_text(render_matrix(results), encoding="utf-8")
    (work / "dogfood-readiness-findings.md").write_text(render_findings(results), encoding="utf-8")
    (work / "dogfood-readiness-report.md").write_text(
        render_report(results, metadata, score_data),
        encoding="utf-8",
    )
    (work / "dogfood-readiness-slide-deck.html").write_text(
        render_deck(results, metadata, score_data),
        encoding="utf-8",
    )
    (work / "dogfood-readiness-mutations.md").write_text(
        "# Agentic Garnet Mutation Log\n\n"
        "- `release-manifest-tamper-detection` mutates `tamper.garnet` after deterministic build and expects `verify` to fail with `source_hash mismatch`.\n"
        "- `check-safe-violation` injects a forbidden safe-mode `raise`/`var` body and expects the checker to reject it.\n"
        "- `check-malformed-agent-source` injects malformed syntax and expects the parser diagnostic to stay actionable.\n"
        "- `check-missing-agent-source`, `eval-unknown-agent-symbol`, and `verify-missing-release-manifest` exercise missing-input and undefined-symbol recovery paths.\n",
        encoding="utf-8",
    )
    status_script = ROOT / "scripts" / "garnet_readiness_status.py"
    if status_script.exists():
        status = run(["python3", str(status_script)], ROOT, timeout=30)
        status_text = status.stdout if status.returncode == 0 else status.stderr
        (work / "readiness-slice-status.md").write_text(status_text, encoding="utf-8")
    else:
        (work / "readiness-slice-status.md").write_text(
            "# Garnet Readiness Slice Status\n\n"
            "Not available in this packaged matrix context.\n",
            encoding="utf-8",
        )
    subprocess.run(
        "find . -type f ! -name MANIFEST.sha256 ! -name MANIFEST.verify.log -print0 | "
        "sort -z | xargs -0 shasum -a 256 > MANIFEST.sha256 && "
        "shasum -a 256 -c MANIFEST.sha256 > MANIFEST.verify.log",
        cwd=work,
        shell=True,
        check=True,
    )


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--garnet-bin", help="path to an existing garnet binary")
    parser.add_argument(
        "--app-executable",
        help="path to a packaged Garnet Studio executable; replaces SwiftPM app probes with packaged-app probes",
    )
    parser.add_argument("--output-dir", help="artifact directory; defaults to /tmp")
    parser.add_argument(
        "--copy-to-desktop",
        action="store_true",
        help="copy the completed artifact directory into ~/Desktop/dogfood",
    )
    parser.add_argument(
        "--strict",
        action="store_true",
        help="exit nonzero when any probe fails; default records findings and exits 0",
    )
    parser.add_argument(
        "--skip-app-workbench",
        action="store_true",
        help="skip SwiftUI/SwiftPM app probes for headless CI runs; local/package gates cover them",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    garnet = ensure_garnet_bin(args.garnet_bin)
    app_executable = Path(args.app_executable).expanduser().resolve() if args.app_executable else None
    if app_executable is not None and not os.access(app_executable, os.X_OK):
        raise SystemExit(f"Garnet Studio executable is not executable: {app_executable}")
    stamp = time.strftime("%Y%m%d-%H%M%S")
    work = (
        Path(args.output_dir).expanduser().resolve()
        if args.output_dir
        else Path(tempfile.gettempdir()) / f"garnet-agentic-dogfood-{stamp}"
    )
    work.mkdir(parents=True, exist_ok=True)
    fixtures = prepare_fixtures(work)

    metadata = {
        "repo": str(ROOT),
        "head": run(["git", "rev-parse", "HEAD"], ROOT).stdout.strip(),
        "branch": run(["git", "branch", "--show-current"], ROOT).stdout.strip(),
        "garnet": str(garnet),
        "app_executable": str(app_executable) if app_executable else "",
        "app_workbench": "skipped" if args.skip_app_workbench else "included",
        "artifact_dir": str(work),
    }

    results: list[ProbeResult] = []
    for item in probe_set(
        garnet,
        work,
        fixtures,
        app_executable=app_executable,
        include_app_workbench=not args.skip_app_workbench,
    ):
        result = item() if callable(item) else run_probe(item, work)
        results.append(result)
        print(f"{result.status.upper():6} {result.probe.domain:28} {result.probe.id}")

    write_outputs(work, results, metadata)
    final_score = score(results)
    print(f"artifact_dir={work}")
    print(f"readiness={final_score['readiness']}")
    print(f"passed={final_score['passed']}/{final_score['total']}")

    if args.copy_to_desktop:
        desktop = Path.home() / "Desktop" / "dogfood" / work.name
        if desktop.exists():
            shutil.rmtree(desktop)
        shutil.copytree(work, desktop)
        print(f"desktop_copy={desktop}")

    return 1 if args.strict and final_score["failed"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
