# NNS Governance Economics, Metrics, and Rewards

## Status

- Status: implemented for the 0.16.0 slice
- Authority: mainnet NNS Governance canister
- Public boundary: native economics, cached metrics, latest reward event, and
  maturity modulation query responses
- Collection guarantee: one live canister query per report

## Authority and Scope

`ic-query` reads four public query methods from the native NNS Governance
canister through `LiveNnsSource`:

| Report | Governance method |
| --- | --- |
| economics | `get_network_economics_parameters` |
| metrics | `get_metrics` |
| reward event | `get_latest_reward_event` |
| maturity modulation | `get_maturity_modulation` |

The public `NnsGovernanceSource` capability exposes the same four operations
for fixture, mirror, proxy, or pre-collected implementations. Every operation
uses the shared `NnsSourceRequest`, and every report records the network,
Governance canister principal, source endpoint, collection timestamp, and
collector.

These reports preserve Governance terminology and values. JSON retains the
full current native response fields, including nested optional wrappers,
signed permyriad maturity modulation, proposal-id records, neuron-subset
metrics, and all metric buckets. The unlabeled Candid bucket pairs are
represented as named `{ "key": ..., "value": ... }` rows without changing
their numeric values. A non-finite floating-point bucket is rejected as an
invalid metrics response because JSON would otherwise replace it with `null`.

The latest reward event is not reward history. Cached Governance metrics are
canister-reported metrics, not certified Registry state or Dashboard
analytics. These reports do not claim authenticated neuron-owner access,
delegation/followee state, private maturity, or historical node-provider
reward coverage.

## Network and Query Contract

Only the mainnet `ic` network is supported. Both public builders and the
direct `LiveNnsSource` methods reject another network before constructing an
agent or making a live call.

Each report comes from one Governance query response. It does not join
independently timed calls and does not inherit a Registry version. The
collection timestamp records when `ic-query` initiated the report, while
native timestamps such as metrics `timestamp_seconds`, reward-event
`actual_timestamp_seconds`, and maturity-modulation
`updated_at_timestamp_seconds` remain unchanged inside the payload.

`get_metrics` can return a typed Governance application error. That error is
preserved separately from agent, Candid encoding, Candid decoding, and local
runtime failures.

## Cache Contract

These four bounded point-value reports are live-only in this slice. They do
not read or write the proposal or neuron complete-collection caches, and they
do not introduce another cache identity, lock, refresh policy, or stale-data
claim. If durable evidence becomes necessary, a future explicit snapshot
operation must retain the same canister, endpoint, collection timestamp, and
native payload timestamps.

## CLI Contract

```bash
icq nns governance economics
icq nns governance metrics
icq nns governance reward-event
icq nns governance maturity-modulation
```

All four commands are explicit live queries, render text by default, accept
`--json` for raw JSON and `--source-endpoint` overrides, and honor the global
NNS network identity.
