# Changelog

All notable changes to `ic-query` will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/), and this
crate follows [Semantic Versioning](https://semver.org/).

## Unreleased

## [0.30.x] - 2026-08-05 - Certified Registry evidence

Detailed release notes: [docs/changelog/0.30.md](docs/changelog/0.30.md)

- `0.30.16` adds explicit confined publication and bounded restoration for the
  certified Registry archive. A streaming publisher writes each canonical
  report once as an owner-only content-addressed object, syncs files and newly
  created directories, and atomically replaces the manifest only after complete
  authenticated replay. Loading bounds and hashes one object at a time,
  reauthenticates all retained evidence locally, and requires an exact recomputed
  manifest. Missing, oversized, noncanonical, unconfined, symlinked, or tampered
  content fails closed; failed final publication preserves the prior manifest.
  There is no default path, refresh policy, network call, CLI, or certified
  catalog promotion.

- `0.30.15` adds a versioned, bounded certified Registry archive-manifest
  contract under `nns-host`. Its builder accepts only locally authenticated
  retained batches, streams canonical report JSON through SHA-256 and byte
  accounting without another encoded copy, enforces explicit batch/per-report/
  total ceilings before replay publication, and emits a manifest only after
  exact-target completion. Pure validation rejects identity, schema, ordering,
  continuity, digest, endpoint, and accounting inconsistencies. Manifests remain
  untrusted indexes until every report is reauthenticated and replayed; this
  adds no filesystem storage, cache, network call, CLI, or certified catalog
  assurance.

- `0.30.14` makes the shared `patch`, `minor`, and `major` release gate run
  `cargo clean` after CI exits, including failed CI runs, while preserving the
  gate's original status if cleanup itself fails. CI guards now use exact,
  named, trapped temporary paths; feature-boundary logs share one owned
  directory and are removed together on success or failure.

- `0.30.13` adds a bounded in-memory replay builder that accepts only sealed,
  locally reauthenticated Registry delta batches. It starts at version zero,
  preserves the existing exact-target, cumulative-limit, provenance, and
  atomic-application contracts, exposes read-only progress, and returns the
  same sealed authenticated replay-session type as live bootstrap only after
  complete reconstruction. It performs no network or filesystem IO and adds
  no archive format, cache, CLI, or certified catalog assurance.

- `0.30.12` adds local reauthentication for retained schema-3 certified
  Registry delta reports. The operation validates the report, verifies its raw
  certificate and mixed-tree commitment against the built-in mainnet root key,
  decodes the committed delta, and compares every version, mutation,
  precondition, and chunk reference before returning a sealed borrowed
  capability. The source endpoint is validated but never called. This adds no
  archive, cache, replay restoration, CLI, or certified catalog assurance.

- `0.30.11` hard-cuts certified Registry delta reports to schema 3 and
  retains each unique hash-verified large-value chunk once in canonical digest
  order. The pure validator now re-hashes every retained chunk, requires the
  table to match the exact referenced digest set, recomputes retained-byte
  accounting, and reconstructs every chunked mutation from its ordered
  evidence. Live calls and ceilings are unchanged. Replay evidence-chain
  digests consequently commit to the stronger report; state digests are
  unchanged. This adds no cache, CLI, or certified catalog assurance.

- `0.30.10` hard-cuts the built-in certified Registry bootstrap to return a
  sealed `NnsAuthenticatedRegistryReplaySession`. Only ic-query's fixed live
  path, which verifies every batch against the mainnet root key, can construct
  it; custom sources and manual replay continue to return ordinary sessions.
  An authenticated catalog-projection wrapper composes the existing projection
  without copying rows or provenance. Callers now use `.replay_session()` to
  inspect the live-bootstrap result or `.into_replay_session()` to explicitly
  discard the capability. This remains in-memory evidence and does not enable
  cached or validated certified catalog assurance.

- `0.30.9` adds a pure, cacheless Subnet Catalog projection from a completed
  exact-target Registry replay session. It decodes the replayed Subnet list,
  routing table, and every referenced Subnet record, then uses the same
  canonical ordering, IC-native classification policy, and routing validation
  as the live catalog. The projection borrows its replay session so version and
  provenance commitments remain attached. Incomplete state and missing,
  malformed, or structurally invalid records fail with typed errors. This does
  not serialize a mirror, publish a catalog, or enable certified assurance.

- `0.30.8` extends exact-target replay sessions with compact provenance needed
  before any certified projection. Each admitted validated report advances a
  domain-separated SHA-256 evidence chain without buffering another full
  report; sessions also retain distinct source endpoint strings in canonical
  order and certificate-time bounds. A separate canonical digest of keys,
  values, mutation positions, and timestamps appears only after the pinned
  target is complete. One public schema constant versions both commitments.
  Failed batches publish none of this candidate provenance. The commitments
  are not serialized, cached, or standalone authentication, and certified
  Subnet Catalog assurance remains unsupported.

- `0.30.7` aligns certified replay with two additional committed Registry
  history rules proven by a complete bounded mainnet reconstruction. Repeated
  keys inside one atomic version are preserved and applied in their stable,
  canonical key order; retained value content on a committed delete remains
  raw evidence but is ignored when rebuilding current state. Decreasing key
  order still fails closed. A cacheless probe reconstructed exact Registry
  target `62948` in 75 batches and 77 calls using 63,814,080 encoded response
  bytes and 22,345,176 bytes of current state; these measurements are evidence,
  not new defaults.

- `0.30.6` adds a bounded diagnostic Registry bootstrap probe. It shares the
  complete bootstrap's pre-call reservation and validation loop but returns a
  typed `Complete` or `CapacityReached` outcome with the accumulated session,
  allowing current-history sizing without another unreserved call. A zero-call
  budget returns explicit empty progress. The complete bootstrap remains
  complete-only and converts capacity exhaustion to its existing typed error;
  probe state is explicitly incomplete, uncached, and not catalog authority.
  Certified replay now also preserves historical non-delete mutations whose
  empty legacy protobuf value is absent on the wire, matching the official
  Registry transport's empty-inline-value interpretation.

- `0.30.5` adds an explicit caller-runtime async Registry bootstrap from
  version zero. Before every built-in source call it reserves capacity for one
  certified query, up to 64 chunk queries, and up to 40 MiB of encoded
  responses; insufficient remaining batch, call, or byte capacity stops before
  the call. The first response pins the exact target and only a complete
  session is returned. Limits have no defaults, custom sources remain
  responsible for their internal work, and no cache, CLI, or certified catalog
  publication is added.

- `0.30.4` adds a pure exact-target Registry replay session. Its first valid
  certified batch pins the selected Registry version; later batches must still
  certify that target and may not move it when the live Registry advances.
  Explicit ceilings bound admitted Registry versions, batches, reported query
  calls, encoded response bytes, and reconstructed state. Root-key changes,
  cumulative-limit failures, and completed-session reuse fail atomically. The
  session performs no source calls and does not yet publish certified catalog
  authority.

- `0.30.3` adds a pure in-memory Registry replay API that applies exactly one
  validated certified delta batch after the state's current version. It uses
  the IC's committed-changelog replacement/delete semantics, requires explicit
  caller ceilings for live entries and combined raw key/value bytes, and
  publishes the candidate state only after the whole batch succeeds. The state
  begins at version zero and carries current values and mutation positions, but
  is not serialized authority evidence. No network loop, cache, CLI, catalog
  projection, or assurance promotion is added.

- `0.30.2` hard-cuts certified Registry delta reports to schema 2 and completes
  certified large-value chunk references under fixed call, chunk, per-value,
  total-value, response-byte, and agent-body ceilings. Reports preserve the
  original absent/inline/chunked encoding, ordered SHA-256 references,
  reconstructed value bytes, and separate certified/chunk/total accounting;
  the pure validator recomputes those invariants. Repeated digests reuse one
  verified chunk response. The shared exact-version Registry value path now
  applies the same bounded hash-verified reconstruction. This adds no cache,
  replay loop, CLI surface, or certified Subnet Catalog claim.

- `0.30.1` adds a mainnet-only caller-runtime async library operation for one
  certified `get_certified_changes_since` batch. It authenticates the shared
  latest-version and delta witness, requires contiguous eight-byte version
  labels, decodes native atomic mutations, preserves raw evidence and exact
  accounting, and fails closed on malformed, oversized, unknown, conflicting,
  or incomplete content. A pure validator recomputes structural report
  invariants for trusted custom sources; cryptographic authentication remains
  the source's responsibility. The operation is one-call, uncached,
  non-paginating, has no CLI surface, and does not promote Subnet Catalog
  assurance.

- `0.30.0` hard-cuts the existing NNS Registry version report and source
  contract to one authenticated `get_certified_latest_version` call. The host
  verifies the mainnet-root-key certificate, Registry canister authority,
  certified-data commitment, bounded protobuf mixed hash tree, canonical
  unsigned-LEB128 version leaf, and certificate time before returning version
  2 evidence. JSON preserves the certificate, witness, root-key digest, time,
  and byte counts; text stays compact. This certifies only the latest version,
  not ordinary Subnet Catalog `get_value` reads. Command grammar and caching
  are unchanged; `nns-host` now has a direct CBOR dependency.

## [0.29.x] - 2026-08-05 - Subnet Catalog authority and embedder hardening

Detailed release notes: [docs/changelog/0.29.md](docs/changelog/0.29.md)

- `0.29.8` completes the family-level library feature split with `nns-host`
  for the complete NNS governance, Registry inventory, component-cache, and
  topology surface, plus `cmc-host` for certified Cycles Minting Canister
  reports. `nns-host` includes `nns-topology-host`; `host` remains the complete
  convenience union. APIs, cache schemas and paths, calls, CLI behavior, and
  default features are unchanged.

- `0.29.7` adds the independent `sns-host` library feature for native SNS
  discovery, targeted reports, complete proposal/neuron caches, reward
  checkpoints, and local reward diffs. It excludes Dashboard, Registry, NNS,
  system-canister, and native ICRC host adapters and has no direct ic-query
  Reqwest, Prost, CBOR, or SHA-256 edge; Reqwest, CBOR, and cryptographic
  packages remain transitive through `ic-agent`. APIs, schemas, paths, calls,
  and CLI behavior are unchanged.

- `0.29.6` adds the independent `icrc-host` library feature for native ICRC
  ledger/index queries, certified-tip verification, and complete
  account-history caches. It excludes Dashboard, Registry, NNS, and SNS host
  adapters and has no direct ic-query Reqwest or Prost edge; Reqwest remains
  transitive through `ic-agent`. APIs, schemas, paths, calls, and CLI behavior
  are unchanged.

- `0.29.5` adds the independent `dashboard-host` library feature for official
  Dashboard REST reports and the confined observed node-status cache. It
  excludes `ic-agent`, Registry protobufs, CBOR certification, and native
  NNS/SNS/ICRC host adapters while `host` remains the complete union. Report
  and cache schemas, paths, live calls, and CLI behavior are unchanged.

- `0.29.4` adds the focused `nns-topology-host` library feature as a strict
  superset of `subnet-catalog-host`. It exposes the exact-version joined NNS
  Subnet/node/operator/provider live and cache API without enabling ic-query's
  direct Dashboard Reqwest or CBOR certification edges; those packages may
  remain transitive through `ic-agent`. Broader independently cached topology
  summaries remain under `host`. Rust report shapes, cache schemas and paths,
  live call behavior, and CLI grammar are unchanged.

- `0.29.3` closes the authority-bearing route API by returning the matched
  validated Subnet classification with each Canister route. Catalog loads can
  require a minimum assurance, reject known-insufficient refresh selections
  before collection, and emit compact persistable authority evidence; a
  convenience constructor covers explicit missing/invalid/stale refresh.
  Defaults, cache schemas and paths, CLI grammar, and ordinary one-endpoint
  call behavior are unchanged. Release automation now runs its complete CI
  gate once before changing version files and does not repeat it after commit
  and tag creation.

- `0.29.2` hard-cuts the Subnet Catalog host API and version-2 cache/report
  evidence to caller-runtime async load/refresh operations with synchronous
  adapters, exact Registry query-call counts, and explicit one-endpoint or
  bounded two-to-three-endpoint source selection. Agreement requires the same
  Registry version and canonical Registry payload from distinct hostnames;
  mismatch never falls back to one endpoint. CLI grammar remains unchanged
  and continues to select one endpoint. Version-1 catalog evidence is not
  migrated; authorized invalid-content read-through replaces it, while
  cache-only callers must refresh explicitly.

- `0.29.1` hard-cuts every managed cache family to capability-rooted load,
  discovery, refresh-lock, and atomic-publication operations. On Unix it
  rejects symlinks, path escapes, nonregular managed files, permissive
  directories, and files not using mode `0600`; new directories and files use
  `0700` and `0600`. Cache paths, schemas, network behavior, and CLI grammar are
  unchanged. Older permissive cache trees are not migrated or repaired
  automatically and must be removed or secured before use.

- `0.29.0` hard-cuts the Subnet Catalog Rust and persisted-report contracts to
  separate raw JSON from validated evidence, label current single-endpoint
  Registry collection `uncertified_query`, retain raw Registry Subnet types,
  bind routes to exact provenance and a canonical digest, and make every cache
  refresh policy and resulting disposition explicit. It also adds a
  caller-runtime async fetch, structured library error classification, and
  refresh/list/info provenance without changing CLI grammar or live call
  counts. Existing catalog caches use the replaced version-1 shape: authorized
  read-through repairs them, while cache-only callers must explicitly refresh.

## [0.28.x] - 2026-08-04 - observed IC node and Subnet status

Detailed release notes: [docs/changelog/0.28.md](docs/changelog/0.28.md)

- `0.28.5` makes `HostCacheError` the single public owner of generic JSON
  cache failures for observed node status and SNS reporting. Family errors
  retain only missing-cache guidance and semantic/identity failures; cache
  schemas, refresh policy, commands, and report output are unchanged.

- `0.28.4` consolidates official Dashboard adapter invariants without changing
  reports: canister page cursors now have one normalization path for public
  builders and direct live-source calls, shared principal validation lives at
  the common source boundary, and bounded metric/ICRC series use one inclusive
  observation-count rule. Commands, JSON, text, calls, and caches are
  unchanged.

- `0.28.3` consolidates the observed-status projection internals without
  changing reports: Subnet aggregates are constructed once before view
  filtering, node/Subnet/provider selection and non-up evidence reuse shared
  helpers, and the redundant status-only counting flow is removed. Every
  bounded Dashboard time-series request also shares one collection-time bound.
  Commands, JSON, text, calls, and cache behavior are unchanged.

- `0.28.2` adds typed less/equal/greater comparisons of each provider's
  unassigned versus assigned up and conservative non-up node counts. Reports
  include all-provider comparison totals and compact text labels, while the
  projection now constructs each provider aggregate once; calls and cache
  identity are unchanged.

- `0.28.1` separates status-report provenance and summaries from their tables
  with a blank line. It also rejects empty live/custom node snapshots,
  noncanonical cached row order, and semantically invalid rows supplied to pure
  projections; existing read-through policies visibly replace recoverably
  invalid caches.

- `0.28.0` adds node-, Subnet-, and node-provider operational views over one
  bounded official Dashboard `/nodes` response. The views share a 60-second
  atomic cache, preserve raw status and assignment evidence, expose explicit
  uncertified/default-scope provenance, and make no per-row follow-up calls.
  Subnet rows keep down-only and conservative non-up fault-distance evidence
  separate, while provider rows retain status-by-assignment comparisons;
  missing, invalid, or stale cache content visibly refreshes.

- The component-cache consistency command and corresponding public topology
  report API are hard-cut from `health` to `check`, avoiding confusion with
  observed machine status. There is no compatibility command, alias, wrapper,
  or cache migration.

```bash
icq nns node status
icq nns subnet status tdb26 --all
icq nns node-provider status --json
icq nns topology check
```

## [0.27.x] - 2026-08-04 - bounded official ICRC analytics

Detailed release notes: [docs/changelog/0.27.md](docs/changelog/0.27.md)

- `0.27.5` makes every live base endpoint credential-, query-, and
  fragment-free before transport construction, disables redirects for official
  Dashboard requests, and caps every native `ic-agent` response at 8 MiB.
  Valid endpoints, call counts, report schemas, and cache behavior are
  unchanged.

- `0.27.4` gives every official Dashboard live adapter one shared 8 MiB
  response-body ceiling. Declared and streamed body sizes are checked before
  JSON decoding, and oversized or incomplete bodies now return distinct
  typed host errors without changing live-only or cache behavior.

- `0.27.3` adds a one-request bounded token-value series with a 24-hour,
  1,000-row default and a hard 90-day, 1,000-row ceiling. Reports preserve
  nullable raw price and 24-hour-volume fields, each external provider and
  URL, exact Dashboard provenance, and explicit possible truncation when the
  requested limit is reached.

```bash
icq icrc analytics token-values mxzaz-hqaaa-aaaar-qaada-cai --limit 100 --json
```

- `0.27.2` adds one-request indexed account and transaction counts alongside
  holder count. All three scalar endpoints now share one typed count kind,
  request, report, source method, builder, renderer, live URL path, and CLI
  dispatch flow; none requests rows, follows a cursor, or creates a cache.

```bash
icq icrc analytics account count mxzaz-hqaaa-aaaar-qaada-cai
icq icrc analytics transaction count mxzaz-hqaaa-aaaar-qaada-cai --json
```

- `0.27.1` adds a one-request official ICRC holder count without requesting or
  enumerating holder rows. Total-supply and holder-count reports now share one
  public analytics ledger target, one CLI target parser, one `LiveIcSource`
  capability, and the existing non-certified Dashboard provenance contract.

```bash
icq icrc analytics holder count mxzaz-hqaaa-aaaar-qaada-cai
```

- `0.27.0` adds a live-only, one-request historical total-supply series for
  one ledger indexed by the official IC Dashboard ICRC analytics API. Public
  requests, reports, renderers, fixture-source builders, and the live adapter
  preserve exact bounds, raw base-unit strings, and explicit non-certified
  Dashboard provenance. Daily queries default to 30 days and all queries are
  capped at 1,000 requested and returned observations; the operation performs
  no enumeration, pagination, follow-up calls, or caching.

```bash
icq icrc analytics total-supply mxzaz-hqaaa-aaaar-qaada-cai
```

## [0.26.x] - 2026-08-03 - SNS maturity reward evidence

Detailed release notes: [docs/changelog/0.26.md](docs/changelog/0.26.md)

- `0.26.21` makes `cache status` distinguish generic header integrity from
  cache age and shows each canonical family's automatic, explicit, or
  missing-only invalid-content recovery policy. The bounded local inventory
  still performs no family-specific semantic validation, full history scan,
  network call, or mutation. Maintained user, library, adapter, and roadmap
  documentation is refreshed against the current CLI and schema contracts.

```bash
icq cache status
```

- `0.26.20` extends recoverable invalid-content refresh from the SNS catalog to
  bounded Subnet and NNS inventory caches and to explicit topology and ICRC
  account-history read-through APIs. Validated replacements remain atomic;
  failed refreshes preserve the original file, cache-only and status operations
  stay local and strict, NNS inventory custom sources are validated before
  publication, and complete Governance histories retain explicit recovery.

- `0.26.19` adds typed native SNS proposal actions and votes, lifecycle-aware
  SNS catalog rows and filtering, terminal-aware metadata health, and shared
  human-readable cycle and byte quantities. `sns list` also atomically repairs
  malformed, incompatible, or invalid joined catalogs. Raw codes, unknown
  evidence, canonical JSON labels, and exact JSON quantities remain lossless.

```bash
icq sns list --all
```

- `0.26.18` replaces duplicated NNS and SNS report provenance strings with
  shared `ReportDataSource` and `ReportResultScope` classifications. Live,
  cache, bounded-live, and complete-cache assembly, validation, rendering, and
  JSON now consume one typed vocabulary without changing output labels.

- `0.26.17` replaces free-form NNS neuron state, visibility, type, and
  recent-ballot vote labels with typed native classifications. Unknown numeric
  codes and omitted optional values remain distinct and lossless; live rows,
  custom sources, and caches now enforce agreement between raw codes and their
  classifications without changing JSON or CLI labels.

- `0.26.17` adds explicit `reported_zero`, `reported_nonzero`, and
  `unavailable` cycle-balance evidence to SNS Root canister reports. A failed
  health query now preserves the discovered inventory with a typed query gap
  instead of discarding it or conflating unavailable balances with zero.

- `0.26.17` hard-cuts every remaining current schema identifier back to `1`,
  including cache-status reports, refresh locks, SNS list/neuron reports and
  neuron snapshots, and NNS topology health. There are no version-2 readers or
  migrations; refresh version-2 SNS neuron snapshots, and verify any lingering
  version-2 refresh lock before removing it manually.

- `0.26.16` replaces free-form NNS proposal topic and ballot-vote labels with
  `NnsProposalTopic` and `NnsProposalVote`. Projection, topic filtering,
  sorting, rendering, and caches now use typed native classifications, and the
  redundant numeric-to-label module is removed without changing JSON or CLI
  labels.

- `0.26.15` replaces free-form NNS proposal decision-status and reward-status
  labels with `NnsProposalStatus` and `NnsProposalRewardStatus`. Native
  projection, filters, sorting, rendering, and caches now share one code/label
  vocabulary while existing JSON and CLI labels remain unchanged.

- `0.26.14` completes `SnsCanisterMethod` coverage for all sixteen fixed
  SNS-native method names used by the live adapters. Discovery, metadata,
  parameters, proposals, neurons, and reward collection now share the same
  typed vocabulary as metrics, Root, swap, and upgrade; the ICRC-106 ledger
  method remains protocol-owned and raw.

- `0.26.13` replaces eighteen free-form native SNS method fields across
  metrics, Root inventory and health, swap, upgrade, and partial-query gaps
  with `SnsCanisterMethod`. Live calls, source validation, reports, gaps, and
  rendering now share the exact native method labels while existing JSON and
  CLI output remain unchanged.

- `0.26.12` replaces free-form SNS metrics and Root-health call-type strings
  with `SnsCanisterCallType`. Source validation, live conversion, reports, and
  rendering now share one typed invocation vocabulary while existing JSON and
  CLI labels remain unchanged.

- `0.26.11` replaces free-form NNS topology-gap classifications and duplicate
  ICRC/SNS ledger-metadata value types with typed domain enums. Gap construction,
  native metadata conversion, ordering, and rendering now consume closed
  vocabularies while existing JSON and CLI labels remain unchanged.

- `0.26.10` replaces free-form ICRC capability status and NNS node Subnet-kind
  strings with typed domain enums. Capability diagnostics, Registry
  projection, node filters, topology counts, and text rendering now consume
  typed classifications while existing JSON, cache, and CLI labels remain
  unchanged.

- `0.26.9` replaces the free-form SNS proposal `decision_state` string with
  `SnsProposalDecisionState`. Live projection, cached filtering, and lifecycle
  sorting now share the typed state while existing cache JSON and CLI labels
  remain unchanged.

- `0.26.8` replaces free-form component-topology assessment, capacity, and
  provider status strings with three public domain-specific enums. Existing
  JSON labels and CLI text are unchanged; typed construction removes invalid
  states and string-based sort fallbacks.

- `0.26.7` consolidates NNS, SNS, and ICRC complete-collection evidence into
  one public `CacheCollectionCompleteness` DTO and validator. Serialized cache
  shapes remain unchanged; the duplicate ICRC-specific type and status
  constant are removed.

- `0.26.6` replaces free-form NNS, SNS, and ICRC refresh-attempt lifecycle
  fields with `CacheRefreshAttemptStatus::{Running, Complete, Failed}`. Writers
  now accept only those states, while existing sidecar/report JSON and CLI text
  labels remain unchanged.

- `0.26.5` replaces the remaining free-form complete-cache validation strings
  in NNS neuron/proposal, SNS neuron/proposal, and ICRC account-history
  summaries with one shared `CacheValidationStatus::{Valid, Invalid}` Rust
  enum. Existing `ok`/`invalid` JSON labels and CLI text are unchanged.

- `0.26.4` replaces free-form global cache and refresh-lock status strings with
  typed Rust enums while preserving the existing JSON labels and CLI output.
  Cache discovery, generic-header parsing, and lock projection now have
  separate internal owners; cache and network behavior is unchanged.

- `0.26.3` extends local-only `cache status` with active, stale, and invalid
  refresh-lock evidence. Refresh locks now record the stale threshold chosen by
  their owner, and competing refreshes honor that recorded policy. NNS
  Registry leaf and Subnet caches now share the `nns/<network>/...` layout used
  by the other domain-scoped caches; typed loaders neither migrate nor reuse
  the replaced paths.

- `0.26.2` adds local-only `cache status`, which inventories known complete
  caches across networks, reports file age and size, and distinguishes managed
  `fresh`/`stale` policies from readable `unmanaged` caches and invalid files.
  It never refreshes or removes cache state.

- `sns list` now reuses one atomic joined discovery catalog for one hour,
  avoiding a repeated metadata fan-out on consecutive calls. `sns refresh`
  forces an explicit replacement; targeted SNS commands retain their bounded
  targeted discovery calls instead of refreshing the all-SNS catalog.

- NNS topology health schema 2 no longer treats sources without an age policy
  as healthy freshness evidence. It reports their count explicitly and marks
  the cache-freshness check non-OK until every source age is assessable.

```bash
icq cache status
icq cache status --json
icq sns refresh
icq sns list
```

- `0.26.1` replaces the abbreviated `sns params` command with
  `sns parameters` as a pre-1.0 hard cut with no alias. Nested Canister,
  neuron, proposal, and reward commands retain the consistent
  family-operation-identifiers grammar.

- Command namespaces without a selected operation now render the same complete
  local help as their explicit `help` subcommand. Every CLI `Commands` section
  is alphabetized, including the generated `help` entry.

```bash
icq sns
icq sns info 1
icq sns parameters 1
icq sns canister list 1
icq sns neuron list 1
icq sns proposal info 1 387
icq sns reward
icq sns reward checkpoint 1 --json
```

- `0.26.0` adds exact live SNS neuron detail and API-exhausted reward
  checkpoints without enlarging the fixed-size neuron list/cache schema. The
  reports preserve native neuron identifiers, permissions, followees,
  maturity state, pending disbursement accounts, complete Governance brackets,
  strict pagination evidence, collection bounds, and typed policy findings.

- It also adds a pure no-host checkpoint validator and local-only reward diff.
  Loaded files are treated as untrusted evidence; allocation is valid only
  when stable target/event evidence, policy observations, non-negative joined
  deltas, and native distributed maturity reconcile exactly. Zero distribution
  and failed invariants remain typed outcomes.

```bash
icq sns neuron info 1 000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f --json
icq sns reward checkpoint 1 --max-pages 10 --json
icq sns reward diff before-checkpoint.json after-checkpoint.json --json
```

## [0.25.x] - 2026-08-01 - Fuller fixed-size SNS neuron evidence

Detailed release notes: [docs/changelog/0.25.md](docs/changelog/0.25.md)

- `0.25.34` separates official Dashboard canister, metric, and bounded
  network-resource request contracts behind the existing IC model facade.
  Public Rust paths, request fields, and behavior are unchanged.

- `0.25.33` separates NNS Governance economics, cached-metrics, and
  reward/modulation text rendering behind the existing facade. Public
  functions and exact human-readable output are unchanged.

- `0.25.32` separates NNS Governance economics, cached-metrics, and
  reward/modulation model families behind the existing explicit facade.
  Public Rust paths, native response shapes, and behavior are unchanged.

- `0.25.31` separates NNS neuron source-independent pagination, provenance,
  and report assembly from the built-in Governance wire adapter. Public Rust
  paths, source calls, cache contracts, and behavior are unchanged.

- `0.25.30` separates ICRC request and report contracts into account,
  indexed account-history, and ledger-wide owners behind the existing explicit
  facades. Public Rust paths, JSON contracts, and behavior are unchanged.

- `0.25.29` separates the NNS proposal model into request, serialized
  report/row, and filter/sort vocabulary owners behind the existing facade.
  Public Rust paths, JSON contracts, and behavior are unchanged.

- `0.25.28` gives SNS proposal and neuron pagination one resolved refresh
  context for request, source, target, attempt, progress, page-cap, incomplete,
  and exhaustion policy. Family-specific cursors, validation, and source calls
  remain separate; refresh behavior is unchanged.

- `0.25.27` centralizes the resolved SNS refresh-attempt lifecycle and reuses
  the shared complete paged-collection result for proposal and neuron
  refreshes, removing duplicate locked adapters and intermediate DTOs.
  Refresh behavior is unchanged.

- `0.25.26` centralizes strict SNS cache loading by stable list id and Root
  canister principal, removing duplicate proposal/neuron lookup flows and the
  redundant neuron lookup-error module. Typed lookup behavior is unchanged.

- `0.25.25` drives SNS cache-list and cache-status reports directly from the
  collection storage marker, removing the duplicate proposal/neuron status
  adapters and remaining list callbacks. Report behavior is unchanged.

- `0.25.24` centralizes complete SNS cache loading and validation behind the
  collection storage contract, deleting the proposal/neuron load modules and
  loader plumbing. Cache errors, reports, and behavior are unchanged.

- `0.25.23` makes each SNS collection marker own its cache schema, field
  allowlist, and missing-cache error, replacing the enum selector and duplicate
  proposal/neuron scan modules. Cache and report behavior are unchanged.

- `0.25.22` replaces duplicated proposal and neuron cache-summary modules and
  their projection macro with one typed SNS snapshot-summary owner. Public
  APIs, cache evidence, report output, and behavior are unchanged.

- `0.25.21` centralizes SNS proposal and neuron cache-list and cache-status
  report assembly, and removes their family-local path forwarders. Public APIs,
  cache identities and schemas, report output, and behavior are unchanged.

- `0.25.20` removes the remaining test-only Subnet duration/error surface,
  compact-renderer visibility bridge, NNS CLI value aliases, and single-use
  test adapters. Tests now use the supported shared duration parser, real host
  errors, command values, and observable report text directly. Production
  behavior is unchanged.

- `0.25.19` removes the final shared test-only Clap error augmenter and three
  System help forwarders. Tests now retain Clap’s native diagnostics without
  cloning and rendering complete commands on parse failures, while System help
  assertions render production builders directly. Production behavior is
  unchanged.

- `0.25.18` removes the ICRC test-support facade and its 13 type-specific
  parser adapters plus 18 help forwarders. ICRC tests now use one generic
  production-Clap harness and render command builders directly; production
  behavior is unchanged.

- `0.25.17` removes SNS option parsers and 22 help-rendering functions that
  existed only for tests. Focused SNS tests now share production-Clap harnesses
  and render the supported command builders directly; production behavior is
  unchanged.

- `0.25.16` removes the remaining test-only helper macro, option parsers, and
  help forwarders from the generic NNS Registry-leaf command families. Node,
  node-provider, node-operator, and data-center tests now exercise production
  Clap builders and option projections directly; production behavior is
  unchanged.

- `0.25.15` removes test-only option parsers and help-rendering modules from
  the NNS Subnet and Topology command owners. Focused tests reuse the shared
  NNS production-Clap harness and render supported command builders directly.
  Production grammar, parsing, dispatch, reports, and source behavior are
  unchanged.

- `0.25.14` removes test-only option parsers and help forwarders from the NNS
  neuron, Governance, and Registry command owners. Focused NNS tests now share
  one production-Clap parsing harness and render supported command builders
  directly. Production grammar, parsing, dispatch, reports, and source behavior
  are unchanged.

- `0.25.13` removes single-use configurable NNS proposal command builders and
  routes proposal option and help tests directly through the supported Clap
  commands. Production grammar, parsing, dispatch, reports, and source
  behavior are unchanged.

- `0.25.12` consolidates repeated Dashboard CLI test parsing for canister,
  metric, and network-resource options behind one IC-family harness. Production
  command parsing, help, dispatch, reports, and source behavior are unchanged.

- `0.25.11` centralizes ledger id, source endpoint, and output selection across
  live ICRC balance, allowance, ledger-transaction, and archive CLI option
  models. Command grammar, parsing behavior, dispatch requests, reports, and
  source behavior are unchanged.

- `0.25.10` separates ICRC account-history cache identity and strict storage,
  refresh and atomic publication, local list/status projection, and attempt
  evidence behind the existing public facade. Public APIs, cache paths and
  schemas, refresh policies, reports, and source behavior are unchanged.

- `0.25.9` removes duplicated test-only parsers from every ICRC option model
  and routes focused command tests through one shared Clap parsing helper.
  Production option parsing, CLI behavior, public APIs, reports, and source
  behavior are unchanged.

- `0.25.8` separates ICRC ledger-wide and account/account-history Clap command
  construction behind the existing command facade. CLI grammar and help,
  option parsing, dispatch, public Rust APIs, reports, and source behavior are
  unchanged.

- `0.25.7` centralizes optional ICRC subaccount normalization across account
  reports and complete-history caches and reuses one typed principal converter
  for live and cached transaction targets. Accepted inputs, errors, cache
  identities, public APIs, reports, and source behavior are unchanged.

- `0.25.6` centralizes common SNS custom-source validation for capability
  errors, exact evidence fields, and canonical principal text. Capability-
  specific invariants, validation errors, public APIs, reports, and source
  behavior are unchanged.

- `0.25.5` centralizes canonical lowercase-hex checks and corrects the focused
  Subnet feature documentation and gate to distinguish direct optional
  dependencies from packages retained transitively through `ic-agent`. It also
  simplifies shared internal feature gates by relying on `host` enabling
  `subnet-catalog-host`. Validation errors, public features and APIs, report
  output, and source behavior are unchanged.

- `0.25.4` separates live ICRC token/account queries, ledger-history and
  archive traversal, and capability probing behind the existing fetch facade.
  Public APIs, report output, source calls, and verification behavior are
  unchanged.

- `0.25.3` separates official Dashboard canister, metrics, and network-resource
  CLI parsing, dispatch, option models, and focused tests behind the existing
  `ic` command facade. CLI behavior, output, source calls, and public Rust APIs
  are unchanged.

- `0.25.2` separates NNS neuron cache models, paths, attempt evidence, page
  collection, publication, refresh orchestration, and cached report projection
  behind the existing public facade. Public APIs, CLI behavior, cache paths and
  schemas, validation, refresh behavior, and network calls are unchanged.

- `0.25.1` corrects schema-version documentation and consolidates shared
  proposal ordering, scalar text formatting, and small NNS leaf command
  adapters. Public APIs, CLI behavior, report JSON, cache schemas, sorting,
  rendering, and network behavior are unchanged.

- `0.25.0` expands every SNS neuron list and complete snapshot row with the
  fixed-size native Governance fields already returned by `list_neurons`:
  source NNS neuron id, auto-stake maturity, aging timestamp, raw dissolve
  state, voting-power percentage multiplier, vesting period, and neuron fees.
  It adds no query, pagination, fanout, or implicit cache behavior.

- This is a pre-1.0 report/cache hard cut. SNS neuron report and cache schemas
  advance to version 2; version-1 neuron caches are rejected rather than
  migrated or read through a compatibility branch and must be explicitly
  refreshed.

- Live custom-source rows, refresh pages, and loaded snapshots now share
  canonical lowercase-id, derived timestamp, uniqueness, and limit validation.
  Cache schema identity is checked before a changed row shape is decoded, so
  stale schemas return the typed unsupported-schema error.

## [0.24.x] - 2026-08-01 - Bounded SNS governance metrics

Detailed release notes: [docs/changelog/0.24.md](docs/changelog/0.24.md)

- `0.24.0` adds `sns metrics` for one SNS resolved by list id or Root
  principal. It uses the official Governance `get_metrics` composite query and
  preserves raw treasury ledger/accounts, current and original e8s amounts,
  per-treasury cached timestamps, voting-power metrics, proposal-window counts,
  genesis time, and latest SNS-ledger block time.
- The proposal-count window defaults to 30 days and is capped at 365 days.
  Including targeted discovery, the command makes three client requests; the
  Governance composite query performs its own bounded latest-block lookup.
  `ic-query` does not enumerate transactions, fan out, create a cache, or claim
  that differently timestamped metrics form one point-in-time snapshot.

```bash
icq sns metrics 1
icq sns metrics 23ten-uaaaa-aaaaq-aabia-cai --window 90d --json
```

## [0.23.x] - 2026-08-01 - Bounded SNS completeness

Detailed release notes: [docs/changelog/0.23.md](docs/changelog/0.23.md)

- `0.23.0` adds `sns swap` for one SNS resolved by list id or Root principal.
  It makes exactly three bounded native queries, retains component failures as
  typed gaps, and never calls `get_state`, enumerates participants, or creates
  a cache.
- Adds `sns upgrade` using Governance `get_running_sns_version` and SNS-W
  `get_next_sns_version`. It preserves all six Wasm hashes and pending-upgrade
  state, distinguishes no blessed successor from a query failure, and makes at
  most four live calls including targeted discovery.
- Direct SNS lookup now enriches only the selected SNS; unknown lookup requests
  no metadata, while `sns list` enriches the complete inventory. Sequential
  swap and upgrade evidence explicitly has no point-in-time guarantee.
- Hardens custom-source evidence and gives swap and upgrade reports focused
  public source traits, DTOs, builders, and text renderers. Metadata text must
  be trimmed and non-empty when present, and payload fields cannot coexist with
  a metadata error.
- As a pre-1.0 Rust API hard cut, replaces `SnsListSource` and
  `MainnetSnsList` with `SnsDiscoverySource`, `MainnetSnsInventory`,
  `MainnetSnsCanisters`, and `MainnetSnsMetadata`. No compatibility alias or
  fallback discovery flow remains.

```bash
icq sns swap 1
icq sns swap 23ten-uaaaa-aaaaq-aabia-cai --json
icq sns upgrade 1
icq sns upgrade 23ten-uaaaa-aaaaq-aabia-cai --json
```

## [0.22.x] - 2026-08-01 - Structural consolidation

Detailed release notes: [docs/changelog/0.22.md](docs/changelog/0.22.md)

- `0.22.7` replaces the mixed Dashboard model with explicit request/query,
  serialized report/row, host source-data, and host error owners behind one
  facade. Existing `ic_query::ic::*` paths, feature availability, constructors,
  fields, derives, report JSON, typed errors, and source contracts are
  unchanged. This completes the planned 0.22 cohesive-module boundary work.

- `0.22.6` separates Dashboard source traits, request normalization, untrusted
  source validation, and report projection into canister, metric, and
  network-resource owners behind one shared provenance facade. Existing public
  trait paths, typed errors, validation and canonical ordering, report fields,
  and custom-source behavior are unchanged.

- `0.22.5` separates the official Dashboard live adapter into canister,
  metric, and network-resource owners behind one shared HTTP transport facade.
  URL construction, wire decoding, and their focused tests now live with the
  capability that owns them. Existing public source traits and builders,
  endpoints, validation order, report output, request bounds, and network-call
  counts are unchanged.

- `0.22.4` separates the former monolithic ICRC text renderer into account,
  ledger-history, and ledger metadata/evidence owners behind one explicit
  internal facade. The facade retains only shared table-section and alignment
  mechanics. Existing public renderer paths, text output, report JSON, CLI
  behavior, feature availability, cache behavior, and network calls are
  unchanged.

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
  `subnet-catalog-host` feature without enabling `ic-query`'s direct optional
  Dashboard Reqwest or CBOR dependencies. Those packages may remain transitive
  through `ic-agent`. The full `host` feature remains a superset. Registry node,
  provider, operator, and data-center reports now share one cache-missing
  refresh driver and one exact-or-unique-prefix resolver, plus common
  network/source-request/fetch/write orchestration. NNS Governance proposal and
  neuron snapshots also share one attempt-sidecar construction, validation,
  status, and failed-progress owner.
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
