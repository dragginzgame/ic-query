# Changelog

All notable changes to `ic-query` will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/), and this
crate follows [Semantic Versioning](https://semver.org/).

## Unreleased

## [0.2.x] - 2026-06-16 - SNS proposal detail expansion

Detailed patch breakdown: [docs/changelog/0.2.md](docs/changelog/0.2.md)

- `0.2.27` moves NNS proposal queries under grouped commands
  `icq nns proposal list` and `icq nns proposal info <proposal-id>`, removes
  the ungrouped `icq nns proposals` and `icq nns proposal <proposal-id>` forms,
  and updates help, README examples, and parser coverage to enforce the grouped
  surface.

  ```bash
  icq nns proposal list --reward-status settled
  icq nns proposal info 132411 --ballots
  icq nns proposal info 132411 --verbose
  ```

- `0.2.26` adds `icq nns proposal <proposal-id> --ballots`,
  `icq nns proposal <proposal-id> --verbose`, and
  `icq nns proposals --reward-status <status>`, including deterministic NNS
  ballot rows in JSON, compact-by-default NNS proposal summaries, and
  API-backed NNS proposal reward-status filtering.

  ```bash
  icq nns proposal 132411 --ballots
  icq nns proposal 132411 --verbose
  icq nns proposals --reward-status settled
  ```

- `0.2.25` adds direct live NNS governance proposal list and detail queries,
  including status/topic filters, local proposal sorting, sort direction
  controls, verbose list details, and text/JSON reports that expose the
  selected view options.

  ```bash
  icq nns proposals --status open
  icq nns proposals --topic governance
  icq nns proposals --sort title --asc
  icq nns proposal 132411 --format json
  ```

- `0.2.24` adds local SNS proposal list sorting by proposer neuron id and
  reward event round, reusing the existing proposal view sorter for bounded
  live rows and complete proposal snapshots before limit truncation.

  ```bash
  icq sns proposals 1 --sort proposer
  icq sns proposals 1 --sort reward-round
  ```

- `0.2.23` adds local SNS proposal list sorting by status, ballot count, and
  reject cost, and centralizes SNS proposal sort direction policy in the
  report sort model so command parsing and report assembly share the same
  API-order versus local-sort behavior.

  ```bash
  icq sns proposals 1 --sort status
  icq sns proposals 1 --sort ballots
  icq sns proposals 1 --sort reject-cost
  ```

- `0.2.22` makes SNS proposal sort direction defaults match the selected
  sort: title/action default ascending, id/tally/timestamp sorts default
  descending, and explicit `--asc`/`--desc` is rejected for API-order views.

  ```bash
  icq sns proposals 1 --sort title
  icq sns proposals 1 --sort title --desc
  icq sns proposals 1 --sort total-votes
  ```

- `0.2.21` adds local SNS proposal list sorting by proposal title, action,
  yes tally, no tally, and total vote tally, reusing the shared proposal view
  sorter for live rows and complete proposal snapshots while keeping cache
  identity, cache paths, and JSON row fields unchanged.

  ```bash
  icq sns proposals 1 --sort title --asc
  icq sns proposals 1 --sort action
  icq sns proposals 1 --sort total-votes
  ```

- `0.2.20` shares SNS neuron/proposal refresh-attempt metadata and progress
  DTO plumbing, snapshot cache path construction, and cache-list lookup flow
  plus snapshot scan/load helpers while keeping family-specific storage error
  mapping, report DTOs, cache paths, report schemas, text output, and JSON
  fields unchanged, and documents the section-style type doc rule for scoped
  public helper types.

- `0.2.19` centralizes shared SNS neuron/proposal cache-status lookup flow
  behind associated-type cache-family traits, keeping family-specific storage
  and report DTOs separate, shares the identical refresh-attempt status DTO,
  and removes duplicated id/root cache-status branching without changing CLI
  behavior, cache paths, report schemas, text output, or JSON fields.

- `0.2.18` adds cache-backed `icq sns proposals --status decided` filtering
  for complete proposal snapshots, rejects combining that synthetic local
  status with topic filters, and documents that adopted/rejected still require
  live fallback because cached proposal rows do not carry the raw governance
  status enum.

  ```bash
  icq sns proposals 1 --status decided
  ```

- `0.2.17` adds `icq sns proposals --sort decided|executed|failed` for
  newest decision, execution, and failure proposal views, adds `--asc` and
  `--desc` direction controls for local proposal sorts, applies sorting
  through the shared proposal view layer for live rows and complete proposal
  snapshots, and keeps cache identity, cache paths, and JSON row fields
  unchanged.

- `0.2.16` moves changelog contribution rules out of the public changelog and
  into `AGENTS.md`, routes SNS nested `neurons`/`proposals` refresh/cache
  dispatch through shared clap helpers, shares cached lookup/cache command
  setup across SNS runners, and removes remaining SNS neuron/proposal cache
  re-export/timestamp helper shims without changing CLI behavior, cache
  behavior, report schemas, or output.

- `0.2.15` removes over-fragmented one-function and single-child modules
  across NNS topology, NNS leaf runtime/cache errors, SNS text/source/live
  helpers, and proposal cache reports, enforces the `module.rs` versus
  `module/mod.rs` layout rule across the touched tree, and shares SNS cache
  summary ordering plus root-principal parsing between neuron and proposal
  cache reports without changing CLI behavior, cache behavior, report schemas,
  or text output.

- `0.2.14` moves deployed SNS list sorting into the report view layer and
  leaves lookup focused on stable id assignment and input resolution, applies
  the code-hygiene module-header and module-granularity standard to SNS
  command/report modules and touched NNS topology modules, and normalizes
  touched SNS/NNS imports away from `super::super` paths without changing
  behavior.

- `0.2.13` centralizes SNS proposal and neuron row view transforms under the
  report view layer, removing cache-local filter/sort helpers without changing
  CLI behavior, cache behavior, or report schemas.

- `0.2.12` adds `icq sns proposals --sort api|id|created`, reports the
  selected proposal sort in text and JSON, and applies cache-backed proposal
  sorting before limit truncation without changing proposal cache identity.

  ```bash
  icq sns proposals 1 --sort created
  icq sns proposals 1 --sort id --limit 50
  ```

- `0.2.11` reports live-versus-cache provenance for SNS proposal list and
  detail reports, including cache path and completeness metadata when a
  complete local proposal snapshot is used.

- `0.2.10` lets `icq sns proposal` reuse an existing complete proposal
  snapshot for detail lookups before falling back to live governance reads.

  ```bash
  icq sns proposals refresh 1
  icq sns proposal 1 42
  ```

- `0.2.9` splits SNS proposal cache-backed report building into focused load,
  filter, report projection, collection fetch, progress, attempt, and state
  modules, splits proposal cache status report building, and moves proposal
  cache input lookup into storage without changing cache behavior or CLI
  behavior.

- `0.2.8` splits SNS proposal report DTOs and proposal cache refresh
  orchestration/storage/attempt handling into focused modules, aligning
  proposal and neuron cache structure and moving proposal cache discovery onto
  the shared deterministic snapshot scanner without changing report schemas,
  cache behavior, or CLI behavior.

- `0.2.7` splits SNS proposal cache internals into focused model, path,
  storage, attempt, collection, and report modules, and shares SNS cache-file
  error formatting between neuron and proposal caches without changing CLI
  behavior.

- `0.2.6` centralizes missing-cache load/refresh policy across subnet catalog,
  cached NNS component reports, and SNS proposal auto-cache creation without
  changing CLI behavior, and documents the explicit-refresh rule for SNS
  neuron complete snapshots.

- `0.2.5` makes normal `icq sns proposals` list views auto-create and reuse
  complete local proposal snapshots, adds manual proposal cache inspection and
  refresh commands, and splits related SNS source/live proposal modules.

  ```bash
  icq sns proposals 1
  icq sns proposals refresh 1
  icq sns proposals cache list
  icq sns proposals cache status 1
  ```

- `0.2.4` splits SNS source traits and live-source implementations into
  focused list, token, params, proposal, and neuron modules without changing
  CLI behavior.

- `0.2.3` splits SNS clap value-enum, report request, and report sort/filter
  model plumbing into focused modules while preserving existing list, neuron,
  and proposal option behavior.

  ```sh
  icq sns list --sort name
  icq sns neurons 1 --sort stake
  icq sns proposals 1 --topic governance
  ```

- `0.2.2` adds `icq sns proposals --topic <topic>` to filter bounded live SNS
  governance proposal listings by SNS topic, reports the selected filter, and
  tightens request-mapping coverage for unfiltered versus concrete topic
  selectors.

  ```sh
  icq sns proposals 1 --topic any
  icq sns proposals 1 --topic governance
  icq sns proposals 1 --topic treasury-asset-management --format json
  ```

- `0.2.1` centralizes cached NNS leaf cache errors and JSON cache helpers,
  removes duplicated cache-error macro plumbing, and tightens command/test
  module hygiene.

  ```sh
  icq nns node list
  icq nns node-provider refresh
  icq sns neurons 1 --owner 2vxsx-fae --sort api
  ```

- `0.2.0` adds direct SNS proposal ballot table output with compact neuron IDs
  by default and full IDs under `--verbose`.

  ```sh
  icq sns proposal 1 387 --ballots
  icq sns proposal 1 387 --ballots --verbose
  ```

## [0.1.x] - 2026-06-13 - Snapshot cache and SNS query growth

Detailed patch breakdown: [docs/changelog/0.1.md](docs/changelog/0.1.md)

- `0.1.49` simplifies topology read command runners and option tests, and
  splits NNS macro plumbing into focused modules.

- `0.1.48` centralizes CLI help/version argument collection, NNS/SNS clap
  usage-error mapping, cached NNS leaf runtime setup, and SNS neuron cache
  command setup.

- `0.1.47` centralizes compact-vs-verbose text/JSON writing for NNS list
  commands, passthrough subcommand-argument extraction for clap dispatch, and
  NNS node/subnet runtime cache-request setup.

- `0.1.46` centralizes SNS lookup-command runtime fields for info, token,
  params, proposal, proposals, neurons, and neuron refresh commands, plus
  shared CLI, SNS, and NNS command args/help/version handling.

- `0.1.45` removes remaining production wildcard imports and centralizes
  clap parse-to-usage, help/version handling, and NNS project-root usage-error
  handling for command dispatch and option parsers.

- `0.1.44` replaces wildcard module re-exports across SNS, cached NNS report
  roots, subnet catalog reports, and topology fixtures with explicit export
  lists.

- `0.1.43` adds a shared NNS leaf refresh-cache writer and migrates node,
  node-provider, node-operator, data-center, and topology cache-request
  adapters onto shared NNS leaf cache helpers.

- `0.1.42` extracts shared locked, paged snapshot refresh and
  attempt-lifecycle orchestration, centralizes SNS neuron attempt writers, and
  migrates SNS neuron complete-refresh paging onto the generic runners.

- `0.1.41` extracts shared snapshot JSON loading/writing, header validation,
  refresh-attempt, and full-collection path scanning helpers and migrates SNS
  neuron cache reads, writes, and attempts onto them.

- `0.1.40` extracts shared snapshot-cache key, path, envelope, completeness,
  and paged-collection state helpers and migrates SNS neuron complete snapshots
  onto them without changing cache JSON shape.

- `0.1.39` splits SNS report source, lookup, live fetch, text helpers, neuron
  models, neuron cache collection, and live proposal conversion helpers into
  focused modules.

- `0.1.38` splits NNS component text rendering plus topology text, relation,
  refresh, gap, and derived report helpers into focused modules.

- `0.1.37` splits NNS registry, node, and cached component report roots into
  focused build, source, model, text, and refresh modules.

- `0.1.36` splits shared NNS leaf command/option/runtime helpers plus NNS
  component command, runtime, spec, and report-adapter wiring into focused
  modules.

- `0.1.35` splits SNS params text, SNS neuron command plumbing, and NNS
  topology provider/command/option plumbing into focused modules.

- `0.1.34` splits live SNS Candid wire types and proposal text rendering into
  focused modules.

- `0.1.33` splits SNS text/build orchestration and NNS topology build/read
  orchestration into focused modules.

- `0.1.32` splits SNS neuron-cache storage, refresh, report, attempt, and
  collection handling into focused modules.

- `0.1.31` splits shared cache-file JSON, refresh-lock, and write helpers into
  focused modules.

- `0.1.30` splits subnet catalog list text rendering, classification model
  enums, and root catalog helpers into focused modules.

- `0.1.29` splits subnet catalog resolver and report model definitions into
  focused modules.

- `0.1.28` splits IC registry live-source, relation-inventory fetch
  orchestration, and public registry models into focused modules.

- `0.1.27` splits IC registry transport helpers, relation helpers, and
  relation inventory tests into focused modules.

- `0.1.26` splits IC registry protobuf wire types and registry domain
  projection mappers into focused modules.

- `0.1.25` continues NNS cleanup by splitting cached leaf report model roots
  plus topology runtime and request wiring into focused modules.

- `0.1.24` continues module cleanup by splitting subnet catalog tests, models,
  and text rendering plus NNS topology clap command construction into focused
  modules.

- `0.1.23` decomposes NNS topology report models, provider/summary assembly,
  and shared topology fixtures into focused modules.

- `0.1.22` splits cached NNS leaf report roots and subnet catalog host/report
  support while preserving existing cache and report behavior.

- `0.1.21` splits major NNS, SNS, registry, subnet catalog, and neuron-cache
  test and orchestration modules into focused files.

- `0.1.20` splits SNS report assembly, report-root orchestration, and SNS
  report tests by command family and cache behavior.

- `0.1.19` splits live SNS fetch, conversion, and report model wiring for
  deployed SNSes, tokens, governance parameters, proposals, and neurons.

- `0.1.18` splits the custom NNS node command and SNS command runtime, clap,
  and parser modules without changing command behavior.

- `0.1.17` deduplicates standard cached NNS leaf report adapters and reuses a
  shared mainnet-only network guard.

- `0.1.16` centralizes cached NNS leaf cache paths, refresh-lock paths, JSON
  cache error mapping, refresh text rendering, and shared JSON cache reports.

- `0.1.15` hardens cache writes, refresh locks, SNS neuron cache paths,
  command-output helpers, and panic-prone timestamp/subnet internals.

- `0.1.14` splits live SNS, NNS topology/subnet CLI, SNS command specs,
  cache-file support, subnet catalog request/report, and neuron-cache modules.

- `0.1.13` tightens shared NNS leaf helpers, JSON cache providers, topology
  dispatch, and SNS lookup result types.

- `0.1.12` splits IC registry relation inventory fetching and async
  live-source querying out of broader registry modules.

- `0.1.11` decomposes IC registry adapters, wire types, transport helpers,
  relation inventory helpers, domain projections, and annotations.

- `0.1.10` splits NNS component report text/model modules and NNS topology
  helper/rendering modules.

- `0.1.9` splits SNS command, report, cache, and live source modules and
  renames the live SNS source to match its broader role.

- `0.1.8` splits SNS neuron cache model/path helpers and live SNS Candid plus
  conversion helpers.

- `0.1.7` corrects release target boundaries, tightens package/publish
  requirements, and splits NNS topology aggregation modules.

- `0.1.6` hardens SNS neuron snapshot cache coverage, splits cache mechanics,
  deduplicates request construction, and normalizes Rust module layout.

- `0.1.5` splits SNS command dispatch and text rendering, deduplicates live SNS
  Candid queries, and reuses shared command helpers.

- `0.1.4` adds bounded SNS governance proposal listings and direct proposal
  detail lookup commands.

- `0.1.3` adds local SNS neuron snapshot cache inspection commands and fixes
  cache-backed neuron sorts to avoid unnecessary live fetches.

- `0.1.2` deduplicates SNS lookup plumbing, moves duration formatting into a
  helper, splits SNS report internals, and preserves failed refresh metadata.

- `0.1.1` adds SNS nervous system parameter lookup and shared token/e8s amount
  rendering.

- `0.1.0` introduces complete SNS neuron snapshots, cache-backed neuron sorts,
  refresh progress output, and Canic-style release helper commands.

## [0.0.x] - 2026-06-13 - Initial IC query extraction

Detailed patch breakdown: [docs/changelog/0.0.md](docs/changelog/0.0.md)

- `0.0.9` adds bounded SNS neuron listings, 0.1 snapshot-cache design docs,
  and stricter SNS lookup validation.

- `0.0.8` adds SNS token metadata lookup, stable SNS-W numeric ids, logo
  payload hiding, and long-value table fixes.

- `0.0.7` deduplicates cached NNS leaf command plumbing and topology read
  request construction.

- `0.0.6` deduplicates cached NNS leaf dispatch and splits topology read
  parsing plus report request/model code.

- `0.0.5` lets clap render top-level help and uses clap range validation for
  subnet list limits.

- `0.0.4` adds release helpers, SNS metadata fallback visibility, SNS list
  sorting, CLI smoke tests, cache-write hardening, and NNS/SNS query cleanup.

- `0.0.3` adds SNS list/info lookups, groups NNS commands under `src/nns/`,
  renames the local cache directory to `.icq/`, and updates README positioning.

- `0.0.1` creates the `icq` executable, extracts the former Canic NNS query
  surface, and adds read-only text/JSON metadata reports.
