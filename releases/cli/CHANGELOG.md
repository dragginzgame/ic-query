# CLI and text presentation changelog

Covers `ic-query-cli` (`icq`) and `ic-query-text`. The CLI's existing finalized
baseline is 0.85.0; earlier coordinated releases remain in [shared history](../history.md).
The new text package starts at the same group baseline without changing existing
package versions or claiming an earlier publication.

Baseline version: `0.85.0`.

## [0.95.0]

### ⚠️ Breaking

- Align rendering and certificate integration with Query’s Host Tools 0.85
  boundary. CLI commands and output remain unchanged. [Details](0.95.md).

### ⬆️ Dependencies

- Adopt Testkit 0.92.0 and Wire 0.17.10. Use one
  Platform 0.24 and Foundation 0.75 family throughout native qualification,
  removing the older transitive copies. [Details](0.95.md).

## [0.94.0] - 2026-11-07

### ⚠️ Breaking

- Align the shared dependency catalog and certificate integration with Platform
  0.24.0 for the next Query minor. Keep CLI commands and rendered output unchanged.
  [Details](0.94.md).

### ⬆️ Dependencies

- Adopt Build 0.78.0 and Host Artifacts, FS, HTTP and Process 0.75.0.
  Align Host Tools 0.84.0 with that Foundation family while retaining cache
  admission, bounded HTTP reads and Governance build/recovery behavior.
  [Details](0.94.md).

### 🩹 Fixed

- Keep frozen-batch fixture detail-note ownership aligned with its simulated
  release, including when Query has a pending minor. [Details](0.94.md).
- Adopt Shared Tooling 0.31.5. Isolate inherited release settings in the
  release-command checker and verify outer batch and captured inner dispatch
  without running a real release from the fixture. [Details](0.94.md).

## [0.93.4] - 2026-11-07

### 🧹 Cleanup

- Reuse common source/output options for ICRC commands while retaining their
  help and defaults. Share private cache fixtures and use Host FS temporary
  directories in text integration tests. [Details](0.93.md).
- Normalize decimal transaction cursors directly while retaining arbitrary-size
  IDs and Clap validation. Group feature-gated public-API test imports without
  changing coverage. [Details](0.93.md).
- Consolidate SNS command definitions and simplify SNS/ICRC cache-status
  rendering, preserving command help, field order and terminal escaping.
  [Details](0.93.md).

### 🩹 Fixed

- Renew the existing Query-only RUSTSEC-2026-0387 acceptance through 20 November
  UTC after maintainer review; retain refusal from 21 November and all other
  dependency checks. The affected dependency remains. [Details](0.93.md).
- Keep fixture-lifecycle checks independent of the wall clock while retaining
  explicit dependency-advisory expiry tests and the real release guard.
  [Details](0.93.md).
- Adopt Shared Tooling 0.31.3, including the Bash 3.2 tool-setup fix and clear
  LOC diagnostics when cleanup removes active scratch inputs. Preserve checksum
  NUL rejection and backend failure status.
  [#115](https://github.com/dragginzgame/ic-query/issues/115). [Details](0.93.md).

### ⬆️ Dependencies

- Update the locked graph to Build 0.77.0, Platform 0.23.6, Wire 0.17.6 and Testkit 0.90.0
  without changing public APIs or schema-1 contracts. [Details](0.93.md).

## [0.93.3] - 2026-11-06

### 🔧 Tooling

- Adopt Shared Tooling 0.31.0 and frozen grouped delivery. Capture selected source
  once, preserve deferred normal-checkout edits, and retain per-group validation,
  supplier-first publication, registry receipts and explicit recovery.
  [Details](0.93.md).
- Use shared candidate setup and batch status/resume/reconcile/retire commands.
  Keep recovery offline, propagate saved publication context, and reject clearing
  the frozen roster for a selected group. [Details](0.93.md).
- Use the current `RELEASE_GROUPS` configuration and shared batch runner. Adopt
  the supplying contributor recovery-journal format and checksum verification
  fixes while keeping Query's public schemas at 1. [Details](0.93.md).
- Bind IC pins to the delivery checkout and prepare its selected IC tools before
  validation, retaining offline recovery admission. [Details](0.93.md).

```bash
make release-patch RELEASE_GROUPS=query,cli
make release-batch-status BATCH=batch-XXXXXXXX
IC_QUERY_RELEASE_BATCH_CASE=admission make release-batch-check
```

### ⬆️ Dependencies

- Align the locked workspace graph on Platform 0.23.4, Runtime 0.19.4,
  Wire 0.17.5 and Testkit 0.89.3, with compatible TOML support patches.
  Preserve published package APIs and Query's schema-1 contracts. [Details](0.93.md).

## [0.93.2] - 2026-11-06

### 🧪 Testing

- Use an immutable smoke-test command fixture to avoid transient executable-file
  launch failures while retaining deadline and process-evidence assertions.
  [Details](0.93.md).

### 🧹 Cleanup

- Reuse scalar report rendering, JSON display conversion, inventory dispatch and
  typed endpoint setup.
  Keep commands, help, validation, text/JSON output and cache behavior unchanged.
  [Details](0.93.md).
- Share topology count-table construction while preserving headers, row order,
  alignment and terminal escaping. [Details](0.93.md).
- Limit selected CLI/Text lint and archives to their own packages against real
  Query dependencies. Remove the duplicate native check after Clippy while
  preserving feature/Wasm and archive checks. Full CI and unqualified release
  hosts retain the complete gate. [Details](0.93.md).

## [0.93.1] - 2026-11-06

### 🧹 Cleanup

- Define report arguments through typed Clap options instead of paired builders
  and string-keyed extraction. Retain commands, validation and report behavior;
  keep Rust option documentation out of user-facing help. [Details](0.93.md).
- Reuse field rendering and typed collection contexts across Dashboard and NNS
  reports, preserving field names, order, formatting and escaping. [Details](0.93.md).

### ⬆️ Dependencies

- Adopt Build 0.75.0 for probe input resolution and release metadata tooling,
  preserving source fingerprints, input limits and metadata contracts. [Details](0.93.md).

## [0.93.0] - 2026-11-05

### 🧹 Tooling

- Adopt the shared Candid boundary check in CI and both release plans, with
  declaration-specific external protocol allowances. Refresh Shared Tooling to
  0.30.4 plus its committed generic-header fix. [Details](0.93.md).

### ⚠️ Breaking

- Rust renderer consumers adopt Query 0.93's dependency generation. CLI text
  and JSON are unchanged. [Details](0.93.md).

### ⬆️ Dependencies

- Align Host Tools 0.83.0 and Testkit 0.87.0 on one Platform 0.23 dependency.
  Migrate the compiled Governance fixture to the async client, with explicit
  instance deletion on success and failed assertions.

## [0.92.0] - 2026-11-05

### ⚠️ Breaking

- Rust text-renderer consumers adopt Query 0.92's model generation, with owned
  models separated from external Candid DTOs. CLI output is unchanged
  ([#114](https://github.com/dragginzgame/ic-query/issues/114), [migration](0.92.md)).

### ⬆️ Dependencies

- Select Platform 0.23.0 for Query and its native protocol fixtures. Native
  Host Tools/Testkit retain their required Platform 0.22 generation.
- Adopt Testkit 0.86.0 and Wire 0.17.2 for native Governance fixtures and the
  development probe/smoke harness. Keep one Wire generation; codec-only builds
  no longer pull in the optional schema derive. Report JSON is unchanged.

## [0.91.0] - 2026-11-05

### ⚠️ Breaking

- Reduce SNS support to explicitly targeted neuron ownership/stake reads. Remove
  discovery/catalog, proposal, parameter, metrics, swap, upgrade and Root health
  reports. ICRC balances, transaction/burn evidence and holder enumeration remain.
  Adopt Governance principals and refresh the new schema-1 neuron snapshots.

```bash
icq sns neuron list "$GOVERNANCE_CANISTER_ID" --owner "$USER_PRINCIPAL" --json
icq sns neuron refresh "$GOVERNANCE_CANISTER_ID"
```

- Refresh one shared Registry snapshot for NNS node/operator/data-centre views.
  Provider refresh reports separate Registry and Governance publications, and
  exports Governance metadata. Topology refresh has three components. JSON
  consumers adopt the replacement shapes; see the [0.91 notes](0.91.md).
- Remove the global `icq cache status` command and its Text renderer. Family cache
  inspection and strict cache/coordination admission remain available.
- Use `icq nns proposal cache status` instead of `cache list`. Status retains
  snapshot validation and refresh-attempt evidence; remove the redundant Text
  cache-list renderer. See the [0.91 notes](0.91.md) for JSON and Rust updates.

- Clip long text to terminal-column budgets through Host Text, using `…`.
  Standardize refresh-attempt text on `attempt_*` labels across NNS, SNS and
  ICRC. Raw JSON and cached evidence retain their existing contracts.

### 🧹 Cleanup

- Adopt Shared Tooling 0.30.0 with the latest compact LOC display, classify
  fixture/tooling source explicitly and
  place the Governance runner and probe with their source roles. Default local
  fixture scratch to `target/tmp/` and replace the blanket pass-by-value lint
  allowance with ownership fixes and scoped, reasoned expectations. Existing
  Cargo target names, report contracts and recovery evidence remain.

- Let Clap derive command paths and own shared typed canister/analytics options,
  principal parsing and SNS sort selection. Reduce duplicated text field blocks
  and refresh-attempt rendering while preserving report-specific policies.

The contributor timing utility now uses Clap and provides native help:

```bash
cargo run -p ic-query-cli --example subnet_catalog_timing -- --help
```

### 🛡️ Security

- Escape untrusted runtime-error and cache-progress fields while preserving
  typed error evidence, Clap usage layout and local refresh advice.

### 🧪 Testing

- Trim compile-only API assertions while retaining fixture-driven downstream,
  serialization, rendering and custom-source coverage.

- Run native and Wasm lint once per canister CI job before Testkit, local NNS
  smoke and bundle checks. Standalone developer commands retain lint gates.

## [0.90.0] - 2026-11-04

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
