# Subnet Catalog Failure Provenance

- Status: implemented for the active 0.41.0 release slice
- Last reviewed: 2026-08-20
- Scope: `subnet-catalog-host` detailed load failures

## Contract

Successful and failed loads answer different questions. A successful
`CacheDisposition` records how the returned validated catalog was supplied. It
does not identify where an unsuccessful operation stopped or what happened to
the cache on that path.

Detailed loads therefore return `SubnetCatalogLoadFailure`. Its request
retains the requested network, selected `CatalogSourceSelection`, and minimum
assurance. `SubnetCatalogLoadStage` identifies the exact failing operation, and
`SubnetCatalogFailureCacheDisposition` records the failure-side cache fact or
attempt. Refresh failures also retain whether missing, rejected, stale, or
forcibly bypassed cache content caused the refresh path.

The failure separately carries:

- `registry_version: Option<u64>`;
- `Option<SubnetCatalogSubject>` with typed Registry record/key, Subnet
  principal, routing range, endpoint, cache path, or field identity;
- stable `SubnetCatalogErrorCode` and `SubnetCatalogErrorCategory` values;
- `SubnetCatalogRetryability`, including `Unknown` with a
  `SubnetCatalogUnknownRetryReason`; and
- the original `SubnetCatalogHostError` as `source`.

No field is inferred by parsing `Display` output. Unknown version, subject, or
retryability evidence remains explicitly unknown.

`SubnetCatalogSubject::RegistryLatestVersion` identifies version acquisition
without pretending that the method is a Registry record. Every
`SubnetCatalogRegistryRecordSubject` has an exact key. An indexed malformed
Registry entry is `RegistryRoutingTableEntry`; `RoutingRange` is reserved for a
complete offending range value.

## Registry Version Boundary

`get_latest_version` is the pinning boundary. Agent construction, Registry
canister selection, or latest-version failures retain `registry_version =
None`. Once the query succeeds, every later Subnet-list, routing-table,
Subnet-record, decoding, routing-range projection, catalog validation, and
multi-endpoint aggregation failure carries exactly that pinned version.

For multi-endpoint agreement, an endpoint failure carries the version pinned
by that endpoint when known. A version or payload mismatch carries the exact
version returned by the offending endpoint and leaves the reference version in
the original typed host error.

## API Boundary

The detailed entry points are:

- `load_cached_subnet_catalog_detailed`;
- `load_subnet_catalog_detailed`;
- `load_subnet_catalog_detailed_async`;
- `load_subnet_catalog_detailed_with_source`; and
- `load_subnet_catalog_detailed_with_source_async`.

`SubnetCatalogSource::fetch_catalog_detailed` lets a caller-supplied source
return `SubnetCatalogSourceFailure`. Its default delegates to the existing
simple source method and truthfully leaves unavailable version/subject
evidence absent. Existing simple load entry points call the detailed core and
return `SubnetCatalogLoadFailure::into_source()`, so the collection and cache
algorithms are not duplicated.

All of these types and functions are available only with
`subnet-catalog-host` (or a broader feature that includes it). They add no
process output, cache schema, migration, CLI surface, or network request.
