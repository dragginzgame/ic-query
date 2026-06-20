# 0.1 Status: Snapshot Cache And Complete SNS Collections

Last updated: 2026-06-16

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

The latest cleanup removes the topology read runner trait and generated
zero-sized runner structs in favor of one direct shared read-command helper.
Topology read command and option tables now use explicit module aliases
instead of long repeated item import lists, topology read option parser tests
share common assertions and a local test macro, and NNS macro plumbing is
split into focused modules under `src/nns/macros/`.

The previous cleanup centralizes CLI argument collection behind one private
helper used by first-argument help/version handlers. Cached NNS leaf list,
info, and refresh runtime paths also share one project-root lookup,
cache-request constructor, and command timestamp setup. SNS neuron cache
list/status runtime paths now share format, network, and project-root setup.
SNS command option parsers share one helper for mapping clap parse failures to
SNS usage errors. NNS option parsers and NNS/SNS required-subcommand dispatch
paths also use namespace-local wrappers for clap usage-error mapping.

The previous cleanup centralizes compact-vs-verbose text/JSON output behind a
shared writer helper. NNS subnet list, custom node list, and cached leaf list
commands now use the same compact/verbose renderer selection instead of
open-coding that branch in each run path. Clap passthrough subcommand-argument
extraction is also shared between required-subcommand parsing and top-level
command dispatch, removing a duplicate hidden argument constant. Custom NNS
node commands and NNS subnet catalog commands now centralize project-root
lookup plus cache-request construction in their runtime cache helpers.

The previous cleanup centralizes SNS lookup command runtime fields behind
shared lookup command parts for format, network, source endpoint, timestamp,
and input. `sns info`, `sns token`, `sns params`, `sns proposal`,
`sns proposals`, `sns neurons`, and `sns neurons refresh` now build their
report requests from the shared lookup parts instead of manually unpacking
`SnsLookupOptions`. SNS runtime commands also share a small command-argument
prelude for collecting args and handling first-argument help/version output.
NNS dispatch, subnet, node, registry, topology, and cached leaf run paths now
share matching command-argument helpers for regular and flag-only help/version
handling. Top-level, NNS, and SNS command adapters use shared CLI helpers for
command-argument collection and first-argument help/version checks.

The previous cleanup removes the remaining production wildcard imports from
SNS governance-parameter text rendering and centralizes clap parse-to-usage
handling behind shared helpers used by top-level, NNS, SNS, subnet, registry,
topology, and leaf option parsers. NNS run paths also share one project-root
helper for converting local root lookup failures into command usage errors.
Shared CLI first-argument help/version handling now uses one private helper
while preserving the existing command-specific version alias behavior.

The previous cleanup tightens module APIs by replacing wildcard re-exports in
SNS report/model/source roots, cached NNS leaf report roots, subnet catalog
report roots, and topology report fixtures with explicit export lists. SNS
nested governance helper structs remain test-only root exports because
production report code only needs the containing governance-parameters model.

The previous cleanup adds shared NNS leaf request accessors and a shared NNS
leaf JSON refresh-cache writer. Node, node-provider, node-operator, and
data-center refresh now use the shared writer for cache-path construction and
`RefreshCacheWriteRequest` setup while preserving their report fields and
cache behavior. Topology NNS component cache-request adapters also use the
shared leaf cache constructor, while subnet catalog cache requests remain
separate. The shared NNS leaf writer has fixture coverage for dry-run output
and real cache replacement behavior.

The previous cleanup extends the shared `snapshot_cache` module with locked
snapshot refresh setup, paged refresh orchestration, and refresh-attempt
lifecycle handling. Complete SNS neuron refresh uses the shared runners for
parent directory creation, refresh locking, replacement detection, progress
lifecycle, max-page cutoff, fetch-failure reporting, running/failed attempt
handling, attempt-progress callbacks, and collection-exhaustion handling while
preserving SNS-specific page fetching, attempt metadata, and error reporting.
SNS neuron attempt-file construction is centralized behind status-specific
starting, running, complete, and failed writers instead of being rebuilt at
each refresh call site.

## Implementation Checklist

- [x] Add a reusable snapshot cache module with logical `SnapshotKey` values.
- [x] Add shared snapshot envelope and completeness primitives for the current
      flattened JSON backend.
- [x] Add a published `SnapshotEnvelope<T>` shape carrying logical domain,
      entity, collection, and scope metadata for newly written complete
      snapshots while preserving compatibility with older local JSON caches.
- [x] Add JSON backend path encoding under `.icq/`.
- [x] Add shared JSON snapshot loading/writing, header loading,
      refresh-attempt, and cache discovery helpers.
- [x] Add refresh-attempt files separate from published complete snapshots.
- [x] Add refresh locking and atomic complete-snapshot commit for SNS neurons.
- [x] Add a reusable paged collection state helper for row de-duplication,
      cursor tracking, and exhaustion checks.
- [x] Add a generic paged collection refresh helper.
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

All passed during the 0.1.49 cleanup, including the direct topology read
command helper, topology read command/option import cleanup, deduplicated
topology read option parser tests, and split NNS macro modules. Prior
validation covered the 0.1.48 cleanup, including shared CLI argument
collection for first-argument help/version handlers and shared cached NNS leaf
project-root lookup, cache-request construction, and command timestamp setup,
plus shared SNS neuron cache command setup, shared SNS option parser
usage-error mapping, and shared NNS/SNS clap usage-error wrappers for option
parsing and required-subcommand dispatch. Prior validation covered the 0.1.47
cleanup, including the shared
compact-vs-verbose text/JSON writer and migration of NNS subnet list, custom
node list, and cached leaf list output onto it, plus shared clap
passthrough-argument extraction for required-subcommand parsing and top-level
dispatch, plus shared custom NNS node and NNS subnet runtime cache-request
setup. Prior validation covered the 0.1.46 cleanup,
including shared SNS lookup command parts and migration of info, token,
params, proposal, proposals, neurons, and neuron refresh runtime request
construction, plus shared SNS command args/help/version handling and shared
NNS command args/help/version helpers. Top-level, NNS, and SNS adapters now
share CLI-level argument collection and first-argument help/version handling.
Prior validation covered the 0.1.45 cleanup, including production
wildcard-import removal, shared clap parse-to-usage helper migration for
top-level, NNS, SNS, subnet, registry, topology, and leaf option parsers,
shared NNS project-root usage-error handling across subnet, node, topology,
and cached leaf run paths, and deduplicated first-argument help/version
handling. Prior validation covered the 0.1.44 cleanup, including
explicit export-list coverage for SNS report/model/source roots, cached NNS
leaf report roots, subnet catalog report roots, and topology report fixtures.
Prior validation covered the 0.1.43 cleanup, including shared NNS leaf
refresh-cache writer extraction and migration of node, node-provider,
node-operator, and data-center refresh paths plus topology NNS component
cache-request adapters onto shared leaf cache helpers, with focused coverage
for dry-run output and real cache replacement behavior. Prior validation
covered the 0.1.42 cleanup, including
shared locked snapshot refresh setup, shared paged snapshot refresh
orchestration, shared refresh-attempt lifecycle handling, and migration of
complete SNS neuron refresh setup, paging, and start/failure attempt handling
onto the generic runners, plus centralized SNS neuron attempt-file writers.
Prior validation covered the 0.1.41 cleanup, including shared snapshot JSON loading/writing,
snapshot-header loading, refresh-attempt envelopes/read-write helpers,
full-collection path scanning, and SNS neuron cache read/write/attempt
migration onto those helpers. Prior validation covered the 0.1.40 cleanup,
including shared snapshot-cache key/path helpers, flattened snapshot
envelope/completeness primitives, shared paged-collection state, and SNS neuron
cache migration with cache JSON shape coverage. Prior validation covered the
0.1.39 cleanup, including the split of SNS report source definitions, lookup
handling, live fetch handling, shared text helpers, neuron report models,
complete neuron collection paging, and live proposal conversion into focused
modules. Prior
validation covered the 0.1.38 cleanup, including the split of NNS node,
node-provider, node-operator, and data-center text rendering into focused
list, info, and refresh modules plus the split of topology summary and capacity
text rendering into focused table-helper modules and the split of topology
health, capacity, refresh, and gap report construction into focused
derived-helper modules, including a shared topology relation index for summary
join coverage and gap detection. Prior validation covered the 0.1.37 cleanup,
including the split of NNS registry, node, and cached component report roots
into focused build, source, model, text, and refresh modules.
Prior validation covered the 0.1.36 cleanup, including the
split of shared NNS leaf command construction, option parsing, and runtime
helpers plus the split of NNS component command/runtime/spec/report-adapter
wiring into focused modules. Prior validation covered the 0.1.35 cleanup,
including the split of
SNS governance-parameter text rendering into focused category modules, the
split of SNS neuron command specification/runtime dispatch into focused root,
cache, refresh, and helper modules, and the split of NNS topology provider
accumulation into focused ingestion, data-center association, and
row-projection modules, plus the split of NNS topology read-command
construction and option parsing into focused read/help/refresh modules. Prior
validation covered the 0.1.34 cleanup, including the split of live SNS Candid
wire types into
focused deployed-SNS, token metadata, proposal, and neuron API modules, plus
the split of SNS proposal text rendering into focused single-proposal,
proposal-list, and shared detail-line modules. Prior validation covered the
0.1.33 and 0.1.30 cleanups, including the split of SNS
text/build orchestration, NNS topology build/read orchestration, subnet catalog
list text rendering into focused compact, verbose, and range-line modules, plus
the split of subnet catalog classification model enums into focused
subnet-kind, specialization, geographic-scope, and source modules, and the
split of subnet catalog root error, JSON parse/render, and principal parsing
helpers into focused modules. Prior validation covered the 0.1.29 cleanup,
including the split of subnet catalog resolver behavior into focused model,
subnet, prefix, and canister-routing modules, plus the split of subnet catalog
report model definitions into focused stale-status, list, info, and refresh
modules. Prior validation covered the 0.1.28 cleanup, including the split of
IC registry live-source queries into
focused agent/canister setup, registry-version, subnet-catalog, governance
node-provider, and node-relation list modules, plus the split of IC registry
relation-inventory fetch orchestration into focused fetch, data-center record,
and registry-key helper modules, and the split of IC registry public model
definitions into focused request, registry-version, node-provider,
node-operator, node, and data-center modules. Prior validation covered the
0.1.27 cleanup, including the split of IC registry transport
helpers into focused version, value, chunk, and protobuf codec modules, the
split of IC registry relation helpers into focused model, subnet assignment,
and count aggregation modules, and the split of IC registry relation inventory
tests into focused node-provider, node-operator, node, and data-center modules.
Prior validation covered the 0.1.26 cleanup, including the split of IC registry
protobuf wire types into focused id, registry, routing, subnet, and
node/data-center modules, plus the split of IC registry domain projection
mappers into focused node-provider, node-operator, node, and data-center
modules while preserving existing internal root imports. Prior validation
covered the 0.1.25 cleanup, including the split of NNS node,
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
