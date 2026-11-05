# Subnet Catalog Failure Provenance

- Status: implemented; canonical load API updated for 0.86.0
- Last reviewed: 2026-11-02
- Scope: `subnet-catalog-host` load failures

## Contract

Detailed load and source errors are boxed at their return boundaries so
successful results do not carry the full failure payload inline. Source
constructors return `Box<SubnetCatalogSourceFailure>`; evidence builders keep
that allocation, and `into_source` consumes it to recover the original error.
All typed provenance fields remain available through ordinary field access.

Successful and failed loads answer different questions. A successful
`CacheDisposition` records how the returned validated catalog was supplied. It
is transient acquisition provenance and is excluded from
`CatalogSnapshotAuthorityEvidence`, which is derived only from the validated
catalog. It does not identify where an unsuccessful operation stopped or what
happened to the cache on that path.

Detailed loads return `Box<SubnetCatalogLoadFailure>`. Its request
retains the requested network, selected `CatalogSourceSelection`, and minimum
assurance. `SubnetCatalogLoadStage` identifies the exact failing operation, and
`SubnetCatalogFailureCacheDisposition` records the failure-side cache fact or
attempt. Refresh failures also retain whether missing, rejected, stale, or
forcibly bypassed cache content caused the refresh path.

`SubnetCatalogSourceFailure` and `SubnetCatalogLoadFailure` carry Registry progress
and the offending subject through one `evidence: SubnetCatalogFailureEvidence`.
For example, read `failure.evidence.registry_version` and
`failure.evidence.subject`. Load stage, cache disposition and the original
`source` remain on the outer failure. Read classifications directly through
`failure.source.code()`, `failure.source.category()` and
`failure.source.retryability()`; the failure does not retain separate copies.

Together these records retain:

- `registry_version: Option<u64>`;
- `returned_registry_value_version: Option<u64>` for an individual
  `get_value` response that progressed far enough to return it;
- `source_endpoint` and individual read `assurance` when known;
- completed `registry_records`, with each record's requested and returned
  Registry versions, exact key/schema/subject, Registry timestamp, endpoint,
  assurance, and inline or chunked representation;
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
None`. Once the query succeeds, every later Subnet-list, routing-key-family,
routing-shard, Subnet-record, decoding, routing-range projection, catalog
validation, and multi-endpoint aggregation failure carries exactly that pinned
requested version. A Registry response's individual value version is retained
separately and is never substituted for the pin.

For multi-endpoint agreement, an endpoint failure carries the version pinned
by that endpoint when known. A version or payload mismatch carries the exact
version returned by the offending endpoint and leaves the reference version in
the original typed host error.

## API Boundary

Every load entry point retains typed failure provenance:

- `load_cached_subnet_catalog`;
- `load_subnet_catalog`;
- `load_subnet_catalog_with_source`; and
- `load_subnet_catalog_with_source_and_progress`.

Live-capable entry points are async on the caller's runtime; the explicit cached
loader is synchronous and performs no network calls.

`SubnetCatalogSource::fetch_catalog_detailed` lets a caller-supplied source
return `Box<SubnetCatalogSourceFailure>`. Its default delegates to the existing
simple source method and truthfully leaves unavailable version/subject
evidence absent. The parallel detailed loader names are removed. Load callers
inspect `failure.source` or explicitly consume `into_source()` when they choose
to discard the acquisition evidence. The collection and cache algorithms have
one implementation; see the [0.86 consumer update](../changelog/0.86.md).

All of these types and functions are available only with
`subnet-catalog-host` (or a broader feature that includes it). They add no
process output, migration, or CLI surface. The schema identifier remains 1
under the repository's pre-1.0 hard-cut policy, while the current schema-1
catalog and report shape includes routing-source and per-record evidence.
