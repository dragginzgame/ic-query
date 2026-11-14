# Topology changelog

Release group: `topology`. Package: `ic-topology`.

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
