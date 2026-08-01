# IC Dashboard Network Metrics

## Status

- Status: implemented
- Authority: official IC Dashboard Metrics REST API
- Collection mode: one bounded live time-series lookup
- Public family: `ic_query::ic`

## Decision

`icq ic metrics <metric>` queries one documented Metrics API resource through
the existing `LiveIcSource`. The first supported aggregate metric set is:

| CLI metric | Raw response series |
| --- | --- |
| `instruction-rate` | `instruction_rate` |
| `message-execution-rate` | `message_execution_rate` |
| `cycle-burn-rate` | `cycle_burn_rate` |
| `block-rate` | `block_rate` |
| `ic-node-count` | `total_nodes`, `up_nodes` |
| `ic-subnet-total` | `ic_subnet_total` |
| `registered-canisters-count` | `running_canisters`, `stopped_canisters` |
| `total-ic-energy-consumption-rate-kwh` | `energy_consumption_rate` |
| `boundary-nodes-count` | `boundary_nodes_count` |

The public `IcMetricSource` capability lets fixtures, mirrors, and proxies use
the same request validation, projection, and rendering path. It is implemented
by `LiveIcSource`; there is no adapter type per metric endpoint.

## Request and Size Contract

Every query supplies an inclusive Unix-second `start`, inclusive `end`, and a
positive `step`. The defaults are the preceding hour ending at collection time
with a 300-second step. The builder enforces the Metrics API's earliest
accepted timestamp (`1620432000`) and maximum step (`259200` seconds), rejects
an end later than collection time, and caps the requested window at 1,000
observations per returned series.

The live source validates the range before parsing the endpoint. It sends
exactly one JSON request containing `format`, `start`, `end`, and `step`. It
does not paginate, follow another endpoint, expand per Subnet, or collect
another metric. Subnet and message-type filtering are outside this first
aggregate-network slice.

## Report Contract

`IcMetricReport` preserves:

- the selected official metric path name;
- the exact requested start, end, and step;
- every documented raw response series;
- each observation's raw Unix timestamp and value string;
- series and observation counts;
- endpoint and retrieval provenance.

Value strings are not converted to floating-point JSON numbers. This avoids
loss of precision and keeps the source representation available to downstream
consumers. Text rendering is a compact timestamp/value view; JSON retains the
raw fields.

Custom sources must echo the exact `IcMetricQuery` and shared
`IcSourceRequest`. Projection requires exactly the documented series for the
selected metric, canonical series-name order, strictly increasing observation
timestamps inside the requested window, no more observations than the request
permits, and non-empty raw values. Additive live response fields are ignored,
while a missing or malformed required series is a typed decode error.

## Authority and Freshness

The default endpoint is:

```text
https://metrics-api.internetcomputer.org/api/v1
```

The endpoint must be HTTP(S), include a host, and contain no query or fragment.
The report uses the shared Dashboard authority and provenance contract:

- `network: ic`;
- `authority: official_ic_dashboard_api`;
- `certified: false`;
- `point_in_time_guaranteed: false`.

The official Dashboard Metrics API is timestamped off-chain analytics. Its
values do not inherit a Registry version, certified state-tree authority, or a
claim that distinct observations and series form one IC state snapshot.

## User-Facing Usage

```bash
icq ic metrics instruction-rate
icq ic metrics cycle-burn-rate \
  --start 1700000000 --end 1700003600 --step 300
icq ic metrics ic-node-count --json
```

Command help is the definitive option and metric-name reference.

## Cache Contract

Metric reports are live-only. They do not read, write, invalidate, or migrate
cache files. A future long-range or multi-metric snapshot would require a
separate explicit design with timestamped identity and a durable size policy;
it must not silently turn this bounded command into a history collector.

## Scope

This report does not:

- query an unbounded history or all Metrics API resources;
- issue one request per Subnet or metric;
- expose per-Subnet or message-type filters;
- claim that Dashboard metrics are certified IC state;
- replace Registry topology, native Governance metrics, or certified ledger
  evidence.
