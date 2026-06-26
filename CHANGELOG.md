# Changelog

All notable changes to `ic-query` will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/), and this
crate follows [Semantic Versioning](https://semver.org/).

## Unreleased

## [0.5.x] - 2026-06-25 - Library boundary cleanup

Detailed patch breakdown: [docs/changelog/0.5.md](docs/changelog/0.5.md)

- `0.5.16` improves the generic ICRC native library path. Downstream crates
  now get public constructors for every generic ICRC request DTO, a public
  default ICRC source endpoint constant, and host-without-CLI smoke coverage
  for the live ICRC report builder exports.

- `0.5.15` opens the SNS native host API without requiring `cli`.
  Downstream crates can now call SNS list/info/token/params, proposal
  list/detail, and neuron report builders directly, inspect SNS proposal and
  neuron caches, resolve their cache/refresh sidecar paths, refresh complete
  SNS proposal and neuron snapshots, and render SNS neuron/proposal cache and
  refresh reports from the library.

- `0.5.14` opens the NNS proposal native host API without requiring `cli`.
  Downstream crates can now inspect complete NNS proposal caches, build
  list/detail reports from a complete cache, call the live NNS proposal report
  builders explicitly, refresh the complete proposal cache, resolve proposal
  cache paths, and render cache/refresh reports directly from the library.

- `0.5.13` improves the native NNS inventory library path for downstream
  crates. Node, data-center, node-provider, and node-operator cache request
  types now have public constructors, and their host APIs expose cache and
  refresh-lock path helpers plus refresh-lock defaults without requiring
  `cli`. The public API smoke tests now seed cached inventory reports and
  exercise the same cache-backed list/info builders that the CLI uses.

- `0.5.12` opens subnet catalog list/info/refresh request, report, row,
  builder, refresh, and text-rendering APIs under `features = ["host"]`
  without requiring `cli`. Native downstream crates can now build and render
  the same cache-backed subnet catalog reports used by `icq nns subnet`
  without spawning the `icq` executable.

- `0.5.11` opens NNS topology request/report/row DTOs and text renderers
  under `--no-default-features`, and exposes topology cache-backed builders
  and refresh execution to `host` users without requiring `cli`. Command
  parsing and dispatch remain behind `cli`. Downstream crates can now build,
  construct, deserialize, and render topology summary, coverage, versions,
  health, gaps, capacity, regions, providers, and refresh reports without
  spawning `icq`.

- `0.5.10` opens NNS data-center, node-provider, and node-operator list/info
  request, report, row, and text-rendering DTOs under `--no-default-features`.
  Downstream crates can now construct and render the cached NNS inventory leaf
  reports without enabling native live-call, cache, refresh, or CLI
  dependencies. This slice also exposes the subnet-catalog host cache request,
  cached load, load-or-refresh, cache path helpers, and default mainnet
  endpoint under `features = ["host"]` so native tools can replace
  `icq nns subnet info` shell-outs with direct library calls.

- `0.5.9` opens NNS node list/info request, report, row, filter, constant,
  and text-rendering DTOs under `--no-default-features`. Downstream crates can
  now construct and render NNS node reports without enabling native live-call,
  cache, or CLI dependencies.

- `0.5.8` opens NNS proposal list/detail request, report, row, filter, sort,
  tally, ballot, and text-rendering DTOs under `--no-default-features`.
  Downstream crates can now construct and render NNS proposal reports without
  enabling native live-call, cache, or CLI dependencies.

- `0.5.7` tightens the generic ICRC no-default library boundary. The public
  API smoke test now constructs and renders token, balance, allowance, index,
  transaction, block-type, archive, tip-certificate, and capability reports
  without enabling native live-call or CLI dependencies.

- `0.5.6` opens SNS governance-parameter report DTOs and text rendering under
  `--no-default-features`, along with SNS proposal list/detail request,
  report, row, filter, sort, and text-rendering DTOs. Downstream crates can
  now construct and render SNS params and proposal reports without enabling
  native live-call, cache, or CLI dependencies. The make-driven Cargo checks
  also disable HTTP multiplexing, use a higher network retry count, and retry
  package verification to make CI less sensitive to transient crates.io HTTP/2
  failures.

- `0.5.5` opens more pure SNS library surface under `--no-default-features`.
  Downstream crates can now construct and render SNS info and token report
  DTOs without enabling native live-call, cache, or CLI dependencies. The SNS
  no-default public API smoke test now covers list, info, and token reports.

- `0.5.4` tightens the feature-boundary guard after the no-default library
  surfaces were opened. The CI script now separately asserts that pure
  no-default builds avoid CLI/live-call dependencies and that `host` without
  `cli` still avoids `clap`; the README and crate docs also clarify that
  no-default is a host/CLI dependency boundary, not a `no_std` promise.

- `0.5.3` opens the pure subnet-catalog model, JSON, and resolver API under
  `--no-default-features` so downstream crates can validate catalog snapshots
  and resolve subnet/canister principals without host cache refresh or CLI
  dependencies. The feature-boundary check now runs the ICRC, NNS, SNS, and
  subnet-catalog public API smoke tests under no-default builds.

- `0.5.2` opens the first pure no-default NNS and SNS library surfaces after
  the crate split. `ic_query::nns::registry` now exposes registry-version
  request/report/text DTOs without `host`, and `ic_query::sns` now exposes SNS
  list request/report/text DTOs without `host`; live builders, cache IO, and
  CLI dispatch remain behind host/CLI features. This slice also makes
  `ic-query --features host --no-default-features` compile without the CLI
  feature, and the feature-boundary CI check now covers that host-only build
  plus the no-default public API smoke tests.

- `0.5.1` corrects release documentation after the library-boundary cleanup
  was published as `0.5.0`. The detailed changelog now has a `0.5.x` ledger,
  the `0.5.0` notes live in the matching release line, and the README library
  dependency example points at the `0.5` release series. This slice also adds
  a CI guard for the library feature boundary so default/no-default
  `ic-query` builds stay free of CLI parsing and native live-call dependencies.

- `0.5.0` tightens the library and CLI package boundary after the initial
  split. The top-level process runner now lives in `ic-query-cli`, while
  `ic-query` keeps only family-level CLI adapters behind the `cli` feature for
  the wrapper. This slice also changes the `ic-query` default feature set to
  empty so plain library dependencies do not pull CLI parsing or native
  live-call adapters.

## [0.4.x] - 2026-06-25 - Library and CLI package split

Detailed patch breakdown: [docs/changelog/0.4.md](docs/changelog/0.4.md)

- `0.4.0` splits the project into a virtual workspace with
  `crates/ic-query` as the reusable library package and `crates/ic-query-cli`
  as the package that installs the existing `icq` binary. This slice adds
  public report facades for generic ICRC reports, NNS registry version reports,
  and deployed SNS list reports so downstream crates can start using typed
  request/report APIs without invoking CLI argument parsing. The library also
  gates CLI and live host-call dependencies behind features so
  `ic-query --no-default-features` compiles for native and
  `wasm32-unknown-unknown` targets without pulling `clap`, `ic-agent`, Tokio,
  or `futures`.

  ```bash
  cargo install ic-query-cli
  ```

## [0.3.x] - 2026-06-24 - Generic ICRC ledger queries

Detailed patch breakdown: [docs/changelog/0.3.md](docs/changelog/0.3.md)

- `0.3.9` adds `--follow-archives` to `icq icrc transactions`, allowing the
  bounded live transaction query to follow returned ICRC-3 archive callbacks.
  Followed archive blocks and archive follow errors are reported separately so
  ledger-returned blocks, callback ranges, and archive fetch results remain
  script-friendly. This slice also cleans up the generic ICRC implementation by
  deduplicating common command option wiring, ICRC-3 block/range row
  conversion, and text table rendering helpers without changing CLI behavior,
  report schemas, or output semantics.

  ```bash
  icq icrc transactions mxzaz-hqaaa-aaaar-qaada-cai --start 0 --limit 1 --follow-archives
  icq icrc transactions mxzaz-hqaaa-aaaar-qaada-cai --start 0 --limit 1 --follow-archives --format json
  ```

- `0.3.8` adds `icq icrc capabilities <ledger-canister-id>` for live generic
  ICRC endpoint probing. The report keeps each probed method independent so
  unsupported optional endpoints appear as `unsupported` rows instead of
  failing the whole command.

  ```bash
  icq icrc capabilities mxzaz-hqaaa-aaaar-qaada-cai
  icq icrc capabilities mxzaz-hqaaa-aaaar-qaada-cai --format json
  ```

- `0.3.7` adds live generic ICRC-3 tip certificate inspection through
  `icrc3_get_tip_certificate`. Text output shows certificate and hash tree byte
  counts plus truncated hex previews when present; JSON keeps
  `certificate_present` plus optional full certificate and hash tree hex
  strings and byte counts.

  ```bash
  icq icrc tip-certificate mxzaz-hqaaa-aaaar-qaada-cai
  icq icrc tip-certificate mxzaz-hqaaa-aaaar-qaada-cai --format json
  ```

- `0.3.6` adds live generic ICRC-3 ledger discovery for supported block types
  and archive ranges. Both reports are live-only, include the queried source
  endpoint, and keep archive range bounds as string fields in JSON.

  ```bash
  icq icrc block-types ryjl3-tyaaa-aaaaa-aaaba-cai
  icq icrc archives ryjl3-tyaaa-aaaaa-aaaba-cai --from qaa6y-5yaaa-aaaaa-aaafa-cai --format json
  ```

- `0.3.5` adds live generic ICRC transaction history pages through the
  ledger's `icrc3_get_blocks` endpoint. Text output shows compact block
  summaries and archive callback ranges; JSON keeps raw ICRC-3 block values
  and block/log indexes as string fields.

  ```bash
  icq icrc transactions ryjl3-tyaaa-aaaaa-aaaba-cai
  icq icrc transactions ryjl3-tyaaa-aaaaa-aaaba-cai --start 100 --limit 50 --format json
  ```

- `0.3.4` cleans up the generic ICRC index slice by sharing ICRC/SNS token
  metadata text formatting, centralizing generic ICRC live query setup, using
  the shared ICRC-106 index-error formatter from SNS token reports, and
  replacing the remaining plain test `unwrap` without changing CLI behavior,
  report schemas, cache paths, or output.

- `0.3.3` adds live ICRC-106 index discovery by ledger canister id, returning
  the configured index canister when available or the ledger-reported index
  discovery error as text/JSON fields.

  ```bash
  icq icrc index ryjl3-tyaaa-aaaaa-aaaba-cai
  icq icrc index ryjl3-tyaaa-aaaaa-aaaba-cai --format json
  ```

- `0.3.2` adds live ICRC-2 allowance queries by ledger canister id, owner
  principal, and spender principal, including optional owner and spender
  subaccounts plus text/JSON reports that keep raw allowance base units and
  expiration nanoseconds script-friendly.

  ```bash
  icq icrc allowance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa aaaaa-aa
  icq icrc allowance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa aaaaa-aa --owner-subaccount 0000000000000000000000000000000000000000000000000000000000000000 --spender-subaccount 0000000000000000000000000000000000000000000000000000000000000000
  ```

- `0.3.1` consolidates duplicated generic ICRC and SNS ledger-token live
  plumbing behind a shared ICRC ledger helper for wire types, token metadata
  calls, metadata conversion, and ledger query error mapping without changing
  CLI behavior, report schemas, cache paths, or output.

- `0.3.0` adds live generic ICRC ledger token metadata and account balance
  queries by ledger canister id, including text/JSON reports that show the
  queried source endpoint and preserve raw base-unit token amounts in JSON.

  ```bash
  icq icrc token ryjl3-tyaaa-aaaaa-aaaba-cai
  icq icrc balance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa --subaccount 0000000000000000000000000000000000000000000000000000000000000000
  ```

## [0.2.x] - 2026-06-16 - SNS proposal detail expansion

Detailed patch breakdown: [docs/changelog/0.2.md](docs/changelog/0.2.md)

- `0.2.42` adds cache-compatible NNS and SNS proposal text search with
  `--query <text>`, reports the selected `query_filter` in proposal list
  text/JSON output, and makes NNS proposal list reports expose `result_scope`
  so bounded live views are distinguishable from complete-cache views.

  ```bash
  icq nns proposal list --query subnet
  icq sns proposals 1 --query treasury
  ```

- `0.2.41` adds proposal list filters for SNS reward eligibility and
  NNS/SNS proposers while keeping them cache-compatible where complete
  snapshots are available.

  ```bash
  icq nns proposal list --proposer 123456789
  icq sns proposals 1 --eligible yes
  icq sns proposals 1 --proposer 00010203
  ```

- `0.2.40` adds NNS and SNS proposal sorting by latest tally timestamp, plus
  SNS proposal sorting by action id, reward eligibility, and reward-event end
  timestamp, while reusing the existing live and complete-cache proposal view
  sorters.

  ```bash
  icq nns proposal list --sort tally-time
  icq sns proposals 1 --sort action-id
  icq sns proposals 1 --sort tally-time
  icq sns proposals 1 --sort eligible
  icq sns proposals 1 --sort reward-end
  ```

- `0.2.39` adds NNS proposal sorting by reward status, voting deadline, and
  total potential voting power while reusing the existing live and
  complete-cache proposal view sorter.

  ```bash
  icq nns proposal list --sort reward-status
  icq nns proposal list --sort deadline
  icq nns proposal list --sort voting-power
  ```

- `0.2.38` adds cache-backed SNS proposal topic sorting with
  `icq sns proposals <id|root-principal> --sort topic`, defaulting to
  ascending topic-label order and applying the sort before `--limit`.

  ```bash
  icq sns proposals 1 --sort topic
  ```

- `0.2.37` makes SNS proposal `--topic <topic>` filters cache-compatible by
  preserving proposal topic labels in complete snapshots, refreshing legacy
  proposal snapshots that lack topic labels before topic-filtered views, and
  allowing decided-status filters to combine with topic filters through the
  complete cache path.

  ```bash
  icq sns proposals 1 --status decided --topic governance
  ```

- `0.2.36` makes SNS proposal `--status adopted|rejected` filters
  cache-compatible for topic-free list views by preserving raw SNS governance
  status codes in proposal rows, refreshing legacy proposal snapshots that
  lack those codes before applying final-status filters, and bumping SNS
  proposal JSON report schemas for the new optional raw `status` field.

- `0.2.35` deduplicates SNS cache inspection plumbing by sharing invalid-cache
  summary fields, valid cache-id lookup, and cache-error text rendering across
  SNS neuron and proposal cache reports without changing CLI behavior, cache
  paths, report schemas, or output semantics.

- `0.2.34` hardens complete snapshot cache inspection so NNS proposal, SNS
  proposal, and SNS neuron cache list/status reports surface malformed,
  unsupported, or identity-mismatched local cache files as invalid local rows
  while keeping normal cache-backed reads strict and live-free cache status
  behavior unchanged.

- `0.2.33` makes crate packaging more intentional by excluding internal
  workflow, agent, governance, toolchain, and dev-only script files from the
  published tarball, and adds a CI guard for package contents without changing
  CLI behavior.

- `0.2.32` validates complete snapshot cache identity fields when present,
  rejecting caches whose recorded domain, entity, collection, or scope does not
  match the logical cache key while preserving compatibility with older caches
  that do not yet contain identity fields, adds NNS/SNS family-level mismatch
  coverage, adds a CI guard against tag-pinned GitHub Actions, standardizes
  README command fences, and removes stale clippy allow-list entries.

- `0.2.31` fixes release documentation drift, clarifies NNS proposal cache
  status wording, documents corrupted refresh-lock handling, and adds a
  changelog/version consistency check to the local and CI gate, pins CI actions
  to exact revisions, documents clap required-value invariants, and adds
  logical identity fields to newly written snapshot caches without changing CLI
  behavior.

- `0.2.30` continues internal cleanup by splitting NNS proposal report assembly,
  labels, and source modules into focused owners, moving shared cache policy
  tests under the cache-file module, tightening clap/parser coverage, and
  preserving CLI behavior, cache paths, report schemas, and output.

- `0.2.29` centralizes shared cache-file, clock, and runtime error formatting
  through typed `thiserror` errors, splits NNS proposal snapshot refresh into
  orchestration, page collection, and publish modules, deduplicates NNS
  proposal cache option parsing, and normalizes shared cache/snapshot helper
  docs without changing CLI behavior, cache paths, report schemas, or output.

- `0.2.28` adds explicit complete NNS proposal snapshot refresh and cache
  inspection commands under `icq nns proposal refresh` and
  `icq nns proposal cache list|status`, reusing the shared snapshot cache
  lock, progress, attempt sidecar, and complete-only publish flow while
  keeping normal `icq nns proposal list` as a bounded live query.

  ```bash
  icq nns proposal refresh
  icq nns proposal refresh --max-pages 5
  icq nns proposal cache status
  ```

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

  ```bash
  icq sns list --sort name
  icq sns neurons 1 --sort stake
  icq sns proposals 1 --topic governance
  ```

- `0.2.2` adds `icq sns proposals --topic <topic>` to filter bounded live SNS
  governance proposal listings by SNS topic, reports the selected filter, and
  tightens request-mapping coverage for unfiltered versus concrete topic
  selectors.

  ```bash
  icq sns proposals 1 --topic any
  icq sns proposals 1 --topic governance
  icq sns proposals 1 --topic treasury-asset-management --format json
  ```

- `0.2.1` centralizes cached NNS leaf cache errors and JSON cache helpers,
  removes duplicated cache-error macro plumbing, and tightens command/test
  module hygiene.

  ```bash
  icq nns node list
  icq nns node-provider refresh
  icq sns neurons 1 --owner 2vxsx-fae --sort api
  ```

- `0.2.0` adds direct SNS proposal ballot table output with compact neuron IDs
  by default and full IDs under `--verbose`.

  ```bash
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
