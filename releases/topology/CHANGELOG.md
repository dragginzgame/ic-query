# Topology changelog

Release group: `topology`. Package: `ic-topology`.

## [0.105.2] — GitHub delivery 2026-11-14

GitHub-only version; not published to crates.io.

### 🔧 Dependencies

- Retain the worktree's registry lock selection of Platform 0.33.4. Its Registry
  and Agent module sources match 0.33.3; the existing compatible manifest range
  remains valid. Platform 0.33.4, Types 0.20.4 and Toko Tooling 0.43.1 were verified
  as the latest stable, non-yanked first-party selections at delivery review.
  Platform's pending [#165](https://github.com/dragginzgame/ic-platform/issues/165)
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
  ([#160 — Add bounded Registry NodeRecord decoding](https://github.com/dragginzgame/ic-platform/issues/160)).
  Retain optional presence, bounded values and mutation evidence without
  inferring attestation, usable hardware, liveness or provider relations.
- Compare registered members with separately observed certified node keys.
  Retain both reports, bounded set differences and explicit non-atomicity;
  preserve configuration on a second-read failure without inferring node health.
- Add bounded Registry subnet configuration: registered members, raw configured
  replica version and subnet type, and halt flags. Preserve schema defaults and
  unknown values; distinguish configuration from running software or downtime.
- Use published Platform for bounded Registry subnet-record decoding
  ([#159 — Add bounded Registry SubnetRecord decoding](https://github.com/dragginzgame/ic-platform/issues/159)).
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
