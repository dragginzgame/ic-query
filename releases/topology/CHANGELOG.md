# Topology changelog

Release group: `topology`. Package: `ic-query-topology` (formerly `ic-topology`).

## [0.110.4] - Unreleased

### 🔧 Tooling

- Adopt Toko Tooling 0.45.0 for topology candidate preparation and proof admission.
  Preserve fresh graph checks, original proof custody and report qualification.
  [Tooling #390](https://github.com/dragginzgame/toko-tooling/issues/390),
  [Tooling #391](https://github.com/dragginzgame/toko-tooling/issues/391).

## [0.110.3] - 2026-11-17

### 🔧 Tooling

- Admit the selected combined product proof once per release group, retaining
  topology's reverse-consumer obligations and fresh source/archive checks.
  Check existing tools before preparation and record detailed admission timings.
  [Query #4](https://github.com/dragginzgame/ic-query/issues/4).

## [0.110.2] - 2026-11-17

### 🔧 Tooling

- Select topology validation together with its CLI and umbrella consumers; retain full validation for shared or unknown changes.
  [Query #3 — Select affected packages and consumers](https://github.com/dragginzgame/ic-query/issues/3).

- Preserve the locked Platform 0.34.14 patch selection; report contracts are unchanged.

## [0.110.1] - 2026-11-17

### 🔧 Changed

- Qualify locked Platform 0.34.13 and Protocol 0.3.5. Their new update and
  management-cycle APIs are not used by Query. Compatible requirements, public
  report contracts, JSON schema 1 and read-only behavior remain unchanged.

## [0.110.0] - 2026-11-17

### ⚠️ Breaking

- Rename the package and directory to `ic-query-topology`; Rust imports become
  `ic_query_topology`. Downstream manifests must select the new package and
  rebuild. There is no old-name alias or forwarding crate. Earlier `ic-topology`
  releases remain immutable historical packages. Report JSON and schema 1 are
  unchanged; retained evidence needs no reset. This new public Rust identity
  requires the minor release despite retaining report behavior.
- Preserve and qualify the working-tree Platform 0.34.12 / Protocol 0.3.4 lock
  update; compatible manifest floors remain unchanged.

## [0.109.2] - 2026-11-17

### 🐛 Fixed

- Replace exact Platform, Protocol and Types requirements with compatible caret
  ranges at 0.34.11, 0.3.2 and 0.21.2. Consumers can select later compatible
  patches without conflicting with Query's own locked supplier versions.
  Public report contracts and schemas remain unchanged. Previously
  published versions retain their original requirements; consumers need this
  patch release to receive the fix.
- Qualify the lockfile's Protocol 0.3.3 patch. Its capture-status fix is not used
  by Query; the compatible manifest requirement remains 0.3.2.

## [0.109.1] - 2026-11-17

### 🔧 Changed

- Adopt published Platform 0.34.11 and Protocol 0.3.2, retaining Types 0.21.2.
  Supplier module-layout changes preserve public paths; the added management
  capabilities are not selected here. Report contracts and schema 1 are unchanged.

## [0.109.0] - 2026-11-17

### ⚠️ Breaking

- Adopt published Platform 0.34.10 and Protocol 0.3.0 at the maintainer's request.
  Public cloud-engine fields and error payloads now carry Protocol 0.3 types:
  callers exchanging these values must align their Protocol dependency and
  rebuild. This Rust type-identity change requires topology 0.109.0. Types 0.21.2,
  report JSON, schema 1 and retained evidence remain unchanged; no data reset is
  required. Topology does not enable SNS or ledger features, and does not use
  Protocol's changed management request/status contracts.

## [0.108.0] - 2026-11-16

### ⚠️ Breaking

- Move all public report/request principals and the CLI to published Types 0.21.2.
  This is a Rust type-identity change: direct callers must align the Types source
  and rebuild for the next incompatible topology release. Principal bytes/text,
  JSON and schema-1 retained evidence are unchanged; no evidence reset is required.
  Remove the duplicate Types alias, cloud-engine principal bridges and certified
  progress text conversion. Retire the legacy Registry DTO/transport boundaries.

- Migrate available-node status/page decoding and bounded page-request encoding
  to published Protocol 0.2.6. Keep traversal, partial results and report JSON unchanged;
  use Protocol's cloud-engine classes directly. `AvailableError::Decode` and
  `Application` now retain Protocol payloads; request failures share `Encode`,
  retiring the temporary `PageEncode` variant (JSON category stays `encode`).
  Rust error consumers require migration at
  the next incompatible topology delivery; retained reports need no reset.

- Migrate marketplace price and network-fee response decoding to published Protocol
  0.2.6. Preserve daily amounts, class/principal JSON fields and partial
  outcomes; add explicit vector/skipping budgets. `PriceError::Decode` changes
  its Rust payload to Protocol's error type, requiring caller migration at the
  next incompatible topology release. Platform supplies native transport and
  Protocol supplies request encoding.

- Select published Platform 0.34.7, Protocol 0.2.6 and Types 0.21.2, removing
  legacy Platform 0.33.7, Agent, temporary aliases and sibling requirements from
  product and test graphs. Selected-engine reports use the supplier identity directly.
  Existing JSON and retained reports are unchanged. Public supplier
  error payload identities change and require consumers to align sources before
  the next delivery.

  Adopt the delivered 313-frame corpus from
  [Platform #10 — Portable Query fixtures](https://github.com/dragginzgame/ic-platform/issues/10).
  Tests replay independent data without a private fixture crate or direct Candid.
  `ReadError::Transport` now carries Platform's native error; Registry/placement
  errors and public cloud-engine classes use Protocol. Retire unreachable
  subnet/metrics identity-conversion errors. Add `ReadError::ClockRange` while
  preserving the JSON `clock` category. Canic must rebuild and update Rust matches;
  schema-1 evidence requires no reset.

  Native reads use the pinned mainnet root/subnet and explicit signing/delegation
  freshness (300 seconds past, 60 future), with no redirects, proxies or retries.
  Progress keeps separate times and its stricter final verification budget.

## [0.107.0] — GitHub delivery 2026-11-16

GitHub-only version; not published to crates.io.

### ⚠️ Breaking

- Advance Protocol 0.1.0 to immutable Git revision
  `8ee1e58ce6577054d58fba729546989e8b6fc041` with `registry-settings`.
  Selected-engine code is unchanged, but publicly exposed Protocol payload types
  now have this dependency identity; direct consumers must align their Git pin.
  Consumers exchanging Protocol types must use the new pin and rebuild. No
  retained Query report reset is required; schemas remain 1.

### ➕ Added

- Add `subnet::progress::fetch` using Git-pinned Platform 0.34.0 certificate-time
  projection from [Platform New #1 — Authenticated subnet certificate time](https://github.com/dragginzgame/ic-platform-new/issues/1).
  Retain both time sources, explicit freshness limits and partial outcomes without
  implying uptime or maintenance completion. Retire the temporary sibling path
  in favor of immutable revision `126479e64adfc13fa21aa217caa19a298bcdc19e`; legacy
  transport remains.
  Align thiserror to the supplier's 2.0.18 selection.


- Add `subnet::compare::fetch_with_settings`: retain the original comparison and
  two independent settings outcomes, continuing after component failures with
  a 200-second asynchronous bound and 776-call maximum. Existing `fetch`, public
  constructors and JSON remain unchanged. No inferred capacity or common version.

- Add `subnet::settings::fetch` for registered features and resource declarations,
  preserving optional presence, explicit zero/false and original Registry evidence.
  One bounded query; no effective-default, capacity or attestation inference.
  Adopts the decoder requested in [Platform #169 — Add bounded Registry subnet settings](https://github.com/dragginzgame/ic-platform/issues/169).

### 🧪 Tests

- Verify exact/exceeded Registry envelope byte limits and preserve source rejection
  even when inline settings are present. Retain nested shared-field budget tests.

### 📚 Documentation

- Document the authenticated state-time contract, immutable supplier
  selection and remaining transport migration. Dashboard status, generic endpoint
  status and canister-only version metadata do not become uptime or completion claims.

## [0.106.1] — GitHub delivery 2026-11-15

GitHub-only version; not published to crates.io.

### ➕ Added

- Add `placement::collection::fetch` for 1–32 unique, explicitly selected canister
  IDs. Preserve argument order, each lookup's source evidence and typed failures
  within a shared 60-second deadline. Distinguish cancelled from unstarted reads;
  complete collection does not establish fleet ownership or an atomic snapshot.

## [0.106.0] — GitHub delivery 2026-11-15

GitHub-only version; not published to crates.io.

### ⚠️ Breaking

- Switch selected cloud-engine request/response codecs to `ic-platform-protocol`
  0.1.0, pinned to Git revision `08c72219ad6f261067c00eece9c8c71852e6b561`
  in `dragginzgame/ic-platform-new`. `EngineError::Encode` and `EngineError::Decode`
  now carry that crate's error types; Rust consumers matching their payloads must
  update those matches. Other readers retain Platform 0.33.7. Report schema 1,
  selected-engine JSON fields and CLI syntax remain unchanged.

### 🔧 Changed

- Apply the replacement decoder's explicit 1,024-element vector ceiling,
  including skipped vectors, alongside the retained byte, header, type, work,
  allocation and skipped-work budgets. Query still admits at most one selected
  engine and 128 nodes per list. Fixture encoding remains with the published
  Platform codec; production decoding uses the pinned protocol implementation.

### ➕ Added

- Derive shared provider and data-center identities for two-subnet comparisons,
  retaining per-side node counts and independent unknown coverage. JSON adds
  `infrastructure_overlap`; a missing distribution remains distinct from an empty
  observed intersection. No new reads, capacity ranking or independence verdict.

### 📚 Documentation

- Document the diagnostic consumption boundary against the current local Canic
  checkout, including explicit network selection and partial evidence. Query
  placement observations do not replace Canic's operation-bound placement checks.

- Define the selected-subnet feature/resource-settings contract for Canic placement
  screening. Implementation awaits the published decoder requested in
  [Platform #169 — Add bounded Registry subnet feature and resource-limit projection for Query](https://github.com/dragginzgame/ic-platform/issues/169).
  Preserve optional settings and replica-default selections without inferring
  usable TEE protection or available capacity.

## [0.105.5] — GitHub delivery 2026-11-15

GitHub-only version; not published to crates.io.

### 🔧 Tooling

- Adopt authenticated Toko Tooling 0.44.3 through the exporter. Retain the focused
  post-push review baseline and restore compact LOC totals: default and `-d 0`
  show repository totals, `-d 1` expands crates and `-d 2` expands modules.
  Update the exact Cargo pin, lockfile and receipt; JSON LOC counts are unchanged.

### ➕ Added

- Add two-subnet placement comparison using existing distribution and metrics
  readers, retaining independent versions and partial outcomes. Move shared
  metrics acquisition outcomes to the metrics owner while retaining maintenance's
  existing type path and serialized shape.

- Implement a bounded selected-subnet maintenance report combining configuration
  changes, certified message-counter observations and an optional configured
  version expectation. Retain partial evidence and independent comparison availability.
  Reuse the configuration comparator and shared message-counter classification.

## [0.105.4] — GitHub delivery 2026-11-15

GitHub-only version; not published to crates.io.

### 📚 Documentation

- Verify the selected-engine topology contract against public `getEngineViews`;
  preserve omitted/hidden outcomes and separate record versus Registry-copy nodes.

- Verify the next cloud-engine pricing/availability source contract directly
  against the public canister: daily price units, all six node-class tags and
  bounded availability pages with copy freshness. Keep availability evidence
  separate from price declarations and usable-capacity claims.

### ➕ Added

- Add `cloud_engine::engine::fetch(EngineId)` using Platform 0.33.7. A validated
  opaque ID selects one public record with subnet/operator bindings, deletion
  evidence and both ordered node lists. Compare admitted sets without inferring
  freshness, liveness or capacity. Keep absent copied membership distinct from
  empty membership. One 20-second query, at most 128 nodes per list, no retry.

- Add `cloud_engine::candidates::fetch(page_limit)`: compose availability then
  prices with their existing 60/30-second budgets and preserve original evidence.
  Match exact class/provider/DC scopes, retaining every overlap through price-row
  indices. Pricing failure retains candidate observations with unknown coverage;
  fee-only failure still permits matching. No tariff selection or billing claim.

- Add `cloud_engine::available::fetch`: public new-engine eligibility in at most
  ten pages/1,000 nodes within 60 seconds. Keep independent copy status, per-page
  cursors/versions and raw chip-registration flags; stop on duplicate nodes,
  repeated cursors, version changes or failures while retaining admitted pages.
  Pagination exhaustion is distinct from an atomic inventory, live capacity or
  TEE attestation. Skip node IP text through the published bounded decoder.
  `fetch_with_page_limit` accepts 1–100 pages before IO, retains the selected
  limits and allows up to 10,000 nodes under the same 60-second deadline.

- Add `cloud_engine::prices::fetch` using published Platform 0.33.6. Retain exact
  daily net/gross amounts, all six raw classes, optional scopes, signed timestamps
  and separate query evidence. Bound prices to 1,000 unique validated identities
  and collection to 30 seconds; retain admitted prices if the fee read fails.
  Gross prices already include their fee. No monthly conversion or quote is inferred.

- Compare selected-node registration observations through `node::changes`, using
  supplied reports or two bounded reads. Preserve absent/empty/raw chip and HostOS
  values, detect operator changes, and retain both inputs on identity, source,
  field-limit or version refusal. This does not infer TEE readiness or node liveness.

- Join one subnet's nodes, operators, providers and registered data centers at
  one nonzero requested Registry version. Bound collection to 128 nodes, four
  concurrent reads and 60 seconds; retain record failures, unfinished reads,
  per-node relationships and independent provider/location unknown counts.
  Registry declarations do not establish verified geography, failure domains,
  capacity, uptime or TEE protection.

### 🔧 Dependencies

- Adopt Platform 0.33.7, adding the strict bounded selected-engine projection
  ([Platform #168](https://github.com/dragginzgame/ic-platform/issues/168)).
  Retain 0.33.6's `cloud-engine-types` capability for the four selected
  public methods ([Platform #166](https://github.com/dragginzgame/ic-platform/issues/166));
  use pricing and availability through the owning reports, with no direct protocol dependencies.

- Adopt published Platform 0.33.5 for bounded global-version, operator and
  data-center decoding ([Platform #165](https://github.com/dragginzgame/ic-platform/issues/165)).
  Reuse existing node/configuration projection policy; retain registry sources
  without sibling paths or direct protocol dependencies.

### 🔧 Tooling

- Refresh the workspace's authenticated Toko Tooling selection to 0.44.1 through
  its exporter. All 50 selected files remain accounted for; the baseline changes
  and the LOC reporter now makes omitted depth equivalent to explicit `-d 0`.
  Query has no imports of
  the retired Python evidence API. Tool/snapshot admission passes; optional
  release automation remains unselected.

## [0.105.3] — GitHub delivery 2026-11-14

GitHub-only version; not published to crates.io.

### 📚 Documentation

- Refine the pending provider/location distribution contract: reject zero as a
  Registry version pin, use the reviewed data-center key rules without trimming,
  and preserve independent provider/location coverage. Record Platform's local
  decoder completion separately from its pending publication; no new command
  or report is implemented.

## [0.105.2] — GitHub delivery 2026-11-14

GitHub-only version; not published to crates.io.

### 🔧 Dependencies

- Retain the worktree's registry lock selection of Platform 0.33.4. Its Registry
  and Agent module sources match 0.33.3; the existing compatible manifest range
  remains valid. Platform 0.33.4, Types 0.20.4 and Toko Tooling 0.43.1 were verified
  as the latest stable, non-yanked first-party selections at delivery review.
  Platform's pending [Platform #165](https://github.com/dragginzgame/ic-platform/issues/165)
  operator/data-center/version codecs remain a separate prerequisite.

### 📚 Documentation

- Specify the next selected-subnet provider/location distribution contract,
  including a common requested Registry version and explicit partial coverage.
  Implementation awaits published Platform version/operator/data-center decoders;
  this adds no query API or CLI command.

### ➕ Added

- Compare two selected-subnet configurations, supplied by the caller or acquired
  with a bounded wait. Retain both reports, membership differences and raw-field
  change flags; refuse regressing/conflicting versions and invalid coverage
  without inferring running software or maintenance completion.

- Collect registered node reports for one selected subnet, retaining configuration,
  per-node typed failures and observation timing. Bound membership to 128, read
  batches to four and overall collection to 60 seconds; distinguish completed,
  cancelled and unstarted reads without claiming an atomic snapshot or node health.
  See the [contract](../../docs/design/subnet-nodes.md).

## [0.105.1] — GitHub delivery 2026-11-14

GitHub-only version; not published to crates.io.

### ➕ Added

- Read a selected node's registered operator, optional raw chip ID and configured
  HostOS through published Platform 0.33.3
  ([Platform #160 — Add bounded Registry NodeRecord decoding](https://github.com/dragginzgame/ic-platform/issues/160)).
  Retain optional presence, bounded values and mutation evidence without
  inferring attestation, usable hardware, liveness or provider relations.
- Compare registered members with separately observed certified node keys.
  Retain both reports, bounded set differences and explicit non-atomicity;
  preserve configuration on a second-read failure without inferring node health.
- Add bounded Registry subnet configuration: registered members, raw configured
  replica version and subnet type, and halt flags. Preserve schema defaults and
  unknown values; distinguish configuration from running software or downtime.
- Use published Platform for bounded Registry subnet-record decoding
  ([Platform #159 — Add bounded Registry SubnetRecord decoding](https://github.com/dragginzgame/ic-platform/issues/159)).
  Remove the temporary Platform and Types sibling paths. Platform 0.33.3 requires
  Types 0.20.4; retain one shared registry identity and the existing Registry limits.
  Platform 0.33.3, Types 0.20.4 and Toko Tooling 0.43.1 were verified as the latest
  stable, non-yanked first-party selections at this delivery review.

## [0.105.0] — GitHub delivery 2026-11-14

GitHub-only version; not published to crates.io.

### ⚠️ Breaking

- Add `SubnetReport::observed_subnet_type` and `SubnetError::SubnetTypeLimit`.
  Update downstream struct literals and exhaustive error matches. JSON remains
  schema 1 with an additive field; no retained evidence reset is required.

### ➕ Added

- Add bounded two-sample subnet activity: retain both certified metrics reports
  and classify replicated-message totals as increased, unchanged or decreased.
  Wait 1–60 seconds between reads; preserve the first sample on a second-read
  error, with no retries, history service, uptime or throughput claims.
- Preserve subnet-certified application, verified-application, system and
  cloud-engine types, unknown strings and absent values in the existing read.
  Bound the type to 128 UTF-8 bytes without truncation. Classification establishes
  neither independent Registry agreement nor deployment permission or capacity.
- Adopt published Platform 0.33.1 for its Agent `SubnetType` export, retaining
  the existing transport and codec boundaries. Registry availability, non-yanked
  status, archive checksum and release provenance were verified on 2026-11-14.
  Platform 0.33.1, Types 0.20.3 and Toko Tooling 0.43.0 remain the latest stable,
  non-yanked first-party selections at delivery review.

## [0.104.1] — GitHub delivery 2026-11-14

GitHub-only version; not published to crates.io.

### ➕ Added

- Add certified single-subnet resource observations using the existing published
  Platform Agent. Retain raw counts, bytes and nominal cycles with source evidence;
  distinguish gauges and cumulative values without estimating capacity or uptime.
- Carry forward legacy precision and source-authority lessons with focused
  serialization and CLI qualification; keep network-wide REST history out of scope.

## [0.104.0] — GitHub delivery 2026-11-14

Delivered through `7695e00`; not published to crates.io.

### ✨ Added

- Add bounded Registry subnet inventory through Platform's published protobuf
  capability. Preserve all registered IDs and raw record mutation version/time;
  reject missing, empty, duplicate, invalid, chunked or excessive records.
- Add bounded selected-subnet node-key observations from authenticated mainnet
  state, with deterministic ordering, explicit incomplete-membership coverage
  and typed identity/count/empty-data refusals. Do not infer routing or capacity
  from self-reported subnet data.
- Add a bounded, read-only mainnet canister placement lookup using published
  Platform Agent and Registry DTOs. Return schema-1 source/time/verification
  evidence and typed failures; distinguish query signatures from certified results.
- Qualify reply projection, JSON precision, malformed/over-budget envelopes,
  source rejection, missing subnet, transport failure and deadline behavior offline.

### 🛠 Changed

- Share actual mainnet trust, response and deadline policy in one internal module.
  Unpublished `PlacementError::Read(ReadError)` replaces duplicated transport,
  deadline and clock variants; placement JSON remains unchanged.

### 🔧 Tooling

- Adopt published Platform 0.33.0 for the bounded Registry protobuf capability;
  external schemas and codecs remain with Platform.
- Establish the virtual `ic-query` workspace and `ic-topology` package shell with
  Rust 1.100.0, shared lints and a Cargo lockfile. Publication remains disabled.
- Adopt authenticated Toko Tooling 0.43.0, its standard Make interface, LOC
  reporter and `deps-list` / `deps-update` commands. Prepare locked dependencies
  explicitly; validate offline. Configure Linux and macOS scaffold checks.
- Declare the existing independent release groups and their owning ledgers so
  `make deps-list` displays group ownership. Keep root changelog navigation separate
  from group release notes and CLI examples; release automation remains deferred.

### 📚 Documentation

- Preserve the earlier unpublished `0.104.2` scaffold's design and tooling work.
  Focus topology on Canic placement, capacity, uptime and maintenance decisions;
  exclude NNS governance and Node Rewards. The first report is the selected
  canister placement lookup; full node/subnet inventory needs additional Platform DTOs.
- Keep SNS migration independent and record consumer-retirement obligations in
  the [restart design](../../docs/design/0.104-restart.md).
