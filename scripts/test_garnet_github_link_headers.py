#!/usr/bin/env python3
"""No-network contract tests for strict GitHub response Link headers."""
from __future__ import annotations
import importlib.util, sys, unittest
from pathlib import Path
from unittest import mock
SCRIPT = Path(__file__).with_name("garnet_github_link_headers.py")
NEXT = "https://api.github.com/repositories/7/actions/runs?branch=main&page=2&per_page=100"
LAST = "https://api.github.com/repositories/7/actions/runs?branch=main&page=4&per_page=100"
class LinkHeaderTests(unittest.TestCase):
    def helper(self) -> object:
        self.assertTrue(SCRIPT.exists(), "strict Link/header helper is absent")
        spec = importlib.util.spec_from_file_location("_garnet_github_link_headers_test", SCRIPT)
        self.assertIsNotNone(spec); self.assertIsNotNone(spec.loader if spec else None)
        module = importlib.util.module_from_spec(spec)
        sys.modules[spec.name] = module; spec.loader.exec_module(module)
        return module
    def assert_invalid(self, fields: object) -> None:
        helper = self.helper()
        with self.assertRaises(helper.HeaderSyntaxError): helper.parse_header_fields(fields)
    def test_repeated_physical_link_fields_are_preserved_and_structured(self) -> None:
        helper = self.helper()
        comma_uri = NEXT.replace("actions/runs", "actions/a,b;c")
        block = helper.parse_header_fields((
            ("Content-Type", "application/json"),
            ("Link", f'<{comma_uri}>; title="a,b;c\\\"d"; rel="next", <{LAST}>; rel=last'),
            ("lInK", '</repositories/7/actions/runs?page=1&per_page=100>; rel=first'),
        ))
        self.assertEqual(block.get("CONTENT-TYPE"), "application/json")
        self.assertEqual(tuple(item.target for item in block.links),
                         (comma_uri, LAST, "/repositories/7/actions/runs?page=1&per_page=100"))
        self.assertEqual(tuple(item.relations for item in block.links),
                         (("next",), ("last",), ("first",)))
        self.assertEqual(block.links[0].parameter("TITLE"), 'a,b;c"d')
    def test_header_names_values_containers_and_singletons_are_strict(self) -> None:
        helper = self.helper()
        self.assertEqual(helper.parse_header_fields((("X_Test-1", "ok\tvalue"),)).get(
            "x_test-1"), "ok\tvalue")
        bad = ([('Content-Type', 'application/json')],
               (("Content Type", "x"),), (("X@Test", "x"),), (("X-Ü", "x"),),
               (("X-Test", "bad\rvalue"),), (("X-Test", "bad\nvalue"),),
               (("Content-Type", "a"), ("content-type", "b")),
               (("Content-Type", object()),), (("Content-Type",),), ("not-a-pair",))
        for fields in bad:
            with self.subTest(fields=repr(fields)): self.assert_invalid(fields)
    def test_physical_link_aggregate_bound_and_duplicate_relations_fail(self) -> None:
        helper = self.helper()
        one, two = f'<{NEXT}>; rel=next', f'<{LAST}>; rel=last'
        with mock.patch.object(helper, "MAX_LINK_HEADER_CHARS", len(one) + len(two) + 1):
            with self.assertRaises(helper.HeaderSyntaxError): helper.parse_header_fields(
                (("Link", one), ("Link", two)))
        self.assert_invalid((("Link", one), ("Link", one)))
    def test_link_grammar_accepts_quotes_ptoken_and_space_relation_lists(self) -> None:
        helper = self.helper()
        value = (f'<{NEXT}>; title="a,b;c\\\"d"; type="text/plain"; '
                 'title*=UTF-8\'en\'%E2%82%AC; anchor="https://example.invalid/context"; '
                 'ReL="prev  next https://rels.example/relation"')
        link = helper.parse_header_fields((("Link", value),)).links[0]
        self.assertEqual(link.relations,
                         ("prev", "next", "https://rels.example/relation"))
        self.assertEqual(link.parameter("type"), "text/plain")
        self.assertEqual(link.parameter("anchor"), "https://example.invalid/context")
        self.assertEqual(link.parameter("title*"), "UTF-8'en'%E2%82%AC")
    def test_each_link_requires_one_valid_rel_and_exact_parameter_grammar(self) -> None:
        bad = (f'<{NEXT}>; title="missing rel"', f'<{NEXT}>; rel=next; REL=prev',
               f'<{NEXT}>; rel=""', f'<{NEXT}>; rel=" next"',
               f'<{NEXT}>; rel="next "', f'<{NEXT}>; rel="next\tlast"',
               f'<{NEXT}>; rel=NEXT', f'<{NEXT}>; rel="next next"',
               f'<{NEXT}>; rel="next,"',
               f'<{NEXT}>; rel=next; type=text/plain',
               f'<{NEXT}>; rel=next; title=a; TITLE=b',
               f'<{NEXT}>; rel=next; title="unterminated', f'<{NEXT}>; rel=next,')
        for value in bad:
            with self.subTest(value=value): self.assert_invalid((("Link", value),))
    def test_every_uri_reference_is_validated_including_non_next(self) -> None:
        helper = self.helper()
        relative = "/repositories/7/actions/a,b;c?page=1&per_page=100"
        self.assertEqual(helper.parse_header_fields((("Link", f'<{relative}>; rel=prev'),
                                                     )).links[0].target, relative)
        bad_targets = ("", "https://example.invalid/a b", "https://example.invalid/%GG",
                       "https://[bad]/x", "https://example.invalid/a\\b",
                       "https://example.invalid/é")
        for target in bad_targets:
            with self.subTest(target=target): self.assert_invalid(
                (("Link", f'<{target}>; rel=prev'),))
        mixed = f'<{NEXT}>; rel=next, <https://example.invalid/%GG>; rel=prev'
        self.assert_invalid((("Link", mixed),))
if __name__ == "__main__": unittest.main()
