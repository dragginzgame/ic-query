# Changelog

All notable changes to `ic-query` will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/), and this
crate follows [Semantic Versioning](https://semver.org/).

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
