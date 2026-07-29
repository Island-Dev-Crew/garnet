# Lane 0 Repair 3 Findings Register

This register preserves the exact scope used before a cure. The machine-readable
path enumeration is `U25_SCOPE.json`; prose or a carried count must not replace
that exact-tree record.

| ID | State | Exact-tree scope | Disposition |
|---|---|---|---|
| U-25 | IN PROGRESS | `efd4f6bae8b3afaba74594e57944b2548142aeae` / tree `e9bce10421c1eac2a514291212b87d61a5289037`; ten CR-bearing text paths enumerated in `U25_SCOPE.json` | Six sealed `proofs/**` paths are closed with no byte change. Three CRLF assets require LF normalization and explicit attributes. One generated Lane 2B HTML file requires generator-owned removal of lone CR bytes. |
| U-45 | PROPOSED | This lane's touched findings must name paths at a full commit and tree before implementation. | Replace count-pinned finding scopes with exact-tree path enumerations. New checkers report violating paths and do not assert a repository-wide total. |

U-25's earlier “eleven files” premise was wrong at authorship. It is superseded
by the exact-tree enumeration above; there is no eleven-file target to recover.
