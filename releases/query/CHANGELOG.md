# Query changelog

Covers `ic-query` and unpublished `ic-query-testkit`. The existing finalized
baseline is 0.85.0; earlier coordinated releases remain in [shared history](../history.md).

Baseline version: `0.85.0`.

## [0.86.0]

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

### ⬆️ Dependencies

- Adopt Foundation 0.74.1, Host Tools 0.78, Metrics 0.27.0 and Testkit 0.80 with
  its matching PocketIC 0.18 client. Development probes and fixtures adopt Wire
  0.15.0, retaining existing protocol bytes and budgets.
- Adopt Shared Tooling 0.27.27 and its existing release-group support, preserving
  full repository CI and the native matrix.

Details and consumer actions: [0.86 notes](0.86.md).

Inspect the selected group version before release:

```bash
make version RELEASE_GROUP=query
```
