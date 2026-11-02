# Query changelog

Covers `ic-query` and unpublished `ic-query-testkit`. The existing finalized
baseline is 0.85.0; earlier coordinated releases remain in [shared history](../history.md).

Baseline version: `0.85.0`.

## [0.88.0]

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
