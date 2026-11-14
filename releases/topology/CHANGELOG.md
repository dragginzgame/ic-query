# Topology changelog

Release group: `topology`. Package: `ic-topology`.

## [0.104.0] — pending

### ✨ Added

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
