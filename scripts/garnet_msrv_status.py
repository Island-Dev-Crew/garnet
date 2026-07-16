#!/usr/bin/env python3
"""Validate Garnet's single, structurally CI-enforced Rust MSRV contract.

The gate is local, deterministic, and stdlib-only. It enumerates every active
Cargo manifest, checks current Rust-version claims, and projects only the
canonical workflow job/step shape it needs. Ambiguous YAML features fail closed
instead of being guessed, so comments, disabled steps, or commands in the wrong
job cannot satisfy CI enforcement.
"""
from __future__ import annotations

import argparse
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
YAML_KEY_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_-]*$")
YAML_ANCHOR_RE = re.compile(r"(^|\s)[&*][A-Za-z0-9_-]+(?=\s|$)")
YAML_BLOCK_RE = re.compile(
    r"^(?:-\s+)?[A-Za-z_][A-Za-z0-9_-]*:\s*[|>][+-]?$"
)
MAX_WORKFLOW_LINES = 2_000
MAX_WORKFLOW_LINE_LENGTH = 4_096


@dataclass
class WorkflowStepProjection:
    fields: dict[str, str]


@dataclass
class WorkflowJobProjection:
    job_id: str
    fields: dict[str, str]
    matrix: dict[str, tuple[str, ...]]
    steps: list[WorkflowStepProjection]

    @property
    def condition(self) -> str | None:
        return self.fields.get("if")

    @property
    def runs_on(self) -> str | None:
        return self.fields.get("runs-on")


@dataclass
class WorkflowFileProjection:
    relative: str
    jobs: dict[str, WorkflowJobProjection]


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


def _strip_inline_comment(value: str) -> str:
    single = False
    double = False
    escaped = False
    for index, char in enumerate(value):
        if double and char == "\\" and not escaped:
            escaped = True
            continue
        if char == "'" and not double and not escaped:
            single = not single
        elif char == '"' and not single and not escaped:
            double = not double
        elif char == "#" and not single and not double and (
            index == 0 or value[index - 1].isspace()
        ):
            return value[:index].rstrip()
        escaped = False
    if single or double:
        raise ValueError("unterminated quoted YAML scalar")
    return value.rstrip()


def _reject_ambiguous_yaml(value: str, line_number: int) -> None:
    stripped = value.strip()
    if stripped in {"---", "..."} or stripped.startswith("%"):
        raise ValueError(f"line {line_number}: YAML directives/documents are unsupported")
    if "<<:" in stripped:
        raise ValueError(f"line {line_number}: YAML merge keys are unsupported")
    if YAML_ANCHOR_RE.search(stripped) or stripped.startswith("!"):
        raise ValueError(f"line {line_number}: YAML anchors, aliases, and tags are unsupported")
    if stripped.startswith("{"):
        raise ValueError(f"line {line_number}: flow mappings are unsupported")


def _unquote_scalar(value: str, line_number: int) -> str:
    value = _strip_inline_comment(value).strip()
    _reject_ambiguous_yaml(value, line_number)
    if not value:
        return ""
    if value[0] not in {"'", '"'}:
        return value
    if len(value) < 2 or value[-1] != value[0]:
        raise ValueError(f"line {line_number}: malformed quoted YAML scalar")
    if value[0] == "'":
        if "''" in value[1:-1]:
            raise ValueError(
                f"line {line_number}: escaped single-quoted YAML is unsupported"
            )
        return value[1:-1]
    try:
        decoded = json.loads(value)
    except json.JSONDecodeError as exc:
        raise ValueError(
            f"line {line_number}: non-JSON double-quoted YAML is unsupported"
        ) from exc
    if not isinstance(decoded, str):
        raise ValueError(f"line {line_number}: YAML scalar must be text")
    return decoded


def _split_yaml_key(content: str, line_number: int) -> tuple[str, str]:
    content = _strip_inline_comment(content)
    if not content:
        raise ValueError(f"line {line_number}: empty YAML mapping entry")
    if ":" not in content:
        raise ValueError(f"line {line_number}: expected a YAML mapping entry")
    key, remainder = content.split(":", 1)
    if not YAML_KEY_RE.fullmatch(key):
        raise ValueError(f"line {line_number}: non-canonical YAML key {key!r}")
    if remainder and not remainder.startswith(" "):
        raise ValueError(
            f"line {line_number}: YAML key/value separator must contain one space"
        )
    return key, remainder[1:] if remainder else ""


def _flow_sequence(value: str, line_number: int) -> tuple[str, ...]:
    value = _strip_inline_comment(value).strip()
    if not (value.startswith("[") and value.endswith("]")):
        raise ValueError(
            f"line {line_number}: workflow matrix values must be one flow sequence"
        )
    inner = value[1:-1].strip()
    if not inner or any(char in inner for char in "[]{}"):
        raise ValueError(f"line {line_number}: matrix flow sequence is ambiguous")
    items = tuple(
        _unquote_scalar(item.strip(), line_number) for item in inner.split(",")
    )
    if any(not item for item in items) or len(set(items)) != len(items):
        raise ValueError(
            f"line {line_number}: matrix values must be unique non-empty strings"
        )
    return items


def _workflow_lines(text: str) -> list[tuple[int, int, str]]:
    if text.startswith("\ufeff") or "\r" in text or "\t" in text:
        raise ValueError("workflow must be BOM-free UTF-8/LF text without tabs")
    raw_lines = text.split("\n")
    if len(raw_lines) > MAX_WORKFLOW_LINES:
        raise ValueError("workflow line count exceeds the MSRV projection bound")
    logical: list[tuple[int, int, str]] = []
    block_parent: int | None = None
    for line_number, raw in enumerate(raw_lines, 1):
        if len(raw) > MAX_WORKFLOW_LINE_LENGTH:
            raise ValueError(f"line {line_number}: workflow line exceeds the bound")
        if not raw.strip():
            continue
        indent = len(raw) - len(raw.lstrip(" "))
        if block_parent is not None and indent > block_parent:
            continue
        block_parent = None
        if indent % 2 != 0 or indent > 40:
            raise ValueError(
                f"line {line_number}: workflow indentation is not canonical"
            )
        content = raw[indent:]
        if content.startswith("#"):
            continue
        content = _strip_inline_comment(content)
        if not content:
            continue
        _reject_ambiguous_yaml(content, line_number)
        logical.append((line_number, indent, content))
        if YAML_BLOCK_RE.fullmatch(content):
            block_parent = indent
    return logical


def _validate_nested_entry(content: str, line_number: int) -> None:
    if content.startswith("- "):
        value = content[2:].strip()
        if not value:
            raise ValueError(f"line {line_number}: empty nested sequence item")
        _reject_ambiguous_yaml(value, line_number)
        return
    key, value = _split_yaml_key(content, line_number)
    if key == "<<":
        raise ValueError(f"line {line_number}: YAML merge keys are unsupported")
    _unquote_scalar(value, line_number)


def _parse_workflow(text: str, relative: str) -> WorkflowFileProjection:
    logical = _workflow_lines(text)
    jobs_markers = [
        index
        for index, (_line, indent, content) in enumerate(logical)
        if indent == 0 and content == "jobs:"
    ]
    if len(jobs_markers) != 1:
        raise ValueError(f"{relative}: workflow must contain exactly one jobs mapping")

    jobs: dict[str, WorkflowJobProjection] = {}
    current: WorkflowJobProjection | None = None
    current_step: WorkflowStepProjection | None = None
    section: str | None = None
    matrix_open = False
    inside_jobs = False

    for line_number, indent, content in logical:
        if indent == 0:
            if content == "jobs:":
                inside_jobs = True
                continue
            if inside_jobs:
                break
            continue
        if not inside_jobs:
            continue
        if indent == 2:
            job_id, value = _split_yaml_key(content, line_number)
            if value:
                raise ValueError(
                    f"line {line_number}: reusable/inline workflow jobs are unsupported"
                )
            if job_id in jobs:
                raise ValueError(f"line {line_number}: duplicate workflow job {job_id}")
            current = WorkflowJobProjection(job_id, {}, {}, [])
            jobs[job_id] = current
            current_step = None
            section = None
            matrix_open = False
            continue
        if current is None:
            raise ValueError(f"line {line_number}: job content has no owning job")
        if indent == 4:
            key, raw_value = _split_yaml_key(content, line_number)
            if key in current.fields:
                raise ValueError(
                    f"line {line_number}: duplicate job key {current.job_id}.{key}"
                )
            value = _unquote_scalar(raw_value, line_number)
            current.fields[key] = value
            section = key if value == "" else None
            current_step = None
            matrix_open = False
            if key in {"steps", "strategy"} and value:
                raise ValueError(
                    f"line {line_number}: {current.job_id}.{key} must be a block"
                )
            continue
        if indent == 6:
            if section == "steps":
                if not content.startswith("- "):
                    raise ValueError(
                        f"line {line_number}: workflow step must start with '- '"
                    )
                key, raw_value = _split_yaml_key(content[2:], line_number)
                current_step = WorkflowStepProjection(
                    {key: _unquote_scalar(raw_value, line_number)}
                )
                current.steps.append(current_step)
                continue
            if section == "strategy":
                key, raw_value = _split_yaml_key(content, line_number)
                value = _unquote_scalar(raw_value, line_number)
                matrix_open = key == "matrix" and value == ""
                if key == "matrix" and value:
                    raise ValueError(
                        f"line {line_number}: strategy.matrix must be a block"
                    )
                continue
            _validate_nested_entry(content, line_number)
            continue
        if indent == 8:
            if section == "steps":
                if current_step is None:
                    raise ValueError(
                        f"line {line_number}: step continuation has no owning step"
                    )
                key, raw_value = _split_yaml_key(content, line_number)
                if key in current_step.fields:
                    raise ValueError(
                        f"line {line_number}: duplicate workflow step key {key}"
                    )
                current_step.fields[key] = _unquote_scalar(raw_value, line_number)
                continue
            if section == "strategy" and matrix_open:
                axis, raw_value = _split_yaml_key(content, line_number)
                if axis in current.matrix:
                    raise ValueError(
                        f"line {line_number}: duplicate workflow matrix axis {axis}"
                    )
                current.matrix[axis] = _flow_sequence(raw_value, line_number)
                continue
            _validate_nested_entry(content, line_number)
            continue
        _validate_nested_entry(content, line_number)

    if not jobs:
        raise ValueError(f"{relative}: workflow jobs mapping is empty")
    for job in jobs.values():
        if not job.runs_on or not job.steps:
            raise ValueError(
                f"{relative}: job {job.job_id} must contain runs-on and steps"
            )
    return WorkflowFileProjection(relative, jobs)


def _workflow_projection(
    root: Path, findings: list[str]
) -> dict[str, WorkflowFileProjection] | None:
    projection: dict[str, WorkflowFileProjection] = {}
    try:
        for relative in (CI_PATH, STUDIO_CI_PATH):
            text = (root / relative).read_text(encoding="utf-8")
            projection[relative] = _parse_workflow(text, relative)
    except (OSError, UnicodeError, ValueError) as exc:
        findings.append(f"stdlib workflow projection failed: {exc}")
        return None
    return projection


def _workflow(
    projection: dict[str, WorkflowFileProjection], relative: str
) -> WorkflowFileProjection | None:
    return projection.get(relative)


def _job(
    workflow: WorkflowFileProjection | None, job_id: str
) -> WorkflowJobProjection | None:
    return workflow.jobs.get(job_id) if workflow is not None else None


def _active_step(
    job: WorkflowJobProjection | None,
    *,
    key: str,
    value: str,
    condition: str | None,
) -> bool:
    if (
        job is None
        or job.condition is not None
        or job.fields.get("continue-on-error") not in {None, "false"}
    ):
        return False
    for step in job.steps:
        if step.fields.get(key) != value:
            continue
        if step.fields.get("continue-on-error") not in {None, "false"}:
            continue
        actual_condition = step.fields.get("if")
        if condition is None and "if" not in step.fields:
            return True
        if condition is not None and actual_condition == condition:
            return True
    return False


def _job_is_linux_matrix(job: WorkflowJobProjection | None) -> bool:
    return (
        job is not None
        and job.condition is None
        and job.fields.get("continue-on-error") in {None, "false"}
        and job.job_id == "test"
        and job.fields.get("name") == "cargo test (${{ matrix.os }})"
        and job.runs_on == "${{ matrix.os }}"
        and set(job.matrix) == {"os"}
        and job.matrix["os"]
        == ("ubuntu-latest", "windows-latest", "macos-latest")
    )


def _job_is_windows(job: WorkflowJobProjection | None) -> bool:
    return (
        job is not None
        and job.condition is None
        and job.fields.get("continue-on-error") in {None, "false"}
        and job.job_id == "windows-studio"
        and job.fields.get("name") == "Windows Studio build + test"
        and job.runs_on == "windows-latest"
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
