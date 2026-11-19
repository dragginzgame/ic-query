# SNS changelog

Release group: `sns-migration`. Package: `ic-query-sns` (formerly `ic-sns-migration`).
The delivery group and historical ledger path remain stable through this rename.

## [0.105.11] - Unreleased

### 🔧 Tooling

- Update the packaged development dependency to Toko Tooling 0.45.21 and apply
  its return spacing and read adapter layout.
- Select the Tokio runtime for tests through `test-util`.

## [0.105.10] - 2026-11-19

### 🔧 Tooling

- Update the packaged development dependency to Toko Tooling 0.45.18.

## [0.105.9] - 2026-11-19

### 🔧 Tooling

- Update the packaged development dependency to Toko Tooling 0.45.16.

## [0.105.8] - 2026-11-19

### 🔧 Tooling

- Select Toko Tooling 0.45.13 and keep the frozen ledger and governance fixtures
  beside the migration tests that replay them.

## [0.105.7] - 2026-11-18

### 🔧 Tooling

- Adopt Toko Tooling 0.45.11 and its Rust layout and dependency-catalog rules.
  Cargo may fetch missing locked dependencies; explicit offline selections remain
  effective, and candidate dependencies are prepared before proof sealing.
- Qualify the locked Platform 0.34.17, Protocol 0.3.6 and Types 0.21.3 graph
  while retaining compatible manifest floors.
- Group imports and move test declarations to module tops; retain migration evidence semantics.

### 🎯 Qualification

- Linux workspace checks pass with the selected graph; release wiring passes
  32 assertions. Native four-group validation executes products once and admits
  fresh reuse for the other three groups. See the [setup evidence](../../docs/development/setup.md).

## [0.105.6] - 2026-11-18

### 🔧 Tooling

- Adopt Toko Tooling 0.45.3 for migration release admission, retaining fresh input
  hashing while reducing repeated JSON processing and improving timing detail.
  [Tooling #394](https://github.com/dragginzgame/toko-tooling/issues/394).

## [0.105.5] - 2026-11-18

### 🔧 Tooling

- Retire the unused standalone provenance helper through the exporter; preserve
  existing publication admission. [Query #5](https://github.com/dragginzgame/ic-query/issues/5).

- Use the shared Rust preparation helper for CI, keeping toolchain selections in
  `rust-toolchain.toml` and preserving setup failure diagnostics.

## [0.105.4] - 2026-11-17

### 🔧 Tooling

- Adopt Toko Tooling 0.45.0 for migration candidate preparation and proof admission.
  Preserve fresh graph checks, original proof custody and migration qualification.
  [Tooling #390](https://github.com/dragginzgame/toko-tooling/issues/390),
  [Tooling #391](https://github.com/dragginzgame/toko-tooling/issues/391).

## [0.105.3] - 2026-11-17

### 🔧 Tooling

- Admit the combined product proof once per group while preserving SNS and
  reverse-consumer selection. Check prepared tools before installation and
  expose admission phase timings. [Query #4](https://github.com/dragginzgame/ic-query/issues/4).

## [0.105.2] - 2026-11-17

### 🔧 Tooling

- Select SNS validation together with its CLI and umbrella consumers; retain full validation for shared or unknown changes.
  [Query #3 — Select affected packages and consumers](https://github.com/dragginzgame/ic-query/issues/3).

- Preserve the locked Platform 0.34.14 patch selection; migration contracts are unchanged.

## [0.105.1] - 2026-11-17

### 🔧 Changed

- Qualify locked Platform 0.34.13 and Protocol 0.3.5 without changing compatible
  requirements, ICRC balance or SNS neuron evidence contracts. Query does not
  call the new update or management-cycle APIs.

## [0.105.0] - 2026-11-17

### ⚠️ Breaking

- Rename the package and directory to `ic-query-sns`; Rust imports become
  `ic_query_sns`. Consumers must change their manifest/imports and rebuild.
  No old-name alias or forwarding crate remains. Previously published packages
  retain their original identity; this first release under the new identity
  advances the minor. JSON schema 1, records and retained evidence need no reset.
- The name leaves room for future SNS capabilities. Current functionality
  remains read-only ICRC balance and selected SNS neuron migration evidence;
  expansion requires explicit contracts. ICRC balance reads still work outside SNS.
- Preserve and qualify the working-tree Platform 0.34.12 / Protocol 0.3.4 lock
  update without narrowing compatible requirements.

## [0.104.3] - 2026-11-17

### 🐛 Fixed

- Allow compatible Platform, Protocol and Types patches through caret requirements
  at 0.34.11, 0.3.2 and 0.21.2 instead of exact pins. Balance/neuron contracts
  are unchanged; upstream requirements still apply. Qualify Protocol 0.3.3 in
  the lockfile without raising its compatible manifest floor.

## [0.104.2] - 2026-11-17

### 🔧 Changed

- Adopt published Platform 0.34.11 and Protocol 0.3.2, retaining Types 0.21.2.
  Balance and neuron projections, bounds and partial outcomes are unchanged.
  The library does not call the added management APIs or enable the new fixture
  feature; ledger retains its supplier-owned management feature dependencies.

## [0.104.1] - 2026-11-17

### 🔧 Tooling

- Add the independent `ic-sns-migration` package, with publication enabled and
  independently selected native-read, ledger and SNS-neuron capabilities.
- Enable this release group's standard publication adapter. The first published
  API advances the unpublished 0.104.0 package baseline to 0.104.1.
- Select published Platform 0.34.10, Protocol 0.3.0 and Types 0.21.2 for the
  initial API. Protocol's changed management APIs are not used by these reports;
  balance and neuron evidence contracts remain unchanged.

### ➕ Added

- Implement exact ICRC account balance reports with independent metadata outcomes,
  and selected SNS neuron lookup/list reports. Accept explicit compatible ledger
  and Governance targets; Dragginz and Toko are examples, not restrictions.
- Use the bounded supplier DTOs/codecs from
  [Platform #21 — SNS neuron and ICRC metadata types](https://github.com/dragginzgame/ic-platform/issues/21).
  Preserve partial evidence, raw optional fields, unknown permission/error codes,
  source/method identity, local observation windows and query authentication.
- Bound listing to 100 neurons per page, 1–20 pages and 60 seconds overall.
  Explicit exclusive cursors support further bounded unfiltered continuations.
  Principal-filtered calls use one page because Governance ignores their cursor.
  Full pages do not establish exhaustion; failed later pages retain the prefix.

### 🧪 Testing

- Qualify migration policy using six frozen independent supplier frames and
  consumer regression tests for exact balances, partial outcomes, ID/cursor
  admission, limits, deadlines and cancellation. Linux live Dragginz balance,
  exact-neuron, two-page and principal-filtered reads succeeded. This does not
  establish a full export, atomic snapshot, Toko interoperability or macOS execution.

### 📚 Documentation

- Define the [migration evidence contract](../../docs/design/migration-evidence.md)
  and retained predecessor invariants. Keep this package independent of
  `ic-topology`; broad SNS governance support and migration execution remain excluded.
