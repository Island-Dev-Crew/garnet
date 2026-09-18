"""Fail-closed tests for the opt-in shadow classifier (no merge authority)."""
import importlib.util
import os
import json
from unittest.mock import patch
from pathlib import Path
import tempfile
import subprocess
import unittest

SPEC = importlib.util.spec_from_file_location('shadow', Path(__file__).with_name('garnet_shadow_lanes.py'))
shadow = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(shadow)


class PolicyTests(unittest.TestCase):
    def test_prose_only_is_advisory(self):
        result = shadow.classify_paths(['docs/internals/example.md'])
        self.assertEqual(result, ('candidate-fast', ['prose-only; all downstream policy checks still required']))

    def test_policy_and_unknown_paths_require_human(self):
        for path in ['AGENTS.md', 'docs/AGENTS.md', '.github/workflows/ci.yml', 'scripts/x.py',
                     'docs/index.html', 'README.md', 'docs/internals/../x.md']:
            with self.subTest(path=path):
                self.assertEqual(shadow.classify_paths([path])[0], 'careful')

    def test_empty_change_is_not_fast(self):
        self.assertEqual(shadow.classify_paths([])[0], 'blocked')

    def test_machine_verdict_checks_every_fail_closed_field(self):
        valid = {'schema': 'garnet.diff-caps.machine/1', 'verdict': 'no-authority-expansion',
                 'authority_expanded': False, 'exit_code': 0, 'skipped_path_count': 0, 'skipped_paths': [],
                 'capability_band': '5/5', 'aggregate_removed': [], 'functions_added': [], 'functions_removed': [], 'aggregate_gained': [], 'wildcard_introduced': False, 'functions_caps_expanded': []}
        self.assertFalse(shadow.validate_diff(valid, 0))
        self.assertFalse(shadow.validate_diff(valid | {'functions_caps_expanded': [{'name': 'f', 'gained': ['fs']}]}, 0))
        for key in ['schema', 'authority_expanded', 'exit_code', 'skipped_path_count', 'skipped_paths']:
            malformed = dict(valid); del malformed[key]
            with self.subTest(key=key), self.assertRaises(ValueError):
                shadow.validate_diff(malformed, 0)
        for patch in [{'skipped_path_count': 1}, {'skipped_path_count': False},
                      {'skipped_paths': [{'rule':'symlink', 'count':1}]},
                      {'capability_band': '2/5'}, {'aggregate_gained': [False]}, {'exit_code': 1}, {'authority_expanded': 'false'}, {'aggregate_gained':['fs']}]:
            with self.subTest(patch=patch), self.assertRaises(ValueError):
                shadow.validate_diff(valid | patch, 0)
        widened = valid | {'verdict':'authority-expanded','authority_expanded':True,'exit_code':1,'capability_band':'2/5','aggregate_gained':['fs']}
        self.assertTrue(shadow.validate_diff(widened, 1))

    def test_duplicate_json_is_rejected(self):
        with self.assertRaises(ValueError):
            shadow.read_json('{"authority_expanded":true,"authority_expanded":false}')


class GitTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.git('init', '-q')
        self.git('config', 'user.name', 'Fixture')
        self.git('config', 'user.email', 'fixture@example.invalid')
        (self.root/'docs/internals').mkdir(parents=True)
        (self.root/'docs/internals/note.md').write_text('before\n')
        self.git('add', '.'); self.git('commit', '-qm', 'base')
        self.base = self.git('rev-parse', 'HEAD').strip()

    def tearDown(self): self.temp.cleanup()

    def git(self, *args):
        return subprocess.check_output(['git', '-C', str(self.root), *args], text=True)

    def commit(self):
        self.git('add', '.'); self.git('commit', '-qm', 'head')
        return self.git('rev-parse', 'HEAD').strip()

    def test_exact_commit_binding_ignores_dirty_tree(self):
        (self.root/'docs/internals/note.md').write_text('after\n'); head = self.commit()
        (self.root/'docs/internals/note.md').write_text('uncommitted\n')
        result = shadow.assess(self.root, self.base, head, None)
        self.assertEqual(result['lane'], 'candidate-fast')
        self.assertEqual(result['base'], self.base); self.assertEqual(result['head'], head)
        self.assertFalse(result['merge_authorized']); self.assertEqual(result['mode'], 'shadow')

    def test_symbolic_refs_rejected(self):
        with self.assertRaises(ValueError): shadow.assess(self.root, 'HEAD', 'HEAD', None)

    def test_symlink_cannot_enter_prose_lane(self):
        (self.root/'docs/internals/note.md').unlink()
        (self.root/'docs/internals/note.md').symlink_to('/etc/passwd')
        result = shadow.assess(self.root, self.base, self.commit(), None)
        self.assertEqual(result['lane'], 'blocked')

    def test_garnet_change_without_explicit_binary_blocks(self):
        (self.root/'main.garnet').write_text('@caps()\ndef main() { 1 }\n')
        result = shadow.assess(self.root, self.base, self.commit(), None)
        self.assertEqual(result['lane'], 'blocked')

    def test_snapshot_rejects_case_aliases_in_files_and_directories(self):
        oid = self.git('rev-parse', self.base+':docs/internals/note.md').strip()
        for paths in [('MAIN.garnet', 'main.garnet'), ('A/one.garnet', 'a/two.garnet'), ('é.garnet', 'e\u0301.garnet'), ('a\\b.garnet', 'x.garnet')]:
            with tempfile.TemporaryDirectory() as dest, self.subTest(paths=paths):
                entries = {p: ('100644', 'blob', oid) for p in paths}
                with self.assertRaises(ValueError):
                    shadow.source_snapshot(self.root, entries, Path(dest))

    def test_directory_garnet_config_is_careful(self):
        (self.root/'Garnet.toml').write_text('edition = "2026"\n')
        self.assertEqual(shadow.assess(self.root, self.base, self.commit(), None)['lane'], 'careful')


@unittest.skipUnless(os.environ.get('GARNET_SHADOW_TEST_BINARY'), 'set explicit trusted binary for integration')
class RealCompilerTests(unittest.TestCase):
    setUp = GitTests.setUp
    tearDown = GitTests.tearDown
    git = GitTests.git
    commit = GitTests.commit

    def test_ambient_manifest_cannot_change_snapshot_edition(self):
        program = self.root / 'main.garnet'
        program.write_text('@caps()\ndef main() { 1 }\n'); base = self.commit()
        program.write_text('@caps()\ndef main() { 2 }\n'); head = self.commit()
        binary = Path(os.environ['GARNET_SHADOW_TEST_BINARY'])
        normal = shadow.assess(self.root, base, head, binary)
        with tempfile.TemporaryDirectory() as parent:
            Path(parent, 'Garnet.toml').write_text('[project]\nedition = "impossible"\n')
            with patch.object(tempfile, 'tempdir', parent):
                self.assertEqual(normal, shadow.assess(self.root, base, head, binary))

    def test_source_changes_never_grant_merge_authority(self):
        program = self.root / 'main.garnet'
        program.write_text('@caps()\ndef main() { "before" }\n')
        base = self.commit()
        binary = Path(os.environ['GARNET_SHADOW_TEST_BINARY'])
        for source, lane, first_reason in [
            ('@caps()\ndef main() { "after" }\n', 'careful', 'no new declared capabilities detected'),
            (f'@caps(fs)\ndef main() {{ fs::write_file({json.dumps(str(self.root / "never"))}, "x") }}\n', 'careful', 'declared authority expanded'),
            ('@caps()\ndef main() { fs::write_file("never", "x") }\n', 'blocked', 'head parse/check failed'),
        ]:
            program.write_text(source)
            result = shadow.assess(self.root, base, self.commit(), binary)
            self.assertEqual(result['lane'], lane, result)
            self.assertEqual(result['reasons'][0], first_reason, result)
            self.assertFalse(result['merge_authorized'])
            self.assertFalse((self.root / 'never').exists())
            repeated = shadow.assess(self.root, base, result['head'], binary)
            self.assertEqual(result, repeated)


if __name__ == '__main__': unittest.main()
