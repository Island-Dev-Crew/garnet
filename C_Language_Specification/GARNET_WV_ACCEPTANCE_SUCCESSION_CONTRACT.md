# Garnet WV acceptance succession contract v1

**Status: normative contract law; not activated.** The four certificate/receipt
schemas and all four conservation predicates in this document are
`OPEN-UNTIL-IMPLEMENTED`. They define the exact shape later L1 acts must trap;
they are not evidence that a producer, registry, verifier, carrier, certificate,
receipt, re-evaluation, or migration already exists.

**Authority:**
`F_Project_Management/W_TRUST/REACCEPTANCE_REDESIGN_BRIEF_v2.md` as adopted and
resolved by
`F_Project_Management/W_TRUST/L1_DECISION_POINTS_RULING_2026-09-01.md`.

## Activation and claim boundary

This act defines text only. R1, R2, and R3 remain ineligible until the later
arc acts land every base-controlled route, producer, append-only registry,
consumer inventory, executable predicate, positive fixture, fail-closed
negative fixture, and gate integration named here. An absent implementation,
an `OPEN-UNTIL-IMPLEMENTED` row, or an unavailable observation is RED. None of
those states upgrades WV-6, a claim class, acceptance, launch state, or release
state.

The two-pair model is exact:

- `native_accepted_pair` is the immutable pair established by the terminal
  native-Windows ceremony and preserved by every record-only succession.
- `successor_observed_pair` is non-authoritative accounting recomputed at the
  successor boundary. It is never native evidence and cannot advance the
  accepted pair.
- Only one exact closed R3 event certificate may establish each later accepted pair,
  and only after its terminal effectiveness transcript becomes effective.

## Canonical JSON and exact-shape law

Every repository JSON document defined here is a regular, non-executable
`100644` blob. Every repository document and the artifact-only eligibility
receipt use exactly the UTF-8 bytes produced by the serializer already enforced
for rolling-review records:

```python
(json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n").encode()
```

Therefore object keys are sorted lexicographically, indentation is two spaces,
non-ASCII characters remain literal UTF-8, line endings are LF, and the file
has exactly one trailing LF. Duplicate, unknown, aliased, or missing keys;
invalid UTF-8; alternate escaping; CRLF; extra members; unsafe paths; symlink,
executable, or gitlink modes; and extra trailing bytes are RED.

The following scalar rules apply recursively:

- `sha256` is a lowercase 64-hex string.
- `commit` and `tree` are full lowercase Git object IDs resolved with
  replacement refs disabled; abbreviations are RED.
- Every field whose JSON type is `integer` is a JSON integer, never a boolean;
  it is positive or nonnegative exactly as its owning row states.
- `repository_id`, PR IDs, review IDs, run IDs, workflow IDs, artifact IDs, and
  user IDs are positive JSON integers, never numeric strings or booleans.
- `git_path` is a nonempty slash-delimited repository-relative path with no
  backslash, empty component, `.` component, `..` component, or absolute form.
- `timestamp` is an RFC 3339 UTC string with a terminal `Z`.
- Set-valued arrays are duplicate-free and sorted by the field named for their
  shape. Order-sensitive transport arrays follow authenticated API page order.
- A field described as `null-or-X` is present and is either JSON `null` or the
  stated shape. Omitting it is RED.

Self-digest fields use one framing rule. Remove only the named self-digest key,
serialize the remaining object with the canonical expression above, and hash
`<domain-as-ASCII> NUL <canonical-bytes>`. `edge_census.sha256` uses domain
`garnet.wv_acceptance.edge_census/v1`; `graph_projection.sha256` uses domain
`garnet.wv_acceptance.graph_projection/v1`;
`historical_edge_census.sha256` uses domain
`garnet.wv_acceptance.historical_edge_census/v1`; and
`historical_native_impact_graph.sha256` uses domain
`garnet.wv_acceptance.historical_native_impact_graph/v1`. An ordinary
`edge_ref.entries_sha256` hashes
`garnet.wv_acceptance.edge_entries/v1 NUL <canonical-associated-entries-array>`.
An `edge_ref` beneath `historical_edge_census` instead hashes
`garnet.wv_acceptance.historical_edge_entries/v1 NUL
<canonical-associated-historical-entries-array>`.
`native_h_input_sha256`, `source_h_input_sha256`, and
`successor_b_input_sha256` respectively hash
`garnet.wv_acceptance.pair_input/native-h/v1 NUL <commit-as-ASCII> NUL
<canonical-blob-binding-array>` and
`garnet.wv_acceptance.pair_input/source-h/v1 NUL <commit-as-ASCII> NUL
<canonical-blob-binding-array>` and
`garnet.wv_acceptance.pair_input/successor-b/v1 NUL <commit-as-ASCII> NUL
<canonical-blob-binding-array>`. `event_c_input_sha256` uses the same framing
with domain `garnet.wv_acceptance.pair_input/event-c/v1`. No other field is
masked. `migration_base_input_sha256` hashes
`garnet.wv_acceptance.pair_input/migration-base-s/v1 NUL
<migration_base_s-as-ASCII> NUL <canonical-migration_base_inventory>`.

`native_root_id` is exactly `sha256:<hex>`, where `<hex>` is the SHA-256 of the
canonical `native_root` bytes. Let `<tiphex>` be the lowercase SHA-256 of the
canonical `predecessor_effective_tip` bytes. A succession `certificate_id` is
exactly `succession:<wv>:<tiphex>:<source_b>`. An event `certificate_id` is
exactly
`event:<wv>:<tiphex>:<event_class>:<certificate_content_head>`. Colons occur
only in these in-object IDs, never in path components.

Mechanical test `wv_schema_canonical_v1` parses with duplicate-key rejection,
requires exact recursive key sets and JSON types, reserializes with the byte
expression above, compares bytes, verifies every scalar rule, and exercises
missing/unknown/duplicate-key, CRLF, escaping, mode, path, and trailing-byte
negatives. Status: `OPEN-UNTIL-IMPLEMENTED`.

## Named embedded shapes

The main schemas may use only the embedded shapes below. Each shape has exactly
the listed keys.

### `pair`

| key | JSON type | invariant |
|---|---|---|
| `path_count` | integer | nonnegative |
| `sha256` | string | `sha256` |

### `blob_binding`

| key | JSON type | invariant |
|---|---|---|
| `git_oid` | string | full blob object ID |
| `mode` | string | exactly `100644` unless the owning field explicitly binds observed history |
| `path` | string | `git_path` |
| `sha256` | string | SHA-256 of the raw blob bytes |

Unless its owning field explicitly states a historical ordering, an array of
`blob_binding` is sorted by `path` and contains each path once.

### `versioned_blob_binding`

`versioned_blob_binding` has the same exact four keys and scalar invariants as
`blob_binding`. Its arrays are sorted by `(path, mode, git_oid)`, contain each
such triple once, and may contain multiple historical object IDs or modes for
one path. A same-blob mode transition therefore preserves both bindings.

### `effective_tip_ref`

| key | JSON type | invariant |
|---|---|---|
| `certificate_blob_sha256` | null-or-string | null only when `certificate_kind=native`; otherwise `sha256` |
| `certificate_kind` | string | exactly `native`, `succession`, or `event` |
| `certificate_path` | null-or-string | null only when `certificate_kind=native`; otherwise `git_path` |
| `effectiveness_blob_sha256` | null-or-string | null only when `certificate_kind=native`; otherwise `sha256` |
| `effectiveness_path` | null-or-string | null only when `certificate_kind=native`; otherwise `git_path` |
| `native_root_id` | string | stable nonempty root identifier shared by the chain |

### Activated law-base selector

Acceptance predecessor and law predecessor are distinct facts. Every succession
and event certificate binds `law_base_commit` and `law_base_tree`; all phrases
in this document such as "predecessor registry", "predecessor inventory",
"predecessor classifier", "predecessor producer/verifier", or "predecessor
transport/class law" mean the exact bytes at this law base.

The selector is closed:

- when `predecessor_effective_tip.certificate_kind=native`,
  `law_base_commit=A`, the unique verified gate-activation boundary defined by
  the genesis law below;
- when the predecessor kind is `succession` or `event`, `law_base_commit=T`,
  the derived authoritative-main introduction commit of that predecessor's
  unique effective transcript.

`law_base_tree=tree(law_base_commit)`. The commit exists on authoritative
upstream main first-parent history with replacement refs disabled. Every
registry, implementation inventory, source closure, matcher, classifier,
digestor, projector, producer, and verifier used for the certificate is loaded
from that exact tree. The authenticated certificate-PR base equals or descends
from `law_base_commit`; the complete intervening census is qualified, and every
law-bearing blob remains object-identical. Candidate bytes, the acceptance
tip's historic native `H`, a current-main shortcut, a merge base, or a later
unbound law revision cannot replace the selected law base. A law-base/tree
mismatch, missing activation, changed intervening law byte, or selector fork is
RED.

Mechanical test `wv_law_base_selector_v1` derives `G`, `A`, and every later
`T`, checks the exact kind-discriminated selection and authoritative-main/tree
bindings, loads every governed byte from that tree, and rejects native-`H`
lookup, candidate/current-tip substitution, ambiguous activation, stale law,
changed intervening law, missing inventory, and tree or commit mismatch.
Status: `OPEN-UNTIL-IMPLEMENTED`.

### `selected_review`

| key | JSON type | invariant |
|---|---|---|
| `commit_id` | string | exact approved head named by the owning field; `Q` in an effectiveness transcript |
| `decisive_event_order` | integer | one-based position among this reviewer's decisive rows sorted by immutable `review_id` |
| `implementation_families` | array of strings | sorted distinct families of every authenticated PR commit author/committer principal |
| `review_id` | integer | positive immutable GitHub review ID |
| `reviewer_id` | integer | positive immutable GitHub user ID |
| `reviewer_family` | string | exact family slug from the bound reviewer-family registry |
| `reviewer_login` | string | current login from the direct user/review objects |
| `state` | string | exactly `APPROVED` |
| `submitted_at` | string | `timestamp` |
| `supplemental_reviews` | array of `supplemental_review` | exact additional decisive reviews required by the owning certificate |

The verifier concatenates completely paginated review rows, rejects duplicate
IDs, filters one reviewer's rows to `APPROVED`, `CHANGES_REQUESTED`, or
`DISMISSED`, sorts those decisive rows by positive immutable `review_id`, and
assigns one-based `decisive_event_order`. The selected row is the final member
of that sequence, is read again through its direct review-object endpoint, and
must project identically. API delivery order and timestamps never break ties or
override the immutable-ID order.

### `supplemental_review`

| key | JSON type | invariant |
|---|---|---|
| `commit_id` | string | same exact approved head as the primary selected review |
| `decisive_event_order` | integer | derived by the same per-reviewer immutable-ID rule |
| `review_id` | integer | positive immutable GitHub review ID distinct from every other selected ID |
| `reviewer_family` | string | exact family slug from the bound reviewer-family registry |
| `reviewer_id` | integer | positive immutable GitHub user ID distinct from every other selected reviewer |
| `reviewer_login` | string | current login from the direct user/review objects |
| `state` | string | exactly `APPROVED` |
| `submitted_at` | string | `timestamp` |

Supplemental reviews are sorted by `(reviewer_family, reviewer_id, review_id)`.
Every primary and supplemental row is the final decisive row for that reviewer,
is read directly, and projects identically to the paginated row.

### Designated-review equality

The primary selected review never substitutes an arbitrary cross-family
approval for the reviewer designated by the applicable canonical structured
record. For a native root, the canonical bytes at
`native_root.structured_review` parse under their registered schema; their
`reviewer_id`, `reviewer_login`, and `review_state=APPROVED` equal
`native_selected_review`, whose `commit_id` equals the native root's exact
approved head. For a succession source, the complete `h_to_r_census` contains
exactly one terminal newly added canonical rolling review record; its designated
reviewer identity/state equal `source_selected_review`, whose `commit_id=R`.
For an ordinary succession or event certificate, the complete `B..Q` or
`C..Q` census contains exactly one terminal newly added canonical rolling
review record. For the DP3 migration, the fact-only bridge binds every review
record in `B..S` and the qualified `S..Q` census contains exactly the terminal
certificate review record. In each case the record's
designated reviewer identity/state equal the effectiveness transcript's primary
`selected_review`, whose `commit_id=Q`.

Each structured record separately passes the predecessor rolling law, including
its own non-self-referential `reviewed_head`, tree, touched set, digest, author
identities, and exact schema. The selected review is reselected from complete
pagination and read directly. An absent/extra record, a different designated
reviewer, login or state disagreement, an approval by another eligible family,
or a selected commit other than the exact owning tip is RED. A bounded-
weakening supplemental review remains the additional distinct review required
by class law; it never replaces the record-designated primary.

### Reviewer-family registry

The predecessor registry path is exactly
`F_Project_Management/W_TRUST/REVIEW_FAMILY_IDENTITIES.json`; its schema ID is
`garnet.review_family_identities/v1`. Its canonical object has exactly
`principals` and `schema`; `schema` equals that ID. Each `principals` member has
exactly `family` and `user_id`: `user_id` is a positive immutable GitHub user
ID, and `family` matches `[a-z][a-z0-9-]{0,63}`. The array is sorted by
`user_id`, contains each ID once, and maps an ID to exactly one family. Every
certificate's `reviewer_family_registry_sha256` equals the SHA-256 of the raw
canonical predecessor-registry bytes. Authenticated current logins are checked
separately and never define family.

For every `selected_review`, `implementation_families` is nonempty and is
derived by mapping the union of every authenticated PR commit `author.id` and
`committer.id` through that registry; an absent principal or mapping is RED.
The primary `reviewer_family` is absent from `implementation_families`. Unless
the bound event direction is `BOUNDED WEAKENING`, `supplemental_reviews` is
exactly `[]`. For `BOUNDED WEAKENING`, it contains exactly one review whose
family differs from the primary family and every implementation family. Thus
the ordinary cross-family review and the additional weakening review are
machine-distinct.

### `weakening_authority`

| key | JSON type | invariant |
|---|---|---|
| `authority_seat` | string | exactly `jon-merge-authority` |
| `decision` | string | exactly `APPROVED` |
| `expiry_predicate_id` | string | exact nonempty predecessor class-rule predicate ID |
| `ruling` | `blob_binding` | immutable predecessor-main Jon ruling record |
| `ruling_id` | string | exact nonempty ID registered by predecessor class law |
| `scope_sha256` | string | exact bounded-event scope digest defined below |

`scope_sha256` hashes
`garnet.wv_acceptance.weakening_scope/v1 NUL <canonical-object>`, where the
object has exactly `allowed_source_delta`, `direction`, `event_action`,
`event_class`, and `wv`, copied recursively from the event certificate. The
entire `weakening_authority` object equals the predecessor class registry's
machine-consumed authority entry; the ruling blob exists at that predecessor
tip. Its `expiry_predicate_id` equals that selected Class A row's exact
`valid_while_predicate_id`; a second predicate name or verifier operation is
RED. The live obligation law below evaluates that predicate while the
activation remains open.

### `review_scope`

This is the exact structured non-extension object proposed for R1.

| key | JSON type | invariant |
|---|---|---|
| `approval_head_selector` | string | exactly `exact-current-pr-head` |
| `attestation_kind` | string | exactly `mechanical_record_succession` |
| `coverage_extension` | array | exactly `[]` |
| `extends_native_coverage` | boolean | exactly `false` |
| `landing_tree_requirement` | string | exactly `tree(Q)==tree(M)` |
| `native_checks_reexecuted` | array | exactly `[]` |
| `reviewed_through` | string | pre-certificate source record head `source_r=R`; never the certificate's own commit |
| `source_b` | string | authoritative content landing `B` |
| `source_h` | string | current succession-segment anchor: native `H` or predecessor transcript `T` |
| `source_r` | string | final source record tip `R` |

### `census_entry`

| key | JSON type | invariant |
|---|---|---|
| `commit` | string | child `commit` |
| `new_blob_sha256` | null-or-string | null only for deletion; otherwise raw new blob SHA-256 |
| `new_git_oid` | string | full new object ID or all-zero object ID for deletion |
| `new_mode` | string | Git raw-diff mode |
| `old_blob_sha256` | null-or-string | null only for addition; otherwise raw old blob SHA-256 |
| `old_git_oid` | string | full old object ID or all-zero object ID for addition |
| `old_mode` | string | Git raw-diff mode |
| `operation_subtype` | string | predecessor-inventoried producer operation |
| `parent` | string | exact parent `commit` for this edge |
| `path` | string | `git_path` |
| `producer_id` | string | predecessor inventory producer `logical_id` |
| `status` | string | exact no-rename raw status |
| `walk_index` | integer | nonnegative index of the owning `edge_ref` |

### `edge_ref`

| key | JSON type | invariant |
|---|---|---|
| `commit` | string | child `commit` |
| `entries_sha256` | string | framed digest of this edge's canonical `census_entry` subarray |
| `entry_count` | integer | nonnegative number of entries for this edge |
| `parent` | string | exact parent `commit` |
| `tree` | string | resolved child `tree` |
| `walk_index` | integer | zero-based index in the canonical edge array |

### `edge_census`

| key | JSON type | invariant |
|---|---|---|
| `closure_open` | array | exactly `[]` for an eligible object |
| `edge_count` | integer | nonnegative and independently derived |
| `edges` | array of `edge_ref` | complete graph edges sorted by `(commit, parent)` with contiguous `walk_index` |
| `end_commit` | string | exact terminal `commit` |
| `entries` | array of `census_entry` | complete raw-diff facts, sorted by `walk_index`, then `path` |
| `entry_count` | integer | equals the array length |
| `sha256` | string | self-digest of the complete census object under the stated framing |
| `start_commit` | string | exact initial `commit` |

`edge_count=len(edges)` and `entry_count=len(entries)`. Each reachable
`(parent, commit)` edge appears once even when its raw diff is empty; its
`entry_count` and `entries_sha256` bind exactly the entries carrying its
`walk_index`. Every entry names one listed edge and agrees on parent/commit.
`edge_count=0` is permitted only when `start_commit=end_commit`; otherwise at
least one edge exists. Empty-diff edges therefore remain visible rather than
being inferred from path rows.

### `historical_edge_census`

The one-time DP3 bridge uses a fact-only historical census rather than
qualifying activation-arc changes as record-producer operations. Its top-level
shape is exactly the `edge_census` key set and count/edge/walk law above, but
its `entries` array contains `historical_census_entry` objects and uses the
historical digest domains stated above. A `historical_census_entry` has exactly
`commit`, `new_blob_sha256`, `new_git_oid`, `new_mode`, `old_blob_sha256`,
`old_git_oid`, `old_mode`, `operation`, `parent`, `path`, `status`, and
`walk_index`. Every shared field has the exact `census_entry` invariant.
`operation` is derived only from the no-rename raw status and modes and is
exactly `add`, `delete`, `modify`, or `type-change`; it never classifies the
change as a record or invokes a producer.

Both bridge-history enumerators walk every authoritative-main first-parent
edge, including empty edges, with replacement refs disabled and independently
derive the same canonical census bytes. A rename/copy status, producer label,
path-only projection substituted for the per-edge object facts, omitted
transient version, absent object, or enumerator disagreement is RED.

### `graph_projection`

This is the squash-durable, self-contained projection of the source graph.
It has exactly these keys:

| key | JSON type | invariant |
|---|---|---|
| `commits` | array | complete graph commits, sorted by `commit` |
| `endpoint_b_inventory` | array of `blob_binding` | complete ordered `(path, blob OID, mode, SHA-256)` stream at `B` |
| `endpoint_h_inventory` | array of `blob_binding` | complete ordered stream at `H` |
| `endpoint_source_inventory` | array of `blob_binding` | complete ordered stream at current `source_h` segment anchor |
| `relevant_blobs` | array of `versioned_blob_binding` | exact non-endpoint support/version union defined below |
| `sha256` | string | self-digest under `garnet.wv_acceptance.graph_projection/v1` |

Each `commits` member has exactly `commit`, `parents`, and `tree`. `commit` and
`tree` are full object IDs; `parents` is the raw commit object's ordered,
duplicate-free array of full parent IDs. Every `census_entry` edge occurs in
this graph, and every graph edge in `source_h..R` occurs in the census. The
native `H` commit is also present even when it is not ancestral to `source_h`;
its endpoint inventory preserves the independent native-pair recomputation.
Endpoint arrays are path-sorted, complete tree inventories rather than changed-
path lists.

Let `E` be the set of `(path, mode, git_oid)` triples in the three endpoint inventories.
Let `U` be the versioned union of: every non-null old and new census side; every
member of the exact chain-preservation set; every source-closure binding for the
selected classifier, digestor, producer, pair enumerators/digestors, and
verifiers; and the predecessor-tree bindings for the succession, effectiveness,
implementation, impact-criterion, record-consumer, producer-graph,
reviewer-family, and transport-projection registry files. `relevant_blobs` is
exactly `U - E`, rendered as
complete `versioned_blob_binding` rows sorted by `(path, mode, git_oid)`. Converting a
census side copies its path, object ID, mode, and raw SHA-256 without inference.
No other selection word such as “relevant” grants discretion: an omitted,
extra, wrong-version, endpoint-duplicated, or candidate-selected member is RED.

### Implementation inventory and source-closure hash

The predecessor implementation inventory path is exactly
`F_Project_Management/W_TRUST/WV_ACCEPTANCE_IMPLEMENTATIONS.json`; its schema
ID is `garnet.wv_acceptance_implementation_inventory/v1`. Its canonical object
has exactly `implementations`, `independence_allowlist`, and `schema`; `schema`
equals that ID.
`implementations` is sorted by `logical_id`, contains each logical ID,
content-derived implementation ID, and `entrypoint` once, and each member has
exactly these keys:

| key | JSON type | invariant |
|---|---|---|
| `dependency_edges` | array | complete source dependency edges sorted by `(from_path, to_path, kind)` |
| `entrypoint` | string | implementation entrypoint `git_path` |
| `entrypoint_blob_sha256` | string | raw SHA-256 of the entrypoint blob |
| `implementation_id` | string | deterministic `implementation:<hex>` ID defined below |
| `logical_id` | string | stable unique role-specific producer/consumer identity |
| `operations` | array | exact operation rows sorted by `operation_id` |
| `roles` | array of strings | nonempty sorted unique closed roles named below |
| `source_closure` | array of `blob_binding` | complete path-sorted fixed-point source closure |
| `source_closure_sha256` | string | framed digest of the complete `source_closure` array |

`independence_allowlist` is sorted by `binding.path`; each member has exactly
`binding` and `kind`, where `binding` is a unique `blob_binding` and `kind` is
exactly `inert-build-config`. The bound path must be reached in every using
closure only by a `build-config` edge and must parse as predecessor-recognized
declarative build metadata with no executable body, import resolver, plugin,
command, generated code, or runtime selector. Repository source modules,
parsers, digestors, enumerators, templates, generated helpers, and scripts can
never be allowlisted. External language/runtime standard-library primitives do
not name repository blobs and therefore are outside the closure intersection.

Each implementation operation has exactly these keys:

| key | JSON type | invariant |
|---|---|---|
| `class_rule_id` | null-or-string | non-null exact predecessor event class-rule ID only for a class-scoped operation |
| `direction_axis` | null-or-string | null for a non-class operation; otherwise exact `object_policy`, `source_effect`, or `toolchain_effect` |
| `direction_value` | null-or-string | null exactly when `direction_axis` is null; otherwise the selected class row's non-null value on that axis |
| `fixture_matchers` | array | exact positive and fail-closed fixture rows sorted by `(expected, matcher.kind, matcher.value)` |
| `input_matchers` | array | sorted unique exact operation matchers |
| `operation_id` | string | globally unique stable operation ID |
| `operation_kind` | string | exactly `classify`, `digest`, `enumerate`, `load`, `observe`, `parse`, `produce`, `project`, `resolve`, or `verify` |
| `output_matchers` | array | sorted unique exact operation matchers |
| `parameter_schema` | null-or-`blob_binding` | exact governed parameter schema; null only when the operation has no parameters |
| `predicate_id` | string | exact nonempty machine-predicate ID enforced for the operation |
| `verifier_logical_id` | string | exact predecessor verifier implementation `logical_id` |

An operation matcher has exactly `kind` and `value`. `kind` is exactly
`exact-path`, `prefix`, `schema`, or `suffix`. The value follows the record-
consumer matcher's exact path/prefix/suffix grammar below; for `schema` it is a
nonempty canonical schema ID. Matcher arrays are sorted by `(kind, value)`.
An `exact-path` matcher matches only byte-equal paths; `prefix` and `suffix`
match only their literal path text; and `schema` matches only a canonically
parsed object's exact top-level `schema` value. A concrete input or output
matches exactly one applicable matcher. Zero matches, overlapping multiple
matches, case folding, glob expansion, or a schema assertion without parsing
the exact blob is RED.

Each `fixture_matchers` member has exactly `expected` and `matcher`.
`expected` is exactly `pass` or `fail`, and `matcher` has the exact operation-
matcher shape. Every `verify` operation for a public mechanical predicate in
the later-act inventory has at least one member of each expected value and
exhausts every repository fixture that invokes that predicate; every other
operation has `fixture_matchers=[]`. A generated in-memory case may supplement
but cannot replace the registered positive and fail-closed repository fixtures.

Every `operation_id` and `(verifier_logical_id, predicate_id)` pair is globally
unique. `verifier_logical_id` selects a row whose `roles` contains `verifier`;
that row has exactly one `verify` operation for `predicate_id`. Every public
mechanical predicate in the later-act inventory has exactly one such verify
operation, owned by a row whose `roles` also contains `record-consumer` and
whose exact consumer-inventory matcher covers the governed bytes it reads.

For an operation with `class_rule_id=null`, both direction fields are null. For
a class-scoped operation, both are non-null and equal one exact non-null axis
and value in that class row. An operation affecting two axes is represented by
two otherwise identical operation rows with distinct globally unique
`operation_id` values, one per axis; combining or inferring axes is RED.

An implementation role admits only corresponding operation kinds:
`classifier` admits `classify`; `digestor` and `pair-digestor` admit `digest`;
the six inventory/graph enumerator roles and `pair-enumerator` admit `enumerate`;
`record-consumer` admits `load` or `parse`; `record-producer`,
`succession-producer`, `event-class-producer`, and `effectiveness-producer`
admit `produce`; `transport-projector` admits `observe` or `project`; and
`impact-resolver` admits `resolve`; and `verifier` admits `verify`. A multi-role implementation may contain the union
for its declared roles and no other kind. Every declared role has at least one
corresponding operation.

Each `dependency_edges` member has exactly `from_path`, `kind`, and `to_path`.
Both paths are `git_path`; `kind` is exactly `build-config`, `import`,
`include`, or `template`. The tuple is unique. `source_closure` is the least
fixed point beginning with `entrypoint` and following every executable-source,
build-configuration, include, import, and template edge resolved by
predecessor-owned language/configuration resolvers. It includes both endpoints
of every reachable edge and no unreachable path. The entrypoint appears
exactly once and its member SHA-256 equals `entrypoint_blob_sha256`.

Governed runtime inputs are deliberately outside this hash domain. A registry,
certificate, receipt, evidence blob, API response, command output, or inventory
consumed as data is bound by its exact schema/path/blob field, the record-
consumer inventory, the transport projection registry, and the predecessor
commit that owns it; it is never a `dependency_edge` or `source_closure`
member. In particular neither `WV_ACCEPTANCE_IMPLEMENTATIONS.json` nor
`WV_ACCEPTANCE_RECORD_CONSUMERS.json` may enter its own or the other's source-
closure preimage. This two-layer rule prevents a registry byte string from
having to contain its own SHA-256. Runtime data exclusion does not hide
executable behavior: a source import, executable include, code generator,
parser implementation, matcher implementation, or configuration that changes
resolution remains in the source closure, while the data selected by that
code is bound at the observation boundary.

An unresolved static source edge, dynamic source-code path construction,
missing source blob, symlink, gitlink, executable mode, cycle with an omitted
member, reachable omitted edge, inventory/self-registry edge, or candidate-
owned resolver is RED; a source cycle itself is valid when its complete
strongly connected component is present.

The closed role strings are `bridge-review-enumerator`, `classifier`,
`digestor`, `effectiveness-producer`, `event-class-producer`,
`historical-impact-enumerator`, `impact-resolver`,
`implementation-inventory-enumerator`,
`pair-digestor`, `pair-enumerator`, `producer-graph-enumerator`, `record-consumer`,
`record-consumer-inventory-enumerator`, `record-producer`,
`registry-genesis-enumerator`, `succession-producer`, `transport-projector`,
and `verifier`. No alias or additional role is valid.
`logical_id` matches `garnet\.[a-z0-9._-]+/v[1-9][0-9]*`; it is the stable
identity used by every identity-bearing contract field. It never supplies a
content hash. `implementation_id` is the content-derived identity of this exact
inventory row and is not an alias for `logical_id`.
`source_closure_sha256` hashes
`garnet.wv_acceptance.source_closure/v1 NUL
<canonical-source_closure-array>`. To derive `implementation_id`, remove only
that key from the entry and hash
`garnet.wv_acceptance.implementation/v1 NUL
<canonical-entry-without-implementation_id>`; the ID is
`implementation:<lowercase-hex>`.

Every producer, enumerator, digestor, projector, or other logical identity and
source-closure hash in this contract is resolved through exactly one
predecessor-inventory row by `logical_id`. A field explicitly
described as producer raw bytes, including certificate/event
`producer_sha256`, `class_producer_sha256`, and effectiveness
`producer_sha256`, equals that row's `entrypoint_blob_sha256`. A field described
as a source-closure, producer-closure, consumer-closure, digestor, or enumerator
hash equals that row's `source_closure_sha256`. No identity or hash can be
supplied without the corresponding exact row. Role binding is exact:

| contract use | required inventory role membership and cardinality |
|---|---|
| each succession `producer_identity` / `producer_sha256` | the selected row contains `succession-producer` |
| each event `class_producer_identity` / `class_producer_sha256` | the selected row contains `event-class-producer` |
| each effectiveness `producer_identity` / `producer_sha256` | the selected row contains `effectiveness-producer` |
| each pair `digestor_identity` / `digestor_sha256` | the selected row contains `pair-digestor`; the owning recomputation array selects exactly two distinct rows |
| each pair `enumerator_identity` / `enumerator_sha256` | the selected row contains `pair-enumerator`; the owning recomputation array selects exactly two distinct rows |
| each `census_entry.producer_id` / `operation_subtype` | `producer_id` equals a row's `logical_id`, its `roles` contains `record-producer`, and `operation_subtype` selects that row's exact `produce` operation |
| each `producer_edge.producer_id` / `producer_sha256` and consumer-inventory producer binding | `producer_id` equals a selected row's `logical_id`, its `roles` contains `record-producer`, and `operation` selects that row's exact operation |
| each consumer-inventory `consumer_path` / `consumer_blob_sha256` | exactly one row whose `roles` contains `record-consumer` has that entrypoint and closure hash |
| implementation-inventory agreement | exactly two distinct rows contain `implementation-inventory-enumerator` |
| predecessor producer-graph agreement | exactly two distinct rows contain `producer-graph-enumerator` |
| record-consumer-inventory agreement | exactly two distinct rows contain `record-consumer-inventory-enumerator` |
| activation-base genesis census | exactly two distinct rows contain `registry-genesis-enumerator` |
| DP3 historical bridge census and native-impact graph | exactly two distinct rows contain `historical-impact-enumerator`; each has one `enumerate` operation whose `predicate_id=wv_dp3_genesis_migration_v1`, and the bridge selects both rows exactly once |
| DP3 rolling-review coverage | exactly two distinct rows contain `bridge-review-enumerator`; each has one `enumerate` operation whose `predicate_id=wv_dp3_genesis_migration_v1`, and the bridge selects both rows exactly once |
| each historical-impact relation resolver | the unique impact-criterion registry row selects one `impact-resolver` implementation and one exact `resolve` operation for that relation |
| each projection-registry `projector_identity` / `projector_sha256` | the selected row contains `transport-projector` |
| every `classifier_sha256` field | exactly one row contains `classifier` and supplies its source-closure hash |
| every `digest_law_sha256` field | exactly one row contains `digestor` and supplies its source-closure hash |

An implementation selected in one required independence set cannot be reused
as the other member or substituted by a row lacking the required role.
For every required pair, the intersection of the two `source_closure` path/blob
sets is exactly a subset of `independence_allowlist`; each shared member is
byte-equal to its binding and reached only as inert build configuration. Any
shared repository executable/parser/enumerator/digest helper, common generated
logic, thin wrapper over a third implementation, output consumption, or
allowlist mismatch is RED.

Pair recomputation independence is pipeline-wide. For one recomputation member,
its `pipeline_logical_ids` are the set containing its enumerator and digestor
identities and its `pipeline_source_closure` is the union of both registered
source closures. Between the two members, the logical-ID sets are disjoint and
the closure intersection is exactly a subset of `independence_allowlist` under
the inert-build-config rule above. Thus an enumerator or digestor cannot reappear
in the other pipeline under the other role, and neither pipeline may import,
invoke, wrap, or consume the other or its output. Cross-role reuse/swap, a
shared executable helper, a third common implementation, or output consumption
is RED.

For a `census_entry`, the concrete `path` matches the selected operation's
input matcher when the old side exists and its output matcher when the new side
exists; an addition has only the output obligation and a deletion only the
input obligation. For a `producer_edge`, `from_path` and `to_path` respectively
match exactly one selected-operation input and output matcher. The operation's
`class_rule_id` is null for R1 record succession and equals the one selected
event-class row for an R3 class-scoped edge. These equalities are recomputed
from the predecessor inventory; labels carried by a certificate are never
authority.

Two separately encoded predecessor-inventoried closure enumerators have
distinct entrypoints and source-closure hashes; neither imports, invokes,
wraps, or consumes the other or its output. They independently derive the
complete inventory bytes from the full predecessor tree and agree byte for
byte. Each enumerator's own closure is derived by the other enumerator. The
mechanical test `wv_implementation_source_closure_v1` enforces the exact
schema, framing, fixed-point and two-enumerator agreement. It rejects a
missing/extra entry, edge, operation, predicate binding, matcher, or class-
direction binding; role drift,
raw/closure-hash substitution, wrong-role identity, role cardinality error,
invented operation ID, implementation dependency,
unresolved/dynamic source dependency, candidate resolver, self/cross-registry
preimage, pair-pipeline cross-role reuse/swap, shared-helper/thin-wrapper
independence failure, self-digest mismatch, or byte disagreement. Status:
`OPEN-UNTIL-IMPLEMENTED`.

### `pair_recomputation`

| key | JSON type | invariant |
|---|---|---|
| `digestor_identity` | string | exact pair-digestor `logical_id` |
| `digestor_sha256` | string | framed source-closure `sha256` |
| `enumerator_identity` | string | exact pair-enumerator `logical_id` |
| `enumerator_sha256` | string | framed source-closure `sha256` |
| `native_h_input_count` | integer | equals `endpoint_h_inventory` length |
| `native_h_input_sha256` | string | framed digest of the complete ordered H input stream |
| `native_h_pair` | `pair` | recomputed native pair at `H` |
| `source_h_input_count` | integer | equals `endpoint_source_inventory` length |
| `source_h_input_sha256` | string | framed digest of the complete ordered current-segment input stream |
| `source_h_pair` | `pair` | recomputed raw observed pair at current `source_h` |
| `successor_b_input_count` | integer | equals `endpoint_b_inventory` length |
| `successor_b_input_sha256` | string | framed digest of the complete ordered B input stream |
| `successor_b_pair` | `pair` | recomputed raw observed pair at `B` |

`pair_recomputations` is exactly two members sorted by `enumerator_identity`.
The two enumerator source-closure hashes differ, the two digestor source-closure
hashes differ, their pipeline logical-ID sets are disjoint, and their pipeline
closure intersection satisfies the exact inert allowlist rule. Both members
bind identical native-H, current-source, and B input-stream digests and produce
identical pairs at all three boundaries.

### `event_pair_recomputation`

| key | JSON type | invariant |
|---|---|---|
| `digestor_identity` | string | exact pair-digestor `logical_id` |
| `digestor_sha256` | string | framed source-closure `sha256` |
| `enumerator_identity` | string | exact pair-enumerator `logical_id` |
| `enumerator_sha256` | string | framed source-closure `sha256` |
| `event_c` | string | exact event content `commit` `C` |
| `event_c_input_count` | integer | nonnegative complete ordered-input length |
| `event_c_input_sha256` | string | framed digest of the complete ordered `C` input stream |
| `event_c_pair` | `pair` | independently recomputed pair at `C` |

An event `pair_recomputations` array has exactly two members sorted by
`enumerator_identity`. Their enumerator and digestor source-closure hashes are
pairwise different; their pipeline logical-ID sets are disjoint; their pipeline
closure intersection satisfies the exact inert allowlist rule; and both bind
the same `C`, input count, input digest, and pair.

### `producer_edge`

| key | JSON type | invariant |
|---|---|---|
| `direction_axis` | null-or-string | exact selected operation axis; null for non-class operation |
| `direction_value` | null-or-string | exact selected operation value; null for non-class operation |
| `from_path` | string | source/input `git_path` |
| `operation` | string | predecessor-ratified producer operation ID |
| `producer_id` | string | predecessor-ratified record-producer `logical_id` |
| `producer_path` | string | exact producer `git_path` |
| `producer_sha256` | string | producer source-closure `sha256` from the implementation inventory |
| `to_path` | string | produced/consumed `git_path` |

Producer-edge arrays are sorted by
`(from_path, to_path, producer_id, operation, direction_axis)` and contain each
tuple once. Both direction fields equal the selected implementation operation;
no edge or certificate may relabel them.
`producer_closure.graph_sha256` hashes
`garnet.wv_acceptance.producer_graph/v1 NUL <canonical-producer-edge-array>`.

### Producer graph registry

The predecessor producer graph path is exactly
`F_Project_Management/W_TRUST/WV_ACCEPTANCE_PRODUCER_GRAPH.json`; its schema ID
is `garnet.wv_acceptance_producer_graph/v1`. Its canonical object has exactly
`edges` and `schema`; `schema` equals that ID and `edges` is the complete sorted
array of `producer_edge` rows. The file's raw SHA-256 is the producer-graph
registry digest.

Every registry edge resolves through one predecessor implementation row whose
`roles` contains `record-producer`: `producer_id`, `producer_path`, and
`producer_sha256` equal its `logical_id`, entrypoint, and source-closure hash;
`operation` selects exactly one of its `produce` operations; and `from_path`
and `to_path` each match exactly one corresponding input/output matcher. The
edge's direction fields equal that operation's exact values. An
operation with non-null `class_rule_id` appears only in that exact class row's
fixed-point graph; a null class rule cannot authorize a class-scoped movement.

Exactly two independent `producer-graph-enumerator` implementations derive the
complete canonical registry bytes from the predecessor implementation
inventory, operation matchers, and full tree, and agree byte for byte. Neither
imports, invokes, wraps, or consumes the other or its output. A candidate graph,
candidate matcher, hand-written reachable-edge list, missing/extra edge,
multi-match, invented operation, wrong producer/hash, or enumerator disagreement
is RED.

Mechanical test `wv_producer_graph_inventory_v1` validates the fixed path and
schema, canonical bytes, raw registry digest, two-enumerator agreement, exact
implementation/operation/matcher/direction bindings, and fail-closed graph
derivation. It rejects a missing, swapped, collapsed, or invented class
direction on any direct or derived edge. Status: `OPEN-UNTIL-IMPLEMENTED`.

### `page_receipt`

| key | JSON type | invariant |
|---|---|---|
| `endpoint` | string | exact bounded API endpoint including page coordinates and, only for attempt-scoped APIs, the run-attempt coordinate |
| `item_count` | integer | nonnegative |
| `next_url` | null-or-string | authenticated next link, null only on terminal page |
| `normalized_sha256` | string | framed digest of the exact `review-page` normalized projection |
| `page` | integer | positive contiguous page number |
| `raw_body_sha256` | string | `sha256` of raw response body |
| `status` | integer | exactly `200` |
| `terminal` | boolean | true exactly on the final page |

### `transport_receipt`

| key | JSON type | invariant |
|---|---|---|
| `auxiliary_sha256` | null-or-string | null for HTTPS; raw standard-error `sha256` for a process observation |
| `endpoint` | string | exact bounded endpoint |
| `item_count` | integer | nonnegative; exactly `1` for a direct object |
| `next_url` | null-or-string | authenticated next link for a collection page; otherwise null |
| `normalized_sha256` | string | framed digest of the exact registered normalized projection |
| `numeric_object_id` | null-or-integer | positive immutable ID exactly for a numeric direct object |
| `object_sha` | null-or-string | full `commit` exactly for a commit/ref direct object |
| `page` | null-or-integer | positive contiguous page number exactly for a collection page |
| `raw_body_sha256` | string | `sha256` of raw response body |
| `status` | integer | exact success code: `200` for HTTPS, `0` for a registered process observation |
| `string_object_id` | null-or-string | nonempty exact subject ID exactly for a string/content direct object |
| `terminal` | boolean | true for a direct object and exactly on a collection's final page |
| `transport_kind` | string | predecessor-enumerated transport role |

For a numeric direct-object projection, `numeric_object_id` is positive,
`object_sha=null`, `string_object_id=null`, `page=null`, `item_count=1`,
`next_url=null`, and `terminal=true`. For a commit/ref direct-object
projection, `numeric_object_id=null`, `object_sha` is the authenticated commit,
`string_object_id=null`, `page=null`, `item_count=1`, `next_url=null`, and
`terminal=true`. For a string/content direct-object projection, both other
object identities are null, `string_object_id` is the exact predecessor-bound
subject, `page=null`, `item_count=1`, `next_url=null`, and `terminal=true`. For
a collection-page projection, all three object identities are null, `page` is
positive, `item_count` equals the projected array length, and `next_url` and
`terminal` agree with the authenticated pagination link. No other combination
is valid.

Every HTTPS receipt has `auxiliary_sha256=null`. Every process receipt has
`auxiliary_sha256` equal to the SHA-256 of its unmodified standard-error bytes;
an eligible process observation requires that digest to equal the SHA-256 of
the empty byte string.

### Transport projection registry and normalization

The predecessor transport projection registry path is exactly
`F_Project_Management/W_TRUST/WV_ACCEPTANCE_TRANSPORT_PROJECTIONS.json`; its
schema ID is `garnet.wv_acceptance_transport_projections/v1`. Its canonical
object has exactly `projections` and `schema`; `schema` equals that ID.
`projections` is sorted by `transport_kind`, contains each kind once, and each
member has exactly `endpoint_template`, `fields`, `item_identity_key`,
`order`, `origin_kind`, `projector_identity`, `projector_sha256`,
`response_kind`, `subject_kind`, and `transport_kind`.

`projector_identity` equals exactly one predecessor implementation-inventory
row's `logical_id`; that row's `roles` contains `transport-projector`, and
`projector_sha256` equals that row's
`source_closure_sha256`.

`origin_kind` is exactly `github-api`, `https-json`, `https-bytes`, or
`process`. `response_kind` is exactly `direct-object`, `collection-page`, or
`raw-bytes`.
`subject_kind` is exactly `numeric-object`, `commit-object`, `string-object`,
or `collection-page`. `github-api` and `https-json` use `direct-object` with a
numeric, commit, or string subject, or `collection-page` with a collection
subject. `https-bytes` and `process` use only `raw-bytes` with a string subject.
No other origin/response/subject tuple exists. A `github-api` endpoint template
is an ASCII path beginning with `/`; an `https-json` or `https-bytes` template is an absolute ASCII
`https://` URL; and a `process` template begins with `process:` followed by one
percent-encoded predecessor-inventoried `logical_id` and one operation ID
separated by `/`.
Its only substitutions are the literal variables `{owner}`, `{repo}`,
`{pull_number}`, `{review_id}`, `{user_id}`, `{sha}`, `{page}`, `{per_page}`,
`{run_id}`, `{attempt}`, `{artifact_id}`, `{crate}`, `{version}`, and
`{toolchain}`, `{logical_id}`, and `{operation}`. Values are derived from
authenticated bound objects or the exact predecessor class row and percent-
encoded once; `per_page=100`. The recorded
endpoint equals the literal template substitution, including query-key order.
Redirects, alternate origins/endpoints, implicit defaults, unregistered query
keys or variables, an ambient executable, or a candidate operation ID are RED.

Each `fields` member has exactly `element_pointer`, `output_key`,
`source_pointer`, and `value_kind`. `output_key` is a unique nonempty ASCII key;
`source_pointer` is an exact RFC 6901 JSON pointer; and `value_kind` is exactly
`boolean`, `canonical-json`, `commit`, `commit-array`, `integer`,
`null-or-string`, `repository`, `sha256`, `string`, `timestamp`, or `tree`.
`element_pointer` is non-null only for `commit-array` and is then the exact RFC
6901 pointer applied to each source-array member; it is JSON null for every
other kind. Fields are sorted by `output_key`. For `raw-bytes`, `fields=[]` and
`item_identity_key=null`. Otherwise, a direct object has
`item_identity_key=null`, while a collection names its exact projected
immutable identity key; every JSON projection has at least one field, and the
collection identity key is one of its output keys. `order` is exactly
`not-applicable` for a direct object or raw bytes and `authenticated` for a
collection page. A missing pointer, wrong type, duplicate identity, or response
item outside the projection is RED.

`commit-array` requires a source array, extracts one full commit ID from every
member through `element_pointer`, and preserves authenticated array order; the
result is duplicate-free. `canonical-json` recursively retains the complete
selected scalar, array, or object subtree, rejects duplicate object keys, and
canonicalizes without dropping, renaming, sorting, or coercing array members.
Thus ordered commit parents and nested Class B rule values remain representable
and every nested extra, missing, reordered, or differently typed value changes
the normalized digest.

For a JSON origin the normalizer rejects duplicate keys, projects exactly the
registered fields, and emits a canonical object for a direct response or a
canonical array for a collection page. A collection's projected members remain
in authenticated response order; later selection algorithms may sort their
own complete working copy but never rewrite the receipt projection. For
`https-bytes` or `process` raw bytes, the normalized projection is exactly the
unmodified response body or standard-output byte string; the separately bound
class parser interprets it. HTTPS origins require `status=200`; a process
origin requires `status=0`, the empty standard-error digest, and the exact
predecessor-inventoried operation. `normalized_sha256` hashes
`garnet.wv_acceptance.transport/<transport_kind>/v1 NUL
<normalized-projection-bytes>`. `raw_body_sha256` independently hashes the
unmodified response body or standard output. Receipts are sorted by
`(transport_kind, page, numeric_object_id, object_sha, string_object_id)`,
treating null before a value.

The projection registry loaded from the exact `law_base_commit` tree is the sole
normalization authority. For an effectiveness transcript the exact forge-role
subset of `transport_receipts` is: one each of `repository-object`,
`head-repository-object`, `pull-request-object`,
`selected-review-primary-object`, `selected-review-primary-user-object`,
`certificate-tip-commit-object`, `certificate-landing-commit-object`,
`landing-parent-commit-object`, and `authoritative-main-ref-object`; one or more
contiguous complete `pull-request-commits-page` rows; and, exactly when the
certificate direction is `BOUNDED WEAKENING`, one each of
`selected-review-supplemental-object` and
`selected-review-supplemental-user-object`. The two supplemental kinds are
absent otherwise. `review_pages` is one or more contiguous complete pages
normalized by the registered `review-page` projection; its page/end-link law
is the same as a collection `transport_receipt`.

For an event's `external_observation`, the exact transport-kind multiset is
selected by the predecessor class row: `registry_yank` has exactly one
`registry-api-object` plus exactly one `sparse-index-object`;
`forge_serialization_drift` has exactly one
`live-ruleset-object`; and `toolchain_lint_activation` has exactly one
`toolchain-version-object`. No other kind is valid. The registered projection
and predecessor class row fix the endpoints, fields, subject kind, complete
page count, and cardinality; candidate bytes cannot add a role or relax a
projection.

For a succession effectiveness transcript, the complete top-level
`transport_receipts` array is exactly the forge-role subset above. For an event
effectiveness transcript, the complete array is exactly the multiset union of
that forge-role subset and one freshly obtained class-selected multiset for
every terminal live obligation after applying the pending event's proposed
next fold state. That post-certificate live set is nonempty. The event chain,
certificate path/blob, class row, subject identity, and class transport
requirements independently derive each expected endpoint and direct-object
identity, so every live receipt maps to exactly one obligation without adding a
receipt tag or transcript key. The pending certificate owns each new or
replacement terminal obligation; otherwise the owner is the previously
effective event that remains terminal. A missing, extra, superseded, multiply
mapped, or wrong-subject receipt is RED.

Expected obligations are sorted by `(event_class, event_certificate_path)`.
Within each `transport_kind`, the producer expands one expected receipt
occurrence per obligation in that order and compares the stored multiset with
that complete occurrence list. Two distinct obligations may yield byte-equal
receipt objects only when all their independently derived endpoint, subject,
and projection facts are equal; both occurrences remain required and neither
can satisfy the other's cardinality. Thus a shared external subject does not
collapse two live predicates or make their receipt ownership ambiguous.

The predecessor effectiveness producer obtains the forge subset and every live
class subset in one invocation, with caches disabled, and evaluates each class
predicate from the newly returned normalized projection before emitting
`verdict=pass`. It accepts no carried receipt, body, digest, timestamp, fixture,
or prior-invocation result as transport input. The transcript durably binds the
fresh normalized and raw-body digests; consistent with the brief's explicit
residual capture-time trust, those digests do not claim that a later clone can
reconstruct the response bytes or retrospectively prove the external service's
faithfulness. Ordinary current-acceptance reporting performs a new live
observation under the same class law.

Mechanical test `wv_transport_projection_v1` validates the registry's exact
shape, fixed path, predecessor ownership, endpoint substitution, field/type
projection, framing, raw-body digest, role discriminants, ordering, complete
pagination, and the exact forge/event/live-obligation multisets above. It
rejects an
unknown/missing/duplicate role, field, page, or object; default/latest endpoint;
redirect; partial page; stale payload; projection/type/order drift; raw versus
normalized substitution; candidate authority; or cardinality mismatch. Status:
`OPEN-UNTIL-IMPLEMENTED`.

### `native_root`

| key | JSON type | invariant |
|---|---|---|
| `artifact_membership` | array of `blob_binding` | exact native artifact set |
| `evidence_destination` | string | exact WV evidence-directory `git_path` |
| `manifest` | `blob_binding` | canonical native manifest |
| `native_accepted_pair` | `pair` | native ceremony pair |
| `native_pull_request_id` | integer | positive immutable PR ID |
| `native_pull_request_number` | integer | positive PR number |
| `native_selected_review` | `selected_review` | authenticated decisive native-root approval |
| `platform` | string | exact native platform identifier |
| `proof_mirrors` | array of `blob_binding` | exact proof mirror set |
| `reporter` | `blob_binding` | accepted reporter blob |
| `reporter_projection` | object | exact keys `EXPECTED_PRODUCT_CONTENT_SHA256`, `EXPECTED_PRODUCT_PATH_COUNT`, `REVIEWED_HEAD`, `REVIEWED_TREE` and their accepted values |
| `required_checks` | array | exact predecessor WV required-check IDs and successful bound evidence, sorted by ID |
| `reviewed_head` | string | native `commit` `H` |
| `reviewed_tree` | string | native `tree` |
| `structured_review` | `blob_binding` | accepted review record |
| `wv_contract` | `blob_binding` | exact governing WV contract blob |

Each `required_checks` member has exactly `evidence` (array of `blob_binding`),
`id` (string), and `result` (exactly `pass`).
`reporter_projection.EXPECTED_PRODUCT_CONTENT_SHA256` is a `sha256`,
`EXPECTED_PRODUCT_PATH_COUNT` is a nonnegative integer, `REVIEWED_HEAD` is a
`commit`, and `REVIEWED_TREE` is a `tree`; no fifth key is permitted.

### WV impact-criterion and native-behavior-root registry

The activated predecessor registry path is exactly
`F_Project_Management/W_TRUST/WV_ACCEPTANCE_IMPACT_CRITERIA.json`; its schema ID
is `garnet.wv_acceptance_impact_criteria/v1`. Its top-level object has exactly
`criteria`, `resolvers`, and `schema`; `schema` equals that ID. `criteria` is sorted by
`(native_root_id, wv, criterion_id)`, contains each tuple once, and has exactly
one WV-6 row for each required-check ID under the sole v1 root: the complete
WV-6 native root independently reconstructed at fixed
`reviewed_head=8426ca761c696c3556190be77cce3e340250b5c7` from its canonical manifest,
artifact membership, mirrors, reporter, structured review, contract, and
required-check evidence. No second root or WV is admitted by this v1 registry;
a future native ceremony requires a new explicit contract/activation version.

`resolvers` is sorted by `relation` and contains exactly one row for each
historical-graph relation: `build-config`, `command`, `dispatch`, `generate`,
`govern`, `import`, `include`, `manifest`, `mirror`, `provenance`,
`runtime-input`, `template`, and `test-evidence`. Each row has exactly
`operation_id`, `relation`, `resolver_identity`, and `resolver_sha256`.
`resolver_identity` selects one law-base implementation row whose roles contain
`impact-resolver`; `operation_id` selects its one `resolve` operation for that
relation; and `resolver_sha256` equals its source-closure hash. The operation's
input/output matchers and parameter schema are the sole relation-specific
resolution law. A second resolver, candidate resolver, relation alias, or
unmapped relation is RED.

Each criterion row has exactly `criterion_id`, `input_matchers`,
`native_behavior_roots`, `native_root_id`, `predicate_id`, `verifier_identity`,
`verifier_sha256`, and `wv`. `input_matchers` is a nonempty sorted unique array
of the exact operation-matcher shape. `predicate_id` selects one law-base
implementation `verify` operation whose input matchers are byte-equal to this
array; `verifier_identity` is that operation's owning verifier/record-consumer
row; and `verifier_sha256` is its source-closure hash.

A `native_behavior_root` has exactly `criterion_ids`, `path`, `root_id`, and
`root_kind`. `criterion_ids` is a nonempty sorted unique array of exact WV
criterion IDs; `path` is a `git_path`; and `root_kind` is exactly
`native-build-input`, `native-executable`, `native-runtime-input`, or
`native-test-input`. `root_id=behavior-root:<hex>`, where `<hex>` hashes
`garnet.wv_acceptance.native_behavior_root/v1 NUL <native_root_id-as-ASCII>
NUL <canonical-root-without-root_id>`. A criterion's root array is sorted by
`root_id` and unique. The complete registry root set is the union by `root_id`;
when a root appears in more than one row, its path/kind are byte-equal and its
`criterion_ids` equals the complete sorted set of rows that name it.

`native_root_id` is the exact `sha256:<hex>` ID of the row's complete canonical
native root and selects its unique `reviewed_head=H`; a bridge uses only rows
whose ID equals its predecessor/native-chain root ID. The exact root formula is
behavior-specific rather than the preservation set `P0`. At that exact native
`H`, both independent historical-impact enumerators expand each
criterion's input matchers over the complete tree, then follow all
implementation dependency edges, required-check producer edges, manifests,
build/runtime inputs, dispatch, generators, and test-harness inputs in the
impact direction until they reach the closed executable/runtime/build/test
subjects. Those terminal subjects, classified by the four `root_kind` values,
are exactly the row's root array. Review records, governance prose, certificate
records, and evidence outputs remain in `P0` and conservation/impact coverage
but are not behavior roots merely because they are preserved. An unmatched
executable input, unsupported dynamic dependency, omitted or extra terminal
subject, evidence output mislabeled as behavior, or enumerator disagreement is
RED.

Mechanical test `wv_impact_criterion_registry_v1` validates the fixed path and
schema, exact criterion/check equality, operation/verifier/input binding,
complete relation/resolver mapping, two-enumerator root derivation, cross-row
root union, and candidate-independent bytes. It includes direct root-path,
transitive source/build/workflow/runtime,
introduced-then-reverted, evidence-only, missing-edge, dynamic-path, and
arbitrary-predicate negatives. Status: `OPEN-UNTIL-IMPLEMENTED`.

### Exact chain-preservation set

All preservation sets are duplicate-free arrays of `versioned_blob_binding`
sorted by `(path, mode, git_oid)`. The native set `P0` is the recursive union of every
`blob_binding` beneath `native_root`: artifact membership, manifest, proof
mirrors, reporter, structured review, WV contract, and every required-check
evidence member. Each binding is converted without changing path, object ID,
mode, or raw SHA-256.

For one effective certificate/transcript pair, `Pnext` is exactly the union of
the predecessor set and:

- the registered certificate and effectiveness-transcript blob bindings;
- every distinct succession-, event-, and effectiveness-registry blob version
  observed at the selected `law_base_commit`, certificate `Q/M` boundary, and
  transcript-introduction `T` boundary, including the `G/A` genesis versions;
- every non-null old/new blob side in the certificate's source/event census,
  the transcript's source-to-`Q` census, and its landing-edge census;
- every certificate impact-evidence, producer-closure input/output,
  weakening-ruling, Class A event, Class C catalog/bundle, and immutable graph-
  projection support binding; and
- the unique canonical rolling review records selected at the source and
  certificate tips, plus the unique canonical rolling review record introduced
  in the terminal transcript bundle at `T`.

Certificate/transcript bindings are reconstructed from their registered path,
raw-byte hash, predecessor/current tree blob OID, and `100644` mode. Census
bindings copy exact recorded old/new facts. The recurrence walks the unique
effective chain oldest-to-newest; it never loads a mutable current registry to
rewrite an older member.

A new succession certificate's `preservation_hashes` is exactly `P` for its
predecessor plus every non-null side of its own `h_to_r_census` and its unique
source review record. In DP3 migration mode it additionally contains every
blob/version binding recursively present beneath `activation_bridge`: the
authority ruling; every bridge-census side; all impact/conservation evidence;
every materialized historical-impact-graph node converted to one
`versioned_blob_binding` by copying only its path, Git OID, mode, and raw
SHA-256 and deduplicating by `(path, mode, git_oid)`; review records; and the complete
`G/A/S` law-base support bindings, including
the impact-criterion registry blob that authorizes the semantic behavior-root
descriptors. No bridge evidence may exist outside this exact preserved
union. A new event certificate's `preservation_hashes` is
exactly predecessor `P` plus every non-null side of its own `edge_census`, all
impact evidence, producer-closure inputs/outputs, weakening ruling if present,
and the selected class registry/catalog/event bindings. The object containing
the set is never included in its own set. The certificate and transcript join
`Pnext` only after the transcript becomes effective.

`wv_succession_certificate_v1`, `wv_event_certificate_v1`, and
`wv_acceptance_chain_v1` independently rederive these unions and reject every
missing, extra, wrong-version, same-path/different-byte ambiguity, mutable-
registry substitution, omitted chain-registry version, omitted `T` review
record, reordered, self-including, or candidate-selected member.

## Succession certificate schema

The certificate schema ID is
`garnet.wv_acceptance_succession/v1`. Certificate paths match exactly
`F_Project_Management/W_TRUST/succession/*.wv-acceptance-succession.json`.
The registry is
`F_Project_Management/W_TRUST/WV_ACCEPTANCE_SUCCESSION.json`, schema
`garnet.wv_acceptance_succession_registry/v1`, with exactly `certificates` and
`schema`; `certificates` is a path-sorted array of exact objects containing
`blob_sha256`, `path`, and `wv`.

A succession certificate has exactly these top-level keys:

| key | JSON type | invariant |
|---|---|---|
| `activation_bridge` | null-or-object | non-null exact DP3 bridge only for the one genesis migration |
| `blocking_findings` | array | exactly `[]` for an eligible certificate |
| `certificate_id` | string | unique stable ID derived from WV, predecessor tip, and `source_b` |
| `certificate_kind` | string | exactly `succession` |
| `classifier_sha256` | string | predecessor classifier `sha256` |
| `digest_law_sha256` | string | predecessor digest-law `sha256` |
| `establishment_mode` | string | exactly `ordinary` or `dp3-528-genesis-migration` |
| `graph_projection` | `graph_projection` | self-contained commit/tree/blob and native-H/current-source/B inventories |
| `h_to_r_census` | `edge_census` | complete authenticated current `source_h..R` graph; fixed key retained for schema compatibility |
| `head_repository` | string | authenticated source PR head repository |
| `head_repository_id` | integer | positive immutable head repository ID |
| `law_base_commit` | string | exact activated predecessor-law `commit` selected below |
| `law_base_tree` | string | resolved `tree(law_base_commit)` |
| `native_accepted_pair` | `pair` | byte-equal to `native_root.native_accepted_pair` for the entire chain |
| `native_root` | `native_root` | complete immutable native root |
| `pair_input_differences` | array | exact explained pair-input deltas, sorted by `path` |
| `pair_recomputations` | array of `pair_recomputation` | exactly two independent native-H/current-source/B computations |
| `predecessor_effective_tip` | `effective_tip_ref` | one linear predecessor whose kind is exactly `native` or `succession`; `event` is RED |
| `preservation_hashes` | array of `versioned_blob_binding` | exact chain-preservation set through the source boundary |
| `producer_identity` | string | base-controlled succession-producer `logical_id` |
| `producer_sha256` | string | producer raw-byte `sha256` |
| `pull_request_id` | integer | positive immutable source PR ID |
| `pull_request_number` | integer | positive source PR number |
| `record_consumer_inventory_sha256` | string | predecessor exhaustive inventory `sha256` |
| `repository` | string | exactly `Island-Dev-Crew/garnet` |
| `repository_id` | integer | positive immutable upstream repository ID |
| `reviewer_family_registry_sha256` | string | raw predecessor reviewer-family registry `sha256` |
| `review_scope` | `review_scope` | exact non-extension attestation |
| `schema` | string | exactly `garnet.wv_acceptance_succession/v1` |
| `source_b` | string | authoritative main first-parent content landing `B` |
| `source_b_tree` | string | resolved `tree(B)` |
| `source_h` | string | current segment anchor: native reviewed `H` for the first succession, predecessor transcript `T` thereafter |
| `source_r` | string | final reviewed and approved source record tip `R` |
| `source_r_tree` | string | resolved `tree(R)`, equal to `source_b_tree` |
| `source_selected_review` | `selected_review` | direct authenticated approval at `R` |
| `successor_observed_pair` | `pair` | two-implementation recomputation at `B`; non-authoritative |
| `verdict` | string | exactly `pass` for an eligible certificate |
| `wv` | string | exact WV contract ID |

### One-time DP3 #528 activation bridge

For `establishment_mode=ordinary`, `activation_bridge=null`. The ordinary
certificate PR base is exactly `source_b`, and the complete `B..Q` census is
qualified certificate/review-only under R1.

`establishment_mode=dp3-528-genesis-migration` is a single Jon-ruled bootstrap
case, not a reusable class. It is eligible exactly once, as the first WV-6
succession certificate, with a native predecessor and
`law_base_commit=A`. Its source facts are fixed to PR #528:

- `native_root.reviewed_head=8426ca761c696c3556190be77cce3e340250b5c7`;
- `source_r=d9d6c163e083b667d3e7beaafcc2f3bb5bde061a`;
- `source_b=0607f7fe8770491bff3d16261628c27c570baa51`;
- `pull_request_number=528`; and
- `tree(source_r)=tree(source_b)`.

The non-null `activation_bridge` has exactly these keys:

| key | JSON type | invariant |
|---|---|---|
| `authority_ruling` | `blob_binding` | exact DP3/DP14 commissioning ruling at `F_Project_Management/W_TRUST/L1_DECISION_POINTS_RULING_2026-09-01.md` |
| `bridge_census` | `historical_edge_census` | complete fact-only authoritative-main first-parent `source_b..migration_base_s` census |
| `bridge_changed_paths` | array of strings | sorted unique path projection of every nonempty bridge edge operation |
| `bridge_impact_matrix` | array | exact law-base WV-6 criterion-to-predicate/verifier mapping, sorted by `criterion_id` |
| `bridge_impact_proof` | array | one independently recomputed row per bound WV-6 required criterion |
| `bridge_pair_recomputations` | array | exactly two pipeline-independent `bridge_pair_recomputation` members, sorted by `enumerator_identity` |
| `bridge_review_coverage` | array | exact one-row-per-routed-main-edge rolling-review coverage, sorted by `walk_index` |
| `conservation_results` | array | exact four named conservation predicate results, sorted by `predicate_id` |
| `genesis_commit` | string | exact derived inactive machinery boundary `G` |
| `genesis_tree` | string | resolved `tree(G)` |
| `law_activation_commit` | string | exact derived activation boundary `A`, equal to `law_base_commit` |
| `law_activation_tree` | string | resolved `tree(A)`, equal to `law_base_tree` |
| `law_base_support_bindings` | array of `versioned_blob_binding` | complete `G/A/S` versions of every schema, implementation, registry, route, fixture, runbook, and governed law consumed by the bridge |
| `migration_base_observed_pair` | `pair` | non-authoritative raw pair recomputed at `migration_base_s` |
| `migration_base_inventory` | array of `blob_binding` | complete ordered `(path, blob OID)` input stream at `S` |
| `migration_base_s` | string | authoritative-main tip `S` used as the certificate PR base |
| `migration_base_tree` | string | resolved `tree(S)` |
| `native_behavior_roots` | array of `native_behavior_root` | exact `root_id`-sorted union from the law-base impact-criterion registry |
| `native_reachability_recomputations` | array | exactly two independent full-history closed-graph results, sorted by `enumerator_identity` |
| `review_records` | array of `versioned_blob_binding` | complete independently enumerated canonical review/landing records for trust-kernel changes in `source_b..S` |
| `schema` | string | exactly `garnet.wv_acceptance_dp3_activation_bridge/v1` |
| `source_landing_b` | string | byte-equal to certificate `source_b` |
| `source_landing_tree` | string | resolved `tree(source_b)` |
| `verdict` | string | exactly `pass` |

`law_base_support_bindings` is a closed, independently derivable set. Its fixed
exact-path seed is:

```text
.github/rulesets/README.md
AGENTS.md
C_Language_Specification/AGENTS.md
C_Language_Specification/GARNET_TRUST_KERNEL_ROLLING_REVIEW.md
C_Language_Specification/GARNET_WV_ACCEPTANCE_SUCCESSION_CONTRACT.md
F_Project_Management/W_TRUST/DP12_WV_ACCEPTANCE_REGISTRY_FORK_RECOVERY_RUNBOOK.md
F_Project_Management/W_TRUST/L1_DECISION_POINTS_RULING_2026-09-01.md
F_Project_Management/W_TRUST/REACCEPTANCE_REDESIGN_BRIEF_v2.md
F_Project_Management/W_TRUST/REVIEW_FAMILY_IDENTITIES.json
F_Project_Management/W_TRUST/U66_COMPANION_U59_EXCEPTION_2026-09-01.md
F_Project_Management/W_TRUST/WV_ACCEPTANCE_EFFECTIVENESS.json
F_Project_Management/W_TRUST/WV_ACCEPTANCE_EVENTS.json
F_Project_Management/W_TRUST/WV_ACCEPTANCE_EVENT_CLASSES.json
F_Project_Management/W_TRUST/WV_ACCEPTANCE_IMPLEMENTATIONS.json
F_Project_Management/W_TRUST/WV_ACCEPTANCE_IMPACT_CRITERIA.json
F_Project_Management/W_TRUST/WV_ACCEPTANCE_PRODUCER_GRAPH.json
F_Project_Management/W_TRUST/WV_ACCEPTANCE_RECORD_CONSUMERS.json
F_Project_Management/W_TRUST/WV_ACCEPTANCE_SUCCESSION.json
F_Project_Management/W_TRUST/WV_ACCEPTANCE_TRANSPORT_PROJECTIONS.json
F_Project_Management/W_TRUST/WV_CLASS_C_REWRITE_CATALOG.json
F_Project_Management/W_TRUST/WV_REGISTRY_YANK_EXCEPTION_EVENTS.json
```

To that seed, the two implementation-inventory enumerators independently add:
every entrypoint, source-closure member, and non-null parameter-schema binding
in the complete implementation inventory; every consumer/producer path in the
record-consumer inventory; every producer-graph endpoint and producer path;
every Class C catalog sibling binding; and every path selected in the complete
`G`, `A`, or `S` tree by every implementation operation's input, output, and
fixture matchers or by every record-consumer matcher. A schema matcher is
expanded by parsing every regular `100644` JSON blob in the tree with duplicate-
key rejection; a suffix/prefix/exact matcher uses its literal law. The union is
keyed by `(path, mode, git_oid)`, sorted accordingly, and each member copies the exact
mode/OID/raw SHA-256 from each boundary tree. A fixed seed path must exist at
all three boundaries; an expanded path contributes every distinct version seen
at any of them. The two enumerators produce byte-identical arrays and have only
inert-allowlist closure intersection. An omitted or extra path/version, route
with zero executable owner, unregistered fixture/runbook, matcher ambiguity,
candidate-only expansion law, or `A..S` law-bearing byte movement is RED.

Each `bridge_review_coverage` member has exactly `base_commit`,
`candidate_head`, `candidate_tree`, `landing_commit`, `landing_marker`,
`landing_tree`, `record`, `review_pages`, `reviewed_head`, `reviewed_tree`,
`rolling_law`, `selected_review`, `touched_paths`, `transport_receipts`,
`verdict`, and `walk_index`. `walk_index` selects one bridge-census main edge;
`base_commit` is that edge's parent, `landing_commit` its child, and
`landing_tree=tree(landing_commit)=candidate_tree`. `record` is the one
canonical rolling-review `blob_binding` introduced by that candidate;
`landing_marker` is null-or-`blob_binding` exactly as the owning rolling law
requires. `rolling_law` is the exact `versioned_blob_binding` for
`GARNET_TRUST_KERNEL_ROLLING_REVIEW.md` at `base_commit`.

The parsed record supplies its exact `reviewed_head` and `reviewed_tree`; the
base-controlled rolling verifier rederives its canonical bytes, digest, author
and touched set, and requires `touched_paths` to equal the complete sorted
old-base trigger projection for the edge. `selected_review.commit_id` equals
the exact record-containing `candidate_head`. Complete `review_pages` and
`transport_receipts` use the registered shapes and bind the repository, PR,
base, candidate head/tree, selected review/user, landing commit, landing parent,
and authoritative-main ref observed during bridge capture. `verdict=pass`.

For each bridge row, `transport_receipts` has exactly one each of
`repository-object`, `head-repository-object`, `pull-request-object`,
`selected-review-primary-object`, `selected-review-primary-user-object`,
`rolling-review-candidate-commit-object`,
`rolling-review-landing-commit-object`,
`rolling-review-landing-parent-commit-object`, and
`authoritative-main-ref-object`, plus one or more contiguous complete
`pull-request-commits-page` rows. Every other role, including either
supplemental-review role, is absent. `review_pages` is separately exactly one
or more contiguous complete `page_receipt` rows normalized by `review-page`.
The selected paginated review row equals the direct selected-review and user
objects. The commit pages exhaust the PR commit collection and supply every
author and committer principal used by the old-base review law.

The PR projection binds the row's immutable PR and repository identities,
`base.sha=base_commit`, `head.sha=candidate_head`, the exact head repository,
merged state, and `merge_commit_sha=landing_commit`. The three rolling-review
commit roles use the same exact commit-object projection shape as the
corresponding certificate roles and bind respectively `candidate_head`,
`landing_commit`, and `parent1(landing_commit)`, including trees and ordered
parents. The main-ref projection descends through `landing_commit`. A missing,
extra, or duplicate role; default/latest endpoint; incomplete page chain;
supplemental role; direct/paginated disagreement; or wrong base, head, tree,
parent, landing, repository, or main descendant is RED.

Exactly two `bridge-review-enumerator` implementations independently walk every
`bridge_census` edge and emit byte-identical coverage arrays. An edge with no
old-base trust-kernel trigger match has no row. An edge with one or more matches
has exactly one row and passes the rolling law loaded from its own base; no
current-law backfill is allowed. `review_records` is exactly the
`(path, mode, git_oid)`-sorted versioned projection of every row's `record` and
non-null `landing_marker`, with no other binding. A missing/extra row, partial
touched set, wrong base law, noncanonical or backdated record, tree mismatch,
unreviewed trust-kernel edge, incomplete transport, stale/non-head review,
missing/extra marker, enumerator dependence/disagreement, or candidate/main
range gap is RED.

A `bridge_pair_recomputation` has exactly `digestor_identity`,
`digestor_sha256`, `enumerator_identity`, `enumerator_sha256`,
`migration_base_input_count`, `migration_base_input_sha256`,
`migration_base_pair`, and `migration_base_s`, with the corresponding identity,
closure-hash, complete ordered-input, and pair invariants of
`event_pair_recomputation`. The two members bind identical inputs and equal
`migration_base_observed_pair`; their pipeline logical-ID sets and closure
unions satisfy the exact independence law. Each count equals
`migration_base_inventory` length and each input hash uses the exact
`migration-base-s/v1` framing above over that same array; an inferred, omitted,
reordered, or differently bound input is RED.

`native_behavior_roots` is byte-for-byte the `root_id`-sorted unique union of
the exact WV-6 criterion rows in the law-base impact-criterion registry. The
registry's raw blob binding is part of `law_base_support_bindings`. A candidate
root list, `P0` substituted for the behavior-specific union, missing criterion
root, or path/kind/criterion disagreement is RED.

A `historical_native_impact_graph` has exactly `bridge_census_sha256`,
`closure_open`, `edges`, `entry_bindings`, `nodes`, `root_matches`, `roots`,
and `sha256`.
`bridge_census_sha256` equals the verified full fact-only census digest;
`closure_open=[]`; `roots` is the sorted unique behavior `root_id` array copied
from `native_behavior_roots`; and `sha256` uses the framing above. A graph
node has exactly `git_oid`, `mode`, `node_id`, `origin`, `path`, `sha256`, and
`tree_commit`. `origin` is exactly `bridge-old`, `bridge-new`, or `dependency`;
all other fields bind one materialized Git blob. `node_id` is
`impact-node:<hex>`, where `<hex>` hashes
`garnet.wv_acceptance.historical_impact_node/v1 NUL` plus the canonical node
with only `node_id` removed. Nodes are sorted by `node_id` and unique.

Node material identity is the exact `(tree_commit, path, mode, git_oid,
sha256)` tuple, and exactly one node exists per tuple. Its deterministic origin
precedence is `bridge-old` when any old-side entry binding selects the tuple,
otherwise `bridge-new` when any new-side entry binding selects it, otherwise
`dependency`. Dependency discovery reuses an existing bridge-origin node rather
than emitting a second node. Because origin is thereby uniquely derived, it may
remain in the `node_id` preimage without allowing two encodings of one blob.

An `entry_bindings` member has exactly `commit`, `node_id`, `operation`,
`parent`, `path`, `side`, `status`, and `walk_index`. `side` is `old` or `new`.
For every non-null old and new side of every `historical_census_entry`, exactly
one member copies its edge coordinates, raw status, operation, and path and
selects a node with byte-equal path, object ID, mode, and raw SHA-256 at
`tree_commit=parent` for `old` or `tree_commit=commit` for `new`. The array is
sorted by `(walk_index, path, side)`; the selected node's origin follows the
material-identity precedence above. It is an occurrence projection, never a
deduplicated changed-path set, so an introduced-then-reverted version remains
present twice at its exact edges.

A graph `edges` member has exactly `from_node_id`, `relation`,
`resolver_identity`, `resolver_sha256`, and `to_node_id`. `relation` is exactly
`build-config`, `command`, `dispatch`, `generate`, `govern`, `import`,
`include`, `manifest`, `mirror`, `provenance`, `runtime-input`, `template`, or
`test-evidence`. For that exact relation, `resolver_identity` and
`resolver_sha256` equal the sole row in the law-base impact-criterion
registry's `resolvers` array; its `operation_id` is the operation applied even
though the edge need not repeat that already unique field. Edges are
sorted by `(from_node_id, to_node_id, relation, resolver_identity)` and unique.

Starting from every `entry_bindings` node, each enumerator independently scans
the complete parent and child tree for every bridge edge and computes the least
fixed point through all law-base implementation dependency edges, producer
input/output matchers, language imports/includes/templates, build and workflow
commands, generators, manifests, mirrors, provenance, runtime inputs, dispatch,
and required-check/test evidence relations. Every materialized dependency
version becomes a node; every resolved relation becomes an edge. An unresolved
dynamic path, unsupported executable/configuration grammar, zero- or
multi-matched operation, missing historical blob/tree, missing node or edge,
or path/version not conservatively classified is recorded in `closure_open`
and therefore makes the bridge RED. Candidate graph law, the endpoint `S`
tree alone, or a path-only graph cannot close the fixed point.

Every graph edge is oriented in the causal impact direction: `from_node_id` is
the upstream changed/dependency/input subject and `to_node_id` is the dependent
consumer/output subject. A producer-graph `from_path -> to_path` retains that
direction. An implementation `from_path -> to_path` dependency edge used to
walk a source closure is reversed here as dependency `to_path -> from_path`.
Imports, includes, templates, build configuration, manifests, runtime inputs,
commands, and dispatch likewise point from the referenced/input subject to the
consumer; generated/provenance/mirror/test-evidence relations point from their
source to their output. The enumerator emits every applicable relation once;
reversing, collapsing, or omitting one leaves `closure_open` nonempty.

Behavior-root identity is semantic and version-independent. Any graph node
whose `path` equals a law-base `native_behavior_root.path` reaches that
`root_id` at distance zero, regardless of node origin, Git blob version, or
tree commit. Transitive reachability then follows only the forward impact edges
above. Thus a direct root-path modification cannot be isolated by giving its
historical node a different ID, and a new/transient dependency cannot disappear
merely because it is absent at endpoint `S`.

That relation is materialized in `root_matches`. Each member has exactly
`node_id` and `root_id`; it selects existing graph/root members whose paths are
byte-equal. The array is sorted by `(node_id, root_id)`, unique, and equals the
complete Cartesian path-equality projection—every matching graph node/root
pair and no other pair. Direct reachability begins with root matches on the
entry-binding nodes; transitive reachability follows graph edges and then tests
root matches on every reached node. An absent direct match, an invented match,
or an ID-only comparison without path equality is RED.

Each `native_reachability_recomputations` member has exactly
`enumerator_identity`, `enumerator_sha256`, `historical_graph`, and
`reachable_native_roots`. The identity selects one of the exact two distinct
law-base `historical-impact-enumerator` rows and its sole registered
`wv_dp3_genesis_migration_v1` enumerate operation. `historical_graph` is the
complete object above. `reachable_native_roots` is the duplicate-free,
`root_id`-sorted `native_behavior_root` subset reachable from any entry-binding
node and is exactly `[]`. Both independently encoded enumerators
rederive byte-identical `bridge_census`, graph, and result; their source
closures intersect only through the inert allowlist. Any historical old/new
version reaching a native behavior root makes the migration ineligible and
routes to a native terminal freeze.

Each `bridge_impact_matrix` member has exactly `criterion_id`, `predicate_id`,
`verifier_identity`, and `verifier_sha256`. The complete array is recursively
byte-equal to the `(criterion_id, predicate_id, verifier_identity,
verifier_sha256)` projection of the exact law-base WV-6 impact-criterion
registry rows and therefore has the same criterion set as
`native_root.required_checks[*].id`. The mapped `predicate_id` selects that
registry row's one implementation `verify` operation, whose input matchers
cover the full historical census and impact graph; the verifier fields equal
its owning verifier/record-consumer row and source-closure hash.
Candidate-selected, remapped, or many-to-one associations are RED.

Every mapped bridge-impact verifier logical ID is distinct from the succession
producer and both historical-impact enumerators. Its source closure intersects
each of those three closures only through the exact inert allowlist, and it does
not import, invoke, wrap, or consume their implementation or unverified output.
The verifier consumes the canonical census/graph bytes and independently
rederives them before evaluating its criterion. Role reuse, executable closure
overlap, or accepting a producer/enumerator assertion as proof is RED.

Each `bridge_impact_proof` member has exactly `bridge_census_sha256`,
`changed_inputs`, `criterion_id`, `evidence`, `historical_graph_sha256`,
`predicate_id`, and `verdict`. The array is sorted by `criterion_id`.
`bridge_census_sha256` and `historical_graph_sha256` equal both independently
verified full-history objects; `changed_inputs` equals all
`bridge_changed_paths` only as an additional human-scale projection, never as
the verifier preimage; and the criterion/predicate pair equals the exact
`bridge_impact_matrix` row. A separate mapped law-base verifier at exact `S`
consumes every census entry occurrence, every old/new path/version/operation,
the complete closed historical graph, the bound WV-6 criterion, and repository
evidence, then independently derives its nonempty path-sorted
`versioned_blob_binding` evidence and `pass` result. Its ID set equals all WV-6
required criteria. Each
`conservation_results` member has exactly `evidence`, `predicate_id`, and
`verdict`; `evidence` is a nonempty path-sorted `blob_binding` array,
`predicate_id` is one of the four conservation IDs exactly once, and
`verdict=pass`. Candidate assertions, current law, or Jon prose cannot replace
either recomputation.

`bridge_census.start_commit=source_b` and `end_commit=S`; `S` descends through
`A`; and current authoritative main equals `S` when the certificate PR is
opened. The certificate PR's authenticated `base.ref=main` and `base.sha=S`.
Its `S..Q` subwalk is certificate/review-only. For this migration only, the
effectiveness `b_to_q_census` retains its fixed key name but is the ordinary
producer-qualified `edge_census` over exactly that authenticated candidate DAG
tail: `start_commit=S`, `end_commit=Q`. The fact-only authoritative-main
`B..S` history remains separately and completely bound by
`activation_bridge.bridge_census`; no combined census or relabeling is
permitted. The transcript verifier checks exact endpoint equality at `S` and
the union of the two disjoint edge sets as the complete `B..S..Q` theorem.
Any gap, overlap, substituted topology, or main movement from `S`, even
record-only movement, voids the candidate and requires restart with a new exact
bridge and exact-head review.

This bridge does not call activation bytes native evidence, move the accepted
pair, extend the native review, or classify non-record changes as records. It
preserves the immutable native pair only because the commissioned one-time
theorem separately proves complete history, zero reachability to native
behavior roots, all WV-6 impact criteria, all four conservation predicates,
and non-authoritative raw-pair accounting at `S`. Any failed or open proof,
second migration, different PR/WV/boundary, reused implementation, unreviewed
trust-kernel segment, reachable native behavior, or claim upgrade is RED.

Mechanical test `wv_dp3_genesis_migration_v1` traps the exact fixed #528 facts,
single-use/native-predecessor mode, `G/A/S` and law-base selection, complete
`B..S..Q` split census, current-main serialization, full bridge changed-path and
review-record sets, two pair and reachability recomputations, five impact rows,
four conservation rows, source-closure independence, zero native reachability,
raw/accepted separation, and restart-on-movement. It rejects every contrary
case named above, including an entry binding redirected to a wrong-path node
that happens to carry the same mode, object ID, and raw bytes. Status:
`OPEN-UNTIL-IMPLEMENTED`.

Each `pair_input_differences` member has exactly `new_git_oid`,
`old_git_oid`, `operation_subtype`, `path`, and `producer_id`. Its path and
object IDs agree with `h_to_r_census`, and the complete sorted array exhausts
the independently recomputed current-`source_h`-to-`B` pair-input difference.

For every difference member, `new_git_oid` and `old_git_oid` are full object IDs
or the all-zero identity on an addition/deletion; `operation_subtype` and
`producer_id` are nonempty predecessor-inventory IDs; and `path` is a
`git_path`. `pair_recomputations[*].native_h_pair` equals
`native_accepted_pair`; both B results equal `successor_observed_pair`. The
chain selector derives the predecessor's accepted pair from the native root or
prior succession certificate and requires it to equal `native_accepted_pair`.

The canonical-byte hash of `native_root` equals
`predecessor_effective_tip.native_root_id`. For a native predecessor, the
predecessor-controlled verifier reconstructs the root from the exact bound WV
contract, repository evidence, reporter projection, structured review, and
authenticated decisive review. For a succession predecessor, `native_root` is
recursively byte-equal to the predecessor certificate's root. A candidate-
selected root, root splice, hash mismatch, or changed native-root byte is RED.

The current segment boundary is exact. When the predecessor kind is `native`,
`source_h=native_root.reviewed_head`, `endpoint_source_inventory` equals
`endpoint_h_inventory`, and every recomputation's `source_h_pair` equals its
`native_h_pair`. When the predecessor kind is `succession`, `source_h` is the
derived introduction commit `T` of that predecessor's unique effectiveness
transcript; the source inventory and pair equal the raw observation independently
derived at that `T` under the predecessor transcript law. In both cases
`h_to_r_census.start_commit=source_h` and `end_commit=source_r`. This makes the
brief's zero-or-more native-rooted succession prefix constructible across
squashes without treating a discarded pre-squash native `H` as an ancestor of
a later PR. The immutable native root and native-H recomputation remain
separate and are never replaced by the segment anchor. A merge base, `B`, `M`,
current main head, or candidate-selected alternative start is RED.

A succession certificate cannot follow an event. Once an effective R3 event
selects `new_accepted_pair`, later qualified record tails are observation-time
accounting under `wv_acceptance_chain_v1` and `wv_record_tail_pair_v1`; they do
not create an R1 certificate or relabel the event pair as native evidence.

`source_selected_review` has `supplemental_reviews=[]`; its primary family is
cross-family from every authenticated source-PR implementation family under the
bound predecessor reviewer-family registry. The later certificate-PR review is
selected and durably anchored by the effectiveness transcript under the same
registry.

`review_scope.reviewed_through=source_r`, and its `source_h`, `source_r`, and
`source_b` equal the certificate's same-named fields. These are all facts that
precede the certificate bytes; no certificate commit ID is embedded in the
certificate. The later effectiveness transcript alone supplies derived `Q` and
`M` and checks the exact structured scope across both stages.

Mechanical test `wv_succession_certificate_v1` validates canonical bytes,
exact recursive shape, registry membership and append-only history, native-root
reconstruction, ID equality and preservation, predecessor uniqueness, source
PR/review identity, complete
per-edge census with the exact native-or-predecessor-`T` segment start,
self-contained graph/tree and endpoint inventories,
consumer-inventory membership, `tree(R)==tree(B)`, first-parent placement of
`B`, two pipeline-independent pair recomputations with bound closure unions,
native-
accepted-pair preservation, certificate-type ordering, complete delta
explanation, pair cross-role reuse/swap, root-splice, alternative-start and
false-native-ancestry negatives,
and every negative named by the R1 contract. Status:
`OPEN-UNTIL-IMPLEMENTED`.

## Four conservation predicates

Each predicate is an activation blocker. A row remains ineligible until the
named mechanical test exists in predecessor-controlled code and its positive
and fail-closed negative fixtures pass at the exact candidate.

| predicate | exact mechanical test | status |
|---|---|---|
| `r1_review_scope_exact_v1` | Parse the exact structured non-extension fields listed in R1, then require authenticated `H/R/B/Q/M` agreement and `coverage_extension=[]`. | `OPEN-UNTIL-IMPLEMENTED` |
| `r2_role_separation_v1` | Require positive immutable reviewer and attempt-2 `triggering_actor` IDs to be pairwise disjoint from one another and every authenticated PR commit author/committer ID. | `OPEN-UNTIL-IMPLEMENTED` |
| `r1_reporter_constant_projection_v1` | In `scripts/smoke_garnet_minimum_shelf.py`, replace only the literal spans for `REVIEWED_HEAD`, `REVIEWED_TREE`, `EXPECTED_PRODUCT_CONTENT_SHA256`, and `EXPECTED_PRODUCT_PATH_COUNT` with typed sentinels; all remaining raw bytes must match and each replacement must be independently derived from bound Git/pair evidence. | `OPEN-UNTIL-IMPLEMENTED` |
| `r1_strict_equal_blob_identity_v1` | Require predecessor/candidate blob-OID equality for `scripts/garnet_github_governance_gate.py` and every predecessor-inventoried comparator consumer; relocation, wrapper, or import substitution is RED. The four reporter literals are not a `_strict_equal` carve-out. | `OPEN-UNTIL-IMPLEMENTED` |

The table above is transcribed verbatim from the brief's four predicate rows.
The later executable tests use the predicate IDs as their public machine names.
In the verbatim `H/R/B/Q/M` shorthand, the `H` slot means the schema's exact
current succession-segment anchor: native `H` only for the first succession and
the predecessor effectiveness introduction `T` for every later succession.
The test authenticates that kind-discriminated value rather than hard-coding
the historic native head. A later succession that supplies native `H`, or a
first succession that supplies `T`, is RED.
They include at least these fail-closed fixtures:

- `r1_review_scope_exact_v1`: missing/unknown field, nonempty coverage extension,
  backdated head, wrong `B`, unequal `Q`/`M` tree, or direct-object disagreement.
- `r2_role_separation_v1`: reviewer/carrier overlap, either identity overlapping
  an author or committer, use of `actor.id` instead of `triggering_actor.id`,
  zero/missing ID, login mismatch, partial commit pagination, or stale review.
- `r1_reporter_constant_projection_v1`: fifth literal, expression rather than
  literal, duplicate assignment, non-derived value, body/import/token movement,
  relocation, wrapper substitution, or candidate-defined masking law.
- `r1_strict_equal_blob_identity_v1`: blob mismatch, missing inventoried
  consumer, new consumer, relocation, wrapper/import substitution, candidate
  inventory, or an attempted four-literal comparator carve-out.

DP10 composes additively with the verbatim singular R2 row. When a bound
`BOUNDED WEAKENING` requires a supplemental decisive reviewer,
`r2_role_separation_v1` evaluates `REVIEWER_IDS` as the primary plus every
supplemental reviewer ID. Every member is positive and pairwise disjoint from
`CARRIER_ID`, every other reviewer, and every commit principal; a carrier can
never satisfy either review role.

The following adopted contract text is transcribed verbatim from the brief:

```text
CONSERVATION RULE.

No succession or bounded re-acceptance MAY delete, relax, bypass, infer, or
silently substitute any fail-closed meta-property enumerated here. Bypass
emptiness, exact head/tree/pair binding, complete authenticated transport,
per-edge drift census, predecessor preservation, append-only linear records,
strict equality, independent decisive review, and producer-closure atomicity
MUST each be re-proved at the new observation boundary.

Every named conservation predicate MUST have executable predecessor-controlled
evidence. OPEN-UNTIL-IMPLEMENTED is ineligible, not an advisory pass.

An R3 certificate MAY alter an object-level policy only when an already-
ratified class names the exact movement and direction. The sole weakening in
this proposal is the Jon-approved, exact-name/version/checksum registry
exception with expiry; it MUST NOT relax any fail-closed meta-property above.

A mechanism that cannot name and recompute one conserved predicate is
ineligible, regardless of event direction or historical precedent. Unknown,
partial, unavailable, conflicting, or not-applicable-by-assertion is RED.
```

Mechanical test `wv_acceptance_conservation_v1` aggregates all four predicate
results plus bypass, binding, transport, census, preservation, linear-record,
review, and closure evidence. It reports ineligible if any result is missing,
open, partial, unavailable, conflicting, or derived from candidate-controlled
law. Status: `OPEN-UNTIL-IMPLEMENTED`.

## Acceptance succession law

The following adopted contract text is transcribed verbatim from the brief:

```text
ACCEPTANCE SUCCESSION LAW.

1. BINDING. A native WV acceptance binds one WV contract and schema, platform
   and scope, exact reviewed head and tree, native-accepted content pair,
   evidence destination, canonical manifest, exact artifact membership and
   hashes, proof mirrors, reporter projection, all required-check results,
   structured review, independent reviewer identity, decisive exact-head
   approval, and preservation state. Acceptance is no broader than those facts.

2. SQUASH. A squash does not preserve branch ancestry and does not implicitly
   carry acceptance. Acceptance survives a squash only through an effective
   R1 certificate proving the complete qualified record projection from the
   accepted head through the final PR tip, exact PR-tip/squash-tree equality,
   authoritative source landing B, exact approved certificate tip Q,
   authenticated first-parent certificate landing M with tree(Q) equal to
   tree(M), complete landing-edge census, unchanged native-accepted pair, two
   independently implemented raw-pair recomputations, and one terminal
   effectiveness transcript anchoring the establishment-time forge facts. The
   source review is neither extended nor backdated. Ordinary verification MUST
   NOT require Q, a pull ref, or live forge review state after that anchor.

3. RECORD SUCCESSOR. Above an effective certificate, later commits preserve
   acceptance only after loading the predecessor effective tip's exhaustive
   record_consumer_inventory. Each sorted entry binds consumer ID, producer
   blob hash, exact path/prefix/suffix/registry matcher, schema, semantic role,
   and permitted operation for every gate or reporter that globs, suffix-
   matches, registry-loads, or parses record bytes, including landed markers
   and the succession, event, and effectiveness registries. Two independent
   inventory enumerators MUST agree. Candidate inventory is never authority.
   Every commit edge MUST be producer-censused, and every operation and
   machinery-consumed record path MUST match one inventory entry and its bound
   predicate. Prefix membership is necessary and never sufficient. An unlisted
   consumer, matcher drift, unmatched claimed record, or operation outside its
   predicate is content change. The accepted pair remains unchanged; any raw-
   pair movement is restated separately and fully explained. Existing records,
   evidence, and chain links remain immutable. A record-path logic or inventory
   change is content, not a record successor.

4. CONTENT CHANGE. The first non-record byte or non-qualified operation ends
   record succession immediately. The prior acceptance becomes
   SUPERSEDED-WITH-PRESERVATION; it is never rewritten or erased. An exact R3
   class MAY establish a new accepted pair through a reviewed, closed delta
   certificate only after its exact approved tip Q and equal-tree first-parent
   landing M are authenticated. If no one class matches, any impact remains
   open, any fail-closed meta-property moves, or weakening lacks Jon's explicit
   ruling, the candidate requires a full native terminal freeze.

5. OBSERVATION. Every current acceptance result binds the native root, complete
   certificate and effectiveness chain, exact current head/tree, current
   accepted pair, current raw observed pair, effective main landing, durably
   anchored decisive review, and fresh external observations still required by
   its classes. Missing, stale, ambiguous, conflicting, forked, partial, or
   unavailable evidence is RED.
```

Mechanical test `wv_acceptance_chain_v1` starts from the native root, loads the
three predecessor registries, independently enumerates every registered and
unregistered suffix match, requires exactly one append-only linear effective
tip, permits only the bounded terminal-construction state defined below,
verifies every effective certificate/transcript pair in order, stops at the
first unqualified content change, and recomputes the current accepted and raw
observed pairs. In that same reporter invocation it freshly executes every
still-live event obligation's complete registered external transport multiset
and emits the current normalized projection and receipt digests. It accepts the
positive Class A `activate` then matching `expire` fold while preserving both
historical receipts and checking only the terminal tuple obligation live. It
also exercises same-subject terminal replacement for Class B ruleset and Class C
toolchain folds without reapplying superseded predicates. A gap, fork, duplicate,
rollback, unmatched object, ambiguous
predecessor, unbounded pending object, stale external observation, current head
outside the chain, premature/wrong-tuple Class A expiry, or candidate-controlled
selector, wrong-subject B/C replacement, two terminal variants/pins, or
superseded-predicate reapplication is RED. Status:
`OPEN-UNTIL-IMPLEMENTED`.

## Later-act test inventory

The arc's later implementation acts trap this contract through these public
machine predicates:

| test ID | later act | required proof |
|---|---:|---|
| `wv_schema_canonical_v1` | 3 | all exact recursive schema/byte/path/mode positives and negatives |
| `wv_succession_certificate_v1` | 3 | R1 producer, succession registry, two pipeline-independent pair computations, graph and preservation fixtures |
| `wv_event_certificate_v1` | 3 | R3 producer, event registry, class/impact/closure positives and class-miss negatives |
| `wv_event_registry_yank_v1` | 3 | exact Class A addition/expiry, dual-source `valid_while`, preservation, ruling, and stale/unknown negatives |
| `wv_class_a_legacy_precondition_v1` | 3 | exact pre-`G` DP7 deny-row removal landing, reversal transport, locked dependency facts, and no event/acceptance authority |
| `wv_event_forge_serialization_v1` | 3 | exact already-ratified leaf variants, complete projection, strict equality, bypass/context preservation, and widening negatives |
| `wv_event_toolchain_lint_v1` | 3 | exact compiler pin plus one predecessor-catalog rewrite and fixed-point closure, with catalog/suppression/stale-proof negatives |
| `wv_acceptance_effectiveness_v1` | 3 | predecessor-base capture producer, terminal receipt registry, review/landing/pagination/non-recursion fixtures |
| `trust_kernel_review_eligibility_v1` | 2 and 3 | attempt-1 artifact emission plus schema/artifact/live-transport negatives |
| `r2_same_run_re_evaluation_v1` | 2 and 3 | sole eligible tuple, same-run/head attempt equality, all-jobs proof, fresh transport, authenticated reporter emission, and separately mandatory Jon procedural read |
| `r1_review_scope_exact_v1` | 3 | exact non-extension structure and authenticated `H/R/B/Q/M` binding |
| `r2_role_separation_v1` | 3 | immutable reviewer/carrier/commit-principal disjointness |
| `r1_reporter_constant_projection_v1` | 3 | exactly four independently derived literal substitutions and byte identity elsewhere |
| `r1_strict_equal_blob_identity_v1` | 3 | predecessor/candidate blob identity for all inventoried comparator consumers |
| `wv_acceptance_conservation_v1` | 3 | aggregate fail-closed conservation result |
| `wv_acceptance_chain_v1` | 3 and 4 | unique-tip chain selection and rolling-reporter integration |
| `wv_law_base_selector_v1` | 3 and 4 | exact inactive-genesis `G`, gate-activation `A`, later transcript `T`, and kind-discriminated predecessor-law selection |
| `wv_acceptance_activation_v1` | 3 and 4 | complete inactive machinery at `G`, later all-green gate/reporter activation at `A`, and no pre-activation certificate authority |
| `wv_acceptance_registry_genesis_v1` | 3 | two independent post-landing `G/parent1(G)` genesis censuses, empty registries, complete inactive machinery surface, append-only history, and fork negatives |
| `wv_dp3_genesis_migration_v1` | 3, 4, and 5 | one fixed #528 bridge with exact `B..S..Q` census, zero native reachability, conservation, current-main serialization, and no pair/claim laundering |
| `wv_acceptance_pending_terminal_v1` | 3 and 4 | bounded certificate-candidate, landed-pending, transcript-candidate, and effective state transitions without authority backdating |
| `wv_implementation_source_closure_v1` | 3 | exact implementation inventory, framed fixed-point closure hashes, and two independent full-tree enumerators |
| `wv_impact_criterion_registry_v1` | 3 | exact fixed #528 WV-6 criterion/verifier/input mapping and two-enumerator semantic behavior-root derivation |
| `wv_producer_graph_inventory_v1` | 3 | canonical predecessor producer graph, exact operation/matcher bindings, and two independent graph enumerators |
| `wv_transport_projection_v1` | 3 | predecessor-owned exact transport projections, discriminated identities, closed role multisets, and pagination negatives |
| `wv_event_class_registry_v1` | 3 | exact five-row class registry, Class A event schema, Class B nested variants, and Class C typed catalog preimages |
| `record_consumer_inventory_twice_v1` | 3 | two independent complete machinery-consumer inventories and candidate-law negatives |
| `wv_acceptance_trigger_digest_routing_v1` | 3 | old-base trigger selection and exact rolling-digest coverage for every schema, registry, producer, verifier, and policy route |
| `wv_record_tail_pair_v1` | 3 | two independent per-certificate source/landing/introduction and final-tail raw-pair computations with accepted-pair separation |
| `wv_cross_family_review_v1` | 3 | authenticated implementation/reviewer family separation plus second-family and Jon-authority weakening proof |

Act 2 may wire only the attempt-1 receipt and attempt-2 verification path. Act 3
owns producers, registries, consumer inventory, executable predicates,
adversarial fixtures, and the registry-fork recovery runbook. Act 4 alone wires
the accepted results into the rolling gate. Act 5 alone may create the #528
migration certificate after serializing against current main. No test ID in
this table is green merely because this specification names it.

## Event certificate schema

The brief did not allocate an event schema ID; this contract completes that
open name as `garnet.wv_acceptance_event/v1`. Certificate paths match exactly
`F_Project_Management/W_TRUST/events/*.wv-acceptance-event.json`. The registry
is `F_Project_Management/W_TRUST/WV_ACCEPTANCE_EVENTS.json`, schema
`garnet.wv_acceptance_event_registry/v1`, with exactly `events` and `schema`;
`events` is a path-sorted array of exact objects containing `blob_sha256`,
`path`, and `wv`.

An event certificate has exactly these top-level keys:

| key | JSON type | invariant |
|---|---|---|
| `allowed_source_delta` | array | exact class-admitted deltas, sorted by `(path, direction_axis)` |
| `blocking_findings` | array | exactly `[]` for an eligible certificate |
| `certificate_content_head` | string | exact pre-certificate content `commit` `C` |
| `certificate_content_tree` | string | resolved `tree(C)` |
| `certificate_id` | string | unique stable ID derived from WV, predecessor tip, class, and `C` |
| `certificate_kind` | string | exactly `event` |
| `class_registry_sha256` | string | raw predecessor event-class registry `sha256` |
| `class_producer_identity` | string | predecessor-ratified event-class-producer `logical_id` |
| `class_producer_sha256` | string | class producer raw-byte `sha256` |
| `classifier_sha256` | string | predecessor classifier `sha256` |
| `digest_law_sha256` | string | predecessor digest-law `sha256` |
| `direction` | object | exact three-axis direction shape below |
| `edge_census` | `edge_census` | complete predecessor-to-`C` per-edge census |
| `event_action` | string | exact action discriminant in the class/direction matrix below |
| `event_class` | string | exactly `registry_yank`, `forge_serialization_drift`, or `toolchain_lint_activation` |
| `external_observation` | object | exact bounded external observation shape below |
| `impact_proof` | array | exactly one entry for each required-check ID in the bound predecessor `wv` contract |
| `law_base_commit` | string | exact activated predecessor-law `commit` selected below |
| `law_base_tree` | string | resolved `tree(law_base_commit)` |
| `new_accepted_pair` | `pair` | independently derived pair at `C` |
| `native_root` | `native_root` | complete immutable native root selected by the chain |
| `old_accepted_pair` | `pair` | exact predecessor accepted pair |
| `pair_recomputations` | array of `event_pair_recomputation` | exactly two independent complete-input computations at `C` |
| `predecessor_effective_tip` | `effective_tip_ref` | one linear predecessor of kind `native`, `succession`, or `event` |
| `preservation_hashes` | array of `versioned_blob_binding` | exact chain-preservation set through event content head `C` |
| `producer_closure` | object | exact fixed-point closure shape below |
| `raw_observed_pair` | `pair` | raw pair at `C`, stated separately from acceptance authority |
| `record_consumer_inventory_sha256` | string | predecessor exhaustive inventory `sha256` |
| `repository` | string | exactly `Island-Dev-Crew/garnet` |
| `repository_id` | integer | positive immutable upstream repository ID |
| `reviewer_family_registry_sha256` | string | raw predecessor reviewer-family registry `sha256` |
| `review_scope` | object | exact event review-scope shape below |
| `schema` | string | exactly `garnet.wv_acceptance_event/v1` |
| `verdict` | string | exactly `pass` for an eligible certificate |
| `weakening_authority` | null-or-`weakening_authority` | non-null exactly for `BOUNDED WEAKENING` |
| `wv` | string | exact WV contract ID |

Each `allowed_source_delta` member has exactly `class_rule_id`,
`direction_axis`, `direction_value`, `new_blob_sha256`, `new_git_oid`,
`old_blob_sha256`, `old_git_oid`, and `path`.
It agrees with the edge census and one predecessor-ratified class rule.
`class_rule_id`, `direction_axis`, and `direction_value` are nonempty strings;
`direction_axis` is exactly `object_policy`, `source_effect`, or
`toolchain_effect`, and `direction_value` equals the top-level direction's
non-null value on that axis. `path` is a `git_path`;
the object IDs are full or all-zero at the corresponding absent side; and each
blob SHA-256 is a `sha256` or JSON `null` exactly when that side is absent. The
array contains one row per `(path, direction_axis)` produced by the exact
operation-direction projection below. Multiple rows may repeat one path only to
state two distinct applicable axes; their old/new object and blob facts must be
identical. Two values for one path/axis, an omitted axis, or an inferred combined
direction is RED.

`external_observation` has exactly `observed_at`, `observation_kind`,
`source_identity`, and `transport_receipts`. `observed_at` is a `timestamp`;
`source_identity` is the predecessor-ratified exact source; and
`transport_receipts` is a nonempty array of `transport_receipt` in the
canonical receipt order defined by the transport projection law; each
collection receipt separately preserves its authenticated member order.
Both `observation_kind` and `source_identity` are nonempty strings whose exact
per-class values come from the predecessor class registry; candidate values
cannot widen either set.

Freshness is an execution property, not a timestamp assertion. During event
production, the predecessor-owned producer opens one invocation, records its
own UTC start, performs every registered transport directly with local caches
disabled, and records `observed_at` from the same clock after the final response
and before invocation completion. No candidate-supplied timestamp, receipt,
response body, fixture, prior invocation, or persisted cache is an input. The
producer emits the exact receipts from that invocation.

The stored `observed_at` and stored receipts are historical establishment
evidence only. Live re-observation is folded over obligations that remain open
at the selected effective tip, never blindly over every historical event. For
Class A, the fold is keyed by the exact locked `(name, version, source,
checksum, resolved_depender)` tuple and walks its exception-event
`predecessor_event` chain. Actions alternate: `activate` opens or replaces the
tuple's live weakening obligation, and its one matching `expire` closes that
activation and becomes the tuple's live strengthening obligation. A later
matching activation may replace that expiry only through a new separately
eligible event. Thus exactly the terminal Class A action per tuple is live;
closed predecessors retain full historical receipt verification but their old
`valid_while` predicates are not re-applied after a valid reversal. A first
`expire`, duplicate action, wrong tuple or predecessor link, fork, or out-of-
order transition is RED. Class B is separately folded by
`(repository_id, live-ruleset-object.numeric_object_id)`: only the terminal
effective variant for that exact governed ruleset is live, while every earlier
receipt remains historical evidence. A later certificate may replace it only
with one of the already-ratified exact variants, exact predecessor subject,
complete new projection, and closed impact proof; a pending replacement is
evaluated as the proposed next fold state. A wrong ruleset subject, two terminal
variants, candidate-created variant, or historical-predicate reapplication is
RED. Class C is folded by `(repository_id,
toolchain-version-object.string_object_id)`: only the terminal effective
catalog/pin event for that exact toolchain subject is live. A later replacement
requires a catalog entry and class variant ratified in a prior separate contract
act, exact predecessor subject, complete fixed-point closure, and proposed-next-
fold evaluation while pending. Earlier Class C receipts remain historical.
An unratified replacement, wrong toolchain subject, two terminal pins, or an
obsolete compiler predicate re-applied after replacement is RED.
The Class A activation's `weakening_authority`, ruling, scope, and matched
expiry/valid-while predicate are live obligations exactly while that activation
is the tuple's terminal action. A matching effective expiry preserves those
historical bytes but discharges their live authority obligation; the expiry
row's strengthening predicate is then the sole live Class A obligation for the
tuple.

At every pending-certificate verification, transcript capture, and ordinary
current-acceptance reporter run, predecessor-owned code derives that live set,
repeats each live row's entire registered transport multiset in the same
invocation, and re-evaluates its exact predicate from the newly returned
normalized projection. Class A repeats action-specific API/index agreement and
`valid_while`; Class B repeats the exact typed live leaf projection; Class C
repeats the exact pinned toolchain observation. A pending matching Class A
expiry is evaluated as the proposed next fold state so the reversal can close
the failed activation obligation, while the prior effective tip remains the
acceptance authority until the expiry transcript becomes effective. The
current reporter emission binds its own invocation time and current receipt
digests. Equality of a stale carried timestamp or stored body never satisfies
this re-observation law. Unavailable transport, an unexecuted transport
operation, a response supplied outside the invocation, cache substitution, a
producer-clock value outside the invocation interval, or a current live
projection that no longer satisfies its terminal obligation is RED.

`producer_closure` has exactly `closure_open`, `edges`, `graph_sha256`, `inputs`,
`outputs`, and `registry_blob_sha256`. `registry_blob_sha256` is the raw SHA-256
of the complete predecessor producer-graph registry; `closure_open` is exactly
`[]`; `edges` is the exact reachable sorted subarray derived from that registry;
`graph_sha256` uses the producer-graph framing above over that subarray; and
`inputs` and `outputs` are path-sorted arrays of `blob_binding` that exhaust the
fixed point. Every changed input is in `inputs`; following every uniquely
matching predecessor edge reaches every output; no reachable edge or output is
omitted; and no candidate edge decides closure.

Each `impact_proof` member has exactly `changed_inputs`, `criterion_id`,
`evidence`, `predicate_id`, and `verdict`. `changed_inputs` is a duplicate-free,
path-sorted array of `git_path`; `criterion_id` and `predicate_id` are nonempty
strings; `evidence` is a nonempty path-sorted array of `blob_binding`; and
`verdict` is the string `pass`. The exact `event_change_set` is the sorted
unique path projection of every `allowed_source_delta` member after that array
has been proved equal to all direct and graph-reachable changed event outputs.
Every impact row's `changed_inputs` equals that complete set; a criterion never
receives a candidate-selected subset.

The top-level array is sorted by `criterion_id`; its ID set equals all distinct
required-check IDs in the bound predecessor `wv` contract (currently five for
both WV-6 and WV-7), and each `predicate_id` is an exact predecessor-ratified
class predicate. That ID resolves one predecessor implementation-inventory
`verify` operation whose input matchers cover every path in `event_change_set`
and whose output matchers cover every evidence binding. Its verifier logical ID
differs from the selected event-class producer and their source closures are
independent under the implementation inventory's allowlist rule. The producer
first emits the row; the separate predecessor verifier executes at exact `C`
over the complete change set, current class transport, bound WV criterion, and
repository evidence, independently derives the evidence path/object/hash set
and verdict, and requires its canonical result to equal the certificate row.
`verdict=pass` is therefore a recomputed result, never candidate assertion. A
missing, extra, duplicate, renamed, or differently ordered criterion; omitted
event input; unrelated, stale, absent, or matcher-ineligible evidence; producer-
verifier dependence; or asserted-only pass is ineligible.

Event `review_scope` has exactly `coverage_extension`, `direction`,
`event_class`, `extends_native_coverage`, `native_checks_reexecuted`, and
`reviewed_through`. `coverage_extension` is a duplicate-free, path-sorted array
of `git_path`; `native_checks_reexecuted` is a duplicate-free,
lexicographically sorted array of nonempty predecessor WV required-check ID
strings; both arrays equal the class proof exactly. `extends_native_coverage`
is `false`; `reviewed_through` equals `certificate_content_head`; `event_class`
equals the top-level value; and `direction` is recursively byte-equal to the
top-level direction object.

Both event pair recomputations bind `event_c=certificate_content_head`, the
complete ordered pair-input stream at `C`, and
`event_c_pair=raw_observed_pair=new_accepted_pair`. `old_accepted_pair` is
selected from the predecessor effective chain and is never inferred from a raw
record-inflated predecessor observation.

`native_root` is never candidate-selected authority. Its canonical-byte hash
equals `predecessor_effective_tip.native_root_id`. For a succession or event
predecessor it is recursively byte-equal to that predecessor chain's immutable
root. For a native predecessor, the predecessor-controlled verifier reconstructs
the exact object from the bound WV contract, reporter projection, evidence
destination, manifest, artifact membership, proof mirrors, required-check
evidence, structured review record, repository identities, and completely
paginated plus directly read decisive review; the candidate object must equal
that reconstruction byte for byte.

The event census boundary is discriminated only by
`predecessor_effective_tip.certificate_kind`. Its `end_commit` equals
`certificate_content_head=C`. For a `native` predecessor, `start_commit` is the
native root's exact authenticated `reviewed_head=H` selected by
`native_root_id`; `H` must resolve and belong to authoritative upstream main's
first-parent history at the candidate base, and `C` must descend from `H` in the
complete authenticated candidate DAG. A discarded, unavailable, or non-main
native `H` cannot begin a direct event: one effective R1 succession is required
first, so its derived transcript-introduction commit `T` supplies the durable
main anchor. This preserves the zero-succession case for a genuinely main-
rooted native acceptance without inventing ancestry for a squashed root. For a
`succession` or `event` predecessor, `start_commit` is the derived `T` of that
predecessor's unique effective transcript. Every graph edge reachable from that
exact start through `C` appears once, including empty-diff edges. No merge base,
prior content landing, raw-observation point, current branch base, candidate-
provided boundary, or alternative ancestry cut is eligible. Record-only edges
after the predecessor boundary are first qualified under the predecessor
consumer inventory; the first non-record byte or unqualified operation and all
following content deltas are classified under the single event class.

Top-level `direction` has exactly `object_policy`, `source_effect`, and
`toolchain_effect`. Each value is a string or JSON `null`, and the complete
class/action matrix is closed:

| `event_class` | `event_action` | `object_policy` | `source_effect` | `toolchain_effect` |
|---|---|---|---|---|
| `registry_yank` | `activate` | `BOUNDED WEAKENING` | null | null |
| `registry_yank` | `expire` | `STRENGTHENING` | null | null |
| `forge_serialization_drift` | `observe-neutral` | `EFFECTIVE-POSTURE NEUTRAL` | null | null |
| `forge_serialization_drift` | `observe-strengthening` | `STRENGTHENING` | null | null |
| `toolchain_lint_activation` | `activate` | null | `BEHAVIOR-NEUTRAL` | `REPRODUCIBILITY-STRENGTHENING` |

No other tuple exists. The predecessor-ratified class variant and every
`allowed_source_delta[*].class_rule_id` select the same row; class, action,
direction, delta direction, and review-scope direction disagreement is RED.

### Event-class registry

The predecessor event-class registry path is exactly
`F_Project_Management/W_TRUST/WV_ACCEPTANCE_EVENT_CLASSES.json`; its schema ID
is `garnet.wv_acceptance_event_classes/v1`. Its canonical object has exactly
`classes` and `schema`; `schema` equals that ID. `classes` is sorted by
`(event_class, event_action, class_rule_id)`, and contains exactly one row for
each of the five matrix tuples above. The corresponding `class_rule_id` values
are exactly:

| `event_class` / `event_action` | `class_rule_id` |
|---|---|
| `registry_yank` / `activate` | `garnet.wv.event.registry-yank-activate/v1` |
| `registry_yank` / `expire` | `garnet.wv.event.registry-yank-expire/v1` |
| `forge_serialization_drift` / `observe-neutral` | `garnet.wv.event.forge-serialization-neutral/v1` |
| `forge_serialization_drift` / `observe-strengthening` | `garnet.wv.event.forge-serialization-strengthening/v1` |
| `toolchain_lint_activation` / `activate` | `garnet.wv.event.toolchain-lint-activate/v1` |

At `A`, the activation registry fixes `ratification_state=active` for the four
non-activation rows. The `registry_yank/activate` row is `active` at `A` if and
only if `wv_class_a_legacy_precondition_v1` passes for a separate landing `L`
strictly before `G` and the row carries one scope-exact Jon weakening authority
already present at `G`; otherwise that row is `blocked` with
`weakening_authority=null`. Those are the only two canonical activation-state
vectors. After `A` is selected, v1 has no free-standing law-update selector:
the vector and any active row's one exact authority remain immutable for that
chain. Activating a blocked row, changing its scope, or authorizing another
Class A addition requires a future explicit contract plus native-root
reactivation; an intervening registry landing cannot authorize its own or a
later candidate under this v1 selector.

Each class row has exactly these keys:

| key | JSON type | invariant |
|---|---|---|
| `class_rule_id` | string | exact unique ID from the table above |
| `delta_rules` | array | exact direct-delta rules sorted by `(path, status, operation_id)` |
| `direction` | object | exact matrix direction object |
| `event_action` | string | exact matrix action |
| `event_class` | string | exact matrix class |
| `impact_predicates` | array | complete `(wv, criterion_id, predicate_id)` mapping |
| `observation_kind` | string | exact class observation kind below |
| `producer_identity` | string | exact event-class-producer `logical_id` |
| `producer_sha256` | string | raw producer entrypoint `sha256` |
| `producer_source_closure_sha256` | string | framed producer source-closure `sha256` |
| `ratification_state` | string | exactly `active` or `blocked` |
| `source_identity` | string | exact external source identity below |
| `transport_requirements` | array | exact closed role/cardinality rows below |
| `valid_while_predicate_id` | null-or-string | exact live predicate for Class A; null otherwise |
| `variant` | object | exact class-discriminated object below |
| `weakening_authority` | null-or-`weakening_authority` | non-null only for an active Class A activation row |

The producer identity resolves through exactly one implementation row whose
`roles` contains `event-class-producer`; both producer hashes equal that row's
entrypoint and source-closure hashes. A `delta_rules` member has exactly
`delta_role`, `direction_axis`, `direction_value`, `operation_id`, `path`, and
`status`. `path` is an exact `git_path`; `status` is exactly `A`, `D`, or `M`;
`operation_id` selects one producer operation carrying this same
`class_rule_id`, axis, and value; and `delta_role` is exactly `compiler-pin`,
`deny-policy`, `event-record`, `fixture`, `procedural-contract`, or
`source-rewrite`. Every direct candidate delta equals one row. Every additional
changed output is reached through the predecessor producer graph from those
direct inputs, and each reachable edge contributes its bound axis/value. Their
sorted `(path, direction_axis)` union equals `allowed_source_delta`; zero,
ambiguous, omitted, or extra path/axis matches are RED.

The direction mapping is closed. Every Class A direct or derived operation uses
`direction_axis=object_policy` and the selected row's exact object-policy value.
Class B has `delta_rules=[]`. For Class C, `compiler-pin` uses
`toolchain_effect=REPRODUCIBILITY-STRENGTHENING`; `source-rewrite` and its
paired `procedural-contract` use `source_effect=BEHAVIOR-NEUTRAL`. A graph-
derived output inherits every distinct axis/value carried by the exact
predecessor edges that reach it and therefore has one allowed-delta row for
each such axis. A swapped axis, null axis, made-up value, collapsed two-axis
row, or output lacking the complete inherited direction set is RED.

Each `impact_predicates` member has exactly `criterion_id`, `predicate_id`, and
`wv`. It is sorted by `(wv, criterion_id)`, unique, and covers every criterion
of every WV contract the class row names. The event `impact_proof` ID set and
predicate mapping equal the subarray for its exact `wv`.

Each `transport_requirements` member has exactly `maximum`, `minimum`, and
`transport_kind`; both bounds are positive integers and equal `1` in v1. The
array is transport-kind-sorted and equals the following closed mapping:

| event class | `observation_kind` | `source_identity` | exact transport kinds |
|---|---|---|---|
| `registry_yank` | `registry-yank-state` | `crates.io-api+sparse-index` | `registry-api-object`, `sparse-index-object` |
| `forge_serialization_drift` | `forge-ruleset-projection` | `github-rulesets-api` | `live-ruleset-object` |
| `toolchain_lint_activation` | `toolchain-version` | `predecessor-pinned-rust-toolchain` | `toolchain-version-object` |

For both Class A rows, `valid_while_predicate_id` is non-null and selects one
exact predecessor verifier operation; it evaluates API/index agreement plus the
locked dependency tuple on every run. Every other row has
`valid_while_predicate_id=null`. A row is selectable only when
`ratification_state=active`. A candidate class-registry change cannot authorize
an event in the same candidate: class selection and
`class_registry_sha256` use the exact law-base tree's complete raw file.
Adding a sixth tuple, changing an ID, or moving a blocked row to active is
outside this v1 chain and requires the future native-root reactivation above.

The event's `class_registry_sha256` equals the SHA-256 of those complete raw
predecessor-registry bytes. Its `class_producer_identity` and
`class_producer_sha256` equal the selected row's `producer_identity` and
`producer_sha256`; the selected implementation row also yields the class row's
exact `producer_source_closure_sha256`. Any valid but different class producer,
hash splice, or candidate-registry value is RED.

The Class A `variant` has exactly `comparator`, `deny_row_template`,
`exception_event_schema`, `locked_fields`, `parser_identity`, `parser_sha256`,
`standing_row_precondition_id`, and `variant_id`. `comparator=Exact`,
`deny_row_template=name:=X.Y.Z`, and
`exception_event_schema=garnet.wv_registry_yank_exception_events/v1`.
`locked_fields` is exactly the sorted set `cargo-lock`, `cargo-manifests`,
`dependency-graph`, `global-yanked-deny`, `resolved-depender`, `source-bytes`,
and `source-checksum`. `standing_row_precondition_id` is exactly
`class-a-standing-arrayref-row-cleared-v1`; `variant_id` is
`registry-yank-activate-v1` or `registry-yank-expire-v1` according to the row.
The parser identity selects a predecessor implementation whose roles contain
`record-consumer` and `verifier`; `parser_sha256` equals its source-closure
hash. At `A`, the activation row remains `blocked` with
`weakening_authority=null` unless the exact precondition landing and one
scope-exact Jon ruling are already fixed predecessor facts. In the sole active
case, its `weakening_authority` is non-null and byte-equal to the one event
certificate's `weakening_authority` field that it can authorize.
All other rows have `weakening_authority=null`.

Mechanical test `wv_class_a_legacy_precondition_v1` scans authoritative-main
first-parent history strictly before `G` and emits one of two exact passing
vectors. If no qualifying DP7 landing exists, the activation row is
`blocked` with `weakening_authority=null`. If and only if both a qualifying
landing `L` and one scope-exact authority already at `G` exist, the row is
`active` with that byte-equal authority. A qualifying `L` is the unique landing
of the separately ceremonied DP7 PR. Relative to its exact base, its content
delta removes only the one complete `arrayref@0.3.9` member from `deny.toml`'s
`bans.skip` array; parsing both sides shows no second comparator or policy
movement. Fresh crates.io API and sparse-index observations agree that the yank
is reversed, while `Cargo.lock`, every Cargo manifest, the locked
source/checksum, resolved depender edge, source bytes, and global `yanked=deny`
posture remain byte-identical. The PR has its own exact-head canonical review
record, authenticated decisive review, and main landing. The removal is
`STRENGTHENING`, uses no Class A event/certificate or exception-registry row,
creates no acceptance movement, and exercises the expiry parser/transport/
locked-tuple subpredicate only as a pre-activation construction proof. The test
rejects a state/authority-vector mismatch, partial or multiple `L`, a
missing/extra row, policy/dependency movement, stale or disagreeing transport,
candidate-only authority, non-main/late landing, or any attempt to use the
historical exception as an eligible Class A precedent. Status:
`OPEN-UNTIL-IMPLEMENTED`.

The Class A exception-event registry path is exactly
`F_Project_Management/W_TRUST/WV_REGISTRY_YANK_EXCEPTION_EVENTS.json`. Its
canonical top-level object has exactly `events` and `schema`, with
`schema=garnet.wv_registry_yank_exception_events/v1`. `events` is append-only
in contiguous positive `sequence` order. Each member has exactly `action`,
`checksum`, `deny_policy`, `exception_id`, `global_yanked_posture`, `name`,
`predecessor_event`, `resolved_depender`, `sequence`, `source`,
`valid_while_predicate_id`, and `version`.

`action` is exactly `activate` or `expire`; `deny_policy` is the exact candidate
`blob_binding`; `name`, `version`, and `source` are nonempty exact Cargo package
identity strings; `checksum` is a `sha256`; and
`global_yanked_posture=deny`. `predecessor_event` is JSON null only for the
first event for one exact `(name, version, source, checksum,
resolved_depender)` tuple and otherwise is the immediately preceding
`exception_id` for that same complete tuple. The valid-while ID equals the
selected Class A row.

`resolved_depender` has exactly `dependent_name`, `dependent_version`,
`dependency_name`, `dependency_source`, `dependency_version`, `lock_checksum`,
and `requirement`. Every string is nonempty; `dependency_name`, source, version,
and `lock_checksum` equal the exception tuple; `lock_checksum` is a `sha256`;
and the complete object equals the predecessor lock/manifest graph projection.
To derive `exception_id`, remove only that key and hash
`garnet.wv.registry_yank_exception_event/v1 NUL
<canonical-entry-without-exception_id>`; the ID is
`registry-yank-event:<lowercase-hex>`.

An activation content delta appends exactly one `activate` row while adding the
one canonical deny-policy exception; an expiry delta appends exactly one
`expire` row while deleting that exact exception. Existing event rows and every
bound dependency fact are byte-immutable. The fresh API and sparse-index
projections equal the row on activation; on expiry they prove reversal while
the lock/source/checksum/depender tuple remains equal. A missing/duplicate row,
noncontiguous sequence, predecessor fork, registry replacement, retained expiry
row, different depender, or prose-only reason is RED. The activation act creates
the exact empty genesis object; no Class A certificate may share that genesis
candidate.

The Class B `variant` has exactly `comparator_identity`,
`comparator_sha256`, `leaf_contract`, and `variant_id`. The comparator selects
a predecessor verifier/record-consumer implementation and binds its source-
closure hash. Each `leaf_contract` member has exactly `expected_value`,
`json_pointer`, `leaf_name`, and `value_kind`. For `observe-neutral`, the exact
two members are `dismissal_restriction` with value kind `disabled-restriction`
and value `{"allowed_actors": [], "enabled": false}`, and
`required_reviewers` with value kind `empty-array` and value `[]`. For
`observe-strengthening`, the sole member is
`require_extra_approval_for_unattributed_changes` with value kind `boolean` and
value `true`. Each JSON pointer is the predecessor row's exact existing nested
rule path and ends in the named leaf. `variant_id` is respectively
`forge-serialization-neutral-v1` or
`forge-serialization-strengthening-v1`; `delta_rules=[]` for both external-
observation rows. Missing, additional, relocated, or differently typed nested
values are RED.

The Class C `variant` has exactly `catalog`, `catalog_entry_id`,
`catalog_schema`, `compiler_pin`, `root_toolchain_paths_absent`,
`rewrite_matcher`, `variant_id`, `verifier_identity`, and `verifier_sha256`.
`catalog` is a `blob_binding` whose path is exactly
`F_Project_Management/W_TRUST/WV_CLASS_C_REWRITE_CATALOG.json`;
`catalog_entry_id=C-RW-0001/rust-u8-chunks2-guarded-v1`,
`catalog_schema=garnet.wv_class_c_rewrite_catalog/v1`,
`rewrite_matcher=CLASS_C_CATALOG_MATCH_V1`, and
`variant_id=toolchain-lint-activate-v1`. The absent-path array is exactly
`["rust-toolchain", "rust-toolchain.toml"]`. `compiler_pin` is the exact
predecessor-ratified compiler/Clippy version. The verifier identity selects a
predecessor verifier/record-consumer implementation, and `verifier_sha256`
equals its source-closure hash. The row's direct delta roles contain exactly
one `compiler-pin`, one `source-rewrite`, and one `procedural-contract`; every
other output is producer-graph-derived.

The Class C catalog is canonical JSON with exactly `entries` and `schema`.
`schema` equals `garnet.wv_class_c_rewrite_catalog/v1`; entries are sorted by
unique `catalog_entry_id` and append-only. The activation version contains
exactly the C-RW-0001 entry above. Each entry has exactly `catalog_entry_id`,
`construction_proof`, `emitted_shape`, `matcher_id`, `minimum_rust_version`,
`old_shape`, `source_language`, `typed_precondition_schema`,
`verifier_identity`, `verifier_sha256`, and `workspace_msrv`.
`construction_proof`, `emitted_shape`, `old_shape`, and
`typed_precondition_schema` are exact `blob_binding` objects consumed under
their registered schemas; `matcher_id=CLASS_C_CATALOG_MATCH_V1`,
`minimum_rust_version=1.88.0`, `source_language=rust`, and
`workspace_msrv=1.95.0`. The verifier fields equal the Class C variant. The
bound typed blobs encode the one unchanged `&str`/`as_bytes` receiver, constant
two-byte chunk, unchanged dominating odd-length guard/error, exact
`chunks_exact(2)` to `as_chunks::<2>().0` replacement, unchanged body with only
indices zero/one, and every forbidden movement stated in the adopted Class C
contract. A hash without its exact registered blob, candidate catalog, second
entry match, or typed/preimage disagreement is RED.

For C-RW-0001 the four catalog bindings have these exact paths:

| catalog field | exact `git_path` |
|---|---|
| `construction_proof` | `F_Project_Management/W_TRUST/class-c/C-RW-0001.construction-proof.json` |
| `emitted_shape` | `F_Project_Management/W_TRUST/class-c/C-RW-0001.emitted-shape.json` |
| `old_shape` | `F_Project_Management/W_TRUST/class-c/C-RW-0001.old-shape.json` |
| `typed_precondition_schema` | `F_Project_Management/W_TRUST/class-c/C-RW-0001.typed-preconditions.json` |

The construction-proof blob has exactly `catalog_entry_id`,
`emitted_shape_sha256`, `obligations`, `old_shape_sha256`, `schema`,
`typed_precondition_sha256`, and `verdict`.
`schema=garnet.wv.class_c_construction_proof/v1`, every SHA field equals the
raw bound sibling blob, and `verdict=pass`. `obligations` is exactly the sorted
array `empty-input`, `even-two-byte-partition`, `first-error-position`,
`msrv-availability`, `no-forbidden-movement`, `odd-guard-and-error`,
`receiver-identity`, and `uses-indices-zero-one`.

The typed-preconditions blob has exactly `body_indices`, `catalog_entry_id`,
`chunk_size`, `dominating_guard`, `forbidden_features`, `receiver_kind`, and
`schema`. `schema=garnet.wv.class_c_typed_preconditions/v1`,
`body_indices=[0, 1]`, `chunk_size=2`,
`dominating_guard=unchanged-odd-length-return`, and
`receiver_kind=unchanged-str-as-bytes`. `forbidden_features` is exactly the
sorted array `allocation`, `api-change`, `dispatch-change`, `guard-change`,
`iterator-remainder`, `lint-suppression`, `public-surface-change`,
`serialized-output-change`, and `unsafe`.

Both source-shape blobs have exactly `body_sha256`, `catalog_entry_id`,
`guard_sha256`, `kind`, `loop_api`, `path`, `receiver_sha256`,
`remainder_used`, `schema`, `span_end`, `span_sha256`, `span_start`,
`unsafe_used`, and `uses_indices`. Their schema is exactly
`garnet.wv.class_c_source_shape/v1`; `path` is the one exact source `git_path`;
span offsets are nonnegative integers with `span_start<span_end`; and
`span_sha256` hashes the exact raw selected bytes. The old row has `kind=old`
and `loop_api=chunks_exact(2)`; the emitted row has `kind=emitted` and
`loop_api=as_chunks::<2>().0`. Their path, receiver, guard, body, and
`uses_indices=[0, 1]` are equal; both booleans are false. The verifier masks
only those exact spans and requires every other candidate source byte to equal
the predecessor. Missing/unknown keys, stale or exchanged sibling hashes,
different paths, a second span, shape drift, or an unregistered schema consumer
is RED. The `catalog_entry_id` inside the construction proof, typed-
preconditions blob, old-shape blob, and emitted-shape blob is byte-equal in all
four places and equals the owning catalog row's exact
`C-RW-0001/rust-u8-chunks2-guarded-v1`; a cross-entry or exchanged binding is
RED.

Mechanical test `wv_event_class_registry_v1` validates the fixed registry and
catalog paths, exact recursive keys, five-row identity/matrix, raw registry
binding, exact activation-state vector, active-state predecessor selection,
producer/parser/verifier operation bindings, direct-delta/closure union, impact
and direction projection, impact mapping, observation/transport
mapping, Class A authority and expiry precondition, Class B nested values, and
Class C typed catalog preimages. It rejects a candidate-selected or widened
row, unknown tuple, blocked row, wrong hash/role/operation/cardinality, broad
path, extra leaf, missing catalog blob, standing-row shortcut, or same-candidate
activation. It also rejects a cross-entry `catalog_entry_id` or sibling-ID
disagreement. Status: `OPEN-UNTIL-IMPLEMENTED`.

For `registry_yank/activate`, `weakening_authority` is non-null and equals the
predecessor class registry's exact Jon authority, scope, and expiry binding; for
every other matrix row it is JSON `null`. At the certificate PR's exact `Q`,
the effectiveness `selected_review` supplies one primary cross-family approval
and, only for that weakening row, exactly one supplemental approval from a
second reviewing family. Both are independent of all authenticated commit
principal families.

Mechanical test `wv_event_certificate_v1` validates canonical bytes, exact
recursive shape, registry membership and append-only history, one-class-only
matching, direction law, complete native-root reconstruction and ID equality,
the discriminated `H-or-T..C` boundary, direct-native main ancestry, complete
edge census, predecessor graph closure, same-invocation fresh external transport,
exact bound-contract
criterion coverage with complete change sets and independently recomputed
evidence/verdicts, preservation, pair derivation by two pipeline-independent
implementation closure unions, review scope, and fail-closed class misses. It
rejects pair cross-role reuse/swap; a
wrong/missing native root; unavailable/non-main native `H`; false ancestry; a
merge base, `B`, `M`, current-head, or candidate-selected `start_commit`; and an
omitted empty-diff edge, carried timestamp/body, uncalled transport, or replayed
receipt. It also rejects an omitted event input, unrelated/stale evidence,
producer-verifier dependence, pass-only assertion, or missing/swapped/collapsed
direct or graph-derived direction. Status:
`OPEN-UNTIL-IMPLEMENTED`.

Mechanical test `wv_cross_family_review_v1` validates the predecessor family
registry hash and unique ID-to-family mappings, reconstructs every
implementation family from authenticated PR commit principals, re-selects and
directly reads the primary and supplemental decisive approvals, and enforces
the non-weakening/weakening cardinalities and family disjointness above. For
`BOUNDED WEAKENING` it also validates the predecessor-registered Jon ruling,
exact scope digest, and exact expiry/valid-while predicate binding at
establishment. It re-evaluates that authority predicate only while the
activation is the terminal live tuple obligation; a matching effective expiry
preserves but discharges it. It rejects unmapped principals,
candidate registry authority, same-family overlap, stale or non-head review,
missing/extra supplemental review, ruling/scope or expiry-predicate mismatch,
expired live authority, premature discharge, or reapplication after a valid
matching expiry.
Status: `OPEN-UNTIL-IMPLEMENTED`.

### Adopted Class A contract

The following proposed contract text is transcribed verbatim from the brief:

```text
REGISTRY-YANK ADDITION is BOUNDED WEAKENING. It is eligible only for one
canonical name:=X.Y.Z row whose predecessor-pinned parser yields one complete
Exact comparator, plus one matching machine-consumed activation event. The
event binds one exact locked name/version/source/checksum and resolved depender,
with Cargo.lock, manifests, dependency graph, source bytes, and global
yanked-deny byte-identical. Every other yank remains denied. The valid_while
predicate is evaluated on every run. Jon's explicit weakening approval is
required before Class A may be ratified.

REGISTRY-YANK EXPIRY is STRENGTHENING only when the exact ignore row is deleted
and the matching expiry event is appended in the same candidate, restoring
unconditional global yank denial after a fresh registry reversal while the
lock/source/checksum and resolved edge remain byte-identical. Movement in any
of those dependency facts makes the exception invalid but is not a Class A
expiry event. If the row remains, the candidate is RED; prose about inertness
cannot keep it eligible.
```

Mechanical test `wv_event_registry_yank_v1` uses the predecessor-pinned parser
and class registry to require the single complete Exact comparator, exact
locked tuple and depender edge, dual fresh API/sparse-index agreement,
`valid_while`, unchanged global denial and dependency closure, one matching
event, and the cross-family/Jon weakening proof. It traps activation and expiry
separately, accepts an effective matching `activate` then `expire` obligation
fold without reapplying the closed activation predicate, and rejects parser/pin
drift, stale or unavailable transport,
API/index disagreement, tuple or dependency movement, a retained false/expired
row, another excepted yank, premature/wrong-tuple/wrong-predecessor expiry,
duplicate action, and an unregistered or duplicate event. The
standing `arrayref@0.3.9` row remains a precondition to remove in its separately
authorized act and is not an eligible construction precedent. Status:
`OPEN-UNTIL-IMPLEMENTED`.

### Adopted Class B contract

The following proposed contract text is transcribed verbatim from the brief:

```text
FORGE SERIALIZATION DRIFT is eligible only when complete authenticated
projection yields exactly the certificate's finite typed leaf set and zero
other divergence; each leaf occupies its exact existing rule path; transport
is complete; _strict_equal and all executable comparison logic are
byte-identical; bypass is []/never; required-context rows remain exact; and
fixtures fail closed for absence, weaker values, wrong types, extra values,
and path displacement. The leaf shape MUST be an already-ratified class
variant; live observation cannot widen the class.

Disabled/empty dismissal_restriction and required_reviewers values are
EFFECTIVE-POSTURE NEUTRAL only at those exact values. A true
require_extra_approval_for_unattributed_changes leaf is STRENGTHENING. An
absent or false value is RED. No updated_at value proves why or when a server
field appeared.
```

Mechanical test `wv_event_forge_serialization_v1` completely authenticates the
fresh same-invocation live projection and accepts only the predecessor-ratified
disabled/empty
`dismissal_restriction` plus `required_reviewers` variant or the exact true
`require_extra_approval_for_unattributed_changes` variant. It enforces strict
comparator-byte identity, `[]/never` bypass, exact required contexts, and the
stated direction; it rejects absence, false/weaker values, wrong types, extra
or displaced leaves, incomplete transport, lossy top-level inference,
candidate projection law, and `updated_at` causation. The class admits zero
autonomous future variants until a prior Jon ruling and separate contract act.
The test also exercises two sequential effective observations for the same
`(repository_id, live-ruleset-object.numeric_object_id)`, requires the later
already-ratified variant to be the sole live fold value, preserves the earlier
receipt as historical evidence, and rejects a subject change, two terminal
variants, or reapplication of the superseded live predicate.
Status: `OPEN-UNTIL-IMPLEMENTED`.

### Adopted Class C contract

The following proposed contract text is transcribed verbatim from the brief:

```text
TOOLCHAIN LINT ACTIVATION is eligible only for one exact compiler/Clippy pin
and one exact CLASS_C_CATALOG_MATCH_V1 rewrite. The predecessor catalog and
verifier MUST match byte-for-byte, the typed preconditions and emitted source
MUST match exactly, and the candidate MUST pass at workspace MSRV and the pinned
lint compiler. The compiler pin is
REPRODUCIBILITY-STRENGTHENING; the source rewrite is BEHAVIOR-NEUTRAL. These
dimensions MUST be stated separately.

Starting from the predecessor's base-controlled graph G(P), the producer census
MUST compute a fixed-point closure from every changed input through every
semantic pin, provenance file, manifest, mirror, runtime-input aggregate, and
proof without consulting candidate graph law.
Every dependent output MUST be regenerated or independently verified at the
exact candidate. Acceptance between cure and closure is void; a proof captured
at an older tree cannot close the current candidate. closure_open MUST equal
[]. A catalog miss or any obligation outside one exact entry requires native
replay.
```

Mechanical test `wv_event_toolchain_lint_v1` binds the predecessor catalog and
verifier bytes, admits exactly `C-RW-0001`, validates every typed precondition
and emitted byte, freshly observes and runs the pinned lint compiler in the same
invocation, runs the workspace MSRV, and derives the complete predecessor-graph
fixed point with `closure_open=[]`. It rejects a
second rewrite/site, catalog or verifier drift, candidate graph law,
suppression, API/behavior/lock/dependency movement, missing or stale dependent
output, older-tree proof, compiler failure, or direction conflation. It also
exercises a later predecessor-ratified replacement for the same
`(repository_id, toolchain-version-object.string_object_id)`, requires one
terminal pin/catalog obligation, preserves earlier receipts historically, and
rejects wrong-subject replacement, two terminal pins, or reapplication of the
superseded compiler predicate. Status:
`OPEN-UNTIL-IMPLEMENTED`.

The following adopted contract text is transcribed verbatim from the brief:

```text
EVENT CERTIFICATE LANDING. Every event certificate, registry entry, class
producer, and governed policy byte MUST be in the base-controlled trust-kernel
trigger set and rolling-review digest. Exact-head approval MUST derive the
certificate PR tip Q. The certificate is ineffective until authenticated main
first-parent landing M exists, tree(Q) equals tree(M), the landing edge is
completely censused, exactly one terminal effectiveness transcript anchors the
establishment-time forge facts, and current HEAD descends through M. Ordinary
verification MUST NOT depend on a surviving Q or live forge review. Any record-
tail raw pair movement MUST be restated separately from the accepted pair at
content head C. Missing review routing or anchor, unequal trees, incomplete
landing transport, or ambiguous ancestry is RED.
```

## Effectiveness transcript schema

The transcript schema ID and top-level keys are fixed by the brief:
`garnet.wv_acceptance_effectiveness/v1`. Transcript paths match exactly
`F_Project_Management/W_TRUST/effectiveness/*.wv-acceptance-effectiveness.json`.
The registry is
`F_Project_Management/W_TRUST/WV_ACCEPTANCE_EFFECTIVENESS.json`, schema
`garnet.wv_acceptance_effectiveness_registry/v1`, with exactly `receipts` and
`schema`; `receipts` is a path-sorted array of exact objects containing
`blob_sha256`, `path`, and `wv`.

An effectiveness transcript has exactly these top-level keys, with no
additional field:

| key | JSON type | invariant |
|---|---|---|
| `b_to_q_census` | `edge_census` | start is `B` for ordinary succession, `S` for the sole DP3 migration, or `C` for event; end is `Q` |
| `blocking_findings` | array | exactly `[]` for an effective transcript |
| `certificate_blob_sha256` | string | raw certificate `sha256` |
| `certificate_kind` | string | exactly `succession` or `event` |
| `certificate_landing_m` | string | authoritative main first-parent `commit` `M` |
| `certificate_path` | string | exact registered certificate `git_path` |
| `certificate_tip_q` | string | exact decisively approved PR head `Q` |
| `certificate_tree` | string | resolved `tree(Q)` |
| `classifier_sha256` | string | predecessor classifier `sha256` |
| `digest_law_sha256` | string | predecessor digest-law `sha256` |
| `head_repository` | string | authenticated certificate PR head repository |
| `head_repository_id` | integer | positive immutable head repository ID |
| `landing_edge_census` | `edge_census` | exact `landing_parent..M` census |
| `landing_parent` | string | exact first parent of `M` |
| `merged_tree` | string | resolved `tree(M)`, equal to `certificate_tree` |
| `predecessor_effective_tip` | `effective_tip_ref` | exact linear predecessor |
| `producer_identity` | string | exactly `garnet.wv_acceptance_effectiveness.producer/v1` |
| `producer_sha256` | string | predecessor-base producer raw-byte `sha256` |
| `pull_request_id` | integer | positive immutable certificate PR ID |
| `pull_request_number` | integer | positive certificate PR number |
| `record_consumer_inventory_sha256` | string | predecessor exhaustive inventory `sha256` |
| `repository` | string | exactly `Island-Dev-Crew/garnet` |
| `repository_id` | integer | positive immutable upstream repository ID |
| `review_pages` | array of `page_receipt` | complete authenticated review pagination |
| `schema` | string | exactly `garnet.wv_acceptance_effectiveness/v1` |
| `selected_review` | `selected_review` | latest decisive exact-`Q` approval |
| `source_landing_b` | string | `B` for succession; by discriminated schema, `C` for event |
| `transport_receipts` | array of `transport_receipt` | complete forge transport plus, for event, the complete fresh live-obligation transport union |
| `verdict` | string | exactly `pass` for an effective transcript |
| `wv` | string | exact WV contract ID |

The fixed key name `source_landing_b` and fixed key name `b_to_q_census` are
discriminated by certificate kind and succession establishment mode. They carry
`B` plus `B..Q` for ordinary succession; `B` plus the separately bound bridge
and producer-qualified `S..Q` tail for the sole DP3 migration; and `C` plus
`C..Q` for event. This completes the brief's generic event use of the R1
transcript without adding a top-level key.

The certificate discriminant closes every cross-document binding. For either
kind, `wv`, `repository`, `repository_id`, `predecessor_effective_tip`,
`classifier_sha256`, `digest_law_sha256`, and
`record_consumer_inventory_sha256` equal the corresponding certificate fields;
the selected-review family mapping is loaded from the exact certificate-bound
`reviewer_family_registry_sha256`;
`certificate_path` and `certificate_blob_sha256` equal the unique registry
entry and raw bytes loaded at that path. For succession,
`source_landing_b=certificate.source_b`; for event,
`source_landing_b=certificate.certificate_content_head`. The census starts at
that exact boundary except that DP3 starts at
`certificate.activation_bridge.migration_base_s`; it ends at
`certificate_tip_q`. `selected_review.commit_id`
equals that same `Q`. `certificate_tree=tree(Q)`,
`landing_parent=parent1(certificate_landing_m)`, the landing census starts at
that parent and ends at `M`, and `merged_tree=tree(M)=certificate_tree`.
Authenticated merge transport identifies that same `M` on authoritative main
first-parent history after the source boundary.

The fixed transport-role multiset closes each forge identity. The
`repository-object`, `head-repository-object`, and `pull-request-object`
numeric IDs respectively equal `repository_id`, `head_repository_id`, and
`pull_request_id`; the pull-request projection also equals the bound number,
base, head repository/head, open-to-merged transition, and authenticated
`merge_commit_sha=M`. Its `base.ref` is exactly `main`. Its `base.sha` is
kind-discriminated: `certificate.source_b` for an ordinary succession,
`certificate.activation_bridge.migration_base_s` for the DP3 migration, and
`certificate.edge_census.start_commit` for an event. Its `head.sha` is exactly
`Q`, and its head repository name and immutable ID equal `head_repository` and
`head_repository_id`. For an event, the complete authenticated PR graph from
that base reaches `C` through the event census and then reaches `Q` through
`b_to_q_census`; for succession it follows the ordinary or migration split
defined above. Any base/head/ref/repository disagreement or unrepresented PR
edge is RED. The primary direct-review and direct-user projections
equal `selected_review`; any supplemental direct-review/direct-user projections
equal its one supplemental row. `review_pages` exhaust the PR's review
collection, contain those same decisive rows, and have no duplicate review ID.

The `pull-request-commits-page` sequence exhausts the certificate PR commit
collection in authenticated order and supplies every author/committer principal
used for role separation. The certificate-tip, landing, and landing-parent
commit objects respectively bind `Q`, `M`, and `parent1(M)`, including their
trees and ordered parent arrays. The authoritative-main-ref projection binds
the exact observed main head, which is on authoritative first-parent history
and descends through `M`. Every direct and paginated projection is loaded under
the predecessor transport registry; a matching raw body without the exact
normalized field projection, identity discriminant, endpoint, role, or complete
page/end-link chain is RED.

The producer derives the transcript introduction commit `T` instead of storing
it. `T` is the unique earliest authoritative-main first-parent commit after `M`
whose tree contains the exact terminal bundle. Relative to `parent1(T)`, that
bundle's complete raw diff has exactly three paths: one new regular `100644`
transcript; one `100644` effectiveness-registry modification consisting solely
of the path-sorted append of that transcript's `path`, raw-byte `blob_sha256`,
and `wv`; and one new canonical rolling-review record whose own exact touched
set is the preceding two paths and whose content digest is computed under the
base-controlled rolling law. No other path, mode, or byte changes. The
transcript can bind only facts strictly preceding `T`, and current `HEAD`
descends through `T`. Its required rolling-review record does not create an
effectiveness `Q_E`, `M_E`, self-reference, second-stage receipt, or receipt for
this receipt.

The transcript contains no accepted-pair or raw-pair field and cannot move
either pair. Ordinary verification selects the accepted pair from the bound
certificate and predecessor chain: succession preserves
`native_accepted_pair`, while an event selects `new_accepted_pair` at `C`.
Two independent implementations then recompute the raw observed pair from
complete ordered inputs at the source boundary, landing `M`, and transcript
introduction `T`. Both source results equal the certificate's
`successor_observed_pair` for succession or `raw_observed_pair` for event; both
implementations agree at `M` and `T`; and every raw movement over the source-to-
`M` and `M`-to-`T` record tails is exhaustively producer-qualified under the
bound predecessor consumer inventory. For the DP3 migration only, the separate
activation bridge accounts for `B..S` and its raw pair; record-tail
qualification begins at `S` and covers `S..M..T`. No activation-bridge byte is
relabeled as a record operation. The transcript's local proof closes at
`T`; it never applies that historical inventory across a later certificate or
event. Those later raw values remain separate from and cannot rewrite the
selected accepted pair.

Mechanical test `wv_acceptance_effectiveness_v1` validates canonical bytes,
the exact fixed key set, registered certificate bytes, complete bounded
transport and pagination, direct selected-review equality, qualified source-to-
`Q` record tail, `Q`/`M` tree equality, authoritative-main first-parent order,
landing-edge census, current-head descent, producer and law hashes, unique
registry append, exact cross-document equality, single-purpose derived
introduction commit, exact same-invocation live-event receipt union and fresh
predicate evaluation,
two-implementation record-tail pair accounting, and
receipt non-recursion. It invokes `wv_dp3_genesis_migration_v1` when the
succession mode selects that one case. It
includes missing/duplicate/forked registry, partial pagination, conflicting
direct object, unequal tree, wrong parent, stale current head, pair field,
self-reference, multi-purpose introduction, accepted/raw-pair conflation,
unexplained event-tail movement, missing/extra/stale/reused live-event receipt,
wrong proposed fold, `Q_E`/`M_E`, and receipt-for-receipt negatives.
Status:
`OPEN-UNTIL-IMPLEMENTED`.

## Attempt-1 eligibility receipt schema

The brief did not allocate a schema ID; this contract completes that open name
as `garnet.trust_kernel_review_eligibility/v1`. The sole channel is one member
named `eligibility.json` in one artifact named exactly
`r2-approval-pending-<run_id>-attempt-1`. The receipt has exactly these keys:

| key | JSON type | invariant |
|---|---|---|
| `artifact_name` | string | exactly `r2-approval-pending-<run_id>-attempt-1` using this receipt's decimal `run_id` |
| `base_ref` | string | exact live base ref from attempt 1 |
| `base_sha` | string | exact live base `commit` |
| `candidate_head` | string | exact record-containing PR head `commit` |
| `candidate_tree` | string | resolved candidate `tree` |
| `event` | string | exactly `pull_request` |
| `finding_codes` | array of strings | normalized, sorted, duplicate-free machine codes |
| `producer_inventory_sha256` | string | raw predecessor required-context producer-inventory `sha256` |
| `pull_request_id` | integer | positive immutable PR ID |
| `pull_request_number` | integer | positive PR number |
| `repository_id` | integer | positive immutable upstream repository ID |
| `review_record_path` | string | exact canonical review-record `git_path` |
| `review_record_sha256` | string | raw review-record `sha256` |
| `run_attempt` | integer | exactly `1` |
| `run_id` | integer | positive immutable workflow-run ID |
| `run_number` | integer | positive workflow run number |
| `schema` | string | exactly `garnet.trust_kernel_review_eligibility/v1` |
| `state` | string | exactly `approval_pending_only` or `ineligible` |
| `workflow_id` | integer | positive immutable workflow ID |
| `workflow_ref` | string | exact workflow ref used by attempt 1 |
| `workflow_sha` | string | exact base-controlled workflow `commit` |

`artifact_name`, rather than GitHub's post-upload numeric artifact ID, is the
receipt's artifact identity. A numeric artifact ID does not exist until after
the receipt is uploaded and would make the receipt self-referential. Attempt 2
authenticates the numeric artifact ID externally, checks the deterministic name,
downloads its raw bytes, and binds both identities in transport evidence.

For R2 only, `producer_inventory_sha256` is the SHA-256 of the complete raw
predecessor-base file `.github/rulesets/required-context-producers.json`, whose
schema is exactly `garnet.required-context-producers/v2`. It is not the WV
implementation inventory. Attempt 1 and attempt 2 independently load that
exact predecessor file and require identical canonical bytes and digest. The
attempt-2 expected job multiset is only the rows whose `workflow` equals the
authenticated workflow path extracted from `workflow_ref` (exactly
`.github/workflows/ci.yml`) and whose `event` equals `pull_request`. Those rows'
base-controlled job/matrix projections expand to the complete expected CI job-
name multiset; every actual attempt-2 job is represented exactly once and no
other producer-inventory row enters that multiset. The fresh head-scoped run
census separately requires every producer row for another workflow to remain at
attempt 1. This preserves act-2 constructibility before the act-3 WV inventories
exist and gives both the all-jobs proof and cross-workflow census exact
authority.

Attempt 2 completely paginates the run-scoped Actions artifacts endpoint and
requires exactly one non-expired artifact with that name, that exact `run_id`,
and a positive numeric artifact ID. The authenticated archive response is
bound by endpoint, status, artifact ID, raw-body SHA-256, and archive-byte
SHA-256. Its ZIP central directory and local headers describe exactly one
unencrypted regular-file member whose name is the literal `eligibility.json`;
the member raw bytes equal the canonical receipt bytes. An extra or duplicate
entry, directory, symlink, device, encrypted member, header/name disagreement,
absolute path, backslash, empty component, or `.`/`..` component is RED.

The base-controlled classifier emits a receipt for every attempt-1 outcome.
Only the exact tuple below grants one re-evaluation:

```text
state = approval_pending_only
finding_codes = [approval-absent]
```

Every other state/code tuple is ineligible. Exit status, log text, step
conclusion, run conclusion, a diagnostic string, and a structurally skipped
context do not grant eligibility.

Mechanical test `trust_kernel_review_eligibility_v1` validates canonical bytes,
the exact key/type set, artifact name/member/path safety, attempt and event
constants, immutable binding equality, normalized codes, the sole eligible
tuple, and missing/duplicate/extra artifact/member and body/API digest
negatives. Attempt-2 gate tests additionally require complete authenticated
artifact enumeration and live equality of the PR, base, head, tree, record,
workflow, run, event, and producer inventory. Status:
`OPEN-UNTIL-IMPLEMENTED`.

Mechanical test `r2_same_run_re_evaluation_v1` traps the complete verbatim R2
block in the rolling-review contract. It requires the unique attempt-1 receipt
and sole eligible tuple; unchanged run/head/tree/base/record/workflow/event and
producer facts; exactly attempt 2; fresh PR, commit, review, selected-review,
artifact, workflow-run, governance, bypass, and required-context transport;
complete head-scoped run census; complete pagination of both attempt-specific
jobs endpoints; equality to the fully expanded predecessor-owned job-name
multiset after the exact workflow/event filter above; exactly one fresh
successful attempt-2 job identity per expected row and no unrepresented job;
`r2_role_separation_v1`; and a fresh reporter emission of the complete final
premerge projection. It rejects a new run, stale payload authority,
partial/default/latest jobs enumeration, missing/duplicate/reused/skipped or
non-success job, movement, identity overlap, context/governance divergence,
partial rerun, attempt 3, or a stale/reused reporter emission. Status:
`OPEN-UNTIL-IMPLEMENTED`.

The machine predicate ends at authenticated reporter emission. Jon's immediate
reading before the merge click is the separate procedural half of DP5: no
schema here claims to authenticate cognition, eliminate the readback-to-click
race, or turn that human act into prevention. Delay or visible UI-state change
requires a new machine emission and another Jon read under `AGENTS.md` and the
U-66 companion, but the reporter proves only what it observed.

## Registry genesis, append-only, and unique-tip law

Before the first certificate, the inactive machinery/genesis act contains
exactly these three canonical chain-registry genesis documents:

```json
{
  "certificates": [],
  "schema": "garnet.wv_acceptance_succession_registry/v1"
}
```

```json
{
  "events": [],
  "schema": "garnet.wv_acceptance_event_registry/v1"
}
```

```json
{
  "receipts": [],
  "schema": "garnet.wv_acceptance_effectiveness_registry/v1"
}
```

Let `G` be the unique authoritative-main first-parent landing that first
introduces the complete inactive machinery/genesis set: the three chain
registries above; the implementation, record-consumer, producer-graph,
transport-projection, reviewer-family, event-class, and impact-criterion
registries; the Class A exception-event genesis; the Class C rewrite catalog;
every producer,
verifier, schema, fixture, and DP12 recovery runbook owned by act 3. No
certificate or transcript may share `G`. Exact-head rolling review covers the
act-3 candidate, but candidate machinery produces no acceptance result.

After `G` lands, two independently encoded `registry-genesis-enumerator`
implementations now owned by `G`, with distinct bound source-closure hashes,
scan the complete tree at `parent1(G)` and report zero matching succession,
event, or effectiveness suffix objects and no pre-existing registry alias. Two
implementation-inventory enumerators owned by `G` derive the complete
inventory at `G`, including one another's source closures, and agree on its
exact bytes. Every support registry is verified at `G`; all three chain
registries and the Class A event registry are exact empty genesis objects.

Let `A` be the unique earliest later authoritative-main first-parent landing
whose predecessor does not pass `wv_acceptance_activation_v1` and whose own
tree does. `A` is the gate-side act-4 boundary: it completes old-base trigger,
rolling-digest, gate, and reporter wiring over the exact current machinery and
registry set; re-runs all implementation, graph, consumer, class, transport,
conservation, fork, duplicate, and adversarial fixtures; and is itself covered
by exact-head rolling review. Surfaces may have been introduced at `G` and
updated before or at `A`; activation is atomic because no certificate is
eligible before the complete predicate first becomes true at `A`, not because
all source files share one introduction commit. Neither `G` nor `A` advances
acceptance, and no certificate, event, transcript, or nonempty chain registry
may appear through `A`.

After `A` lands, its implementations are predecessor-owned and rederive the
complete activated state from `G..A`. This post-landing verification is never
candidate authority or an acceptance claim for `A`. The first certificate's
native predecessor therefore selects `A`, not historic native `H`, as its law
base. A missing or ambiguous `G/A`; missing atomic surface, route, inventory,
fixture, runbook, or exact-head review; nonempty pre-genesis census or
pre-activation chain registry; disagreement; extra matching path;
self-referential closure; partial tree walk; or certificate/receipt introduced
through `A` is RED.

`wv_acceptance_activation_v1` emits an exact canonical in-memory result with
only `blocking_findings`, `genesis_commit`, `schema`, `test_results`, and
`verdict`. `schema=garnet.wv_acceptance_activation/v1`,
`genesis_commit=G`, `blocking_findings=[]`, and `verdict=pass`.
`test_results` is sorted by `test_id`; each member has exactly `test_id` and
`verdict`, with `verdict=pass`. Its test-ID set is exactly:

```text
r1_reporter_constant_projection_v1
r1_review_scope_exact_v1
r1_strict_equal_blob_identity_v1
r2_role_separation_v1
r2_same_run_re_evaluation_v1
record_consumer_inventory_twice_v1
trust_kernel_review_eligibility_v1
wv_acceptance_chain_v1
wv_acceptance_conservation_v1
wv_acceptance_effectiveness_v1
wv_acceptance_pending_terminal_v1
wv_acceptance_registry_genesis_v1
wv_acceptance_trigger_digest_routing_v1
wv_class_a_legacy_precondition_v1
wv_cross_family_review_v1
wv_dp3_genesis_migration_v1
wv_event_certificate_v1
wv_event_class_registry_v1
wv_event_forge_serialization_v1
wv_event_registry_yank_v1
wv_event_toolchain_lint_v1
wv_implementation_source_closure_v1
wv_impact_criterion_registry_v1
wv_producer_graph_inventory_v1
wv_record_tail_pair_v1
wv_schema_canonical_v1
wv_succession_certificate_v1
wv_transport_projection_v1
```

The migration row at `A` is fixture proof only; it grants no certificate.
`wv_law_base_selector_v1` and `wv_acceptance_activation_v1` are excluded from
their own preimage to avoid recursion. Every listed ID is executable under the
act-4 tree, no longer reports `OPEN-UNTIL-IMPLEMENTED`, runs its complete
positive and fail-closed fixture set, and emits pass. The aggregate also checks
that all required gate/reporter contexts consume these results, all chain
registries remain empty, and the exact current implementation/consumer/graph/
transport/class registries are canonical and mutually bound.

Mechanical test `wv_acceptance_activation_v1` validates that exact result and
set equality, independently executes every named test, verifies the false-to-
true first-parent transition at the candidate `A` tree and the post-landing
replay, and rejects a missing/extra/skipped/open/duplicate test, fixture-only
substitution for an ordinary test, nonempty chain registry, absent gate or
reporter consumer, candidate-produced acceptance, parent already active,
candidate/post-landing disagreement, or any non-pass result. Status:
`OPEN-UNTIL-IMPLEMENTED`.

Each registry array is path-sorted and contains one exact entry per discovered
matching file. Paths and raw-byte hashes are unique. The loaded certificate ID
is unique across the succession and event registries. A new entry's path sorts
after every existing path so the canonical array mutation is a literal one-row
append; existing entries and referenced blobs are immutable. A deletion,
replacement, reorder, rollback, interior insertion, alias, unregistered file,
or entry without a matching file is RED.

Every effective succession/event certificate has exactly one effectiveness
transcript, and every effectiveness transcript binds exactly one registered
certificate. The certificate predecessor relation and native root yield exactly
one linear effective tip per WV. Succession kinds form only a native-rooted
prefix; after the first event, every later certificate is also an event. A
duplicate transcript, out-of-order kind, duplicate certificate ID, predecessor
gap, cycle, fork, second tip, or receipt targeting another receipt is RED.

WV identity is never inferred from the untyped `effective_tip_ref`. Every
certificate and transcript `wv` selects exactly one row from its exact
`native_root.wv_contract`, the owning chain-registry row's `wv`, and, for a
non-native predecessor, the recursively loaded predecessor certificate,
effectiveness transcript, and registry-row `wv`. For a native predecessor, the
same ID equals the fixed native ceremony's WV selection. The contract blob is
the canonical `garnet.wv_acceptance_contracts/v2` object at
`F_Project_Management/LAUNCH/WV6_WV7_ACCEPTANCE_CONTRACTS.json`; after duplicate-
key rejection, exactly one `contracts` member has `id=certificate.wv`.
`native_root.evidence_destination` equals that row's `evidenceDestination`, and
the sorted `native_root.required_checks[*].id` set equals its complete
`requiredChecks[*].id` set. The native manifest path equals that destination
plus the row's exact `evidenceManifest`, and its platform/scope facts satisfy
the selected row's bounded Windows evidence contract. A missing or duplicate
row, another row's destination/checks/manifest, path/hash-valid tip from another
WV, cross-WV root splice, or registry/certificate/transcript WV disagreement is
RED and is exercised by `wv_acceptance_chain_v1` and both certificate schema
tests.

One bounded terminal-construction automaton prevents the certificate/receipt
ceremony from deadlocking without backdating authority:

1. `CERTIFICATE-CANDIDATE`: the base has no pending certificate; the candidate
   appends exactly one certificate and its registry row plus the one canonical
   rolling-review record required by base law; the certificate predecessor is
   the current effective tip; every source-to-candidate operation is qualified;
   and the current effective tip remains the only acceptance authority.
2. `ESTABLISHED-BUT-INEFFECTIVE`: authoritative main first-parent landing `M`
   has `tree(M)=tree(Q)` for that one certificate candidate, the registered
   certificate has no transcript, current authoritative main head equals `M`,
   and the prior effective tip remains the only acceptance authority.
3. `TRANSCRIPT-CANDIDATE`: the explicit base is that exact `M`; the candidate is
   the three-path terminal bundle defined by the effectiveness schema and binds
   the sole pending certificate; no other candidate is eligible while pending.
   The prior effective tip still remains authoritative before landing.
4. `EFFECTIVE`: authoritative main introduces the exact terminal bundle at
   derived `T`; the transcript verifies; only then does the chain selector
   advance to the certificate and clear the pending state.

There is at most one pending certificate, it is the sole registered tail after
the current effective tip, and no second certificate, unrelated main commit,
candidate path, acceptance movement, or claim upgrade is permitted before `T`.
Elapsed wall-clock time grants nothing: a pending certificate may deny
availability while main remains exactly `M`, but it cannot yield false
acceptance. Integrity wins over availability; the separately required DP12
recovery runbook may prescribe a new governed repair act but cannot make a fork
or abandoned pending object eligible.

Mechanical test `wv_acceptance_registry_genesis_v1` verifies the two independent
genesis censuses over `parent1(G)`, unique `G` derivation, atomic empty-registry
genesis and complete inactive act-3 surface (including the reviewer-family and
impact-criterion registries), old-base routing/review, post-landing inventory
agreement, exact
suffix routing, path/blob/ID uniqueness, append-only history, complete
discovered-file equality, one transcript per effective certificate, the
bounded pending tail, and one linear effective tip. It includes candidate-
produced acceptance, certificate-at-genesis/activation, nonempty genesis,
missing/extra/duplicate/aliased activation surface or object, registry rollback,
fork, gap, cycle, duplicate receipt, receipt-for-receipt, shared-helper, and
enumerator-disagreement negatives. Status: `OPEN-UNTIL-IMPLEMENTED`.

When called inside activation, this test accepts the candidate `A` only as an
upper bound for proving that no certificate, transcript, or nonempty chain
registry exists through that tree. It does not derive `A` or call
`wv_acceptance_activation_v1`; the aggregate activation test and
`wv_law_base_selector_v1` own the false-to-true transition and uniqueness.

Mechanical test `wv_acceptance_pending_terminal_v1` exercises all four states
and their exact transitions. It rejects two pending certificates, a predecessor
other than the current effective tip, a prelanding acceptance advance, a
landed-pending head beyond `M`, any candidate other than the exact terminal
bundle, an extra/missing bundle or review-record path, a transcript for another
certificate, an unrelated byte or mode, duplicate/late receipt, and an advance
before `T` is authoritative-main first-parent and fully verified. Status:
`OPEN-UNTIL-IMPLEMENTED`.

## Record-consumer inventory schema

The inventory path is exactly
`F_Project_Management/W_TRUST/WV_ACCEPTANCE_RECORD_CONSUMERS.json`; its schema ID
is `garnet.wv_acceptance_record_consumer_inventory/v1`. The canonical top-level
object has exactly `consumers` and `schema`. `schema` equals that ID and
`consumers` is an array of exact entries with these keys:

| key | JSON type | invariant |
|---|---|---|
| `conservation_predicate_ids` | array of strings | sorted unique conservation predicates whose verdict consumes this row |
| `consumer_blob_sha256` | string | framed source-closure `sha256` of the consumer implementation |
| `consumer_git_oid` | string | exact consumer blob object ID |
| `consumer_id` | string | deterministic `consumer:<hex>` ID defined below |
| `consumer_path` | string | exact consumer implementation `git_path` |
| `matcher` | object | exact matcher discriminant defined below |
| `permitted_operation` | string | exactly `classify`, `digest`, `enumerate`, `load`, `parse`, `produce`, or `verify` |
| `producer_blob_sha256` | string | predecessor producer framed source-closure `sha256` |
| `producer_id` | string | predecessor record-producer `logical_id` |
| `producer_path` | string | exact producer implementation `git_path` |
| `schema_id` | null-or-string | exact consumed schema ID, or null only for a byte/path-only consumer |
| `semantic_role` | string | exactly `classifier`, `digestor`, `producer`, `registry-loader`, `reporter`, or `verifier` |

`matcher` has exactly `kind` and `value`. `kind` is exactly `exact-path`,
`prefix`, `suffix`, or `registry`; `value` is a `git_path` for `exact-path` and
`registry`, a `git_path` followed by one terminal `/` for `prefix`, and a
nonempty ASCII token beginning with `.` and containing no slash, backslash, or
NUL for `suffix`. The matcher is applied literally; glob syntax and implicit
case folding are RED.

For one entry, remove `consumer_id`, serialize the remaining object canonically,
and hash
`garnet.wv_acceptance.record_consumer/v1 NUL <canonical-entry-without-id>`;
`consumer_id` is exactly `consumer:<lowercase-hex>`. The array is sorted by
`(consumer_path, matcher.kind, matcher.value, schema_id, permitted_operation,
producer_id)`, treating JSON null before strings, and contains each tuple once.
The bound `record_consumer_inventory_sha256` in every certificate/transcript is
the SHA-256 of the complete raw canonical inventory file bytes at the selected
certificate `law_base_commit`; a transcript derives that same base through its
bound certificate.

Each consumer entry resolves through exactly one predecessor implementation-
inventory row: `consumer_path`, `consumer_git_oid`, and
`consumer_blob_sha256` equal that `record-consumer` row's entrypoint path,
entrypoint blob OID, and source-closure hash. `producer_id`, `producer_path`,
and `producer_blob_sha256` equal one `record-producer` row's `logical_id`,
entrypoint path, and source-closure hash. The governed record matched by the
entry remains runtime data outside both source-closure preimages.

`conservation_predicate_ids` is a subset of exactly
`r1_reporter_constant_projection_v1`, `r1_review_scope_exact_v1`,
`r1_strict_equal_blob_identity_v1`, and `r2_role_separation_v1`. It contains one
ID if and only if that predicate's predecessor implementation consumes this
exact row's matched records on a verdict-bearing path. The two independent
inventory enumerators derive the array from the complete implementation
inventory, operation matchers, source closures, and statically resolved call
paths, then exercise the predicate's registered positive fixture and one
per-row fail-closed mutation that must change the verdict. An untraceable
dynamic call remains an open inventory finding; a candidate label, hand-written
allowlist, passing positive alone, or marker copied from another row is RED.

For `r1_strict_equal_blob_identity_v1`, the exact comparator-consumer subject
set is the named path `scripts/garnet_github_governance_gate.py` plus every
predecessor inventory row whose `conservation_predicate_ids` contains that ID,
deduplicated by `(consumer_path, consumer_git_oid, consumer_blob_sha256)`. The
predicate requires predecessor/candidate blob-OID equality for every subject
and rejects an omitted marker, unmarked verdict-bearing comparator, extra
marked non-consumer, relocation, wrapper, or import substitution. Thus the
brief's phrase "every predecessor-inventoried comparator consumer" is a closed
machine set rather than a semantic guess.

The predecessor inventory is separately recomputed by exactly two independent
`record-consumer-inventory-enumerator` implementations with distinct bound
source-closure hashes. Each walks the complete predecessor tree and emits byte-
identical inventory bytes covering every gate or reporter that globs, suffix-
matches, registry-loads, parses, produces, or verifies record bytes. Neither
implementation imports, invokes, wraps, or consumes the other or its output.
Candidate inventory bytes and candidate matcher law are never authority.

Mechanical test `record_consumer_inventory_twice_v1` rejects an unlisted
consumer, extra or missing matcher, schema/operation or conservation-marker
drift, candidate-owned inventory, source-hash equality, implementation
dependency, incomplete call/tree walk, noncanonical ordering, ID mismatch, or
byte/digest disagreement. Status:
`OPEN-UNTIL-IMPLEMENTED`.

## Trigger and rolling-digest routing law

Before any first instance is eligible, predecessor-base policy separately
routes the succession, event, and effectiveness suffixes; all three registries;
all four exact schemas in this document; the fixed implementation, record-
consumer, producer-graph, transport-projection, reviewer-family, event-class,
impact-criterion, Class A exception-event, and Class C catalog registries; the
DP7 precondition verifier and fixtures; every
certificate, class, effectiveness, inventory, census, pair, closure, and
transport producer/verifier; and every governed policy byte as trust-kernel
triggers and exact rolling-review digest inputs. Record-class or digest-
exclusion membership never substitutes for this route. Trigger classification
uses the predecessor implementation and the rolling digest includes the raw
candidate blobs selected through that old-base route.

Mechanical test `wv_acceptance_trigger_digest_routing_v1` runs the predecessor
classifier and digestor against positive fixtures for every named route, proves
the exact candidate bytes change the digest, and rejects each route removed,
renamed, aliased, moved behind candidate policy, omitted from the digest, or
covered only by a broad record prefix. It also rejects an unknown matching
suffix or registry object, either inventory participating in its own source-
closure preimage, and a producer/verifier/policy byte that reaches a consumer
without appearing in both the trigger set and digest projection.
Status: `OPEN-UNTIL-IMPLEMENTED`.

Mechanical test `wv_record_tail_pair_v1` performs the two independent complete-
inventory, pipeline-disjoint pair computations for each certificate segment at its source
boundary, `M`, and `T`, closes that segment at `T`, then lets
`wv_acceptance_chain_v1` advance through each later effective certificate in
order. Only the final effective tip's predecessor inventory qualifies the
trailing `T..current HEAD` record segment and derives the current raw pair. The
test rejects implementation dependence, input-stream disagreement, a
historical inventory applied across a later event, any unexplained record-tail
delta, pair cross-role reuse or shared executable closure, accepted/raw-pair
conflation, and especially an event certificate whose
pair accepted at `C` is silently replaced by a record-inflated raw pair at `M`.
Status: `OPEN-UNTIL-IMPLEMENTED`.
