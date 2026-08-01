# Changelog

All notable changes to `ic-query` will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/), and this
crate follows [Semantic Versioning](https://semver.org/).

## Unreleased

## [0.22.x] - 2026-08-01 - Structural consolidation

Detailed release notes: [docs/changelog/0.22.md](docs/changelog/0.22.md)

- `0.22.3` separates ICRC request contracts and constructors from serialized
  report/row contracts behind one explicit internal facade. Existing public
  `ic_query::icrc::*` paths, feature availability, type fields and derives,
  constructors, report JSON, cache behavior, and network calls are unchanged.

- `0.22.2` separates ICRC account-history live orchestration, arbitrary-size
  cursor validation, complete collection state, generic ICRC-index decoding,
  and deployed ICP-index decoding into cohesive internal owners. The generic
  and ICP wire contracts remain deliberately distinct while sharing only their
  request envelope and protocol-neutral projection helpers. Public Rust paths,
  report JSON, cache schemas, pagination and index-discovery semantics, and
  network calls are unchanged.

- `0.22.1` replaces staged CLI parsing with one composed Clap command tree and
  typed family/leaf dispatch. Clap now owns nested help, propagated version,
  required-subcommand diagnostics, and top-level network validation; reduced
  passthrough grammars, raw argument scans, and the hidden network-forwarding
  option are removed. Native nested `icq help <path>` is supported, while the
  old leaf-trailing positional shortcut (for example, `icq sns list help`) is
  removed as a pre-1.0 hard cut; use `-h`, `--help`, or
  `icq help <path>`. Command names, report options and output, network
  authority, cache behavior, and live calls are unchanged.

  ```bash
  icq help nns topology
  ```

- `0.22.0` establishes the ordered structural-consolidation design for one
  composed Clap grammar, shared NNS Registry inventory orchestration, shared
  snapshot lifecycle mechanics, and cohesive module boundaries. The first
  slice replaces the NNS neuron wildcard export with an explicit current API
  list, table-drives repeated topology non-mainnet tests, makes published README
  documentation links resolve through GitHub, and adds a focused
  `subnet-catalog-host` feature without Dashboard Reqwest or CBOR dependencies.
  The full `host` feature remains a superset. Registry node, provider, operator,
  and data-center reports now share one cache-missing refresh driver and one
  exact-or-unique-prefix resolver, plus common network/source-request/fetch/write
  orchestration. NNS Governance proposal and neuron snapshots also share one
  attempt-sidecar construction, validation, status, and failed-progress owner.
  SNS proposal and neuron snapshots now share target lookup, lock acquisition,
  complete-cache provenance, atomic publication, and attempt finalization while
  retaining their distinct cursors and row validation. Public Rust paths, CLI
  behavior, report JSON, cache schemas, refresh policy, and network calls are
  unchanged.

## [0.21.x] - 2026-08-01 - Certified Cycle Minting Canister reporting

Detailed release notes: [docs/changelog/0.21.md](docs/changelog/0.21.md)

- `0.21.1` hard-cuts every report-producing CLI command from
  `--format <text|json>` to text-by-default output with `--json` selecting raw,
  script-friendly JSON. The shared flag and output-selection path now covers
  all IC Dashboard, ICRC, NNS, SNS, and system-canister report leaves and
  aligns the spelling with Canic. The removed `--format` form has no alias or
  compatibility parser. Clap now also validates the finite mainnet-only
  `--network ic` contract before staged command dispatch, so help-like option
  values cannot bypass an invalid network or an unsupported family option.
  Report JSON, text rendering, valid mainnet requests, cache identity and
  contents, and the public Rust API are unchanged.

  ```bash
  icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai --json
  icq nns topology summary --json
  icq system cycles --json
  ```

- `0.21.0` adds bounded live `system xdr` and `system cycles` reports backed
  by one native Cycle Minting Canister
  `get_icp_xdr_conversion_rate` query. The host adapter authenticates the
  application-level certificate for the mainnet CMC, verifies the
  certified-data hash-tree commitment and native rate leaf, and preserves the
  raw permyriad rate, market timestamp, endpoint, collection provenance, and
  certificate evidence. The cycles view derives cycles per ICP exactly from
  the certified rate and the IC protocol constant of one trillion cycles per
  XDR. These live-only reports reject non-mainnet networks before agent
  construction, do not scrape uncertified CMC metrics, and never enumerate or
  create a cache. Shared canister certified-data authentication is reused by
  CMC and ICRC tip verification without changing the ICRC report contract.

  ```bash
  icq system xdr
  icq system cycles --format json
  ```

## [0.20.x] - 2026-07-31 - Bounded Dashboard network metrics

Detailed release notes: [docs/changelog/0.20.md](docs/changelog/0.20.md)

- `0.20.2` adds a one-request, explicitly bounded Dashboard v3 daily network
  activity report. It defaults to seven days, caps requests at one year and
  responses at 366 rows, and preserves raw daily average/maximum query,
  update, total transaction, and block-rate strings. Rows are canonically
  ordered and retain exact query, endpoint, retrieval, and non-certified
  provenance. The report is live-only and never paginates, fans out, fills
  missing days, or creates a cache. This is a pre-1.0 Rust-API hard cut for
  custom `IcNetworkSource` implementations, which must implement the new
  daily-statistics capability.

  ```bash
  icq ic network daily-stats
  icq ic network daily-stats \
    --start 1784937600 --end 1785542400 --format json
  ```

- `0.20.1` adds the finite official Dashboard v4 boundary-node data-center
  report. It makes exactly one request, preserves raw data-center ids, names,
  owners, regions, coordinates, and node-count strings, retains zero-node
  locations, and derives canonical row ordering plus checked data-center and
  node totals. The public library adds typed request/report/source DTOs,
  `IcNetworkSource` on the existing `LiveIcSource`, custom-source validation,
  and text rendering. The report is explicitly off-chain, non-certified,
  live-only, and never paginates, follows up, or creates a cache.

  ```bash
  icq ic network boundary-node-data-centers
  icq ic network boundary-node-data-centers --format json
  ```

- `0.20.0` adds one-request bounded time-series reports for nine official
  Dashboard network metrics: instruction and message execution rates, cycle
  burn, block rate, node and Subnet counts, registered canister counts, total
  energy-consumption rate, and boundary-node count. Queries default to the
  preceding hour at a five-minute step and are capped at 1,000 observations
  per returned series. Reports preserve raw value strings, explicit query
  bounds, endpoint/retrieval provenance, and the Dashboard's non-certified,
  no-point-in-time guarantees. The public library adds typed metric
  request/report/source DTOs, `IcMetricSource` on `LiveIcSource`, custom-source
  validation, and text rendering. Metrics are live-only and never fan out,
  paginate, or create a cache. The shared invalid-source error prefix is
  generalized so metric failures are not mislabeled as canister failures.

  ```bash
  icq ic metrics instruction-rate
  icq ic metrics cycle-burn-rate --start 1700000000 --end 1700003600 --step 300
  icq ic metrics ic-node-count --format json
  ```

## [0.19.x] - 2026-07-31 - Bounded Dashboard canister discovery

Detailed release notes: [docs/changelog/0.19.md](docs/changelog/0.19.md)

- `0.19.1` consolidates repeated Dashboard source and report provenance into
  the existing `IcSourceRequest` and one flattened
  `IcDashboardReportProvenance`, and shares collection option parsing,
  transport execution, and text provenance rendering. Report JSON, CLI
  grammar and output, validation, live request counts, and cache behavior are
  unchanged. This is a breaking pre-1.0 Rust-API hard cut: Dashboard report
  provenance is now accessed through `.provenance`, and source-data DTOs echo
  the source request through `.source`; the replaced fields are removed
  without aliases.

- `0.19.0` adds filtered canister counts and one-page canister discovery
  through the official Dashboard v4 API. Count fetches no rows; page is fixed
  to canister-id order, defaults to 50 rows, is capped at 100, and never follows
  a cursor or writes a cache automatically. Typed library requests, reports,
  capability traits, custom-source validation, and renderers preserve raw
  Dashboard values and explicit off-chain provenance.

  ```bash
  icq ic canister count --has-name true
  icq ic canister page --query ledger --limit 25 --format json
  icq ic canister page --after ryjl3-tyaaa-aaaaa-aaaba-cai --limit 25
  ```

## [0.18.x] - 2026-07-30 - Official Dashboard canister reporting

Detailed release notes: [docs/changelog/0.18.md](docs/changelog/0.18.md)

- `0.18.1` consolidates live HTTP(S) endpoint parsing across `ic-agent` and
  official Dashboard sources, while retaining Dashboard-specific base-URL
  validation and typed errors. Top-level CLI error handling now owns usage
  exit codes and broken-pipe detection consistently for every command family.
  The README and supporting guides are reorganized around authority,
  collection mode, cache behavior, the current command hierarchy, and the
  distinction between CLI topology diagnostics and the exact-version joined
  topology library API. Command behavior, report JSON, cache schemas, and the
  public Rust API are unchanged.

- `0.18.0` adds the official Dashboard authority family and a bounded live
  canister detail report. Text and JSON preserve raw canister classification,
  name, controllers, Subnet, language, module hash, Dashboard update time, and
  nullable proposal-linked upgrade history alongside endpoint and retrieval
  provenance. Reports explicitly state that Dashboard evidence is not
  certified and is not one point-in-time snapshot. The public library exposes
  `LiveIcSource`, `IcCanisterSource`, typed source/request/report/error DTOs,
  custom-source construction, validation, and rendering. This report is
  live-only and does not add a cache.

  ```bash
  icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai
  icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai --format json
  ```

## [0.17.x] - 2026-07-30 - SNS Root inventory and health

Detailed release notes: [docs/changelog/0.17.md](docs/changelog/0.17.md)

- `0.17.1` hardens live and custom-source boundaries. Malformed or non-HTTP(S)
  endpoints now return typed agent-build errors instead of reaching
  `ic-agent` parser panics, and the global mainnet option is forwarded
  consistently to every NNS and SNS command family. ICRC account-transaction
  pages, SNS discovery and Root inventory, and NNS Registry version results
  now validate returned identity, provenance, limits, ordering, and authority
  claims before projection. Duplicate SNS health summaries no longer select
  status by response order. Complete NNS Governance, SNS Governance, and ICRC
  index caches reject unknown or duplicate top-level fields and impossible
  point-in-time claims. This is a pre-1.0 Rust-API and cache-validation hard
  cut: new typed error variants are added, previously tolerated invalid custom
  results and extra cache fields are rejected, and no compatibility path is
  retained.

  ```bash
  icq --network ic nns governance economics
  icq --network ic nns neuron list --limit 25
  icq --network ic sns canister list 1
  ```

- `0.17.0` adds `SnsCanisterSource` on the existing `LiveSnsSource` and a
  typed live SNS Root inventory and operational-health report. Root's
  `list_sns_canisters` query remains the membership authority; the health
  ingress always sends `update_canister_list = false`. Reports preserve native
  roles and status, raw module hashes and operational values, canonical
  ordering, explicit typed relation gaps, and the lack of a point-in-time
  guarantee. The shared `SnsSourceRequest` now carries network identity, a
  breaking pre-1.0 hard cut that lets direct built-in SNS source calls reject
  non-mainnet networks before constructing an agent. The report is live-only
  and does not alter existing caches.

  ```bash
  icq sns canister list 1
  icq sns canister list 23ten-uaaaa-aaaaq-aabia-cai --format json
  ```

## [0.16.x] - 2026-07-30 - Native NNS Governance reports

Detailed release notes: [docs/changelog/0.16.md](docs/changelog/0.16.md)

- `0.16.0` adds live native NNS Governance economics, cached-metrics, latest
  reward-event, and maturity-modulation reports. The library exposes one
  focused `NnsGovernanceSource` capability on `LiveNnsSource`, typed public
  payloads and errors, shared flattened query provenance, and text renderers.
  JSON preserves current Governance fields and raw numeric values, including
  named `key`/`value` projections for unlabeled Candid metric buckets. Every
  entry point rejects non-mainnet networks before a source or agent call.
  These bounded point-value reports are explicitly live-only and do not read
  or write the proposal or neuron caches.

  ```bash
  icq nns governance economics
  icq nns governance metrics --format json
  icq nns governance reward-event
  icq nns governance maturity-modulation
  ```

## [0.15.x] - 2026-07-30 - Public NNS neuron reporting

Detailed release notes: [docs/changelog/0.15.md](docs/changelog/0.15.md)

- `0.15.1` consolidates the identical proposal and neuron collection
  contracts into shared `NnsGovernanceRefreshRequest`,
  `NnsGovernanceCacheRequest`, and `NnsGovernanceRefreshAttemptStatus` types,
  and exposes one typed `NnsGovernanceQueryError` instead of copying its
  transport variants and mapping flow into each report family. Shared
  Governance cache provenance and validation are also defined once, and
  proposal cache JSON/operation failures use the shared `HostCacheError`.
  This is a breaking Rust-API hard cut: the replaced family-specific request,
  attempt, transport-error, and duplicated proposal cache-error variants are
  removed without aliases. CLI behavior, report JSON, and cache schemas are
  unchanged.

- `0.15.0` adds native public NNS neuron list and detail queries, complete
  atomic refreshes, cache-preferred reads, and cache-only status reporting.
  Reports preserve raw Governance neuron state, visibility, type, vote,
  stake, staked maturity, voting-power, known-neuron, ballot, endpoint, and
  collection provenance. Complete collections validate canonical neuron-id
  pagination and API exhaustion while explicitly recording
  `point_in_time_guaranteed: false` because Governance exposes no stable
  collection version. The existing `LiveNnsSource` gains the focused
  `NnsNeuronSource` capability, and proposal and neuron calls now share one
  internal Governance transport.

  ```bash
  icq nns neuron list --limit 25
  icq nns neuron info 123456789 --verbose
  icq nns neuron refresh --page-size 300
  icq nns neuron cache status --format json
  ```

## [0.14.x] - 2026-07-30 - Complete snapshots and shared cache contracts

Detailed release notes: [docs/changelog/0.14.md](docs/changelog/0.14.md)

- `0.14.1` moves every CLI cache from repository-local `.icq` state to one
  user-level cache root resolved from `ICQ_CACHE_ROOT`,
  `$XDG_CACHE_HOME/ic-query`, or `$HOME/.cache/ic-query`. Library cache
  requests now accept the actual cache root and consistently use `cache_root`
  terminology. The patch also consolidates repeated ICRC ledger requests, NNS
  inventory requests, SNS cache inspection DTOs, JSON cache error mapping,
  refresh policies, and attempt progress. Account-history collection reduces
  full-history memory overhead, requires custom sources to preserve an
  explicitly requested index, and retains the resolved index in failed
  collection evidence. This is a breaking CLI-storage and Rust-API hard cut:
  former project discovery, `ICQ_ICP_ROOT`, family-specific shared DTOs, old
  field and builder names, compatibility aliases, and cache migration are not
  retained. CLI command grammar and report schemas are unchanged.

- `0.14.0` replaces the live-only plural account-history command with explicit
  live-page, complete-refresh, cache-only list, and cache-status operations.
  Complete refreshes resolve and verify one index, exhaust backward pagination,
  validate canonical unique transaction ids and stable pagination evidence,
  and atomically publish one account snapshot. Failed or capped refreshes
  preserve the last complete cache and record attempt progress. This is a
  breaking CLI and library hard cut with no aliases for the replaced plural
  command or public page types. Complete snapshots explicitly report that the
  index API provides no point-in-time guarantee.

  ```bash
  icq icrc account transaction page mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa --limit 25
  icq icrc account transaction refresh mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa
  icq icrc account transaction list mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa --sort oldest --limit 100
  icq icrc account transaction cache status mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa
  ```

## [0.13.x] - 2026-07-29 - Canonical query command hierarchy

Detailed release notes: [docs/changelog/0.13.md](docs/changelog/0.13.md)

- `0.13.2` authenticates live ICRC-3 tip certificates against the IC root of
  trust and requested ledger canister, enforces certificate freshness, and
  validates that the returned hash-tree digest and required tip leaves match
  the certified data. The capability probe now reports a tip certificate as
  available only after the same verification. CLI and JSON shapes are
  unchanged.

- `0.13.1` adds typed, live ICRC index account-history queries. The command
  discovers the index through ICRC-106 unless one is supplied, verifies the
  index reports the requested ledger, preserves ledger/index/account/endpoint
  provenance and the full typed transaction payload in JSON, and exposes an
  exclusive backward-pagination cursor. It supports both generic index-ng and
  the deployed ICP index interface. Account-history caching remains a roadmap
  item.

  ```bash
  icq icrc account transactions mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa --limit 25
  icq icrc account transactions ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa --index-canister-id qhbym-qaaaa-aaaaa-aaafq-cai --format json
  ```

- `0.13.0` makes the roadmap command hierarchy current: SNS governance uses
  singular `proposal` and `neuron` families with explicit operations, while
  generic ICRC queries are separated into ledger-wide and account-scoped
  families. This is a breaking CLI hard cut; the replaced direct and plural
  forms have no aliases. The public NNS proposal unsupported-network error now
  retains the rejected network identity. Report models, JSON shapes, and cache
  schemas are unchanged.

  ```bash
  icq sns proposal list 1
  icq sns proposal info 1 387
  icq sns neuron list 1
  icq icrc ledger transactions ryjl3-tyaaa-aaaaa-aaaba-cai
  icq icrc account balance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa
  ```

## [0.12.x] - 2026-07-29 - Canonical NNS reporting adapter

Detailed release notes: [docs/changelog/0.12.md](docs/changelog/0.12.md)

- `0.12.3` centralizes the built-in mainnet network policy used by NNS, SNS,
  topology, and subnet-cache adapters so unsupported-network rejection cannot
  drift between report families.

- `0.12.2` rejects non-mainnet global network identities before NNS or SNS
  family dispatch and makes the current mainnet-only target contract explicit
  in top-level help and documentation.

- `0.12.1` adds a living 1.0 roadmap that tracks reporting coverage, caching
  and follow-up-query policy, prioritized workstreams, and the completion bar.
  CLI help now identifies live, cache-backed, cache-preferred, cache-only,
  forced-refresh, and view-dependent collection behavior. Global `--network`
  is honored by NNS proposals and rejected for ICRC commands instead of being
  silently ignored. `make install` now replaces an existing local `icq`
  binary.

  ```bash
  icq --network ic nns proposal list
  ```

- `0.12.0` replaces nine family-specific live NNS host adapters with
  `ic_query::nns::LiveNnsSource` and replaces four duplicate source request
  DTOs with
  `ic_query::nns::NnsSourceRequest`. Direct live Registry, subnet-catalog,
  inventory, proposal, and topology capability calls enforce their network
  contract consistently. This is a breaking library hard cut with no aliases;
  report schemas, cache formats, CLI behavior, and JSON output are unchanged.

## [0.11.x] - 2026-07-29 - Exact-version NNS topology

Detailed release notes: [docs/changelog/0.11.md](docs/changelog/0.11.md)

- `0.11.4` centralizes NNS node, data-center, node-operator, and node-provider
  cache-path, refresh-lock-path, network-validation, and typed cache-load
  wrappers. Public paths, errors, cache schemas, and refresh behavior are
  unchanged.

- `0.11.3` centralizes the shared NNS node, data-center, node-operator, and
  node-provider refresh-report and text projections. Public report fields,
  JSON and text output, cache behavior, and CLI behavior are unchanged.

- `0.11.2` consolidates duplicated SNS neuron and proposal cache error mapping,
  summary projection, and refresh-attempt persistence while preserving public
  APIs, cache and sidecar schemas, errors, and CLI behavior. Automatic
  missing-cache refreshes for the four NNS inventory families now use their
  existing refresh-request constructors instead of repeating defaults.

- `0.11.1` consolidates Registry inventory source requests, exact relation
  resolution, host cache errors, freshness/provenance projection, and SNS
  refresh-attempt reads. All public NNS Registry inventory live sources now
  reject non-mainnet requests before agent construction. This is a breaking
  library hard cut: the five family-specific inventory source-request types
  become `ic_query::nns::NnsInventorySourceRequest`, and component cache
  failures are exposed through `HostCacheError` and `CacheFileError`. Report
  schemas, cache formats, and CLI behavior are unchanged.

- `0.11.0` adds an exact-Registry-version `NnsSubnetTopologyReport` with raw
  Subnet kinds, canonical per-Subnet node-provider counts, strict relation and
  count validation, atomic joined caching, and distinct load, refresh,
  refresh-missing, and refresh-stale library APIs. This is a breaking library
  release because `NnsTopologyProvidersReport` gains the required
  `registry_versions` field; aggregate provider reports now retain every
  component Registry version instead of dropping their source provenance. The
  public live source rejects non-mainnet requests before agent construction,
  and required node and node-operator `key_not_present` responses become
  relation-specific typed errors while preserving other Registry failures.
  The native live-call stack also updates to `ic-agent` 0.49.2.

## [0.10.x] - 2026-07-12 - Canonical library and process boundaries

Detailed release notes: [docs/changelog/0.10.md](docs/changelog/0.10.md)

- `0.10.4` centralizes hosted and local CI on one sequential gate, separates
  CI, publication, and release-script regressions, and makes workspace
  publishing resumable across crates.io indexing delays. Publishing now
  requires the release tag at `HEAD`, package-list failures propagate, and
  pre-bump cleanliness is checked before CI starts.

  ```bash
  make ci-scripts-check
  make publish-guards-check
  ```

- `0.10.3` makes the public-rustdoc debt check independent of forced Cargo
  color in hosted CI, covers that environment in the local release guards,
  reruns the complete CI gate on the release commit before pushing it, and
  fully verifies the CLI package against an unpublished same-workspace library
  version instead of requiring that version to exist on crates.io first. The
  push gate also rejects a current-version tag that does not point to `HEAD`.

- `0.10.2` fixes CI setup by installing the pinned `ripgrep`, `cargo-audit`,
  and `cargo-machete` development tools through one shared target. The library
  process-boundary check now reports how to install its missing `rg`
  prerequisite instead of failing opaquely.

  ```bash
  make install-dev
  ```

- `0.10.1` removes the remaining stdout/stderr ownership from `ic-query`.
  Paged refreshes expose structured `QueryProgressEvent` callbacks while the
  `ic-query-cli` crate owns terminal detection, same-line progress, and
  missing-cache notices. It also shares the duplicated SNS cache identity
  model and validation, centralizes complete-snapshot path discovery, closes
  the 0.10 documentation drift, and adds CI guards for the process boundary
  and the existing public-rustdoc backlog.

- `0.10.0` makes each NNS and SNS family root its only public library path.
  Redundant nested `report` modules are private, topology requests, reports,
  builders, sources, constants, and renderers now live at
  `ic_query::nns::topology`, and the node-provider endpoint constant is named
  `DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT`. This is a hard cut with no old
  path re-exports, constant alias, deprecated wrapper, or compatibility shim.

## [0.9.x] - 2026-07-11 - Library and CLI ownership cut

Detailed release notes: [docs/changelog/0.9.md](docs/changelog/0.9.md)

- `0.9.0` removes the library `cli` feature and moves all Clap definitions,
  command parsing and dispatch, process output, project-context discovery, and
  CLI-only errors into `ic-query-cli`. `ic-query` now exposes only reusable
  duration helpers, requests, report models, source adapters, cache/refresh
  mechanics, builders, and renderers through its default and `host` surfaces.
  This is a hard cut: downstream crates must use the typed library API or the
  `ic-query-cli` executable crate; no compatibility feature, command aliases,
  re-export shims, or deprecated wrappers are retained.

## [0.8.x] - 2026-07-11 - Hard cache and runtime cut

Detailed release notes: [docs/changelog/0.8.md](docs/changelog/0.8.md)

- `0.8.3` reorganizes the generic ICRC implementation around explicit command
  dispatch, option parsing, report assembly, source adaptation, live fetching,
  public contracts, source data, error, and subaccount-validation boundaries.
  The cleanup preserves the existing CLI, public Rust API, JSON reports, and
  schema versions, and removes the remaining production parser panic.

- `0.8.2` makes numeric SNS proposal and neuron cache lookup read lightweight
  snapshot headers before loading only the matching complete snapshot, and
  rejects duplicate cache ids instead of selecting one by filesystem order.
  It also adds exhaustive current-contract coverage for snapshot completeness,
  routing-range order, terminal control escaping, and failed atomic-write
  cleanup.

- `0.8.1` restores the detailed 0.8 release ledger omitted from the 0.8.0
  commit and makes release automation fail closed: target-version notes and
  the complete CI gate must pass before metadata changes, package failures are
  preserved, untracked files fail the clean-tree check, and release steps stay
  sequential. It also completes the hard cut by removing positional version
  shortcuts and redundant public request/status aliases, requiring stable SNS
  row identifiers, validating persisted lock, attempt, and snapshot invariants,
  resetting every current report schema to version 1, preserving raw ICRC
  metadata in JSON while escaping terminal controls in text, and rejecting
  ambiguous routing catalogs.

- `0.8.0` hardens live host execution, complete snapshot publication, and cache
  operations. Live builders no longer panic when called from an existing
  Tokio runtime; paged refreshes reject stalled pages and invalid page sizes;
  failed proposal refreshes preserve progress; stale locks require explicit
  operator cleanup; CLI parse errors retain their diagnostics; and malformed
  attempt sidecars remain visible. This is an intentional breaking release:
  NNS/SNS complete snapshot schema is reset to version 1 and requires logical
  identity fields. Snapshot files outside the current shape and removed
  refresh-lock field names are rejected, with no compatibility aliases or
  automatic migrations.

## [0.6.x] - 2026-07-01 - Public source adapters

Detailed patch breakdown: [docs/changelog/0.6.md](docs/changelog/0.6.md)

- `0.6.9` fixes documentation drift after the source-adapter pass. The README
  now describes `host` as the feature for live calls, cache-backed builders,
  refresh helpers, and custom source adapters, and the library guide now leads
  with the 0.6 public source-adapter model instead of old 0.5-era boundary
  wording.

- `0.6.8` adds a compile-tested downstream source-adapter example to the
  library guide. The example shows how host users can implement a custom
  `NnsRegistrySource` for fixture, mirror, or proxy-backed report assembly
  without enabling `cli` or routing through the built-in live adapter.

- `0.6.7` completes the NNS topology source-adapter path. Host users can now
  implement `NnsTopologyRefreshSource` and pass it to topology refresh without
  routing through the built-in live topology adapter, while the default refresh
  builder now uses the same source boundary internally.

- `0.6.6` opens the NNS topology read-report source-adapter path. Host users
  can now implement `NnsTopologySource` and pass it to topology summary,
  coverage, versions, health, gaps, capacity, regions, and providers builders
  without routing through the built-in live topology adapter.

- `0.6.5` completes the SNS source-adapter path for neuron reports. Host users
  can now implement `SnsNeuronsSource` and pass it to SNS neuron list builders
  and complete neuron-cache refresh without routing through the built-in live
  SNS adapter.

- `0.6.4` extends the SNS source-adapter path to governance proposals. Host
  users can now implement `SnsProposalSource` and `SnsProposalsSource` and
  pass them to SNS proposal detail/list builders and complete proposal-cache
  refresh without routing through the built-in live SNS adapter.

- `0.6.3` opens the first SNS source-adapter path. Host users can now
  implement `SnsListSource`, `SnsTokenSource`, and `SnsParamsSource` and pass
  them to SNS list, info, token, and governance-parameter builders without
  routing through the built-in live SNS adapter.

- `0.6.2` opens the NNS proposal source-adapter path. Host users can now
  implement `NnsProposalSource` and pass it to proposal list/detail builders
  and complete-cache refresh without exposing private governance wire DTOs or
  routing through the built-in live NNS governance adapter.

- `0.6.1` extends the public source-adapter line to NNS inventory reports.
  Host users can now implement custom node, data-center, node-provider, and
  node-operator sources and pass them to the matching list/info builders and
  refresh wrappers without routing through the built-in live NNS registry
  adapters.

- `0.6.0` starts the public source-adapter line. Generic ICRC host users can
  now implement `IcrcSource` and call `build_icrc_*_report_with_source`
  builders directly, and subnet catalog host users can implement
  `SubnetCatalogSource` for cache refresh, load-or-refresh, and report
  builders. NNS registry version host users can also implement
  `NnsRegistrySource` and call `build_nns_registry_version_report_with_source`
  without routing through the built-in live NNS registry adapter.

## [0.5.x] - 2026-06-25 - Library boundary cleanup

Detailed patch breakdown: [docs/changelog/0.5.md](docs/changelog/0.5.md)

- `0.5.24` closes the 0.5 downstream-library documentation pass. The library
  usage guide now includes compile-tested SNS proposal, neuron, and
  cache-status examples for host users that do not enable `cli`, and it
  clarifies that custom source traits remain internal until a later public
  source-adapter design.

- `0.5.23` tightens the final library-boundary cleanup for the 0.5 line.
  CLI-only stdout/project-root helpers now compile only with `cli`, CLI
  duration parsing is no longer included in host-only downstream builds, and
  the generic NNS leaf refresh constructor is limited to CLI dispatch paths
  while host refresh accessors remain available.

- `0.5.22` refreshes downstream library documentation. The README now keeps a
  shorter feature-boundary summary and links to a dedicated library usage guide
  with canic-style migration notes, host/no-default dependency guidance, and
  examples for replacing `icq` process shell-outs with public request
  constructors and report builders. A downstream-usage integration test now
  keeps those example patterns compiling under no-default and host-only builds.

- `0.5.21` completes the remaining small NNS request constructor pass.
  Registry-version and topology read/refresh requests now have public
  constructors, topology refresh has a dry-run setter, and registry/topology CLI
  dispatch uses those constructors.

- `0.5.20` improves subnet catalog request ergonomics for downstream library
  users. Cache, list, info, and refresh requests now have public constructors
  and builder-style setters for filters, range display, forced resolution,
  dry-run, and output controls; subnet CLI and topology adapters use the same
  constructors.

- `0.5.19` improves NNS inventory request ergonomics for downstream library
  users. Node, data-center, node-provider, and node-operator list/info/refresh
  requests now have public constructors; node list requests also have
  builder-style filter setters, and the CLI construction paths use the public
  constructors.

- `0.5.18` improves NNS proposal request ergonomics for downstream library
  users. NNS proposal list/detail requests now have public constructors and
  builder-style setters for filters, sort controls, verbosity, and ballot
  display, and the CLI dispatch path now uses those library constructors.

- `0.5.17` improves SNS request ergonomics for downstream library users. SNS
  list, lookup, proposal list/detail, and neuron list requests now have public
  constructors and builder-style setters for optional view and cache controls.

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
