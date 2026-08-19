# Evidence 99 — Fleet credential-lifecycle doctrine gap

> Relocation note: This finding was relocated from the gate-topology addendum
> under the drift-class ruling in the U-57 disposition
> (`ops/gate-topology/FINDINGS.md:17-20`): a later non-record content merge
> “never widens the record class or weakens the verifier.” This relocation uses
> the existing `ops/lane1/**` record-class surface; it does not amend that class
> or the verifier.

- Status: OPEN; doctrine gap recorded, not cured here.
- Finding: fleet push-credential expiry is not yet tracked.
- Disposition: credential lifecycles (mint date, expiry, seat) belong in the
  fleet doctrine beside token E's mint-at-point-of-use rule.
