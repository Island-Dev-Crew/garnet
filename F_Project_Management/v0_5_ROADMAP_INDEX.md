# Garnet v0.5 Roadmap Index

Date: 2026-05-07
Status: active planning and implementation control surface

This index is the table of contents for moving Garnet from a credible
research-grade prototype toward the original language/toolchain ambition.

## Canonical Roadmap Files

| File | Purpose |
|---|---|
| `F_Project_Management/GARNET_LANGUAGE_COMPLETION_IMPLEMENTATION_PLAN.md` | Canonical current-vs-deferred implementation plan for making the original Garnet ambition executable. |
| `F_Project_Management/ROADMAPS/GARNET_v0_5_LANGUAGE_COMPLETION_ROADMAP.md` | Implementation plan for the seven real-plan milestones. |
| `F_Project_Management/ROADMAPS/GARNET_v0_5_PHASE_OWNERSHIP_REGISTER.md` | Phase ownership, risk, verification, and handoff ledger. |
| `F_Project_Management/ROADMAPS/GARNET_v0_5_CI_VERIFICATION_MATRIX.md` | Exact CI and local commands required by each phase. |
| `F_Project_Management/DOGFOOD/GARNET_v0_5_DOGFOOD_READINESS_PHASE_LOG.md` | Dogfood readiness gates and current phase status. |
| `C_Language_Specification/GARNET_v0_4_2_Conformance_Suite.md` | Current executable conformance handles. |
| `C_Language_Specification/GARNET_v0_4_2_Conformance_Matrix.md` | Descriptive implementation-vs-spec truth table. |

## Progress Rule

A Mini-Spec row may advance only when the same commit or PR includes:

1. parser/checker/runtime implementation evidence appropriate to the row,
2. an active conformance test or an explicitly ignored handoff test,
3. updated matrix status,
4. an example/template or dogfood probe when the feature is user-facing,
5. local verification output and, before merge, CI verification.
