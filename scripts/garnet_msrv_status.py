#!/usr/bin/env python3
"""Validate Garnet's single, structurally CI-enforced Rust MSRV contract.

The gate is local and deterministic. It enumerates every active Cargo manifest,
checks current Rust-version claims, and consumes the repository's pinned typed
workflow projection so comments, disabled steps, or commands in the wrong job
cannot satisfy CI enforcement.
"""
from __future__ import annotations

import argparse
import importlib.util
import json
import re
import shutil
import sys
import tomllib
from dataclasses import asdict, dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MSRV = "1.95"
EXCLUDED_ACTIVE_MANIFESTS = (
    "apps/garnet-studio/src-tauri/Cargo.toml",
    "garnet-parser-v0.3/fuzz/Cargo.toml",
)
MANIFEST_SCAN_IGNORED_PARTS = {".git", "archive", "node_modules", "target"}
CURRENT_SURFACES = {
    "README.md": ("Rust 1.95+",),
    "CONTRIBUTING.md": ("**Rust** 1.95+",),
    "FAQ.md": ("Rust 1.95+",),
    "docs/getting-started.html": ("Rust 1.95+",),
    "docs/index.html": ("Rust 1.95+",),
    "garnet-parser-v0.3/README.md": ("**Rust version:** 1.95+",),
}
RUST_VERSION_CLAIM_RE = re.compile(
    r"(?i)\brust(?:c)?\b"
    r"(?:(?:\s|[*_`<>=:/()—–-]){0,12}(?:version|toolchain))?"
    r"(?:\s|[*_`<>=:/()—–-]){0,20}"
    r"v?([0-9]+\.[0-9]+)(?:\.[0-9]+)?\+?"
)
CI_PATH = ".github/workflows/ci.yml"
STUDIO_CI_PATH = ".github/workflows/macos-studio.yml"
ROOT_AGENT_PATH = "AGENTS.md"
ROOT_CI_COMMAND = (
    "cargo +1.95.0 check --workspace --all-targets --all-features --locked"
)
STUDIO_CI_COMMAND = (
    "cargo +1.95.0 check --locked --manifest-path "
    "apps/garnet-studio/src-tauri/Cargo.toml --all-targets"
)
INSTALL_COMMAND = "rustup toolchain install 1.95.0 --profile minimal"
REPORTER_TEST_COMMAND = "python3 -I scripts/test_garnet_msrv_status.py"
REPORTER_GATE_COMMAND = "python3 -I scripts/garnet_msrv_status.py --gate"
STABLE_ACTION = "dtolnay/rust-toolchain@stable"
LINUX_STEP_CONDITION = "runner.os == 'Linux'"
AGENT_ANCHOR = 'Cargo `rust-version = "1.95"` is the single workspace MSRV'
_SCHEMA_POLICY: object | None = None


@dataclass
class MsrvStatus:
    schema: str
    msrv: str
    workspace_member_count: int
    workspace_members_inheriting: int
    active_manifest_count: int
    active_manifest_set_exact: bool
    excluded_manifests_declaring: int
    current_surfaces_aligned: bool
    workflow_projection_valid: bool
    stable_tracking_preserved: bool
    exact_msrv_ci_check: bool
    studio_exact_msrv_ci_check: bool
    reporter_ci_wired: bool
    rust_toolchain_file_absent: bool
    procedural_contract_present: bool
    findings: list[str] = field(default_factory=list)
    ok: bool = False


def _read(path: Path, findings: list[str], label: str) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except OSError as exc:
        findings.append(f"{label} is unreadable: {exc}")
        return ""


def _toml(path: Path, findings: list[str], label: str) -> dict[str, object]:
    text = _read(path, findings, label)
    if not text:
        return {}
    try:
        value = tomllib.loads(text)
    except tomllib.TOMLDecodeError as exc:
        findings.append(f"{label} is invalid TOML: {exc}")
        return {}
    if not isinstance(value, dict):
        findings.append(f"{label} root must be a table")
        return {}
    return value


def _package(
    manifest: dict[str, object], findings: list[str], label: str
) -> dict[str, object]:
    package = manifest.get("package", {})
    if not isinstance(package, dict):
        findings.append(f"{label} is missing [package]")
        return {}
    return package


def _inherits_msrv(value: object) -> bool:
    return isinstance(value, dict) and value.get("workspace") is True


def _workspace_members(
    root_manifest: dict[str, object], findings: list[str]
) -> list[str]:
    workspace = root_manifest.get("workspace", {})
    if not isinstance(workspace, dict):
        findings.append("root Cargo.toml is missing [workspace]")
        return []
    members = workspace.get("members", [])
    if not isinstance(members, list) or not all(
        isinstance(item, str) for item in members
    ):
        findings.append("root Cargo.toml workspace.members must be a string array")
        return []
    if len(set(members)) != len(members):
        findings.append("root Cargo.toml workspace.members contains duplicates")
    return list(members)


def _active_manifests(root: Path, findings: list[str]) -> set[str]:
    discovered: set[str] = set()
    for path in root.rglob("Cargo.toml"):
        relative = path.relative_to(root)
        if not relative.parts or relative == Path("Cargo.toml"):
            continue
        if any(part in MANIFEST_SCAN_IGNORED_PARTS for part in relative.parts):
            continue
        label = relative.as_posix()
        if path.is_symlink() or not path.is_file():
            findings.append(f"active Cargo manifest is not a regular file: {label}")
            continue
        discovered.add(label)
    return discovered


def _load_schema_policy() -> object:
    global _SCHEMA_POLICY
    if _SCHEMA_POLICY is not None:
        return _SCHEMA_POLICY
    path = Path(__file__).with_name("garnet_workflow_schema_policy.py")
    spec = importlib.util.spec_from_file_location("_garnet_msrv_workflow_schema", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load workflow schema policy from {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    _SCHEMA_POLICY = module
    return module


def _workflow_projection(root: Path, findings: list[str]) -> object | None:
    try:
        schema = _load_schema_policy()
        yaml_policy = schema.yaml_policy
        documents = []
        for relative in (CI_PATH, STUDIO_CI_PATH):
            content = (root / relative).read_bytes()
            documents.append(
                yaml_policy.WorkflowDocument(
                    relative=relative,
                    mode="100644",
                    object_id="working-tree",
                    root=yaml_policy._document(content),
                )
            )
        snapshot = yaml_policy.WorkflowYamlSnapshot(tuple(documents), ())
        projection = schema.project_snapshot(snapshot)
    except Exception as exc:  # fail closed on pinned-parser or structural failure
        findings.append(f"typed workflow projection failed: {exc}")
        return None
    if projection.problems:
        findings.extend(f"typed workflow projection: {item}" for item in projection.problems)
        return None
    return projection


def _workflow(projection: object, relative: str) -> object | None:
    return next(
        (
            workflow
            for workflow in projection.workflows
            if workflow.source.relative == relative
        ),
        None,
    )


def _job(workflow: object | None, job_id: str) -> object | None:
    if workflow is None:
        return None
    return next((job for job in workflow.jobs if job.job_id == job_id), None)


def _node_value(node: object) -> str | None:
    value = getattr(node, "value", None)
    return value if isinstance(value, str) else None


def _step_values(step: object) -> dict[str, object]:
    items = getattr(step, "items", ())
    return dict(items) if isinstance(items, tuple) else {}


def _active_step(
    job: object | None,
    *,
    key: str,
    value: str,
    condition: str | None,
) -> bool:
    if job is None or job.condition is not None:
        return False
    for step in job.steps:
        values = _step_values(step)
        if _node_value(values.get(key)) != value:
            continue
        actual_condition = _node_value(values.get("if"))
        if condition is None and "if" not in values:
            return True
        if condition is not None and actual_condition == condition:
            return True
    return False


def _job_is_linux_matrix(job: object | None) -> bool:
    if job is None or job.condition is not None or job.matrix is None:
        return False
    values = {_node_value(value) for value in job.matrix.values}
    return (
        job.job_id == "test"
        and _node_value(job.runs_on) == "${{ matrix.os }}"
        and job.matrix.axis == "os"
        and "ubuntu-latest" in values
    )


def _job_is_windows(job: object | None) -> bool:
    return (
        job is not None
        and job.condition is None
        and job.job_id == "windows-studio"
        and _node_value(job.runs_on) == "windows-latest"
    )


def _surface_claims(text: str) -> list[str]:
    return [match.group(1) for match in RUST_VERSION_CLAIM_RE.finditer(text)]


def read_status(root: Path = ROOT) -> MsrvStatus:
    findings: list[str] = []
    root_manifest = _toml(root / "Cargo.toml", findings, "root Cargo.toml")
    workspace = root_manifest.get("workspace", {})
    if not isinstance(workspace, dict):
        workspace = {}
    workspace_package = workspace.get("package", {})
    if not isinstance(workspace_package, dict):
        workspace_package = {}
    if workspace_package.get("rust-version") != MSRV:
        findings.append(
            f'root [workspace.package] rust-version must be exactly "{MSRV}"'
        )

    members = _workspace_members(root_manifest, findings)
    expected_manifests = {
        *[f"{member}/Cargo.toml" for member in members],
        *EXCLUDED_ACTIVE_MANIFESTS,
    }
    discovered_manifests = _active_manifests(root, findings)
    active_manifest_set_exact = discovered_manifests == expected_manifests
    for relative in sorted(discovered_manifests - expected_manifests):
        findings.append(f"unlisted active Cargo manifest: {relative}")
    for relative in sorted(expected_manifests - discovered_manifests):
        findings.append(f"expected active Cargo manifest is missing: {relative}")

    inheriting = 0
    for member in members:
        relative = f"{member}/Cargo.toml"
        manifest = _toml(root / relative, findings, relative)
        package = _package(manifest, findings, relative)
        if _inherits_msrv(package.get("rust-version")):
            inheriting += 1
        else:
            findings.append(
                f"{relative} must inherit the workspace MSRV with "
                "rust-version.workspace = true"
            )

    excluded_declaring = 0
    for relative in EXCLUDED_ACTIVE_MANIFESTS:
        manifest = _toml(root / relative, findings, relative)
        package = _package(manifest, findings, relative)
        if package.get("rust-version") == MSRV:
            excluded_declaring += 1
        else:
            findings.append(
                f'{relative} must declare rust-version = "{MSRV}" directly'
            )

    surfaces_aligned = True
    for relative, required in CURRENT_SURFACES.items():
        text = _read(root / relative, findings, relative)
        missing = [marker for marker in required if marker not in text]
        conflicting = sorted(
            {claim for claim in _surface_claims(text) if claim != MSRV}
        )
        if missing or conflicting:
            surfaces_aligned = False
        if missing:
            findings.append(
                f"{relative} is missing current MSRV marker(s): {missing}"
            )
        if conflicting:
            findings.append(
                f"{relative} carries conflicting Rust version claim(s): {conflicting}"
            )

    projection = _workflow_projection(root, findings)
    workflow_projection_valid = projection is not None
    ci_workflow = _workflow(projection, CI_PATH) if projection is not None else None
    studio_workflow = (
        _workflow(projection, STUDIO_CI_PATH) if projection is not None else None
    )
    test_job = _job(ci_workflow, "test")
    agent_job = _job(ci_workflow, "agent-contracts")
    studio_job = _job(studio_workflow, "windows-studio")

    stable_tracking = (
        _job_is_linux_matrix(test_job)
        and _active_step(test_job, key="uses", value=STABLE_ACTION, condition=None)
        and _job_is_windows(studio_job)
        and _active_step(studio_job, key="uses", value=STABLE_ACTION, condition=None)
    )
    if not stable_tracking:
        findings.append(
            "moving stable must execute in ci.yml:test and "
            "macos-studio.yml:windows-studio"
        )

    exact_ci = (
        _job_is_linux_matrix(test_job)
        and _active_step(
            test_job,
            key="run",
            value=INSTALL_COMMAND,
            condition=LINUX_STEP_CONDITION,
        )
        and _active_step(
            test_job,
            key="run",
            value=ROOT_CI_COMMAND,
            condition=LINUX_STEP_CONDITION,
        )
    )
    if not exact_ci:
        findings.append(
            "ci.yml:test is missing active Linux-only exact Rust 1.95 install/check steps"
        )

    studio_exact_ci = (
        _job_is_windows(studio_job)
        and _active_step(
            studio_job, key="run", value=INSTALL_COMMAND, condition=None
        )
        and _active_step(
            studio_job, key="run", value=STUDIO_CI_COMMAND, condition=None
        )
    )
    if not studio_exact_ci:
        findings.append(
            "macos-studio.yml:windows-studio is missing active exact Rust 1.95 "
            "install/check steps"
        )

    reporter_ci_wired = (
        agent_job is not None
        and agent_job.condition is None
        and _active_step(
            agent_job, key="run", value=REPORTER_TEST_COMMAND, condition=None
        )
        and _active_step(
            agent_job, key="run", value=REPORTER_GATE_COMMAND, condition=None
        )
    )
    if not reporter_ci_wired:
        findings.append(
            "ci.yml:agent-contracts is missing active MSRV reporter test/gate steps"
        )

    toolchain_absent = not (root / "rust-toolchain.toml").exists() and not (
        root / "rust-toolchain"
    ).exists()
    if not toolchain_absent:
        findings.append("the moving-stable policy forbids a repository toolchain pin")

    agent_text = _read(root / ROOT_AGENT_PATH, findings, ROOT_AGENT_PATH)
    procedural_contract = AGENT_ANCHOR in agent_text
    if not procedural_contract:
        findings.append("AGENTS.md is missing the procedural MSRV contract")

    return MsrvStatus(
        schema="garnet.msrv_status/v2",
        msrv=MSRV,
        workspace_member_count=len(members),
        workspace_members_inheriting=inheriting,
        active_manifest_count=len(discovered_manifests),
        active_manifest_set_exact=active_manifest_set_exact,
        excluded_manifests_declaring=excluded_declaring,
        current_surfaces_aligned=surfaces_aligned,
        workflow_projection_valid=workflow_projection_valid,
        stable_tracking_preserved=stable_tracking,
        exact_msrv_ci_check=exact_ci,
        studio_exact_msrv_ci_check=studio_exact_ci,
        reporter_ci_wired=reporter_ci_wired,
        rust_toolchain_file_absent=toolchain_absent,
        procedural_contract_present=procedural_contract,
        findings=findings,
        ok=not findings,
    )


def copy_contract_surface(source: Path, destination: Path) -> None:
    """Copy the deterministic MSRV surface for mutation tests."""
    root_manifest = tomllib.loads((source / "Cargo.toml").read_text(encoding="utf-8"))
    members = root_manifest["workspace"]["members"]
    paths = {
        "Cargo.toml",
        *[f"{member}/Cargo.toml" for member in members],
        *EXCLUDED_ACTIVE_MANIFESTS,
        *CURRENT_SURFACES,
        CI_PATH,
        STUDIO_CI_PATH,
        ROOT_AGENT_PATH,
    }
    for relative in sorted(paths):
        target = destination / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source / relative, target)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero unless every structural MSRV contract holds",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    status = read_status(args.root.resolve())
    print(json.dumps(asdict(status), indent=2, sort_keys=True))
    if args.gate and not status.ok:
        print("garnet-msrv gate FAILED: " + "; ".join(status.findings), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
