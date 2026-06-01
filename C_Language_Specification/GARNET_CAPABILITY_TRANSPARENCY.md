# Garnet capability transparency log (S68)

A program's authority should have an auditable *history*, not just a current
surface. S68 seeds that with `garnet caps-log`: an append-only, BLAKE3-chained
log of capability-manifest entries — Certificate-Transparency / Sigstore-Rekor
in spirit, tamper-evident in mechanism.

## The log + the cross-language standard seed

`garnet caps-log <file.garnet> --log <path>` appends one entry per line (JSONL):

```json
{"index":1,"program":"c_stat","caps":["ffi","fs"],
 "caps_blake3":"5cd166…","prev_blake3":"cb7069…"}
```

- `caps_blake3` — BLAKE3 of the program's deterministic capability manifest (S36).
- `prev_blake3` — BLAKE3 of the **previous log line** (the chain link;
  `"genesis"` for entry 0).

`garnet caps-log --verify <path>` recomputes the chain: each entry's
`prev_blake3` must equal the hash of the prior line. Flipping a single byte in
any earlier entry breaks the chain at the next entry (exit 1) — append-only
tamper-evidence.

The entry schema is deliberately **language-agnostic** — `program`, `caps`,
`caps_blake3`, `prev_blake3`, `index`. Any toolchain that can emit a capability
surface can append to (and verify) the same log shape; this is the seed of a
**cross-language capability-manifest standard** (the GRAFT in the reconciliation).

S98 adds the explicit draft/reference profile in
`GARNET_CAPABILITY_MANIFEST_STANDARD.md` and exposes it through:

```text
garnet caps --standard-profile <file-or-package>
```

That profile is still a seed over the declared capability surface. It is not an
accepted standard and it does not prove absence of undeclared authority.

## Honest scope (do not soften)

This is a **local, hash-chained STUB**, **not** a distributed or witnessed
transparency log. There is **no** public log server, **no** signed tree head,
**no** gossip/witness network, and **no** inclusion proof against an external
root. It gives tamper-evidence for a *local* append-only file — useful for a repo
or a release pipeline — but it is not Rekor and makes no availability or
non-equivocation guarantee. A real transparency-log deployment (a public,
witnessed Merkle log) is out of scope; this seeds the format and the mechanism.
