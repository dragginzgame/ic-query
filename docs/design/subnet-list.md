# Registered subnet inventory

Status: implemented using published Platform 0.33.0.

`icq topology subnet list [--json]` enumerates the NNS Registry's registered
subnet IDs so Canic can select candidates for subsequent topology inspection.
It does not identify which subnets admit a particular application's deployment,
or establish capacity, uptime, maintenance status or TEE availability.

## Source and ownership

Read the fixed mainnet Registry `rwlgt-iiaaa-aaaaa-aaaaa-cai` through
`https://ic0.app`, using its protobuf `get_value` query with key `subnet_list`
and no requested historical version. This returns the latest value for that key.
Decode the inline value as `SubnetListRecord`; do not substitute a static list,
dashboard API or a set derived from canister routing ranges.

The external contracts reviewed at IC revision
`48b6f629d74fdc4dca2c1f8e14810a9342f72fbb` are the
[Registry transport schema](https://github.com/dfinity/ic/blob/48b6f629d74fdc4dca2c1f8e14810a9342f72fbb/rs/registry/transport/proto/ic_registry_transport/pb/v1/transport.proto)
and [subnet schema](https://github.com/dfinity/ic/blob/48b6f629d74fdc4dca2c1f8e14810a9342f72fbb/rs/protobuf/def/registry/subnet/v1/subnet.proto).
The current response is `HighCapacityRegistryGetValueResponse`, which can return
either inline bytes or chunk references. The initial report must reject chunked
content explicitly; it must not mistake absent inline bytes for an empty list.

Platform owns the external request/response types, protobuf encoding/decoding,
wire validation and bounded allocation. Query owns the fixed key, endpoint,
deadline, report projection and error policy. Published `ic-platform` 0.33.0
supplies this capability through `nns_registry_types::protobuf`. Query declares
no external protobuf schema or decoder.

## Report and bounds

Return schema `1`, deterministically sorted subnet IDs and explicit evidence:
endpoint, source canister, method, key, local observation time, query-signature
verification, result certification and complete coverage of the selected record.
Preserve the raw record mutation version and timestamp. The response's version
is when this key last changed, not necessarily the Registry's latest global
version; label it `record_version`. Query-signature verification does not make
the value consensus-certified.

Use the established anonymous mainnet Agent policy, 20-second overall deadline
and 1 MiB HTTP response ceiling. Bound protobuf decoding to 64 KiB of reply bytes,
4096 fields per decoding pass, 128 KiB of codec allocation, and at most 1024 subnet
entries. Platform borrows response blobs and counts entries before allocating
their reference vector. Query's identity vector is separately bounded by those
1024 entries. Reject malformed
protobuf, invalid principal bytes, duplicates, an empty list, missing content,
Registry errors and excess entries without returning a partial success. Do not
silently skip invalid IDs or truncate results. Codec limits must take effect
before attacker-controlled lengths cause allocation.

Completeness means every entry in the selected returned record was decoded and
retained. It is not independent proof that the query reflects the newest chain
state, that all entries are live, or that the caller can deploy to them.

## Qualification

Platform must qualify the wire contract, bounds, unknown fields, malformed
lengths/varints, error envelopes and chunk-reference handling. Query must qualify
projection, sorting, duplicates, invalid identities, empty/excess entries, raw
version/timestamp JSON and typed failures. CLI tests must cover local namespace
help and list argument admission; a separate mainnet smoke observation must
verify the published capability against the live source before reporting success.

The 2026-11-14 live smoke read through `https://ic0.app`, using published Platform
0.33.0, returned 78 unique registered subnet IDs at local observation time
`1794653786` Unix seconds. The record mutation version was `65711` and its raw
timestamp was `1794608317679020873` nanoseconds. Query signatures verified;
the result remains uncertified. This is one successful source observation, not
continuous availability or independent latest-state proof.

Existing `subnet show` continues to report its separate certified node-key
observation contract. The old direct-ID syntax is replaced by `subnet show` and
`canister show`; report schemas remain `1` and existing report JSON is unchanged.
