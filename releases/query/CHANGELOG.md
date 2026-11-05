# Query changelog

Covers `ic-query` and unpublished `ic-query-testkit`. The existing finalized
baseline is 0.85.0; earlier coordinated releases remain in [shared history](../history.md).

Baseline version: `0.85.0`.

## [0.91.0]

### ⚠️ Breaking

- Reduce SNS support to explicitly targeted neuron ownership/stake reads. Remove
  discovery/catalog, proposal, parameter, metrics, swap, upgrade and Root health
  reports. ICRC balances, transaction/burn evidence and holder enumeration remain.
  Adopt Governance principals and refresh the new schema-1 neuron snapshots.

```bash
icq sns neuron list "$GOVERNANCE_CANISTER_ID" --owner "$USER_PRINCIPAL" --json
icq sns neuron refresh "$GOVERNANCE_CANISTER_ID"
```

- Collect NNS nodes, operators and data centres through one pinned Registry
  snapshot and cache. Custom adapters use `NnsRegistryInventorySource`; first
  reads collect the replacement schema-1 snapshot instead of older per-view files.
- Keep Governance provider metadata separate and derive its node counts from that
  shared Registry snapshot. Provider reports expose both acquisition records;
  provider and topology refresh outcomes identify the separate publications.
- Remove the global cache-inventory API. Family cache validation, confinement,
  refresh exclusion and local collection-status APIs remain available.
- Inspect the single complete NNS proposal snapshot through cache status. Remove
  the redundant cache-list report and builder; Rust consumers use
  `build_nns_proposal_cache_status_report` and its optional `cache` summary.
- Read detailed Subnet Catalog failure classifications from the retained source
  error instead of copied fields. Registry, node, operator, data-centre and
  topology reports retain their behavior. See the [0.91 notes](0.91.md).

### 🔧 Changed

- Adopt Shared Tooling 0.30.0 with the latest compact LOC display, classify
  fixture/tooling source explicitly and
  place the Governance runner and probe with their source roles. Default local
  fixture scratch to `target/tmp/` and replace the blanket pass-by-value lint
  allowance with ownership fixes and scoped, reasoned expectations. Existing
  Cargo target names, report contracts and recovery evidence remain.

- Simplify topology provider aggregation and share subnet-kind counting while
  retaining registration, capacity, location and unknown-reference evidence.
  Provider distributions and ordering retain their existing meaning.

### 🧹 Cleanup

- Implement native SNS source capabilities beside their IO owners, removing the
  central forwarding layer with the retained neuron scope described above.

### 🧪 Testing

- Run native and Wasm lint once per canister CI job before Testkit, local NNS
  smoke and bundle checks. Standalone developer commands retain lint gates.

## [0.90.0] - 2026-11-04

### ⚠️ Breaking

- Align public certificate/hash-tree and Host error identities on Platform 0.22
  and Host Tools 0.82. Rust consumers passing those values must align their direct
  dependencies. Certificates and hash trees share upstream identity with Agent;
  remove the former conversions. Types 0.18, Runtime 0.19 and schema-1 formats remain.

- Share Dashboard canister payloads through `IcCanisterData` and remove redundant
  `upgrade_count`, status returned-row counts and API-boundary-node `node_count`
  fields. Derive counts from retained rows; independent snapshot totals remain.
  Report schemas stay at 1; consumers must adopt the replacement shape.

- Make NNS proposal-activity and neuron-distribution reports output-only. Remove
  report read-back validators, their error types and analytics deserialization;
  retain source/collection validation, builders and unchanged schema-1 JSON output.
- Remove CloudEngine Type4 node-list `node_count`; derive it from `nodes`. Human
  output and the independent aggregate summaries remain.

- Remove 62 assignment-only request/query setters over public fields. Configure
  those fields after construction; retain validating setters, proposal sort
  direction resets and cache-route selection. Update Rust callers for this cut.
- Remove redundant CloudEngine price/provider counts, operator binding/domain
  summary fields, and topology region/referenced-provider counts. Derive them
  from retained values; independent source totals and human output remain.

### 🔧 Changed

- Align Foundation requirements with locked 0.74.3 and Build with 0.74.3;
  retain Runtime/macros/guard 0.19.1, existing report formats and the compiler floor.
- Adopt Testkit 0.85.1's owned PocketIC client and explicit server endpoint
  contract; retain Query's managed startup and remove the separate client
  dependency. Prepare the matching CLI with `make install-testkit`; the
  PocketIC 16.1.0 server and Wire 0.16 selection remain. All adapters now share
  Platform 0.22 without the older facade dependency.
- Use Shared Tooling 0.29.3's release plans and evidence checks to scope qualified
  CLI/Text-only releases to their product tests and real library integration.
  Query changes retain CLI consumers; full CI, native and archive obligations
  remain. Unknown scope or unqualified hosts keep the full gate. See
  [#112 — Query/CLI release plans](https://github.com/dragginzgame/ic-query/issues/112) and the [0.90 notes](0.90.md).

### 🧪 Testing

- Keep unit tests and shared fixtures in dedicated test files so source-size
  reports separate production code from tests.

## [0.89.0] - 2026-11-03

### ⚠️ Breaking

- Remove Registry delta collection, archive/replay and archive-backed certified
  Subnet Catalog APIs, including their dedicated feature. The certified latest
  Registry version query remains available through `nns-host`.
- Remove archive-only catalog assurance/provenance fields. Preserve historical
  archives and affected catalog caches before adopting the strict schema-1 shape;
  current readers do not migrate evidence.
- Keep live convenience builders and one custom-source/progress entry point for
  complete Governance/ICRC refreshes, Subnet Catalog reports and CloudEngine lists.
  Remove their intermediate source-only and progress-only forwarding variants.

- Retire SNS reward checkpoint/diff APIs. Preserve historical checkpoint files
  with their former reader; current Query does not migrate or delete them.
- Derive remaining returned-array counts and topology presentation from raw
  evidence. JSON/cache consumers must adopt the schema-1 replacement described
  in the detailed notes; independent totals remain.

### 🧹 Cleanup

- Retire archive-only transport, storage helpers and fixtures, and duplicate cached
  inventory scenarios already covered by their owning tests.

### 🔧 Changed

- Align Platform 0.19, Host Tools 0.79, Testkit 0.81, Wire 0.16 and the latest
  published Foundation/Build packages; adopt committed Shared Tooling 0.28.0.
  Keep Rust 1.99.1 and the existing release/native qualification scope.

Details: [0.89 notes](0.89.md).

## [0.88.0] - 2026-11-02

### ⚠️ Breaking

- Replace the four Registry inventory error types with `NnsInventoryError` and
  typed `NnsInventoryCollection` context, including topology failures.
- Store raw SNS reward-checkpoint evidence once. Derive summaries, policy findings,
  counters and UTC labels; preserve historical files and coordinate consumers
  before adopting the replaced schema-1 format.
- Derive exact SNS neuron policy findings and permission labels, returned-vector
  counts, NNS ballot counts and ICRC snapshot bounds. Permission JSON carries raw
  integer codes; preserve affected retained files before adopting the new shapes.
- Derive Dashboard snapshot totals and exact Subnet topology totals from rows.
  Derive Registry summaries, certification display metadata and mutation kinds
  from raw evidence; coordinate consumers and preserve earlier authenticated
  archives before switching capture roots.

### 🧹 Cleanup

- Decode Dashboard account and holder records directly into raw source DTOs.
  Pass Registry inventory rows/provenance directly to report construction.

Details and consumer actions: [0.88 notes](0.88.md).

## [0.87.0] - 2026-11-02

### ⚠️ Breaking

- Keep presentation options in Text, parse SNS selectors once, and explicitly
  admit live versus cached proposal views. Update Rust callers and JSON consumers
  for derived proposal facts, typed cache summaries, neuron provenance, ICRC
  account identities and structured progress.
- Preserve required historical evidence before explicitly refreshing SNS proposal
  and ICRC account-history snapshots with replaced schema-1 row shapes. Other
  snapshot formats, cache keys, collection budgets and refresh policies remain unchanged.

### 🧹 Cleanup

- Evaluate and fingerprint Subnet classification from the same typed policy data,
  preserving the current digest. Retain independent source/cache validation and
  original reply evidence.

Details and consumer actions: [0.87 notes](0.87.md).

## [0.86.0] - 2026-11-02

### ⚠️ Breaking

- Import text renderers from `ic-query-text` under their command-family namespace.
  Query retains typed reports, JSON contracts, collection, validation and caches.
- Adopt Platform 0.18, Types 0.17 and Runtime 0.18. Update consumers exchanging
  their public identities and align canister runtime users.
- Catalog loads consistently retain typed failure provenance. Use typed report
  origins, SNS lifecycle outcomes, grouped reward-diff observations and account
  history contexts; update Rust/JSON consumers together.
- Preserve and replace deployed-SNS catalogs, and finish or archive older portable
  NNS continuations before starting fresh. Their schema-1 shapes change; no
  migration, fallback reader or automatic deletion is added.

### 🧹 Cleanup

- Derive continuation status, consolidate option vocabularies and retain explicit
  source/cache admission and raw evidence. Independent release metadata keeps
  presentation-only releases from republishing Query. Release fixtures select
  their own groups explicitly under inherited runner arguments.
- Own percentage rounding and neuron-distribution counts and sums locally,
  preserving numeric results, optional coverage and overflow diagnostics. Remove
  Query's direct Metrics dependency; Testkit's private dependency remains upstream.

### ⬆️ Dependencies

- Adopt Foundation 0.74.1, Host Tools 0.78 and Testkit 0.80 with
  its matching PocketIC 0.18 client. Development probes and fixtures adopt Wire
  0.15.0, retaining existing protocol bytes and budgets.
- Adopt Shared Tooling 0.27.27 and its existing release-group support, preserving
  full repository CI and the native matrix.

Details and consumer actions: [0.86 notes](0.86.md).

Inspect the selected group version before release:

```bash
make version RELEASE_GROUP=query
```
