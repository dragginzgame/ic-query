# NNS Governance Economics, Metrics, and Rewards

## Status

- Status: implemented for the 0.16.0 slice; canister runtime added in 0.38.0
- Authority: mainnet NNS Governance canister
- Public boundary: native economics, cached metrics, latest reward event, and
  maturity modulation query responses
- Collection guarantee: one live replica query or replicated inter-canister
  call per report

## Authority and Scope

`ic-query` reads four public methods from the NNS Governance canister through
`LiveNnsSource` on native hosts or `CanisterNnsSource` in replicated canister
execution:

| Report | Governance method |
| --- | --- |
| economics | `get_network_economics_parameters` |
| metrics | `get_metrics` |
| reward event | `get_latest_reward_event` |
| maturity modulation | `get_maturity_modulation` |

The portable async `NnsGovernanceSource` capability exposes the same four
operations for fixture, mirror, proxy, or pre-collected implementations. Every
operation uses `NnsGovernanceRequest`, which selects either a replica query or
a replicated inter-canister call. Every report records the network, fixed
Governance canister principal, collection timestamp, and one tagged
`NnsGovernanceSourceProvenance` value. Replica provenance carries the endpoint
and collector label; inter-canister provenance carries the executing collector
canister principal. Execution assurance is derived from that variant rather
than stored independently.

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

Only the mainnet `ic` network is supported. Public builders and both built-in
source adapters reject another network before constructing a transport or
making a live call. Replica endpoints must be credential-free absolute
HTTP(S) URLs without a query or fragment. The inter-canister selection cannot
override the mainnet Governance principal or claim an HTTP endpoint.

Each report comes from one Governance response. The native adapter submits an
ordinary unreplicated replica query. The canister adapter uses one
`Call::bounded_wait` from replicated execution, attaches no cycles, performs
no retry, bounds the raw response before Candid decoding, and records the
executing canister principal. Composite-query collection is not supported.
Neither transport joins independently timed calls or inherits a Registry
version. The caller-supplied collection timestamp is report provenance, while
native timestamps such as metrics `timestamp_seconds`, reward-event
`actual_timestamp_seconds`, and maturity-modulation
`updated_at_timestamp_seconds` remain unchanged inside the payload.

`get_metrics` can return a typed Governance application error. That error is
preserved separately from agent calls, inter-canister rejects, Candid encoding,
Candid decoding, response-size validation, and local runtime failures.

## Cache Contract

These four bounded point-value reports are live-only. They do not read or
write the proposal or neuron complete-collection caches, and they do not
introduce another cache identity, stable-memory layout, lock, refresh policy,
or stale-data claim. A consuming process or canister owns scheduling, retries,
cycle budgeting, and persistence of the returned Serde report.

Proposal and public-neuron collection separately expose serializable schema-1
continuations. Each advance performs one bounded family-specific page call and
returns the page with candidate next state. Each state binds source identity
and concrete collector provenance, accounts cumulative pages and rows under an
explicit page ceiling, and distinguishes API exhaustion from a page-limit
stop. Proposal cursors descend; neuron cursors ascend and remain exclusive.
Neither continuation prescribes a storage or publication format, and
sequential pages are not point-in-time.

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
