# Garnet research corpus map

This directory is the canonical root map for Garnet research. It does not
silently rename or relocate the legacy A–F corpus: those paths remain live until
a dedicated, link-checked migration proves every inbound reference. Only the
repo-held June 2026 reassessment is canonicalized here in this slice.

## A–F lineage

| Lineage | Canonical live location | Current treatment |
|---|---|---|
| A | [`A_Research_Papers/`](../A_Research_Papers/) | Research papers remain at their tracked legacy paths. The existing README owns reading order. |
| B | [`B_Four_Model_Consensus/`](../B_Four_Model_Consensus/) | Consensus and adjudication documents remain live in place. |
| C | [`C_Language_Specification/`](../C_Language_Specification/) | Normative language and trust contracts remain live in place and are not moved by corpus cleanup. |
| D | [`D_Executive_and_Presentation/`](../D_Executive_and_Presentation/) | Presentation artifacts remain live in place; no binary rewrite or rename occurs here. |
| E | Repository-root Rust workspace and application directories | The historical `E_Engineering_Artifacts/` wrapper was flattened. Active crates and apps stay at the repository root; historical references keep their historical meaning. Do not recreate the wrapper or move engineering crates as part of research migration. |
| F | [`F_Project_Management/`](../F_Project_Management/) | Episodic plans, handoffs, evidence records, and launch ledgers remain live in place. |

## Canonicalized June 2026 material

- [`2026-06/GARNET_REASSESSMENT_2026-06-11.md`](2026-06/GARNET_REASSESSMENT_2026-06-11.md)
  is the only June source present in the repository at the Lane 0 base
  `231aefa91985e5a0520c493c7f0fc3e54d74efc8`.
- The former
  [`F_Project_Management/RESEARCH/GARNET_REASSESSMENT_2026-06-11.md`](../F_Project_Management/RESEARCH/GARNET_REASSESSMENT_2026-06-11.md)
  path is an explicit compatibility pointer.
- Builder transcripts and external reports cited by the reassessment are
  citations, not tracked corpus files. Lane 0 imports none of them.

## Standing research contracts

- [`DIRECTIVES_LEDGER.md`](DIRECTIVES_LEDGER.md) reconciles Directives 1–16
  against current repository evidence without treating a plan as implementation.
- [`QUARTERLY_COMPETITIVE_WATCH.md`](QUARTERLY_COMPETITIVE_WATCH.md) activates
  the quarterly watch contract. Its first report remains planned for 2026 Q3,
  due 2026-09-30.

## Migration rule

Future moves require a before/after inbound-reference inventory, a deterministic
link probe, compatibility pointers where historical artifacts cannot be
rewritten, and a separate reviewed change. A map is not proof that migration is
complete.
