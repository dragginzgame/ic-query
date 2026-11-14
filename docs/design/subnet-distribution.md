# Selected-subnet provider and data-center distribution

Status: accepted next slice; source contract prepared, implementation blocked on
published Platform codecs. This is not an implemented command or report.

## Canic decision and scope

Expose which providers and registered locations a selected subnet depends on,
so Canic can compare concentration across candidate placements. Collect only
the subnet's registered nodes and the distinct operators/data centers they
reference. Provider identity comes from each operator's Registry record; no
Governance names, registration lists, rewards or payment accounts are needed.
Registry location and ownership declarations are not independently verified
physical locations or proof that providers have independent failure domains.

Proposed additive surface: `subnet::distribution` and the CLI action
`topology subnet distribution`. Existing `subnet nodes`, configuration and
membership reports retain their contracts. No existing report schema, saved JSON
or published version is replaced by this design.

## Collection and evidence

1. Query the fixed mainnet Registry's `get_latest_version` once, retaining its
   raw version and local observation time. Query-signature verification remains
   distinct from consensus certification.
2. Read the selected subnet record using `get_value` with that explicit version.
3. Read its admitted registered nodes at the same requested version.
4. Deduplicate referenced operator IDs and read each operator once at that version.
5. Deduplicate admitted data-center IDs and read each data center once at that version.

Retain the requested Registry version separately from each record's returned
mutation version and timestamp. Older mutation versions are valid; a returned
mutation version above the pin is refused. Never substitute a subnet/node
mutation version for the global latest version, retry at a newer version or
silently fall back to an unversioned read. A common requested version provides
Registry version consistency, not consensus certification or simultaneous live
machine observations. Do not promote this to a general `atomic_snapshot` claim.

Use the existing fixed endpoint/root-key policy and leaf byte/field/time limits.
Cap membership, distinct operators and distinct data centers at 128 each; read
in batches of at most four under one 60-second overall asynchronous deadline.
The maximum is 386 logical queries: one version, one subnet, 128 nodes,
128 operators and 128 data centers. Agent discovery/transport exchanges are not logical queries.
Excess membership fails before fan-out; no silently truncated successful report.

Keep successful observations and explicit failed, cancelled, not-started and
dependency-unavailable outcomes. A failed operator read must retain every node
that referenced it; it does not erase those nodes or imply an absent provider.
Empty provider bytes are missing identity, not the management principal. Validate
any populated operator identity against its queried key. Preserve original
data-center reference text and distinguish absent references from failed reads.
Admit key IDs under the reviewed Registry key contract before joining; do not
copy legacy trim/lowercase normalization without verifying its applicability.
Compare the returned data-center ID with the admitted key identity.

Group only resolved relationships. Publish selected membership, resolved counts
and unknown counts alongside provider/data-center totals. A complete provider
mapping can coexist with incomplete location evidence; expose these separately.
Do not extrapolate network-wide provider counts, spare node capacity, uptime,
cloud-engine prices or deployment eligibility from this selected membership.

Text should use compact tables and shared keys, retaining full evidence in JSON.
Partial output remains explicit and exits 1, following the node collection.
Initial version/configuration/admission failure produces no success report.

## Published Platform prerequisite

On 2026-11-14 the public Cargo sparse index identified `ic-platform 0.33.3` as
the latest non-yanked stable release, checksum
`74c6e194ce3a80487320edef0041e286fd89d6d96e8a9f641811c7e38b856e51`.
Its published `nns_registry_types::protobuf` supports explicit versions in
`encode_get_value_request`, inline envelopes, subnet records and node records.
It does not expose the following required bounded decoders:

| Protocol | Selected fields |
| --- | --- |
| `RegistryGetLatestVersionResponse` | Raw `version` uint64, field 1 |
| `NodeOperatorRecord` | Operator principal bytes (1), provider principal bytes (3), data-center ID text (4) |
| `DataCenterRecord` | ID text (1), region text (2), owner text (3) |

Request these within the existing `nns-registry-types` feature and explicit
protobuf `Limits`, following the current borrowed-result/error policy. Preserve
proto3 defaults, reject duplicate selected singular fields, malformed lengths,
wrong wire types and invalid selected UTF-8; skip unknown fields within bounds.
Do not expose rewards, node allowances, GPS or protocol-wide generic invocation
for this slice. Query owns identity admission, field-length policy, joins,
deadlines, authentication and completeness; Platform owns external decoding.

Protocol facts were reviewed at IC revision
`904f4dde68006d3cccee4dd3c3ecb4b3798d8790`:
[transport schema](https://github.com/dfinity/ic/blob/904f4dde68006d3cccee4dd3c3ecb4b3798d8790/rs/registry/transport/proto/ic_registry_transport/pb/v1/transport.proto),
[operator schema](https://github.com/dfinity/ic/blob/904f4dde68006d3cccee4dd3c3ecb4b3798d8790/rs/protobuf/def/registry/node_operator/v1/node_operator.proto),
[data-center schema](https://github.com/dfinity/ic/blob/904f4dde68006d3cccee4dd3c3ecb4b3798d8790/rs/protobuf/def/registry/dc/v1/dc.proto).
These are schema facts, not imported third-party implementation or tests.

No Query-local protobuf decoder, new direct external codec dependency or sibling
path substitution is authorized by this design. Adopt the published supplier
capability before introducing the live readers and composition. This document
specifies the consumer contract;
[Platform issue #165](https://github.com/dragginzgame/ic-platform/issues/165)
owns the supplier prerequisite and acceptance checks.

## Legacy lessons and acceptance cases

Read-only review used legacy `5b4951b23c2adfcd3c615961f131e766c4b50283`:
[version selection](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/ic_registry/transport/version.rs),
[relation acquisition](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/ic_registry/source/relation_inventory.rs),
[operator projection](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/ic_registry/projection/node_operator.rs),
[data-center projection](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/ic_registry/projection/data_center.rs),
[topology regressions](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/ic_registry/projection/subnet_topology/tests.rs)
and [location regressions](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/ic_registry/tests/inventory/data_center.rs).
Previously reviewed history `b89d917d5e94f5bd73771076a59347ea29abd328` removed
redundant provider-count acquisition; `0f242c5e467a5e2ec1d4ab28de68678f4cc0bcd8`
moved protocol handling toward owned codecs.

Retain one version selection, shared relation reads, exact counts, identity
checks and failure context. Adapt the global complete-snapshot contract into a
bounded selected-subnet report with explicit partial coverage; restore neither
global scanning nor caches. Carry forward missing operator/provider and mismatched
data-center cases, shared-operator/provider counting, missing location and distinct
provider/location counts. Add version-skew rejection, deduplicated call counts,
dependency-failure coverage, cancellation and raw/escaped presentation cases.
Global duplicate cross-subnet assignment checks require a multi-subnet scope and
are not a claim this selected-subnet report can establish.

Only documentation and supplier discovery are complete. No new product behavior
or implementation qualification is claimed here.
