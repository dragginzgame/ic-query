# Query umbrella changelog

Release group: `query`. Package: `ic-query`.

## [0.104.1] - Unreleased

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
