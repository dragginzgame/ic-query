# Subnet Catalog Downstream Adoption

## Purpose

`ic-query` 0.41.1 hard-cuts the pre-1.0 schema-1 Subnet Catalog contract to
retain current Registry routing authority and per-value provenance. Version
0.41.2 adds portable typed constructors for downstream fixtures and custom
sources. Downstream crates do not need a routing compatibility adapter, but
direct Rust struct literals, copied evidence projections, and persisted older
schema-1 JSON need an explicit update.

`ic-query` 0.42.0 hard-cuts load authority into stable snapshot
identity and transient acquisition provenance. Downstream code using the
removed `CatalogAuthorityEvidence` or `authority_evidence()` surface must adopt
the boundary below; no compatibility alias is retained.

This document is the adoption checklist. It does not define a legacy reader,
Serde default, migration, or monolithic-routing fallback.

## Production compatibility

Callers that use the public load, refresh, list, info, and canister-resolution
functions without constructing their returned structs remain source
compatible. `SubnetCatalogRegistryRecordKind::RoutingTable` continues to name
routing evidence from either schema; the exact key, routing source, and
optional shard lower-bound subject distinguish modern shards from the legacy
record.

Modern records have all of the following properties:

- `provenance.routing_source` is `CanisterRanges`;
- `record.kind` is `RoutingTable`;
- `record.key` is the exact `canister_ranges_*` Registry key;
- `record.canister_range_start` is the shard's decoded lower-bound principal;
- `requested_registry_version` is the shared pinned version;
- `returned_registry_version` is the individual value version;
- endpoint, assurance, Registry timestamp, and inline/chunked value encoding
  are copied from the actual read.

Live collection never selects legacy evidence: an empty pinned
`canister_ranges_*` family fails closed. `LegacyRoutingTable`, exact key
`routing_table`, and no shard lower-bound subject occur only in explicitly
historical replay or in a caller-supplied custom collection. Certified replay
requires `AllowHistoricalTarget` before it permits that legacy source;
`RequireLatestObserved` requires modern shards.

## Stable snapshot authority

Use `CatalogLoadOutcome::snapshot_authority()` when persisting or comparing the
authority identity of a successfully validated catalog. It returns
`CatalogSnapshotAuthorityEvidence`, whose only fields are `registry_version`,
`catalog_digest`, `assurance`, and canonical `source_endpoints`. The same value
is available from `ValidatedSubnetCatalog::snapshot_authority()`.

Treat `CatalogLoadOutcome::path` and `CatalogLoadOutcome::disposition` as
acquisition provenance for that individual load. A repaired missing or invalid
cache and a later cache hit intentionally produce equal snapshot authority and
different `CacheDisposition` values. Do not copy the disposition into a stable
authority projection.

## Direct Rust fixtures

Downstream literals for `SubnetCatalogLoadFailure` must include the new fields.
Use values that describe the fixture rather than inventing live provenance. A
failure before any Registry value read normally uses:

```rust
returned_registry_value_version: None,
source_endpoint: None,
assurance: None,
registry_records: Vec::new(),
```

Every `SubnetCatalogRegistryRecordSubject` literal also includes:

```rust
canister_range_start: None,
```

Use `Some(lower_bound)` only for a known `canister_ranges_*` shard subject.
Prefer `SubnetCatalogSourceFailure::new` for caller-supplied source failures;
attach real value evidence with `with_registry_evidence` when the source has
it.

For successful Registry reads, prefer the portable 0.41.2 constructors over
raw subject literals or hand-built key strings:

```rust
use candid::Principal;
use ic_query::subnet_catalog::{
    SubnetCatalogRegistryRecordEvidence, SubnetCatalogRegistryRecordSubject,
    SubnetCatalogRegistryValueEncoding,
};

let subnet = Principal::from_text(
    "pzp6e-ekpqk-3c5x7-2h6so-njoeq-mt45d-h3h6c-q3mxf-vpeq5-fk5o7-yae",
)?;
let record = SubnetCatalogRegistryRecordSubject::subnet_record(subnet);
let evidence = SubnetCatalogRegistryRecordEvidence::uncertified_query(
    record,
    63_438,
    63_300,
    1_780_531_200_000_000_000,
    "https://icp-api.io",
    SubnetCatalogRegistryValueEncoding::Inline,
);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Use `subnet_list`, `legacy_routing_table`, or `canister_ranges` for the other
typed subjects. The public `SUBNET_LIST_KEY`, `ROUTING_TABLE_KEY`,
`CANISTER_RANGES_KEY_PREFIX`, and `SUBNET_RECORD_KEY_PREFIX` constants remain
available when a fixture needs to assert the exact wire key.

## Typed downstream projections

A downstream failure DTO that claims complete upstream evidence needs to copy:

- `returned_registry_value_version: Option<u64>`;
- `source_endpoint: Option<String>`;
- `assurance: Option<String>` from `CatalogAssurance::as_str()`;
- `registry_records`, without collapsing records into one summary;
- `subject.canister_range_start` for Registry-record subjects.

Each projected Registry record needs the exact kind, key, optional Subnet,
optional shard lower bound, requested version, returned version, Registry
timestamp, endpoint, assurance, and value encoding. Stable text labels are
available from `CatalogAssurance::as_str()`,
`SubnetCatalogRegistryRecordKind::as_str()`, and
`SubnetCatalogRegistryValueEncoding::as_str()`.

Success-side projections that expose collection authority likewise copy
`routing_source` and `registry_records`. Testing only the aggregate pinned
version is insufficient because individual Registry values commonly have
older returned versions.

## Custom sources

`UncertifiedCatalogCollection::new` remains source compatible for simple
custom sources and produces legacy-routing/empty-record defaults. That
intermediate collection is not valid authority until complete evidence is
attached. A custom source claiming modern routing authority must call
`with_registry_evidence` with `CanisterRanges` and its complete fetched-record
evidence. It must not use the defaults to describe a modern collection.

Detailed custom-source failures use `SubnetCatalogSourceFailure::new` and then
`with_registry_evidence` whenever an individual returned version, endpoint,
assurance, or completed record is known.

## Persisted schema-1 data

Older Subnet Catalog or report JSON lacks the new required fields and is not
accepted as the current schema-1 shape. Delete or explicitly refresh those
caches under the caller's existing invalid-content repair policy. Do not add a
fallback reader, alias, automatic migration, or fabricated default routing
source.

## Canic audit

Canic has confirmed that the modern-first routing change integrates without a
compatibility adapter. Its typed projection can retain the returned value
version, failing endpoint and assurance, completed Registry records, shard
lower-bound subject, and retry classification without parsing strings or
fabricating provenance. Older cache shapes correctly fail closed and require
refresh. The 0.41.2 constructors remove the remaining need for Canic fixtures
to spell Registry keys manually. Adopting the current hard cut requires
replacing `authority_evidence()` with `snapshot_authority()` and reading `path`
and `disposition` separately; routing, refresh, cache, and failure adapters
remain unnecessary.
