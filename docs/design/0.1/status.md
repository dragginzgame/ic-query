# 0.1 Status: Snapshot Cache And Complete SNS Collections

Last updated: 2026-06-13

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
`sns token`, and bounded `sns neurons` queries. SNS neurons do not yet have a
complete local snapshot cache. Sorting or ranking neurons across the full SNS
collection is therefore not implemented as authoritative behavior.

The intended 0.1 direction is to reuse the existing cache-file primitives while
introducing a more general snapshot abstraction that can later be backed by
SQLite without changing CLI behavior.

## Implementation Checklist

- [ ] Add a reusable snapshot cache module with logical `SnapshotKey` values.
- [ ] Add a `SnapshotEnvelope<T>` carrying schema, provenance, scope, and
      completeness metadata.
- [ ] Add JSON backend path encoding under `.icq/`.
- [ ] Add refresh-attempt files separate from published complete snapshots.
- [ ] Add generic refresh locking and atomic complete-snapshot commit.
- [ ] Add a generic paged collection refresh helper.
- [ ] Implement SNS neuron full-collection paging.
- [ ] Add `icq sns neurons refresh <id|root-principal>`.
- [ ] Preserve previous complete SNS neuron snapshots when a page fails.
- [ ] Reject incomplete attempts for global sort/ranking commands.
- [ ] Add client-side SNS neuron sorting over complete snapshots.
- [ ] Add tests for schema rejection, failed refresh preservation, stale lock
      handling, and complete-only sorting.
- [ ] Update README cache documentation once the command surface lands.
- [ ] Record implementation deltas in this file before 0.1 closeout.

## Current 0.0.x Inputs

The following already-landed work informs 0.1:

- `.icq/` is the project-local cache directory.
- NNS subnet/node/provider/operator/data-center/topology caches already use
  refresh commands and atomic replacement.
- `icq sns list` preserves SNS-W deployment order for numeric ids.
- `icq sns token` performs bounded token metadata reads.
- `icq sns neurons` performs bounded `list_neurons` reads with clap-validated
  `--limit` and `--owner`.
- The SNS governance `list_neurons` API exposes `of_principal`, `limit`, and
  `start_page_at`, but no caller-selected ordering field.

## Open Decisions

1. Whether missing complete SNS neuron snapshots should auto-refresh on first
   complete-only sort or require an explicit `refresh` command.
2. The default page size for complete SNS neuron refresh.
3. Whether owner-scoped neuron snapshots are in scope for the first 0.1 slice
   or deferred until full snapshots are stable.
4. Whether refresh attempts should retain partial rows for debugging or only
   retain progress/error metadata.
5. Whether a cross-SNS aggregate command belongs in 0.1 or a later release.

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

No 0.1 implementation has landed yet. This documentation was added as the
planning baseline for the snapshot cache work.
