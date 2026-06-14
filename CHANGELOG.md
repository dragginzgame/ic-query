# Changelog

All notable changes to `ic-query` will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/), and this
crate follows [Semantic Versioning](https://semver.org/).

## [0.1.11] - 2026-06-14

### Changed

- Splits IC registry adapter models, errors, Candid wire types, transport
  helpers, relation inventory helpers, catalog and domain projection helpers,
  and mainnet annotations into focused submodules.
- Replaces the registry client wildcard parent import with explicit internal
  imports.

## [0.1.10] - 2026-06-14

### Changed

- Splits NNS node, node-provider, node-operator, and data-center text
  rendering into focused report text submodules.
- Splits NNS node, node-provider, node-operator, and data-center request,
  report, cache, and error models into focused report model submodules.
- Splits NNS topology summary, coverage, versions, and host-error helpers out
  of the topology report orchestration module.
- Splits NNS topology text rendering into focused per-report renderer
  submodules.

## [0.1.9] - 2026-06-14

### Changed

- Splits SNS command clap specification, usage rendering, value parsers,
  command value enums, and parsed option types into focused commands
  submodules.
- Splits SNS report lookup/list ordering helpers and report-object assembly
  into focused report submodules.
- Splits SNS report model types into focused request, report, sort/filter, and
  error submodules.
- Splits SNS neuron cache loading, discovery, summaries, and cached sort
  ordering into a focused cache storage submodule.
- Splits generic live SNS agent/query helpers out of the live SNS fetch
  orchestration module.
- Renames the internal live SNS source to match its broader list, token,
  governance, proposal, and neuron query role.

## [0.1.8] - 2026-06-14

### Changed

- Splits SNS neuron cache path, attempt, error, and cache model helpers into
  focused submodules while preserving the existing cache behavior.
- Splits live SNS Candid request and response wire types into a focused live
  source submodule.
- Splits live SNS response conversion and metadata/proposal formatting helpers
  into a focused live source submodule.

## [0.1.7] - 2026-06-14

### Changed

- Corrects release Make targets so `make patch`, `make minor`, and `make major`
  only gate and bump version files, while `make release-patch`,
  `make release-minor`, and `make release-major` stage, commit, tag, and push.
- Requires a clean worktree for `make package` and `make publish`, and documents
  publishing after the release commit/tag push.
- Splits NNS topology health, gap, capacity, region, provider, and refresh
  aggregation into focused report submodules.

## [0.1.6] - 2026-06-14

### Changed

- Hardens SNS neuron snapshot cache coverage for unsupported schemas, stale
  refresh locks, failed refresh preservation, and complete-cache sorting.
- Splits SNS neuron cache storage, refresh, status, and cached-sort mechanics
  into a dedicated report submodule.
- Deduplicates SNS command request construction for timestamps and project
  cache-root lookup.
- Normalizes Rust module layout to avoid `foo.rs` plus `foo/` collisions,
  removes `#[path = "..."]` module shims, and records module/test layout rules
  in `AGENTS.md`.

## [0.1.5] - 2026-06-14

### Changed

- Splits SNS command dispatch into its own module while keeping the public SNS
  module focused on exports.
- Splits SNS text rendering into focused helper, neuron/cache, params, and
  proposal modules.
- Deduplicates typed live SNS Candid query handling across SNS-W and governance
  calls.
- Reuses shared help/version and SNS cache-path helpers in older NNS and SNS
  call sites.
- Makes `make patch`, `make minor`, and `make major` push the release commit
  and tag automatically after the release bump succeeds.

## [0.1.4] - 2026-06-14

### Added

- Adds `icq sns proposals <id|root-principal>` for bounded live SNS governance
  proposal listings, with `--limit`, `--before`, `--status`, `--verbose`, and
  JSON output support.
- Adds `icq sns proposal <id|root-principal> <proposal-id>` for direct SNS
  governance proposal detail lookup.

## [0.1.3] - 2026-06-14

### Added

- Adds `icq sns neurons cache list` and `icq sns neurons cache status
  <id|root-principal>` to inspect local complete neuron snapshots and latest
  refresh-attempt metadata without live SNS-W or governance calls.

### Fixed

- Uses local complete neuron cache metadata for cache-backed `icq sns neurons
  <id|root-principal> --sort ...` reports instead of re-fetching the live SNS
  list before reading the cache.

## [0.1.2] - 2026-06-13

### Changed

- Deduplicates SNS lookup command dispatch, clap command construction, and
  shared lookup option parsing.
- Moves shared duration display formatting into the duration helper module.
- Splits SNS model types, source contracts, text rendering, and live IC API
  querying out of the main SNS report orchestration module.

### Fixed

- Preserves SNS neuron refresh attempt progress metadata when a refresh stops
  before publishing a complete snapshot.

## [0.1.1] - 2026-06-13

### Added

- Adds `icq sns params <id|root-principal>` to query SNS governance nervous
  system parameters, with readable text output and raw Candid fields in JSON.

### Changed

- Moves shared token/e8s text amount rendering into a reusable helper for SNS
  token, neuron, and parameter reports.

## [0.1.0] - 2026-06-13

### Added

- Adds `icq sns neurons refresh <id|root-principal>` to materialize complete
  SNS governance neuron snapshots under `.icq/sns/ic/<root>/neurons/full.json`.
- Adds cache-backed `icq sns neurons <id|root-principal> --sort
  id|stake|maturity|created` over complete neuron snapshots.
- Adds a shared stderr progress-line helper and uses it to show SNS neuron
  refresh page/row counters while long refreshes are running.
- Adds Canic-style `make release-patch`, `make release-minor`,
  `make release-major`, and `make release-push` helpers.

### Changed

- Keeps default `icq sns neurons` output on the bounded live API path with
  `--sort api`, while whole-collection sorts require the complete cache.
- Allows larger `--limit` values for cache-backed SNS neuron sorts while
  keeping live API queries capped at 100 rows.
- Records SNS neuron refresh attempts separately from published complete
  snapshots so failed or capped refreshes do not replace the last complete
  cache.
- Shortens SNS neuron IDs to eight characters in text tables by default, with
  `--verbose` preserving full neuron IDs.
- Renders current SNS token amounts, including token fee, total supply, stake,
  maturity, and staked maturity, as two-decimal token amounts in text output
  while keeping raw base-unit and e8s fields in JSON.

## [0.0.9] - 2026-06-13

### Added

- Adds `icq sns neurons <id|root-principal>` for SNS governance neuron listings
  with clap-validated `--limit` and `--owner` filters.
- Adds `docs/design/0.1/` planning docs for reusable complete snapshot caches
  and SNS neuron refresh semantics.

### Changed

- Deduplicates SNS lookup command option parsing and shared SNS resolution,
  while clap-validating lookup ids and root principals.
- Lets global `--network ic` forward to all networked SNS lookup commands.

## [0.0.8] - 2026-06-13

### Added

- Adds `icq sns token <id|root-principal>` for SNS ledger token metadata,
  supply, fee, supported standards, minting account, and index canister lookup.

### Changed

- Uses the SNS-W `list_deployed_snses` response order for SNS numeric ids,
  preserving deployment order through concurrent metadata enrichment.
- Reports SNS `icrc1:logo` token metadata as a boolean presence value instead
  of emitting the raw logo payload.
- Removes the drifting crates.io version badge from the README.

### Fixed

- Fixes table rendering for long SNS token metadata values without relying on
  dynamic format widths.

## [0.0.7] - 2026-06-13

### Changed

- Further deduplicates cache-backed NNS leaf command plumbing through shared
  request and test-helper macros.
- Collapses repeated NNS topology read request structs into a shared read
  request type while preserving the existing command-specific request names.
- Deduplicates NNS topology read-runner and component request construction
  helpers.

## [0.0.6] - 2026-06-13

### Changed

- Deduplicates cached NNS leaf command dispatch for data-center, node-provider,
  and node-operator commands.
- Deduplicates NNS topology read command parsing and splits topology report
  request/model code into focused submodules.

## [0.0.5] - 2026-06-13

### Changed

- Lets clap render top-level command help from registered command metadata
  instead of maintaining a separate hand-formatted command list.
- Uses clap's ranged numeric parser for
  `icq nns subnet list --range-limit` and removes unused CLI parsing helpers.

## [0.0.4] - 2026-06-13

### Added

- Adds `make patch`, `make minor`, and `make major` release helpers that run
  `make test` before any version or tag mutations, bump the crate version,
  commit it, and create an annotated version tag.
- Adds SNS metadata fallback visibility in text and JSON reports when a
  governance metadata query fails but the deployed SNS row can still be shown.
- Adds `icq sns list --sort id|name` while keeping numeric SNS ids stable by
  root principal.
- Adds exact snapshot-style tests for top-level and SNS list CLI help output.
- Adds binary smoke tests for top-level help, SNS list help, NNS topology help,
  and version output.

### Changed

- Moves the `icq sns list` row id to the first text table column.
- Assigns SNS list row ids after deterministic root-principal ordering so ids
  are not reshuffled by metadata name changes or metadata lookup failures.
- Hardens release bumps by validating the bumped package before commit/tag and
  restoring `Cargo.toml` and `Cargo.lock` if a post-mutation release step fails.
- Uses more collision-resistant temporary filenames and cleanup for atomic cache
  writes.
- Centralizes CLI output-format handling, top-level command metadata, and the
  current-thread async runtime wrapper used by live IC query adapters.
- Deduplicates topology component request construction and mainnet agent/
  canister setup in live registry and SNS queries.
- Splits live registry fetch orchestration into `ic_registry::client` and moves
  NNS topology text rendering into its own report submodule.
- Splits subnet catalog host/cache refresh code, text rendering, and timestamp
  helpers into focused submodules.
- Deduplicates repeated NNS topology read-option parsing.
- Reuses a shared refresh-lock guard helper for pre-fetch subnet catalog
  refreshes.

## [0.0.3] - 2026-06-13

### Added

- Adds `icq sns list` and `icq sns info` for deployed mainnet SNS instances
  queried from the SNS-W canister and SNS governance metadata.
- Resolves SNS names from governance metadata, shows compact five-character
  canister ID prefixes by default, and supports full canister IDs with
  `icq sns list --verbose`.
- Supports `icq sns info <id|root-principal>` lookups by list row id or SNS
  root canister principal.

### Changed

- Groups NNS command implementation modules under `src/nns/` by command family.
- Renames project-local `icq` cache/state directory from `.ic-query/` to
  `.icq/`.
- Updates README positioning so `ic-query` is documented as a standalone IC
  metadata lookup tool, with Canic mentioned only as an optional integration
  example.
- Clarifies cache behavior for subnet versus node/provider/operator/data-center
  commands, including first-use cache population and source endpoint notices.

## [0.0.1] - 2026-06-13

### Added

- Adds the `icq` executable.
- Adds the `icq nns` command family for read-only Internet Computer mainnet
  metadata queries.
- Adds cache-backed subnet list, subnet info, and subnet refresh commands.
- Adds cache-backed NNS registry version, data-center, node, node-provider,
  node-operator, and joined-topology commands.
- Adds text and JSON output for automation-friendly metadata reports.

### Changed

- Moves the former `canic nns ...` command surface into `icq nns ...`.
- Folds the former subnet catalog model and resolver into `ic-query`, so the
  metadata query tool owns subnet classification and lookup.
