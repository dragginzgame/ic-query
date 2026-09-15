# Subnet Catalog Routing Authority

- Status: implemented for the active 0.41.1 release slice
- Last reviewed: 2026-08-21
- Scope: live and certified-replay Subnet Catalog routing collection

## Upstream contract

Current DFINITY Registry clients obtain routing through
`RegistryClient::get_routing_table`: enumerate the complete key family under
`CANISTER_RANGES_PREFIX` at the selected version, return no modern table when
that family is empty, otherwise fetch every value at the same version, decode
each value as a `RoutingTable` shard, flatten the shards, and validate the
result. The key suffix is lowercase hexadecimal for a raw canister-ID lower
bound. DFINITY permits a shard's first range to start above that bound; every
range must remain below the next shard key.

The implementation follows those semantics rather than deriving a new shard
format:

- [DFINITY routing-table Registry helper](https://github.com/dfinity/ic/blob/master/rs/registry/helpers/src/routing_table.rs)
- [DFINITY Registry key construction](https://github.com/dfinity/ic/blob/master/rs/registry/keys/src/lib.rs)
- [DFINITY Registry delta pagination](https://github.com/dfinity/ic/blob/master/rs/registry/canister/canister/canister.rs)
- [DFINITY routing-shard mutation semantics](https://github.com/dfinity/ic/blob/master/rs/registry/canister/src/mutations/routing_table.rs)
- [DFINITY routing-table protobuf conversion](https://github.com/dfinity/ic/blob/master/rs/registry/routing_table/src/proto.rs)

## Live selection

The ordinary-query collector first pins `get_latest_version`. It reconstructs
the present `canister_ranges_*` key set at that pin from complete,
size-bounded `get_changes_since` pages starting at version zero. The response
`version` is the Registry latest version, not the page watermark, so each next
page starts after the highest returned mutation version. Mutations after the
pin are ignored and deletions through the pin are applied. A response that
does not reach the pin, a missing-version or no-progress page, an explicit
resource-ceiling violation, or malformed or contradictory family evidence
fails closed.

Selection is exact:

1. A nonempty modern family selects `CanisterRanges`. Every listed shard is
   fetched with `get_value(key, pinned_version)`.
2. An empty modern family fails closed as missing current routing authority.
3. The live collector never queries or considers `routing_table`, whether the
   modern family is present, empty, malformed, or incomplete.

The existing bounded, hash-verified `get_chunk` path completes any large shard
or other catalog value. The returned individual mutation version and Registry
timestamp remain distinct from the requested pin.

## Validation

Modern collection rejects a missing listed shard, an empty or malformed shard,
a noncanonical or non-routable key, a range below its shard's lower bound, or a
range that reaches the next shard's key interval. Flattened routing then passes
the same catalog validation as legacy and certified replay: required fields
and principals, nonempty ranges, canonical ordering, no duplicates or
overlaps, and membership of every routed Subnet in the current Subnet list.
Persisted record evidence must be complete for every represented endpoint and
must bind each typed subject to its exact Subnet-list, Subnet-record, legacy
routing, or decoded shard-lower-bound key. No missing Subnet is synthesized.

The public diagnostic replay projection is an explicitly historical boundary:
it uses modern shards when present and may inspect a pre-shard legacy
`routing_table` when the reconstructed family is empty. Certified promotion
reuses the caller's existing version policy. `RequireLatestObserved` requires
nonempty modern shards; only `AllowHistoricalTarget` permits a legacy table.
Replay does not perform ordinary `get_value` calls, so its per-value ordinary-
query evidence list is empty; its authority remains the archive/replay
commitments already required by certified projection.

## Persisted provenance

The current pre-1.0 schema-1 catalog is hard-cut in place. Provenance records
the selected routing source and every completed ordinary Registry value read:
requested pin, returned value version, timestamp, exact key, protobuf schema,
typed subject, endpoint, individual assurance, and inline or chunked encoding.
Typed failures additionally retain the load stage, failing record subject,
returned version when available, completed record evidence, and truthful retry
classification. No migration, legacy reader, or compatibility alias is kept.
