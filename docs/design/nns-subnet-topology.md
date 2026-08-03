# Exact-Version NNS Subnet Topology

## Status

- Status: implemented in 0.11.0
- Scope: host-side, read-only public-IC Registry observation
- Cache: one joined snapshot under the caller-provided shared cache root

## Decision

`ic_query::nns::topology` exposes `NnsSubnetTopologyReport` as the canonical
Subnet-oriented topology snapshot. Each `NnsSubnetTopologyRow` contains the
raw Registry `SubnetKind`, total assigned node count, and canonically ordered
`NnsSubnetNodeProviderRow` values with per-provider node counts.

The live source accepts the shared `ic_query::nns::NnsSourceRequest`, rejects
non-mainnet requests before
agent construction, resolves the latest Registry version exactly once, then
reads the Subnet list, Subnet records, assigned node records, and node-operator
records at that version. Provider membership comes only from the Registry
node-to-operator-to-provider relation. Governance names, registration state,
reward accounts, capacity summaries, data-center labels, and regions are
enrichment and are not fields in this canonical snapshot.

Every report carries the Registry canister, Registry version, fetch timestamp,
source endpoint, and fetcher. `SubnetKind` remains
`Application | CloudEngine | System | Unknown`; consumers own any narrower
management classification.

## Validation

Reports reject:

- non-canonical or duplicate Subnet rows;
- non-canonical or duplicate provider rows within a Subnet;
- invalid or non-canonical principals;
- zero-count provider rows;
- provider counts that do not sum to the Subnet node count;
- report-level Subnet or node totals that do not match the rows; and
- unsupported schemas or wrong network/Registry identity.

Live collection translates Registry `key_not_present` responses for required
node and node-operator record reads into `MissingNodeRecord` and
`MissingNodeOperatorRecord`. A missing operator error carries the canonically
ordered principals of every fetched node that references it. Other Registry
errors retain their transport details, and an empty provider principal remains
`MissingNodeProviderPrincipal`. Projection also returns typed failures for
duplicate cross-Subnet node assignments and incomplete fixture inventories.
No missing relation is silently omitted.

## Cache Lifecycle

The joined report is published atomically at:

```text
<cache-root>/nns/<network>/subnet-topology/report.json
```

One sibling `refresh.lock` covers version resolution, the complete joined
fetch, validation, serialization, and atomic replacement. A failed refresh
does not replace the last complete snapshot.

Cache policy is explicit in the public API:

- `load_cached_nns_subnet_topology` never makes a live call;
- `refresh_nns_subnet_topology` always performs a live refresh;
- `load_or_refresh_missing_nns_subnet_topology` refreshes only absence; and
- `load_or_refresh_stale_nns_subnet_topology` refreshes absence or
  caller-defined staleness.

`nns_subnet_topology_freshness` derives freshness from caller-provided time
and policy without changing cache state.

The existing component-cache topology reports remain useful diagnostics for
version skew and enrichment. They are not placement authority. The aggregate
provider report now retains the Registry version metadata of each component
input so its provenance is not discarded.
