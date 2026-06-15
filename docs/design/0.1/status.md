# 0.1 Status: Snapshot Cache And Complete SNS Collections

Last updated: 2026-06-15

## Purpose

This file tracks implementation status for the 0.1 snapshot-cache design. The
design document captures the intended contract; this file records what has
landed, what remains open, and where the implementation differs from the plan.

Design: [0.1-design.md](0.1-design.md)

## Current State

`ic-query` already has cache-backed NNS component commands under `.icq/`.
Those caches use JSON files, schema versions, refresh locks, and atomic
replacement.

SNS support currently includes live deployed-SNS listing, `sns info`,
`sns token`, direct `sns proposal` lookup, bounded `sns proposals`, bounded
`sns neurons` queries, and the first complete full-collection SNS neuron
snapshot refresh path. `icq sns neurons refresh` writes
`.icq/sns/ic/<root-principal>/neurons/full.json` only after the SNS governance
API is exhausted. Failed and capped attempts leave the published complete cache
untouched.

Cache-backed SNS neuron sorting is implemented for `id`, `stake`, `maturity`,
and `created`. The default `api` sort remains a bounded live query.
Refresh emits a same-line stderr progress counter with pages and rows fetched
when stderr is attached to a terminal. Text neuron tables shorten neuron IDs to
eight characters by default; `--verbose` preserves full IDs. Text output
renders current SNS token amounts, including fee, supply, stake, and maturity,
as two-decimal token amounts while JSON keeps raw base units.
`icq sns neurons cache list` and `icq sns neurons cache status
<id|root-principal>` inspect local complete snapshots and latest
refresh-attempt metadata without making live SNS-W or governance calls.

The first implementation reuses the existing cache-file primitives directly in
the SNS report layer. The reusable cross-command snapshot abstraction remains
open follow-through.

## Implementation Checklist

- [ ] Add a reusable snapshot cache module with logical `SnapshotKey` values.
- [ ] Add a `SnapshotEnvelope<T>` carrying schema, provenance, scope, and
      completeness metadata.
- [x] Add JSON backend path encoding under `.icq/`.
- [x] Add refresh-attempt files separate from published complete snapshots.
- [x] Add refresh locking and atomic complete-snapshot commit for SNS neurons.
- [ ] Add a generic paged collection refresh helper.
- [x] Implement SNS neuron full-collection paging.
- [x] Add `icq sns neurons refresh <id|root-principal>`.
- [x] Preserve previous complete SNS neuron snapshots when a page fails.
- [x] Reject incomplete attempts for global sort/ranking commands.
- [x] Add client-side SNS neuron sorting over complete snapshots.
- [x] Add terminal progress output for long SNS neuron refreshes.
- [x] Compact SNS neuron IDs in text tables with a `--verbose` full-ID escape
      hatch.
- [x] Add local cache list/status visibility for complete SNS neuron snapshots
      and refresh attempts.
- [x] Add tests for schema rejection, failed refresh preservation, stale lock
      handling, and complete-only sorting.
- [x] Update README cache documentation once the command surface lands.
- [x] Record implementation deltas in this file before 0.1 closeout.

## Current 0.0.x Inputs

The following already-landed work informs 0.1:

- `.icq/` is the project-local cache directory.
- NNS subnet/node/provider/operator/data-center/topology caches already use
  refresh commands and atomic replacement.
- `icq sns list` preserves SNS-W deployment order for numeric ids.
- `icq sns token` performs bounded token metadata reads.
- `icq sns proposal` performs direct live `get_proposal` reads by proposal id.
- `icq sns proposals` performs bounded live `list_proposals` reads with
  `--limit`, `--before`, and status filtering.
- `icq sns neurons` performs bounded `list_neurons` reads with
  semantically-validated `--limit` and `--owner`; the 100-row cap applies to
  live API sorting, while cache-backed sorts can show larger local snapshots.
- The SNS governance `list_neurons` API exposes `of_principal`, `limit`, and
  `start_page_at`, but no caller-selected ordering field.

## Open Decisions

1. Whether missing complete SNS neuron snapshots should auto-refresh on first
   complete-only sort or require an explicit `refresh` command.
   Current implementation requires an explicit `refresh` command.
2. The default page size for complete SNS neuron refresh.
   Current implementation defaults to 100.
3. Whether owner-scoped neuron snapshots are in scope for the first 0.1 slice
   or deferred until full snapshots are stable.
   Current implementation defers owner-scoped snapshots.
4. Whether refresh attempts should retain partial rows for debugging or only
   retain progress/error metadata.
   Current implementation retains progress/error metadata only.
5. Whether a cross-SNS aggregate command belongs in 0.1 or a later release.
   Current implementation defers cross-SNS aggregation.

## Known Risks

- The source SNS governance canister may mutate while `ic-query` pages through
  neurons. Unless the API guarantees point-in-time pagination, `ic-query` can
  only report that the API was exhausted during the scan.
- A high page count can create operator-visible latency and API load. Refresh
  commands need clear output and bounded safety controls.
- Adding SQLite too early would increase dependency and packaging complexity
  before the query workload proves it is needed.
- Caching command slices would create versioning and invalidation pressure.
  0.1 should cache collections, not views.

## Validation Status

Current focused validation:

```text
make clippy
make test
cargo fmt --all -- --check
git diff --check
```

All passed during the 0.1.27 cleanup, including the split of IC registry
transport helpers into focused version, value, chunk, and protobuf codec
modules, the split of IC registry relation helpers into focused model, subnet
assignment, and count aggregation modules, and the split of IC registry
relation inventory tests into focused node-provider, node-operator, node, and
data-center modules. Prior validation covered the 0.1.26 cleanup, including
the split of IC registry protobuf wire types into focused id, registry,
routing, subnet, and node/data-center modules, plus the split of IC registry
domain projection mappers into focused node-provider, node-operator, node, and
data-center modules while preserving existing internal root imports. Prior
validation covered the 0.1.25 cleanup, including the split of NNS node,
node-provider, node-operator, and data-center report model roots into focused
request, report, and host-error modules while preserving existing public report
exports, plus the split of NNS topology runtime handling into focused dispatch,
read-runner, and refresh modules, and the split of topology report request
wiring into focused model, cache, list, and refresh request-builder modules.
Prior validation covered the 0.1.24 cleanup, including the
split of subnet catalog core tests into focused schema, resolver, validation,
and fixture modules, the split of subnet catalog domain models into focused
classification, type, and validation modules, the split of subnet catalog text
rendering into focused list, info, refresh, and principal-compaction modules,
and the split of NNS topology clap command construction into focused root,
read, refresh, and usage modules. Earlier validation covered the 0.1.23 cleanup,
including the split of NNS topology
report models into focused per-report modules, the split of topology provider
report assembly into focused report, accumulator, and provider-status modules,
and the split of topology summary report assembly into focused count,
join-coverage, and registry-version helper modules, plus the split of shared
topology report test fixtures into focused subnet, node, node-provider,
node-operator, and data-center fixture modules. Earlier validation covered the
0.1.22 cleanup, including the split of NNS node, node-provider, node-operator,
and data-center report roots into focused cache, refresh,
resolve, source, and node-filter modules while preserving the existing report
API and cache behavior, plus the split of subnet catalog host support into
focused cache, error, path, refresh, and live-source modules while preserving
refresh locking and cache behavior, and the split of subnet catalog report
support into focused model, list, info, and rate/applicability modules.
Earlier validation covered the 0.1.21
cleanup, including the split of NNS topology report tests into focused
summary, coverage, versions, health, gaps, capacity, regions, providers, and
refresh modules with shared fixtures, the split of broad NNS CLI parsing,
help, and local-network rejection tests into focused command-family modules,
the split of NNS node-provider report tests into focused list, info, text,
refresh, and fixture modules, the split of IC registry and subnet catalog
tests into focused behavior modules with shared fixtures, the split of SNS
command parsing and usage tests into focused command-family modules, and the
split of SNS neuron-cache complete-collection paging and progress reporting
out of refresh orchestration. Earlier validation covered the 0.1.20 SNS report
cleanup, including the split of SNS report assembly into focused list/info, token,
parameters, proposal, proposals, and neurons modules, the split of SNS report
builder orchestration out of the report module root, and the split of the SNS
report test suite into focused list/info, token, parameters, proposal, and
neuron test modules with decomposed shared request and source fixtures. SNS
neuron tests are further grouped by live rendering, cache refresh, cache
status, and cache error behavior, and cached NNS report tests reuse the shared
unique temp-directory helper. Earlier validation also covered the split of live
SNS fetch wiring into deployed SNS listing, ledger token metadata, governance
parameters, proposals, and neuron paging modules, focused live SNS
conversion modules for deployed SNS canisters, token metadata, proposal rows,
and neuron rows, focused SNS report model modules for list, token, governance,
proposal, and neuron reports, shared NNS cache path construction, JSON-cache
error mapper reuse, shared cached JSON report use, refresh text rendering
reuse, standard cached-leaf report adapter reuse, common NNS mainnet network
enforcement, shared NNS macro module organization, and the 0.1.18 command
module cleanup. The cache tests cover schema rejection, failed refresh
preservation, stale lock recovery, complete cache sorting, and malformed subnet
routing error handling. The prior live read-only proposal smoke against
`https://icp-api.io` also succeeded outside the sandbox.
