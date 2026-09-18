#!/usr/bin/env python3
"""Opt-in T-L shadow classifier. No network, PR mutation, or merge authority.

Reads exact committed blobs; never imports scripts or runs programs from either
candidate tree. An explicitly supplied trusted Garnet binary may parse/check
source and diff declarations. Current checker coverage cannot prove all call
shapes, so Garnet changes always require human review even on a clean diff.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import tempfile
import unicodedata

LIMIT = 2 * 1024 * 1024
SCHEMA = 'garnet.shadow-lanes/1'


def command(args, cwd=None):
    env = {k: v for k, v in os.environ.items() if not k.startswith('GIT_')}
    env.update(GIT_NO_REPLACE_OBJECTS='1', GIT_CONFIG_NOSYSTEM='1',
               GIT_CONFIG_GLOBAL=os.devnull, GIT_TERMINAL_PROMPT='0')
    result = subprocess.run(args, cwd=cwd, env=env, capture_output=True, timeout=30, check=False)
    if len(result.stdout) + len(result.stderr) > LIMIT:
        raise ValueError('command output exceeds shadow evidence limit')
    return result


def git(repo, *args):
    result = command(['git', '-C', str(repo), *args])
    if result.returncode:
        raise ValueError('Git input unavailable: ' + ' '.join(args[:2]))
    return result.stdout


def read_json(raw):
    def pairs(items):
        result = {}
        for key, value in items:
            if key in result:
                raise ValueError('duplicate JSON field: ' + key)
            result[key] = value
        return result
    return json.loads(raw, object_pairs_hook=pairs)


def classify_paths(paths):
    if not paths:
        return 'blocked', ['empty change has no classification']
    for path in paths:
        parts = PurePosixPath(path).parts
        if ('..' in parts or PurePosixPath(path).is_absolute() or
                not path.startswith('docs/internals/') or not path.endswith('.md') or
                PurePosixPath(path).name in {'AGENTS.md', 'CLAUDE.md'}):
            return 'careful', ['changed paths exceed the prose-only shadow allowlist']
    return 'candidate-fast', ['prose-only; all downstream policy checks still required']


def validate_diff(value, status):
    if not isinstance(value, dict) or value.get('schema') != 'garnet.diff-caps.machine/1':
        raise ValueError('unsupported diff schema')
    expanded = value.get('authority_expanded')
    if type(expanded) is not bool:
        raise ValueError('missing boolean authority verdict')
    expected = 1 if expanded else 0
    if value.get('capability_band') != ('2/5' if expanded else '5/5'):
        raise ValueError('inconsistent capability band')
    if (status != expected or type(value.get('exit_code')) is not int or
            value['exit_code'] != expected or value.get('verdict') !=
            ('authority-expanded' if expanded else 'no-authority-expansion')):
        raise ValueError('inconsistent diff verdict/exit status')
    if type(value.get('skipped_path_count')) is not int or value['skipped_path_count'] != 0 or value.get('skipped_paths') != []:
        raise ValueError('diff walk incomplete or unknown')
    for key in ('aggregate_gained', 'aggregate_removed', 'functions_added', 'functions_removed'):
        items = value.get(key)
        if (not isinstance(items, list) or any(type(item) is not str for item in items)
                or items != sorted(set(items))):
            raise ValueError('invalid diff dimension: ' + key)
    functions = value.get('functions_caps_expanded')
    if not isinstance(functions, list):
        raise ValueError('missing per-function diff')
    for item in functions:
        if (not isinstance(item, dict) or set(item) != {'name', 'gained'} or
                type(item['name']) is not str or not isinstance(item['gained'], list) or
                not item['gained'] or any(type(cap) is not str for cap in item['gained'])):
            raise ValueError('invalid per-function diff')
    if type(value.get('wildcard_introduced')) is not bool:
        raise ValueError('missing wildcard verdict')
    if expanded != bool(value['aggregate_gained'] or value['wildcard_introduced']):
        raise ValueError('inconsistent diff dimensions')
    return expanded


def tree(repo, commit):
    entries = {}
    for entry in git(repo, 'ls-tree', '-rz', '--full-tree', commit).split(b'\0'):
        if not entry:
            continue
        meta, raw = entry.split(b'\t', 1)
        mode, kind, oid = meta.decode('ascii').split()
        path = raw.decode('utf-8', errors='strict')
        if PurePosixPath(path).is_absolute() or '..' in PurePosixPath(path).parts:
            raise ValueError('unsafe committed path')
        entries[path] = (mode, kind, oid)
    return entries


def source_snapshot(repo, entries, destination):
    count = 0
    total = 0
    portable_paths = {}
    for path, (mode, kind, oid) in entries.items():
        if path.endswith('.garnet') or PurePosixPath(path).name == 'Garnet.toml':
            if mode != '100644' or kind != 'blob':
                raise ValueError('source/config is not a plain regular blob')
            data = git(repo, 'cat-file', 'blob', oid)
            total += len(data)
            if total > LIMIT:
                raise ValueError('source snapshot exceeds shadow limit')
            if '\\' in path or ':' in path:
                raise ValueError('nonportable source path')
            parts = PurePosixPath(path).parts
            for length in range(1, len(parts) + 1):
                prefix = '/'.join(parts[:length])
                portable = unicodedata.normalize('NFD', prefix).casefold()
                if portable in portable_paths and portable_paths[portable] != prefix:
                    raise ValueError('source paths alias on case/normalization-insensitive filesystems')
                portable_paths[portable] = prefix
            target = destination / path
            target.parent.mkdir(parents=True, exist_ok=True)
            with target.open('xb') as output:
                output.write(data)
            count += path.endswith('.garnet')
    if not count:
        raise ValueError('source snapshot is empty')


def assess(repo, base, head, binary):
    for commit in (base, head):
        if not re.fullmatch('[0-9a-f]{40}', commit):
            raise ValueError('base and head must be full lowercase 40-character commit IDs')
        if git(repo, 'rev-parse', '--verify', commit + '^{commit}').decode().strip() != commit:
            raise ValueError('commit identity mismatch')
    if command(['git', '-C', str(repo), 'merge-base', '--is-ancestor', base, head]).returncode:
        raise ValueError('base is not an ancestor of head')
    old, new = tree(repo, base), tree(repo, head)
    paths = sorted(p for p in old.keys() | new.keys() if old.get(p) != new.get(p))
    lane, reasons = classify_paths(paths)
    result = dict(schema=SCHEMA, mode='shadow', merge_authorized=False, base=base, head=head,
                  base_tree=git(repo, 'rev-parse', base+'^{tree}').decode().strip(),
                  head_tree=git(repo, 'rev-parse', head+'^{tree}').decode().strip(),
                  changed_paths=paths, lane=lane, reasons=reasons,
                  policy_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                  scope='advisory only; declarations are not complete behavior or ownership proof')
    for path in paths:
        for entries in (old, new):
            if path in entries and entries[path][0:2] != ('100644', 'blob'):
                result.update(lane='blocked', reasons=['changed symlink, executable, or submodule requires separate assessment'])
                return result
    if not any(path.endswith('.garnet') for path in paths):
        return result
    if binary is None:
        result.update(lane='blocked', reasons=['Garnet changes require an explicit trusted local --garnet binary'])
        return result
    executable = Path(binary)
    if not executable.is_absolute() or not executable.is_file() or executable.is_symlink():
        raise ValueError('--garnet must name an absolute regular trusted executable')
    binary_hash = hashlib.sha256(executable.read_bytes()).hexdigest()
    result['garnet_sha256'] = binary_hash
    with tempfile.TemporaryDirectory(prefix='garnet-shadow-') as temp:
        root = Path(temp)
        # Empty manifest terminates upward edition lookup at the trusted
        # extraction boundary while preserving the compiler default.
        (root / 'Garnet.toml').write_text('# shadow default-edition boundary\n')
        before, after = root/'base', root/'head'
        before.mkdir(); after.mkdir()
        source_snapshot(repo, old, before)
        source_snapshot(repo, new, after)
        # verify only parses/checks; no run/test/build from the candidate tree.
        for label, snapshot in [('base', before), ('head', after)]:
            checked = command([str(executable), 'verify', label], cwd=root)
            result[label+'_check'] = {'exit_code': checked.returncode,
                                     'output_sha256': hashlib.sha256(checked.stdout+checked.stderr).hexdigest()}
            if checked.returncode != 0:
                result.update(lane='blocked', reasons=[label+' parse/check failed'])
                return result
        diff = command([str(executable), 'diff-caps', '--machine', 'base', 'head'], cwd=root)
        try:
            payload = read_json(diff.stdout)
            expanded = validate_diff(payload, diff.returncode)
        except (ValueError, UnicodeError) as error:
            result.update(lane='blocked', reasons=[str(error)])
            return result
        if hashlib.sha256(executable.read_bytes()).hexdigest() != binary_hash:
            raise ValueError('trusted executable changed during assessment')
        result['diff'] = payload
        result.update(lane='careful', reasons=[
            'declared authority expanded' if expanded else 'no new declared capabilities detected',
            'checker call-shape and mode-boundary coverage unproven; human review required'])
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, default=Path('.'))
    parser.add_argument('--base', required=True)
    parser.add_argument('--head', required=True)
    parser.add_argument('--garnet', type=Path)
    args = parser.parse_args()
    try:
        result = assess(args.repo, args.base, args.head, args.garnet)
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        result = dict(schema=SCHEMA, mode='shadow', merge_authorized=False,
                      base=args.base, head=args.head, lane='blocked', reasons=[str(error)])
    print(json.dumps(result, sort_keys=True, ensure_ascii=False))
    return {'candidate-fast': 0, 'careful': 1, 'blocked': 2}[result['lane']]


if __name__ == '__main__':
    raise SystemExit(main())
