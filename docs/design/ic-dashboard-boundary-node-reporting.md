# IC Dashboard Boundary-Node Reporting

## Status

- Status: implemented
- Authority: official IC Dashboard REST API
- Collection mode: one finite live resource lookup
- Public family: `ic_query::ic`

## Decision

`icq ic network boundary-node-data-centers` queries the official v4
`boundary-node-data-centers` resource through the existing `LiveIcSource`.
The public `IcNetworkSource` capability owns bounded network-resource reports;
`fetch_boundary_node_data_centers` remains distinct from its bounded
daily-statistics operation without adding another concrete live adapter.

The API returns data-center aggregates, not individual boundary-node records.
The command and public types therefore say `data-centers` explicitly rather
than implying that a row identifies one node.

## Report Contract

`IcBoundaryNodeDataCentersReport` preserves:

- the raw Dashboard data-center id, name, owner, and region label;
- raw latitude and longitude strings;
- the raw per-data-center `total_nodes` string;
- locations that currently report zero nodes;
- a derived data-center count and checked sum of per-row node counts;
- endpoint and retrieval provenance.

Rows are canonically ordered by data-center id. Projection rejects more than
1,000 rows, duplicate ids, empty identity/location labels, coordinates that
are not finite decimal values in geographic range, non-canonical unsigned
node-count text, and a node-count sum that overflows `u64`.

Region text is deliberately not parsed, corrected, or normalized. It is
off-chain Dashboard data and may contain labels that look surprising or
internally inconsistent. Preserving that raw value is more honest than
silently substituting Registry geography or a local interpretation.

Custom sources must echo the exact shared `IcSourceRequest`. The live decoder
requires every current response field while tolerating additive fields at the
row and response-object levels.

## Authority and Freshness

The default endpoint is:

```text
https://ic-api.internetcomputer.org/api/v4
```

The base endpoint must be HTTP(S), include a host, and contain no query or
fragment. The report uses the shared Dashboard provenance contract:

- `network: ic`;
- `authority: official_ic_dashboard_api`;
- `certified: false`;
- `point_in_time_guaranteed: false`.

The response is the complete result of a resource with no pagination
parameters, but that does not make it certified Registry inventory or prove
that every physical boundary node is represented at one point in time.

## User-Facing Usage

```bash
icq ic network boundary-node-data-centers
icq ic network boundary-node-data-centers --format json
```

The command makes exactly one request and performs no per-location follow-up.
Command help is the definitive option reference.

## Cache Contract

The report is live-only. It does not read, write, invalidate, or migrate cache
files. The resource is currently small and does not require pagination or a
multi-call snapshot flow.

## Scope

This report does not:

- expose individual boundary-node principals, domains, IP addresses, or
  operational health;
- join Dashboard locations to Registry node or data-center records;
- correct or reinterpret raw Dashboard region labels;
- claim certified or point-in-time-complete topology;
- issue hidden follow-up calls or create a cache.
