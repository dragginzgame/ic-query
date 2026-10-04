# NNS Neuron Reporting

## Status

- Status: implemented through the 0.40.0 slice
- Authority: mainnet NNS Governance canister
- Public boundary: unauthenticated `NeuronInfo` views
- Collection guarantee: API exhaustion without a point-in-time guarantee

## Authority and Scope

`ic-query` reads the native NNS Governance `get_neuron_index` and
`get_neuron_info` methods through `LiveNnsSource` on native hosts or
`CanisterNnsSource` in replicated canister execution. The list and detail
reports preserve the public `NeuronInfo` fields exposed by Governance,
including raw state, visibility, neuron-type, and vote discriminants; stake;
staked maturity; dissolve delay; age; voting-power fields; public timestamps;
known-neuron metadata; and recent ballots.

These are publicly readable limited neuron views. They are not authenticated
owner views and do not claim access to controllers, followees, private
unstaked maturity, disbursement state, or other fields that Governance does
not expose through `NeuronInfo`. Dashboard analytics are not blended into the
on-chain report.

Only the mainnet `ic` network is supported. Public builders and the direct
live source reject another network before constructing an agent or making a
request.

## Pagination and Validation

Governance accepts an exclusive neuron-id cursor and caps each page at 300
rows. `ic-query` preserves that cursor contract and requires every page to:

- contain no more than the requested page size;
- be strictly ascending with unique neuron ids;
- begin after the requested exclusive cursor; and
- expose the last row id as the next cursor exactly when a full page leaves
  collection exhaustion unproven.

Complete refresh walks the same endpoint and Governance canister until a
short or empty page proves API exhaustion. Page admission validates row
classifications and strict order; the exclusive cursor preserves that order
across pages. Publication moves the admitted rows without revalidating them.
The collector retains the row vector without allocating a second full-history
id set. Disk reads and caller-owned analytics inputs validate their rows at
their own trust boundaries.

The portable `NnsNeuronCollectionState` and
`advance_nns_neuron_collection_with_source` API expose this walk as one bounded
call per advance. The schema-1 state binds request identity, concrete source
provenance, cursor, timestamps, and cumulative page and row accounting under a
caller-selected page ceiling. `complete` means observed API exhaustion;
`page_limit_reached` retains incomplete progress. Native refresh uses the same
engine and follows its completion status rather than independently evaluating
page length and cursor presence. Canister callers own page retention,
scheduling, and publication.
Request and restored-state collection timestamps must parse as valid UTC
dates and times with second precision before another source call is admitted.
After page admission, the next continuation is derived without repeating the
incoming-state validator. Native list and detail report construction derives
cache origin from the optional cache path; the public `from_cache` field remains
available in both Rust reports and JSON.

## Portable Distribution Contract

`build_nns_neuron_distribution_report` accepts a valid final collection state
and every retained row accounted for by it. It makes no network or cache call.
The schema-1 output preserves collection and source provenance, the false
point-in-time guarantee, and the row retrieval-time range.

The report provides raw-code state, optional visibility, and optional neuron-
type counts with effective-stake totals. It also keeps reported and unreported
coverage separate for optional staked maturity, deciding voting power, and
potential voting power. Known-neuron metadata and Neurons' Fund join-timestamp
presence are factual counts, not owner or membership inference.

Fresh reports and restored caller-owned reports use the same pure validator.
The validator checks both collection timestamps as well as page and row
accounting before accepting a projected report.
API exhaustion and internally consistent aggregates do not authenticate the
retained rows, recover private neuron fields, or establish an atomic
Governance balance.

## Cache Contract

The complete snapshot identity is:

```text
domain: nns
network: ic
entity: governance
collection: neurons
scope: full
```

It is stored at:

```text
<cache-root>/nns/ic/governance/neurons/full.json
```

The page size, list limit, exclusive cursor, verbosity, and output format are
operation or view choices and do not alter snapshot identity. Refresh uses
the shared snapshot lock and attempt sidecar, publishes only after API
exhaustion, and leaves a previous complete snapshot unchanged after a failed
or diagnostically capped attempt.

Governance does not expose a stable collection version for this walk. Neurons
may change between pages, so snapshots and derived reports always state
`point_in_time_guaranteed: false`. API exhaustion means the ordered walk
completed; it does not mean every row describes one Governance instant.

List and detail commands prefer an existing valid complete snapshot and make
a bounded live query when the snapshot or requested row is absent. Cache
parse, schema, identity, and validation failures remain visible. `refresh` is
the only CLI operation that writes this snapshot, while `cache status` is
strictly local.

## Adapter Contract

`NnsNeuronSource` is the portable async capability for fixture, mirror, proxy,
or pre-collected implementations. It accepts `NnsGovernanceRequest` and
returns tagged source provenance; native and canister adapters share validation,
row conversion, and report assembly with proposal reporting. Native calls are
ordinary replica queries. The canister adapter uses one response-bounded
replicated inter-canister call per page or exact detail, records the executing
collector principal, attaches no cycles, and performs no retry or persistence.
Neither transport certifies the returned public neuron rows.

Proposal and neuron native complete collections also share the
public `NnsGovernanceRefreshRequest`, `NnsGovernanceCacheRequest`,
`NnsGovernanceRefreshAttemptStatus`, and `NnsGovernanceQueryError` contracts;
their page validation, cache identities, and reports remain family-specific.
Custom sources must honor the same ordered pagination and exact detail-id
contracts; report builders validate their results before projection or cache
publication.
