# Reproduce the source-binding journey

The website checks the filesystem example with its committed WASM package.
`record.json` is a separate recording of seven actual CLI commands, built from
current source at `aba11b329e3284845ba7c9cd4c870998503a9030` on 5 October 2026.
That source implements seal/v2; the published v0.8.2 release implements seal/v1.

Download `replay.py`, `before.garnet`, `accepted.garnet`, and `changed.garnet`
into one directory. Build the CLI from the pinned source, then run:

```
python3 replay.py /absolute/path/to/garnet
```

The script checks the missing declaration, checks the repaired declaration,
records the +fs capability diff, emits a seal twice and compares its bytes,
verifies the original source, and refuses a changed source. An authority
expansion makes diff-caps exit 1 by design; it is a review signal. The script
asserts every expected exit code and does not run the filesystem program.

These seals are unsigned. Verification checks source and declared-capability
content binding, not a signature, identity, factual truth, complete safety, or
a human acceptance attestation. Browser Accept/Reject is a local demonstration
of a review choice and is not included in the CLI seal.
