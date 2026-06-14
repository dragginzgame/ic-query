# 0.1 Status: Snapshot Cache And Complete SNS Collections

Last updated: 2026-06-14

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

All passed during the 0.1.18 command-module cleanup, including the split of the
filtered NNS node command into command construction, option parsing, and
runtime dispatch modules, plus the split of SNS command runtime dispatch,
command error definitions, runtime handlers, clap command construction, and
option parsing into focused submodules. The prior validation also covered
shared NNS cache path construction, JSON-cache error mapper reuse, shared
cached JSON report use, refresh text rendering reuse, standard cached-leaf
report adapter reuse, common NNS mainnet network enforcement, and shared NNS
macro module organization. The cache tests cover schema rejection, failed
refresh preservation, stale lock recovery, complete cache sorting, and
malformed subnet routing error handling. The prior live read-only proposal
smoke against `https://icp-api.io` also succeeded outside the sandbox.
