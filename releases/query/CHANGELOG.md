# Query umbrella changelog

Release group: `query`. Package: `ic-query`.

## [0.104.14] - Unreleased

### 🔧 Tooling

- Update the packaged development dependency to Toko Tooling 0.46.8.

## [0.104.13] - 2026-11-25

### 🔧 Tooling

- Update the packaged development dependency to Toko Tooling 0.46.4.

## [0.104.12] - 2026-11-21

### 🔧 Tooling

- Update the packaged development dependency to Toko Tooling 0.46.0.

## [0.104.11] - 2026-11-19

### 🔧 Tooling

- Update the packaged development dependency to Toko Tooling 0.45.21.

## [0.104.10] - 2026-11-19

### 🔧 Tooling

- Update the packaged development dependency to Toko Tooling 0.45.18.

## [0.104.9] - 2026-11-19

### 🔧 Tooling

- Update the packaged development dependency to Toko Tooling 0.45.16.

## [0.104.8] - 2026-11-19

### 🔧 Tooling

- Select Toko Tooling 0.45.13 as the workspace development marker.

## [0.104.7] - 2026-11-18

### 🔧 Tooling

- Adopt Toko Tooling 0.45.11 and its Rust layout and dependency-catalog rules.
  Cargo may fetch missing locked dependencies; explicit offline selections remain
  effective, and candidate dependencies are prepared before proof sealing.
- Qualify the locked Platform 0.34.17, Protocol 0.3.6 and Types 0.21.3 graph
  while retaining compatible manifest floors.
- Retain empty defaults and direct specialist re-exports across the feature matrix.

### 🎯 Qualification

- Linux workspace checks pass with the selected graph; release wiring passes
  32 assertions. Native four-group validation executes products once and admits
  fresh reuse for the other three groups. See the [setup evidence](../../docs/development/setup.md).

## [0.104.6] - 2026-11-18

### 🔧 Tooling

- Adopt Toko Tooling 0.45.3 for umbrella release admission, retaining fresh input
  hashing while reducing repeated JSON processing and improving timing detail.
  [Tooling #394](https://github.com/dragginzgame/toko-tooling/issues/394).

## [0.104.5] - 2026-11-18

### 🔧 Tooling

- Retire the unused standalone provenance helper through the exporter; preserve
  existing publication admission. [Query #5](https://github.com/dragginzgame/ic-query/issues/5).

- Use the shared Rust preparation helper for CI, keeping toolchain selections in
  `rust-toolchain.toml` and preserving setup failure diagnostics.

## [0.104.4] - 2026-11-17

### 🔧 Tooling

- Adopt Toko Tooling 0.45.0 for the umbrella's candidate preparation and proof
  admission. Preserve feature qualification and original proof custody.
  [Tooling #390](https://github.com/dragginzgame/toko-tooling/issues/390),
  [Tooling #391](https://github.com/dragginzgame/toko-tooling/issues/391).

## [0.104.3] - 2026-11-17

### 🔧 Tooling

- Preserve the umbrella feature obligations in a single combined release-proof
  admission, removing repeated sealing of the same proof. Record admission
  phases and check prepared tools before installation.
  [Query #4](https://github.com/dragginzgame/ic-query/issues/4).

## [0.104.2] - 2026-11-17

### 🔧 Tooling

- Select the umbrella feature matrix independently for isolated umbrella changes; retain full validation for shared or unknown changes.
  [Query #3 — Select affected packages and consumers](https://github.com/dragginzgame/ic-query/issues/3).

## [0.104.1] - 2026-11-17

### 🔧 Changed

- Compose topology 0.110.1 and SNS 0.105.1, qualified with locked Platform
  0.34.13 and Protocol 0.3.5. Empty defaults, optional `topology`/`sns` features
  and direct re-export type identities remain unchanged.

## [0.104.0] - 2026-11-17

### ⚠️ Breaking

- Replace the legacy 0.103.1 API with a thin umbrella over `ic-query-topology`
  and `ic-query-sns`. Defaults are empty; opt in with `topology`, `sns`, or both.
  The exports are `ic_query::topology` and `ic_query::sns`, with exactly the
  specialist crates' types. Direct specialist dependencies remain supported.
- Retire the legacy modules and feature names (`host`, `nns-host`, `sns-host`,
  `icrc-host` and other former adapters); there are no compatibility aliases.
  Change dependency requirements, feature selections and imports explicitly.
  The umbrella adds no transport, policy, caching, CLI, governance operations or
  broader SNS implementation. Features select existing specialist capabilities.
- Existing specialist report schema 1 is unchanged and needs no reset. Legacy
  reports/caches are not accepted as current reports or silently converted:
  retain required historical evidence and explicitly retire old cache workflows
  when migrating. This release does not authorize deleting retained data.

After publication, consumers can select either or both capabilities:

```toml
[dependencies]
ic-query = { version = "0.104.0", features = ["topology", "sns"] }
```

```rust
use ic_query::{sns, topology};
```

### 🧪 Testing

- Qualify empty defaults, each feature independently and both together; compile
  examples prove the re-exports preserve the specialist report type identities.
- Inspect the predecessor at
  [5b4951b23c2adfcd3c615961f131e766c4b50283](https://github.com/dragginzgame/ic-query-legacy/tree/5b4951b23c2adfcd3c615961f131e766c4b50283):
  `crates/ic-query/src/lib.rs`, its manifest and `tests/downstream_usage.rs`.
  Retain the consumer-facing invariant that libraries do not require CLI
  presentation, and use direct re-exports to preserve current types/JSON.
  No legacy implementation is copied. Legacy default report modules and host
  feature breadth are intentionally replaced, not claimed as compatible.

## Legacy history through 0.103.1

Published legacy versions remain intact. Their original implementation and
[release history](https://github.com/dragginzgame/ic-query-legacy/tree/5b4951b23c2adfcd3c615961f131e766c4b50283/releases/query)
are read-only migration references, not the current API contract.
