# U-35 cure RED — traps fail before the exclusion is added

- Recorded: 2026-07-27 (UTC ~23:20Z), at HEAD db6ab65 (post verdict 05, pre-cure)
- Implementer: Claude Code Opus 4.8 — fleet-fork identity
- The three U-35 traps added to scripts/test_garnet_minimum_shelf_provenance.py FAIL
  before b"ops/lane1/" is appended to FROZEN_MUTABLE_PREFIXES:

```
Ran 6 tests in 0.709s
FAILED (failures=3)
```

Cause: ops/lane1/ is product-digest-included, so (d) the exclusion tuple
lacks the Lane 1 prefix, (b) a lane-1-only change moves the digest, and (a)
the crux pair does not hold. The cure (exclusion + rederived pair) turns all
three green in the same reviewed series; recorded here before the cure per the
RED-first discipline.
