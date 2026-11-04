# CLI and text presentation changelog

Covers `ic-query-cli` (`icq`) and `ic-query-text`. The CLI's existing finalized
baseline is 0.85.0; earlier coordinated releases remain in [shared history](../history.md).
The new text package starts at the same group baseline without changing existing
package versions or claiming an earlier publication.

Baseline version: `0.85.0`.

## [0.90.0]

### ⚠️ Breaking

- Adopt Query's Platform 0.22/Host Tools 0.82 public dependency identities.
  Rust embedders align Query/Text and direct certificate/error dependencies;
  command behavior and report/cache formats are unchanged.

- Use one options-aware renderer for each NNS inventory and Subnet Catalog list;
  retire the separate verbose renderer functions. Verbose output keeps full
  identities and report provenance without repeating metadata on every row.
- Keep concise proposal/neuron summaries and tables. Remove exhaustive per-row
  text dumps and proposal `--ballots`; use the existing `--json` output for raw
  details and ballots. `TextOptions` now contains only `verbose`.
- Adopt Query's shared canister payload and derived-count report changes; report
  schemas stay at 1 and JSON consumers must adopt the replacement shape.

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
  qualify Platform 0.22.0 with Runtime/macros/guard 0.19.1. CLI commands,
  output and the compiler floor are unchanged.
- Adopt Testkit 0.85's owned PocketIC client and explicit server endpoint
  contract; retain Query's managed startup and remove the separate client
  dependency. Prepare the matching CLI with `make install-testkit`; the
  PocketIC 16.1.0 server and Wire 0.16 selection remain.
- Use Shared Tooling 0.29.3's release plans and evidence checks to scope qualified
  CLI/Text-only releases to their product tests and real library integration.
  Query changes retain CLI consumers; full CI, native and archive obligations
  remain. Unknown scope or unqualified hosts keep the full gate. See
  [#112 — Query/CLI release plans](https://github.com/dragginzgame/ic-query/issues/112) and the [0.90 notes](0.90.md).

### 🧪 Testing

- Keep standalone Governance probe fixtures independent of enclosing Cargo
  workspaces, including when temporary directories are retained inside a checkout.

- Keep unit tests and shared fixtures in dedicated test files so source-size
  reports separate production code from tests.

## [0.89.0] - 2026-11-03

### ⚠️ Breaking

- Adopt the [Query 0.89 API and catalog contract cut](../query/0.89.md). Preserve
  historical archives and affected catalog evidence before updating consumers.

### 🧹 Cleanup

- Delegate integer quantity scaling, rounding and unit promotion to Host Text
  0.74.1. Retain Query's cycle/byte units and existing human-facing output.

- Consolidate binary help assertions and retire duplicate cached inventory tests
  while retaining command paths, renderer contracts and external source coverage.

### 🗑️ Removed

- Remove the SNS reward checkpoint/diff commands and renderers. Preserve existing
  files with their former reader; no current command migrates or deletes them.
- Adopt Query's remaining count/presentation-field removals in JSON and inventory
  caches. Human-facing values for retained commands are derived from raw evidence.

### 🔧 Changed

- Align Platform 0.19, Host Tools 0.79, Testkit 0.81, Wire 0.16 and the latest
  published Foundation/Build packages; adopt committed Shared Tooling 0.28.0.
  Keep Rust 1.99.1 and the existing release/native qualification scope.

Details: [0.89 notes](0.89.md).

## [0.88.0] - 2026-11-02

### ⚠️ Breaking

- Adopt Query's shared inventory errors and raw SNS reward-checkpoint JSON.
  Derive checkpoint text summaries while preserving the existing valid output.
- Adopt raw SNS permission codes and derived policy findings, report counts and
  ICRC snapshot bounds. JSON consumers use returned arrays for their counts;
  preserve affected retained files before adopting the new schema-1 contracts.
- Derive status/topology totals and Registry display metadata from Query's raw
  contracts. Preserve earlier authenticated archives and coordinate capture-root
  retirement before adopting the Registry hard cut.

### 🧹 Cleanup

- Define ICRC account arguments and request extraction through typed Clap records,
  retaining command names, defaults, bounds and collection policies.

Details and consumer actions: [0.88 notes](0.88.md).

## [0.87.0] - 2026-11-02

### ⚠️ Breaking

- Adopt Query 0.87's Rust and JSON contracts. Pass display options directly to
  Text renderers; update script consumers and preserve required evidence before
  refreshing changed SNS proposal and ICRC account-history snapshots.

### 🧹 Cleanup

- Preserve existing commands, flags and compact/expanded rendering while keeping
  display choices out of collected reports. Format typed paging progress in one
  CLI owner and omit invented metadata for invalid caches.

Details and consumer actions: [0.87 notes](0.87.md).

## [0.86.0] - 2026-11-02

### ⚠️ Breaking

- Import reusable report renderers from `ic-query-text`; the CLI consumes that
  package alongside Query's typed reports. Text presentation retains its existing
  output while JSON adopts the [Query 0.86 contract changes](../query/0.86.md).
- Release and publication commands require explicit `RELEASE_GROUP=cli` or
  `RELEASE_GROUP=query`. Each group owns its versions, notes and selected package
  batch within the existing workspace.

### 🧹 Cleanup

- Use library-owned SNS/ICRC choices and move report presentation to one owner.
  CLI arguments, process IO, progress and terminal policy remain in the CLI.
- Keep metadata fixtures independent of the groups' current versions and notes
  paths, so full CI continues to qualify each group after the other advances.

### ⬆️ Dependencies

- Adopt Build/Text 0.74, Foundation 0.74.1 and Platform 0.18 through the shared
  root catalog. Query owns report arithmetic without Metrics. Development fixtures adopt Wire
  0.15.0 with unchanged protocol bytes and budgets. Keep the full CI gate and use
  Shared Tooling 0.27.27 for independent release selection.
- Select Testkit 0.80.1 and Runtime/macros/guard 0.18.1 in the lockfile. Testkit
  removes its private Metrics dependency, leaving the workspace Metrics-free.

Details: [0.86 notes](0.86.md).

Inspect the selected group version before release:

```bash
make version RELEASE_GROUP=cli
```
