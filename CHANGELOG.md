# Changelog

All notable changes to `ic-query` will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/), and this
crate follows [Semantic Versioning](https://semver.org/).

## [0.75.14]

### 🧹 Cleanup

- Simplify Registry evidence handling and SNS report construction while preserving
  public APIs, validation and report formats.
- Consolidate CLI defaults, cache-selection policy and text formatting, retaining
  accepted options and existing help.

Detailed release notes: [docs/changelog/0.75.md](docs/changelog/0.75.md)

## [0.75.13] - 2026-10-29

### 🔧 Changed

- Adopt IC Metrics 0.17.2, preserving percentage calculations, report formats
  and the Rust 1.91 minimum. IC Platform remains at 0.7.6.

### 🧹 Cleanup

- Consolidate external protocol types, report provenance and CLI value parsing
  while preserving public APIs, report formats and accepted options.
- Simplify Registry relation processing and remove redundant ordering work,
  retaining validation and complete missing-relation diagnostics.

Detailed release notes: [docs/changelog/0.75.md](docs/changelog/0.75.md)

## [0.75.12] - 2026-10-29

### 🔧 Changed

- Adopt IC Metrics 0.17.1 and IC Platform 0.7.6, preserving report formats,
  numeric behavior and bounded Candid decoding policy.

### 🧹 Cleanup

- Consolidate CLI option parsing and report setup while preserving supported
  sort values, defaults, help text and diagnostics.
- Simplify SNS report construction and validation, inventory lookup and proposal
  provenance. Public APIs, report fields and stored formats remain unchanged.

Detailed release notes: [docs/changelog/0.75.md](docs/changelog/0.75.md)

## [0.75.11] - 2026-10-29

### 🧹 Cleanup

- Simplify CLI dispatch, help metadata and report construction while preserving
  supported commands, diagnostics, public APIs and report formats.
- Reduce duplicate index-result handling and inventory request conversion,
  retaining the separate ICP and generic ICRC wire protocols.

Detailed release notes: [docs/changelog/0.75.md](docs/changelog/0.75.md)

## [0.75.10] - 2026-10-29

### 🔧 Changed

- Require IC Platform 0.7.5, including its bounded external Candid nesting guard.
  Existing report formats and reply-arity policy remain unchanged.

### 🧹 Cleanup

- Simplify NNS and SNS command handling, report construction and snapshot
  publication while preserving public APIs, output and stored formats.
- Share ICRC account-history pagination with the existing collection lifecycle,
  retaining index identity, failure evidence, progress and page-limit diagnostics.

Detailed release notes: [docs/changelog/0.75.md](docs/changelog/0.75.md)

## [0.75.9] - 2026-10-29

### 🔧 Changed

- Adopt IC Metrics 0.17.0 with standard validated numeric transport and native
  Wire metadata. Existing report APIs and formats remain unchanged.

Detailed release notes: [docs/changelog/0.75.md](docs/changelog/0.75.md)

## [0.75.8] - 2026-10-29

### 🧹 Cleanup

- Consolidate topology relation checks and derive overall status from individual
  check results, preserving report fields and diagnostics.
- Simplify SNS and NNS command orchestration, cache provenance and Subnet
  resolution. Public APIs, Candid replies, command output and cache formats remain
  unchanged.

Detailed release notes: [docs/changelog/0.75.md](docs/changelog/0.75.md)

## [0.75.7] - 2026-10-28

### 🧹 Cleanup

- Consolidate NNS and SNS collection, cache-status and analytics flows while
  preserving pagination, typed errors and failure evidence.
- Simplify CLI handlers and metadata, principal resolution and cache inspection.
  Public APIs, command output and stored formats remain unchanged.

Detailed release notes: [docs/changelog/0.75.md](docs/changelog/0.75.md)

## [0.75.6] - 2026-10-28

### 🔧 Changed

- Adopt IC Platform 0.7.2, Host/Testkit 0.69.5 and Wire 0.7.19.
- Adopt IC Build 0.69.6 with the macOS compilation correction and remove its
  temporary compatibility pin. [Tooling #262](https://github.com/dragginzgame/ic-tooling/issues/262)
- Refresh the reviewed Shared Tooling 0.27.3 snapshot with clarified source
  and package self-containment guidance.

### 📚 Documentation

- Refresh the affected dependency graph and remediation ownership in the
  advisory documentation, retaining the **7 November 2026** review deadline.
  [Platform #50](https://github.com/dragginzgame/ic-platform/issues/50)
- Clarify that external Candid work quotas do not guarantee Wasm recursion-depth
  bounds. [Platform #73](https://github.com/dragginzgame/ic-platform/issues/73)

Detailed release notes: [docs/changelog/0.75.md](docs/changelog/0.75.md)

## [0.75.5] - 2026-10-28

### 🔧 Changed

- Adopt IC Tooling 0.69.4 across Host, Build and Testkit.
- Adopt Shared Tooling 0.27.3 with source-commit snapshot verification and
  restricted unattended maintenance. Requalify existing maintenance launcher
  configuration before resuming schedules.

Verify the snapshot against the reviewed source commit:

```bash
bash scripts/ci/verify-shared-tooling-snapshot.sh --consumer . \
  --source ../shared-tooling \
  --revision cc81a58a5b2dd287c577cbffd2c576c178b6785c
```

### 🧪 Testing

- Extend installed Governance probe coverage for oversized replies and bounded
  Candid decoding. [#108](https://github.com/dragginzgame/ic-query/issues/108)

Detailed release notes: [docs/changelog/0.75.md](docs/changelog/0.75.md)

## [0.75.4] - 2026-10-28

### 🩹 Fixed

- Keep cache paths containing non-UTF-8 components unrecognised instead of
  assigning another cache family's recovery or freshness policy.

### 🔧 Changed

- Adopt IC Tooling 0.69.3, Platform 0.7.1, Wire 0.7.18 and Metrics 0.16.3.
- Adopt Shared Tooling 0.27.2 with clarified native setup ordering and shared
  dependency issue ownership.

### 🧹 Cleanup

- Consolidate inventory and SNS refresh flows, reuse admitted NNS collections
  for analytics, and simplify request construction and topology projections.
  Public APIs and cache formats remain unchanged.

Detailed release notes: [docs/changelog/0.75.md](docs/changelog/0.75.md)

## [0.75.3] - 2026-10-27

### ➕ Added

- Add source-injectable NNS neuron and proposal builders that prefer complete
  cached reports and query live sources when the requested report is absent.

### 🧹 Cleanup

- Reuse admitted node-status snapshots for internal projection and consolidate
  request conversion, Registry decoding, principal checks and sort ordering.
- Simplify NNS analytics bookkeeping while preserving report fields, cache
  formats, diagnostics and CLI behavior.

Detailed release notes: [docs/changelog/0.75.md](docs/changelog/0.75.md)

## [0.75.2] - 2026-10-27

### 🩹 Fixed

- Respect native filesystem support for non-UTF-8 cache names in the SNS cache
  fixture, retaining typed macOS refusal errors. [#107](https://github.com/dragginzgame/ic-query/issues/107)

### 🔧 Changed

- Adopt IC Tooling 0.69.2 across Host, Build and Testkit, including the prepared
  Cargo metadata editor and Testkit server tools.
- Adopt Shared Tooling 0.27.0 to restore formatting-result reuse for ordinary
  conditional Rust attributes, while retaining full checks for external modules.

### 🧹 Cleanup

- Consolidate cache identity, SNS reward checks and request construction, and
  topology inventory plumbing while preserving public APIs and stored formats.

Detailed release notes: [docs/changelog/0.75.md](docs/changelog/0.75.md)

## [0.75.1] - 2026-10-27

### 🔧 Changed

- Adopt Shared Tooling 0.26.24 with visible formatting passes and hooks that
  preserve opaque submodules, retaining prepared-tool checks and failure logs.

### 🩹 Fixed

- Align CI formatting fixtures with per-pass output and timed failure diagnostics.

### 🧹 Cleanup

- Reduce repeated work in certified catalog recovery, catalog sealing and SNS
  reward reconciliation while preserving validation and retained evidence.
- Consolidate Registry limits, permission checks and ICRC internals without
  changing commands, report fields or cache formats.
- Simplify Registry acquisition, SNS conversion and topology calculations while
  preserving source evidence, report ordering and diagnostics.

Detailed release notes: [docs/changelog/0.75.md](docs/changelog/0.75.md)

## [0.75.0] - 2026-10-27

### ⚠️ Breaking

- Adopt IC Tooling 0.69.0. Certificate parse errors and authenticated ledger-tip
  results now use Host Tools 0.69 Rust types; consumers naming those types must
  update their direct `ic-host-tools` dependency to 0.69.

### 🔧 Changed

- Align Host, Build and Testkit with Platform 0.7.0, removing the older Platform
  selection. Retain external protocol encodings, report fields and cache formats.
- Adopt Shared Tooling 0.26.22 with explicit external Candid boundaries and
  strengthened error, source-inspection and subprocess guidance.

### 🧹 Cleanup

- Simplify archive validation, cache decisions and report construction while
  preserving rejection diagnostics, failure evidence and text output.

Detailed release notes: [docs/changelog/0.75.md](docs/changelog/0.75.md)

## [0.74.5] - 2026-10-27

### 🩹 Fixed

- Use fixture-owned executable tools for advisory-review checks and retain
  per-date refusal diagnostics and failed preparation evidence. [#105](https://github.com/dragginzgame/ic-query/issues/105)
- Archive failed CI fixtures before upload so unusual filenames cannot prevent
  retention of validation logs and original evidence. [#105](https://github.com/dragginzgame/ic-query/issues/105)

### 🧹 Cleanup

- Derive catalog-load and development smoke-command error chaining while
  preserving original causes, retained evidence and display behavior. [#106](https://github.com/dragginzgame/ic-query/issues/106)

Detailed release notes: [docs/changelog/0.74.md](docs/changelog/0.74.md)

## [0.74.4] - 2026-10-27

### 🧹 Cleanup

- Consolidate proposal CLI value mappings, Governance classification codecs and
  report assembly while preserving command behavior and public report fields.
- Simplify Registry evidence coverage and SNS enrichment validation, retaining
  endpoint-specific evidence, validation order and diagnostics.
- Share NNS and Dashboard text preambles, borrow node evidence during rendering
  and narrow ICRC block-query arguments without changing output or wire contracts.

Detailed release notes: [docs/changelog/0.74.md](docs/changelog/0.74.md)

## [0.74.3] - 2026-10-26

### 🧹 Cleanup

- Consolidate cache inspection and Registry decoding while preserving read
  budgets, recovery policies and failure evidence.
- Simplify Governance collection validation, proposal sorting and SNS canister
  matching without changing public reports, ordering or diagnostics.
- Share ICRC capability handling and first-page evidence; derive topology
  coverage directly from inventories while preserving source-call order.

Detailed release notes: [docs/changelog/0.74.md](docs/changelog/0.74.md)

## [0.74.2] - 2026-10-26

### 🔧 Changed

- Adopt Shared Tooling 0.26.21 with task-specific engineering guidance and
  emoji sections for current release notes.

### 🧹 Cleanup

- Share live and cached SNS list projection and common NNS command options,
  preserving report fields, source provenance and command behavior.
- Reduce repeated routing-range scans and temporary SNS/node-status copies;
  retain Unicode name ordering, range pagination and cache validation.

Detailed release notes: [docs/changelog/0.74.md](docs/changelog/0.74.md)

## [0.74.1] - 2026-10-26

- Consolidate Candid error projection, source provenance checks and Registry
  relation counts while preserving public errors, report shapes and validation.
- Centralize topology coverage labels and SNS collection storage identity;
  reuse common CLI source/output options without changing help or parsing.
- Simplify public-API signature fixtures and remove a redundant routing-order
  helper, preserving supported source lifetimes and cache contracts.

Detailed release notes: [docs/changelog/0.74.md](docs/changelog/0.74.md)

## [0.74.0] - 2026-10-26

### 📝 Summary

- **Breaking:** NNS proposal rows retain raw timestamps only; text output derives
  UTC labels. Preserve required old snapshots and explicitly refresh the proposal
  cache after updating consumers; schema identifiers remain 1.
- **Breaking:** NNS proposal cache listing now surfaces invalid refresh-attempt
  errors and retains attempts without a complete snapshot, matching cache status.
- Simplify route resolution, report provenance and analytics validation without
  changing authority evidence or classification policy.
- Avoid unnecessary copies during proposal conversion, cached neuron selection
  and CLI help preparation; consolidate SNS public-signature test declarations.

Detailed release notes: [docs/changelog/0.74.md](docs/changelog/0.74.md)

## [0.73.5] - 2026-10-26

- Include the private Testkit workspace member when preparing release lockfiles,
  keeping prepared-metadata checks consistent with Cargo.
- Adopt Platform 0.6.18 and Wire 0.7.16 as the minimum dependency versions.

Detailed release notes: [docs/changelog/0.73.md](docs/changelog/0.73.md)

## [0.73.4] - 2026-10-26

- Adopt Shared Tooling 0.26.19 so archived Draft labels no longer block a new
  release; current candidate conflicts remain errors.
- Align dependency minimums with Host/Build/Testkit 0.68.6, Platform 0.6.17 and
  Wire 0.7.15, preserving public type identities and protocol bytes.
- Avoid unused-import warnings in standalone Subnet Catalog host builds.

Detailed release notes: [docs/changelog/0.73.md](docs/changelog/0.73.md)

## [0.73.3] - 2026-10-26

- Share ICRC read-through refresh orchestration, SNS enrichment scheduling and target
  validation, refresh CLI construction and Subnet range rendering.
- Adopt Shared Tooling 0.26.18 with Host/Build/Testkit 0.68.5, Platform 0.6.16,
  Metrics 0.16.2 and Wire 0.7.12.
- Consolidate NNS proposal activity and neuron-distribution table rendering while
  preserving headings, empty sections, alignment and token amounts.
- Start normal releases from current committed metadata with fresh validation;
  historical plans no longer block them. Use `release-resume` for explicit recovery.

Detailed release notes: [docs/changelog/0.73.md](docs/changelog/0.73.md)

## [0.73.2] - 2026-10-25

- Reuse admitted snapshot handles in SNS, NNS neuron/proposal and ICRC cache
  inspection, avoiding repeated opens while preserving missing and invalid-cache reporting.
- Remove redundant Registry and SNS proposal forwarding layers without changing
  public entry points.
- Consolidate topology refresh orchestration, proposal sorting, Dashboard time-window
  requests, Registry text provenance and snapshot sidecar paths.
- Reuse Dashboard daily-stat rows and shared hexadecimal validation while preserving
  wire fields, report JSON and text output.

Detailed release notes: [docs/changelog/0.73.md](docs/changelog/0.73.md)

## [0.73.1] - 2026-10-25

- Inspect refresh coordination-file size and lock state through one file handle.
- Consolidate cache-attempt error/progress handling, Governance row bounds and
  Dashboard request validation while preserving family-specific behavior.
- Remove redundant private account-history cursors and replica rollout wire rows;
  simplify SNS cache request mapping and consume Host timestamps directly.

Detailed release notes: [docs/changelog/0.73.md](docs/changelog/0.73.md)

## [0.73.0] - 2026-10-25

- **Breaking:** SNS neuron/proposal cache listing now returns malformed or invalid
  refresh-attempt errors, matching direct cache status. Missing attempts remain
  valid absence; invalid snapshots remain visible as invalid summary rows.
- Share Governance snapshot consistency checks and identical Subnet catalog
  evidence-rendering blocks without changing report shapes or text ordering.

Detailed release notes: [docs/changelog/0.73.md](docs/changelog/0.73.md)

## [0.72.2] - 2026-10-25

- Share certified Registry archive collection and Dashboard observation checks,
  preserving authentication, resource limits and family-specific validation.
- Remove redundant Dashboard and ICRC conversion types; share SNS refresh request
  construction and let shared cache reports own their schema identifier.
- Simplify dependency-boundary checks without changing their effective rules.

Detailed release notes: [docs/changelog/0.72.md](docs/changelog/0.72.md)

## [0.72.1] - 2026-10-24

- Adopt IC Metrics 0.16.0 for topology percentage calculations. The arithmetic
  API, rounding, overflow handling and dependency-free runtime are unchanged.

Detailed release notes: [docs/changelog/0.72.md](docs/changelog/0.72.md)

## [0.72.0] - 2026-10-24

- **Breaking:** adopt Host, Build and Testkit 0.68.0 throughout. Consumers naming
  Query's exposed Host certificate errors or ledger-tip values must align to
  Host 0.68; evidence bytes and report shapes are unchanged.
- Select IC Wire 0.7.2 for the Governance probe and native qualification tools;
  retain string payloads, framing, explicit resource limits and original reply evidence.
- Adopt Shared Tooling 0.26.14 with distinct busy-tool admission and optional
  bounded read-only waits. Normal release retries can advance from clean,
  manually committed preparation despite missing earlier tags or delivery. Preserve
  the old plan as unconfirmed and validate the next release afresh.
- Preserve native filesystem paths when reading SNS refresh attempts, including
  cache roots containing non-UTF-8 bytes; reuse SNS root-identity extraction.
- Use one SNS cache-family contract for collection paths, snapshot data and schema;
  derive published row counts from that data, preserving stored JSON and report shapes.
- Bound cache-status content inspection to 1 GiB across the inventory while
  retaining prefix-only inspection of large history files.
- Track the Testkit manifest and feature-case inventory during CI input-drift checks.
- Require explicit review of the temporary RUSTSEC-2026-0387 acceptance from
  7 November 2026 UTC; the upstream IC Agent selection remains unchanged.

Detailed release notes: [docs/changelog/0.72.md](docs/changelog/0.72.md)

## [0.71.0] - 2026-10-24

- Temporarily accept RUSTSEC-2026-0387 to retain upstream IC Agent versions;
  the crypto-bigint vulnerability remains unresolved, including potential risk
  for caller-supplied signing identities. See [advisory review](docs/security-advisories.md).

- Adopt Shared Tooling 0.26.10 with shared external-IC dependency enforcement,
  literal formatter catalog parsing and improved offline tool diagnostics.
- **Breaking text output:** SNS metadata failures use the status `error` regardless
  of diagnostic wording. ICRC capability details retain the collected diagnostic
  instead of interpreting byte counts from prose. JSON evidence is unchanged.
- Share Governance request construction and page accounting across native and
  canister adapters, retaining family-specific cursors and transport errors.
- Use one SNS inventory assembly path with explicit health-query gaps; share SNS
  and ICRC token text rendering. Derive CLI value lists from the accepted enums.
- Isolate simulator qualification in the unpublished `ic-query-testkit` package,
  keeping PocketIC out of ordinary library test dependencies.
- Use one feature-case inventory for compilation and dependency-graph checks.
  Retain Query adapter coverage while relying on Shared Tooling for the generic
  release-recovery matrix and invoke completion assertions without source slicing.
- **Breaking:** select Host, Build and Testkit 0.67.0; consumers naming exposed
  Host types must align to Host 0.67. Retain Platform 0.6.13, Wire 0.6.12 and
  Metrics 0.15.8. Reprepare selected tools before offline admission.
- Bound Governance probe configuration discovery to 1 MiB per file, 8 MiB total,
  128 files and 16 include levels with Build's explicit configuration limits.
- Repair publication guard fixtures to provide a self-contained Cargo workspace
  for real manifest validation, avoiding missing-member failures during release CI.
- Refresh Shared Tooling to commit `4c0f198e` for Cargo setup phase/timing
  diagnostics on stderr; path-only stdout and offline checks are unchanged.

Detailed release notes: [docs/changelog/0.71.md](docs/changelog/0.71.md)

## [0.70.0] - 2026-10-23

- **Breaking:** adopt Host, Build and Testkit 0.66.0. Consumers naming Host
  types exposed by Query must select Host 0.66, including certificate decode
  errors and authenticated ledger-tip values.
- Use Testkit's borrowing accessor while retaining the simulator owner for
  the entire Governance fixture, preserving client/server lifetime ordering.
- Select Wire 0.6.7 for the probe and its callers, retaining native frame bytes
  and explicit admission budgets.
- Select Platform 0.6.10 and Metrics 0.15.7 and refresh Shared Tooling to 0.26.8,
  fixing setup-lock cleanup races while preserving runtime and report contracts.
- Run `make install-tools install-testkit` once after this snapshot update to
  admit the selected executables under its changed Cargo build policy.

Detailed release notes: [docs/changelog/0.70.md](docs/changelog/0.70.md)

## [0.69.0] - 2026-10-23

- **Breaking:** remove the implicit `ic-host-tools` Cargo feature. Select
  `certificate-host`, `ledger-evidence-host`, or the relevant reporting feature
  instead. Supported capabilities retain their existing roles and behavior.
- **Breaking:** adopt Host, Build and Testkit 0.65.0. Consumers naming Host
  types exposed by Query must select Host 0.65, including certificate decode
  errors and authenticated ledger-tip values.
- Select Platform 0.6.9, Wire 0.6.5 and Metrics 0.15.6, preserving external IC
  types, probe frames and dependency exclusions.
- Adopt Shared Tooling 0.26.7, including bounded Cargo-tool setup contention
  handling and corrected isolated validation companions. Tool pins are unchanged.
- Reprepare the selected executables with
  `make install-release-tools install-testkit` before offline release and
  installed-canister checks.

Detailed release notes: [docs/changelog/0.69.md](docs/changelog/0.69.md)

## [0.68.1] - 2026-10-22

- Adopt published IC Wire 0.6.0 for the Governance probe and its test tooling,
  retaining native Serde frames, method identity, resource limits and raw reply
  evidence. Production library and CLI dependencies remain unchanged.

Detailed release notes: [docs/changelog/0.68.md](docs/changelog/0.68.md)

## [0.68.0] - 2026-10-22

- **Breaking:** adopt the published Host, Build and Testkit 0.64 family.
  Consumers naming Query's public certificate decode errors or authenticated
  ledger-tip values must also select Host Tools 0.64. Preserve certificate
  limits, authentication, error categories and report/cache formats
  ([#100 — Align public authenticated LedgerTip with Host Tools 0.64](https://github.com/dragginzgame/ic-query/issues/100)).
- Import native PocketIC clients and types through Platform, retaining Testkit
  for simulator lifecycle, installation and reply fixtures. Keep simulator
  dependencies confined to native development builds.
- Reprepare the selected Build and Testkit executables with
  `make install-release-tools install-testkit` before offline release or
  installed-IC checks.
- Keep Query-specific setup guidance in the local host guide, preserving the
  canonical Shared Tooling setup document and its release integrity check.

Detailed release notes: [docs/changelog/0.68.md](docs/changelog/0.68.md)

## [0.67.6] - 2026-10-22

- Own private query errors while preserving original causes, diagnostics and
  consumer retry decisions. Exhaustive upstream mappings require review when
  Agent adds error variants
  ([#97 — Own the private query transport error contract](https://github.com/dragginzgame/ic-query/issues/97)).
- Fix Governance build, bundle and smoke admission after the Shared Tooling
  installer interface change by using Query’s selected Make tool check.
- Contain ordinary Agent query dispatch and structural error inspection behind a
  private boundary, preserving raw bytes, Registry retries and ledger capability
  decisions ([#95 — Contain Agent query dispatch and error inspection](https://github.com/dragginzgame/ic-query/issues/95)).
  Public APIs and cryptographic verification are unchanged.

Detailed release notes: [docs/changelog/0.67.md](docs/changelog/0.67.md)

## [0.67.5] - 2026-10-21

- Adopt Shared Tooling 0.26.4, explicitly retaining didc, ic-wasm, ICP CLI, Quill
  and wasm-opt. Preserve failed-run evidence and improve failure-log reporting.
- Follow the shared asynchronous cadence for ordinary authorized PR merges;
  preserve enforced checks and separate native/release qualification.
- Read the minimum Rust version from Cargo's workspace metadata in local and CI
  checks. Remove superseded host-setup instructions and obsolete adoption blockers.

Detailed release notes: [docs/changelog/0.67.md](docs/changelog/0.67.md)

## [0.67.4] - 2026-10-21

- Read the development compiler from rust-toolchain.toml in native and canister CI, retaining explicit Wasm targets and the independent minimum-Rust gate.

## [0.67.3] - 2026-10-21

- Adopt published IC Tooling 0.63.2 and its bounded HTTP reader for Dashboard
  responses. Preserve status precedence, byte limits and typed errors while
  making response-buffer allocation fallible
  ([#93 — Adopt ic-host-http for bounded native JSON response reads](https://github.com/dragginzgame/ic-query/issues/93)).

Detailed release notes: [docs/changelog/0.67.md](docs/changelog/0.67.md)

## [0.67.2] - 2026-10-21

- Extend Dashboard HTTP regression coverage for status precedence and empty/exact
  response budgets while preserving the current transport and typed errors
  ([#93 — Adopt ic-host-http for bounded native JSON response reads](https://github.com/dragginzgame/ic-query/issues/93)).
- Exercise the shared cache-lock facade in its regression tests, keeping
  dashboard-only test builds clean under strict Clippy.

Detailed release notes: [docs/changelog/0.67.md](docs/changelog/0.67.md)

## [0.67.1] - 2026-10-21

- Qualify the public authenticated ledger-tip return type against Host Tools
  directly under the isolated ledger-evidence feature, retaining byte-budget
  refusal before certificate processing or transport
  ([#92 — Align authenticated LedgerTip with published Host Tools 0.63](https://github.com/dragginzgame/ic-query/issues/92)).

Detailed release notes: [docs/changelog/0.67.md](docs/changelog/0.67.md)

## [0.67.0] - 2026-10-21

- **Breaking:** adopt coordinated IC Tooling 0.63.0. Consumers naming Host types
  exposed by Query must select Host Tools 0.63, including certificate decode
  errors and certified ledger-tip values. Report/cache formats remain unchanged.
- Prepare Build and Testkit executables with their required `cli` feature,
  keeping library consumers free of that optional CLI dependency. Re-run
  `make install-release-tools install-testkit` before their offline checks.
- Adopt published Platform 0.6.0, preserving IC wire types, decoder limits,
  native transport and canister runtime behavior.
- Consolidate complete-snapshot admission and paged refresh failure handling,
  preserving validation order, original errors, checkpoints and progress.
- Keep the Futures umbrella in development dependencies for test executors;
  production collection uses its existing `futures-util` implementation directly.
- Limit the library's direct Tokio dependency to Registry retry delays. Other
  host adapters retain their transport runtime without redundant feature edges;
  runtime construction and test macros remain with CLI and test consumers.
- Use Clap for Governance smoke commands and per-command help, sharing Query's
  alphabetical command ordering. Keep argument validation ahead of build and
  network effects, with successful help and unchanged failure exit status.

```bash
cargo run -p ic-query-cli --example governance_smoke --locked --offline -- help inspect-wasm
```

Detailed release notes: [docs/changelog/0.67.md](docs/changelog/0.67.md)

## [0.66.0] - 2026-10-20

- **Breaking:** select Host Artifacts, FS, Process, Text and Tools 0.32. Callers
  naming Host types exposed by Query must select the same minor, including
  `CertificateVerificationError::Decode`'s error and certified ledger-tip values.
  Certificate checks, report/cache formats and external IC wire bytes are unchanged.
- Align Host and Testkit 0.60 on published Platform 0.5.0, removing the older
  Platform package from the selected dependency graph.
- Select IC Build 0.24 for the smoke harness's matching Host 0.32 boundary and
  Wire 0.5.2 for probe fixtures, preserving cleanup policy and protocol bytes.

Detailed release notes: [docs/changelog/0.66.md](docs/changelog/0.66.md)

## [0.65.2] - 2026-10-20

- Adopt published Platform 0.5.0, preserving external IC types and call behavior.
  Infer Host's Registry Principal identity at its API boundary so its Platform
  selection can differ without changing report or cache formats.
- Adopt Shared Tooling 0.25.8. Run the existing Clippy gate before expensive
  CI/release checks and standard canister build, deployment and publication
  operations, stopping those recipes on lint failures.
- Consolidate cache preview, export and publication policy for Subnet Catalog
  and NNS inventories. Preserve export protection, byte ceilings, lock lifecycles
  and typed failures while reusing the confined path helpers.
- Retain Testkit setup stderr and partial downloads in CI failure artifacts,
  preserving file modes and archiving links without following them.
- Keep generic validation-runner regression matrices at Shared Tooling while
  retaining Query's Make admission and recursive-execution checks
  ([PR #90 — Consolidate tooling checks and retain Testkit setup evidence](https://github.com/dragginzgame/ic-query/pull/90)).

Detailed release notes: [docs/changelog/0.65.md](docs/changelog/0.65.md)

## [0.65.1] - 2026-10-20

- Consolidate SHA-256 hashing on Host Artifacts while preserving cache identities,
  canonical JSON digests and Registry provenance commitments.
- Decode Registry certificates through Host's bounded decoder, preserving
  authentication and raw evidence while rejecting malformed fields consistently.
  Direct CBOR use is now limited to fixtures.
- Simplify Governance smoke cancellation with Tokio and validate report timestamps
  through Host Text. Keep existing deadlines, signal cleanup and precise receipt
  timestamps.
- Select Testkit 0.59, sharing the current Host and Platform dependency families
  instead of retaining their older package copies.

Detailed release notes: [docs/changelog/0.65.md](docs/changelog/0.65.md)

## [0.65.0] - 2026-10-20

- Prepare pinned Cargo Audit and Cargo Machete during fresh release preflight,
  avoiding a later tools-check failure after shared build-policy changes.
  Standalone offline preflight checks prepared tools without installing them.
- Adopt published Platform 0.4.0 and Host 0.31.1. Convert Registry shard identities
  explicitly at Host's owned Principal boundary. Use owned Principals for runtime calls while
  preserving external Candid, Agent and ledger DTO identities and wire formats.
- Require IC Build 0.22.1 for Host 0.31's process boundary, preserving the smoke
  runner's TERM grace and bounded reaping policy.
- **Breaking:** consume duration helpers directly from Host Text and ledger value/
  chain evidence directly from Host Tools. Query's ledger-tip authentication now
  requires `ledger-evidence-host` and returns Host's `LedgerTip`. Update consumer
  imports; no compatibility reexports remain
  ([#71](https://github.com/dragginzgame/ic-query/issues/71),
  [#78](https://github.com/dragginzgame/ic-query/issues/78)).
- Use Host's canonical unsigned LEB128 decoder for certified Registry and API
  boundary-node state, preserving field context and Query's typed failures
  ([#63](https://github.com/dragginzgame/ic-query/issues/63)).

- Select Rust 1.99.1 for development and matching CI checks, retaining existing
  package MSRVs.

- Adopt Shared Tooling 0.25.4, including explicit Cargo-tool feature selections,
  actionable admission diagnostics and failure-evidence guidance. Re-run explicit
  tool setup after adoption; prior tool generations and failure evidence remain retained.
  Optional selected-tool checks can specify Cargo features; they require matching
  explicit setup and do not add tools to Query's default catalog:

```bash
bash scripts/dev/install-rust-tools.sh --consumer "$PWD" \
  --package ic-host-fs --version 0.31.1 --bin ic-host-file \
  --profile release --features cli --check
```

- Bind Governance probe compiler provenance to its captured build observation,
  avoiding a second compiler probe after compilation
  ([PR #84](https://github.com/dragginzgame/ic-query/pull/84)).
- Use Host's bounded HTTP fixtures for native Agent, Candid and ICRC tests
  ([#67](https://github.com/dragginzgame/ic-query/issues/67)).
- Consume Host's Registry protobuf projections and routing-shard checks, removing
  duplicate private implementations while preserving report and cache formats
  ([#66](https://github.com/dragginzgame/ic-query/issues/66)).

Detailed release notes: [docs/changelog/0.65.md](docs/changelog/0.65.md)

## [0.64.0] - 2026-10-19

- **Breaking:** use Host Artifacts, FS, Text and Tools 0.30. Certificate decode
  errors now carry Tools' 0.30 `ParseError`; callers naming that type must match it.
  Keep the existing decoding, authentication and cache behavior.
- Complete Host Process 0.30 adoption with published IC Build 0.22, preserving
  the smoke harness's cleanup policy.
  See [IC Build #9 — Qualify consumers against the exposed Host boundary](https://github.com/dragginzgame/ic-build/issues/9).
- **Breaking:** adopt Platform 0.2's maintained runtime and hard-cut the owned
  Governance probe and runner to Wire. Rebuild the probe with its matching runner;
  new receipts retain `.wire` evidence. External NNS Candid calls and report JSON
  remain unchanged.
- Build Governance reply fixtures through Testkit, removing the WAT parser tree.
  See [#81 — Adopt Testkit raw reply fixtures](https://github.com/dragginzgame/ic-query/issues/81).
- Keep the Futures executor test-only; production retains async joins and streams.
- Use Host's native agent construction and endpoint binding, retaining Query's
  endpoint admission, response limits and historical certificate policy.
  See [#82 — Adopt Host native agent construction and endpoint binding](https://github.com/dragginzgame/ic-query/issues/82).

Detailed release notes: [docs/changelog/0.64.md](docs/changelog/0.64.md)

## [0.63.0] - 2026-10-19

- Fix release-cache fixtures when the parent guard test supplies a metadata editor.
- **Breaking:** adopt Host Tools 0.29 certificate decoding. Remove the depth
  argument from `certificate::authenticate` and the `depth` field from
  `CertificateEvidenceLimits`; CBOR recursion now follows the IC Agent codec.
  Keep explicit byte allowances, authentication and witness checks.
- Use Host FS capability handles for managed caches, preserving permissions,
  byte/scan limits and typed errors while removing direct `cap-*` dependencies.
- Compile Governance fixture WAT through Host Artifacts with explicit limits,
  and delegate smoke-harness signal registration/restoration to Host Process.

- Remove the unused Ciborium workspace declaration; fixtures already use
  `serde_cbor`; production certificate decoding remains Host-owned.
- Adopt Shared Tooling 0.25.0 and its grouped formatting defaults, preserving
  dependency-role sections and the existing tool catalogs.
- Use published IC Build 0.21.0 for release editing instead of the retired Perl renderers.
  Release preflight prepares the editor; offline checks require explicit setup:

```bash
make install-release-tools
make release-tools-check
```

Detailed release notes: [docs/changelog/0.63.md](docs/changelog/0.63.md)

## [0.62.4] - 2026-10-19

- Check dependency graphs before feature builds and tests, and identify the
  failing profile when a graph check refuses the selected dependencies.

- Use `serde_cbor` for CBOR test fixtures and remove Query's direct `ciborium`
  dependency. Host retains production certificate decoding.

- Adopt published Host Tools 0.28.0 for certificate-only use, keeping Candid
  service parsing out of production host profiles.

- Run Governance runner fixtures once per complete CI gate, preserving the early
  default-feature compilation check.

### Pending integration

These PRs have passed their complete nine-job CI matrices and remain unmerged:

- [#80](https://github.com/dragginzgame/ic-query/pull/80): bounded ICP transfer
  decoding; callers retain authentication and expected-transfer checks.
- [#84](https://github.com/dragginzgame/ic-query/pull/84): use the admitted build's
  compiler identity in Governance probe provenance.
- [#86](https://github.com/dragginzgame/ic-query/pull/86): select Rust 1.99.1 for
  development and CI, retaining consumer MSRV 1.91.0.

Detailed release notes: [docs/changelog/0.62.md](docs/changelog/0.62.md)

## [0.62.3] - 2026-10-18

- Refresh the selected dependency graph to IC Build 0.17.6, Host 0.27.6 and
  IC Platform 0.1.13, preserving the existing dependency requirements.
- Follow the selected IC tool catalog in host setup guidance, removing stale
  ICP CLI and Binaryen version instructions.

Detailed release notes: [docs/changelog/0.62.md](docs/changelog/0.62.md)

## [0.62.2] - 2026-10-18

- Explicitly release refresh locks after errors and cancellation, even when a
  duplicated descriptor remains open. Fix the cache-status release test race.

- Reuse Host's canonical UTC, exact decimal rounding and protected export writers.
  Preserve report text, cache freshness rules and typed export errors while
  removing the competing local implementations.

- Reuse Shared Tooling’s committed graph checker (6eb8c41, PR #285) while
  retaining local roots, feature selections and exclusion policy. This selects
  unmerged topic source, not a new Shared release.
  Keep graph-only qualification independently runnable, preserving direct
  dependency depth and forbidden dependency-feature edges.

- Adopt published IC Platform 0.1.11 for Candid, certification and native Agent
  APIs, preserving wire types, authentication and caller policies. Remove direct
  Agent declarations and the remaining Agent hash-tree imports.
- Route Governance calls and the typed probe entrypoint through Platform's CDK
  facade and generator, removing the final direct Candid/CDK dependencies.
  Preserve bounded calls, reply limits, wire types and upstream runtime behavior.
- Use Testkit 0.54 and Platform's codec in compiled Governance tests, retiring
  the separate decoder dependency without changing reply limits or call behavior.
- Route standard ledger account, metadata, allowance, block/archive and certificate
  types through Platform, preserving upstream identities and wire formats.
  Keep stable-memory storage integration excluded.
- Enforce direct dependency ownership with Shared Tooling's boundary checker,
  including renamed and target-specific dependencies.

- Adopt Shared Tooling 0.24.0 and its two-tool common Rust setup.
- Label dependency roles and preserve their groups through formatting, hooks
  and source-bound release manifest checks.
- Reject testing/tooling packages in the CLI and Governance probe dependency
  graphs. Document the remaining deployment evidence-binding gap.

Detailed release notes: [docs/changelog/0.62.md](docs/changelog/0.62.md)

## [0.62.1] - 2026-10-18

- Fix internal descriptor-lock exports for focused library tests without the
  aggregate `host` feature.

- Remove the retired test-only certificate decoding path. CMC and ICRC
  tests exercise shared commitment verification and their own leaf policies;
  bounded decoding remains covered by the certificate module.

- Use Host's retaining fixture-directory owner in Governance harness tests.
  Leave failed-test evidence available for diagnosis.
- Keep generic tooling regression suites with Shared Tooling; retain Query's
  Make, hook, workflow, release-adapter and product integration checks.
- Clarify library/tooling ownership and document the adopted bounded Candid
  codec and Host HTTP fixtures. Align dependency examples with the current
  0.62 API and remove the obsolete synchronous-runtime bridge claim.

Detailed release notes: [docs/changelog/0.62.md](docs/changelog/0.62.md)

## [0.62.0] - 2026-10-17

- Align the compiled Governance fixture with Testkit 0.53’s explicit Candid
  argument-list helpers, preserving the selected dependency update.

- **Breaking:** Cache refreshes use persistent empty coordination files with
  Host descriptor locking. Remove lock-age arguments, constants and CLI options;
  cache status reports `active`, `available` or `invalid` from current exclusion.
  Before upgrading, stop all writers and explicitly retire incompatible nonempty
  lock files. Keep cache/attempt evidence; never unlink live coordination files.

- **Breaking:** ICRC balance and allowance reports compose their existing payloads
  in `value`; Rust callers use `report.value`. Text and flat JSON stay unchanged.
- **Breaking:** CloudEngine node-class JSON uses the native Candid spelling
  (`type4_1` through `type4_5`). Marketplace keys and text retain dotted labels.
- **Breaking:** CloudEngine list rows store `operator` as one lookup result:
  a native principal, successful absence, or a bounded error. Update Rust/JSON
  consumers; schema remains 1. Invalid custom-source evidence still fails the report.

- Adopt Shared Tooling 0.23.0 and its common ICP 1.7.0 catalog. Retain Query's
  IC-tool setup/checks and documentation-debt gate under their local owners.

- **Breaking:** SNS neuron, proposal and ballot rows retain native timestamps
  without stored UTC strings. Text derives those strings when rendered. Update
  Rust/JSON consumers, preserve evidence and explicitly refresh affected caches.
- Share Dashboard token-value response decoding with its existing source row,
  preserving nullable raw values and validated report timestamps.

- **Breaking:** SNS lifecycle and treasury records retain native numeric codes
  without stored labels or a parallel treasury enum. Update JSON/Rust consumers
  and explicitly refresh SNS catalogs after preserving required evidence.
- **Breaking:** SNS metrics and swap reports compose their existing source
  payloads in `metrics` and `swap`. Payloads are available without host features;
  report JSON remains flat apart from the removed derived labels.

- **Breaking:** SNS proposal rows and ballots retain native action IDs and vote
  codes without duplicated classifications. Labels remain available in text and
  filtering. Update Rust/JSON consumers and explicitly refresh proposal caches.
- **Breaking:** ICRC account-page reports compose `page: IcrcAccountTransactionPageData`;
  existing text and flat JSON fields remain unchanged.
- Preserve the existing flat SNS reward-checkpoint format for historical
  before/after reconciliation. Continue validating aggregates against raw evidence.

- **Breaking:** ICRC and SNS token reports compose their shared metadata as
  `token: IcrcTokenData`. Update Rust field access; text and flat report JSON
  preserve their existing fields and raw values.
- **Breaking:** Shared complete snapshots store explicit `metadata` and `data`
  objects, and refresh-attempt files store explicit `metadata`. Preserve any
  needed existing evidence, then explicitly refresh the affected caches. Schemas
  remain 1; readers reject unknown and duplicate fields without migration.

Detailed release notes: [docs/changelog/0.62.md](docs/changelog/0.62.md)

## [0.61.0] - 2026-10-17

- **Breaking:** SNS token reports and custom sources use
  `icrc::IcrcTokenStandardRow` and `icrc::IcrcTokenMetadataRow` directly. Remove
  the equivalent SNS row types and redundant ledger-to-report conversions.
  `MainnetSnsToken` composes its metadata as `token: IcrcTokenData`.
- **Breaking:** `IcrcBlockTypesSource` and `IcrcArchivesSource` return vectors
  of their existing row types directly; update custom source implementations.
- **Breaking:** `CmcCyclesReport` retains the complete certified XDR report in
  `xdr`; access provenance, rate and certification through that field. JSON
  remains flat with the existing fields.
- **Breaking:** NNS neuron and proposal rows, ballots and aggregate buckets
  retain raw codes without duplicated classification fields. Text labels are
  derived when rendered. Update Rust/JSON consumers and explicitly refresh
  neuron and proposal snapshots; strict row decoding rejects the replaced
  shapes. Schemas remain 1.
- **Breaking:** ICRC tip-certificate sources return the standard optional byte
  pair; reports derive their existing hex and length fields during assembly.
- **Breaking:** SNS joined, list and info models compose their five canister
  principals in `canisters: MainnetSnsCanisters`. JSON remains flat.
- Include the MIT license text in both published crate packages.
- Align feature-boundary checks with Host's unconditional Candid parser while
  keeping portable and canister builds free of that dependency.
- Keep CI input-drift fixtures bound to their own tool-pin catalog when run
  from a parent CI or release process.
- Use typed Cargo selections and fresh source-evidence validation in the
  Governance probe build harness.
- Align Governance probe builds with IC Build 0.15 and Host 0.26, resolving
  the shared process-cleanup policy type mismatch.
- Fix native host compilation with Host 0.25 while preserving confined,
  atomic cache publication and failure evidence. Update canister fixtures for
  Testkit 0.51’s explicit installation fields and caller principal.
- Select ICP CLI 1.7.0 for development and canister tooling. Run
  `make install-ic-tools` before offline tool checks.
- Share SNS version-hash validation while preserving upgrade hex admission and
  the stricter 32-byte reward-checkpoint requirement.

Detailed release notes: [docs/changelog/0.61.md](docs/changelog/0.61.md)

## [0.60.0] - 2026-10-17

- **Breaking:** NNS neuron/proposal cache status and summary reports share
  `NnsGovernanceCacheStatusReport`, `NnsGovernanceCacheSummary` and
  `nns_governance_cache_status_report_text` under `nns::governance`.
  Status identifies its collection; proposal cache-list attempt evidence moves
  to the report level. Update Rust imports and JSON field access. Stored
  snapshots and attempt sidecars retain their schema-1 contracts.
- **Breaking:** Native ICRC balance, allowance, transaction, archive and account
  page requests compose `ledger: IcrcLedgerRequest`. Constructors accept that
  scope before operation settings. Report JSON and clock-free cache identity
  remain unchanged.
- Reuse the validated CMC XDR report when deriving cycles and share rate text
  formatting, preserving one source call, certificate evidence and exact arithmetic.

- **Breaking:** Address targeted SNS cache reads, cache status, cached neuron
  sorts and CLI proposal lists by root principal. Numeric selectors remain for
  live lookup and explicit refresh; numeric proposal-detail CLI calls use live
  lookup. Existing complete snapshots and sidecars stay schema 1 with no reset.
- Use canonical ICRC ledger account, metadata and allowance wire types. Account
  evidence now requires 32-byte subaccounts; preserve raw decimal/hex report JSON.
- Standardize Dashboard text on the complete provenance preamble already used by
  CloudEngine; JSON is unchanged.

- Share CLI progress frames with published Host Text 0.23.1, preserving terminal
  admission, stderr ownership and non-TTY silence while escaping unsafe controls
  and clearing Unicode display columns correctly. [#70](https://github.com/dragginzgame/ic-query/issues/70).
- Align the Governance smoke harness with published IC Build 0.11 and Host 0.23,
  resolving the cleanup-policy type mismatch without changing TERM/reap policy.

- **Breaking:** SNS custom sources return `SnsNeuronDetail`, `SnsProposalRow`
  and vectors of existing neuron/proposal rows directly. Remove the five
  single-payload `MainnetSns*` result wrappers and update source implementations;
  the neuron page retains its cursor. Source validation, report JSON and stored
  snapshots remain unchanged.

- Fix test compilation with Testkit 0.50 by using Host Process HTTP readers
  directly and supplying explicit Candid reply budgets for the Governance probe.
  Preserve fixture deadlines, request-size limits and raw response behavior.
  [#72](https://github.com/dragginzgame/ic-query/pull/72).

Detailed release notes: [docs/changelog/0.60.md](docs/changelog/0.60.md)

## [0.59.0] - 2026-10-16

- **Breaking:** Governance probe source fingerprints now cover the complete
  captured inventory through IC Build. Regenerate probe artifacts and matching
  receipts together; metadata schemas remain 1.
- Share topology percentage arithmetic with IC Metrics, preserving nearest
  one-decimal formatting. Overflowing coverage totals now render `-`.
- Align the Governance harness with IC Build 0.8 and Host Process 0.21 while
  retaining its TERM grace and bounded reaping.
- Share the remaining CLI and topology test-directory allocation with Host FS.
  Cache-header inspection uses captured completion state while preserving bounded
  prefix reads and parse failures.

- **Breaking:** Remove the public `NnsGovernanceQueryError`; native NNS queries
  now return `NnsGovernanceError` directly. Update imports and error conversions.
- Share CloudEngine, NNS, SNS and ICRC Candid transport and bounded decoding.
  Consolidate hex byte decoding and NNS dimension validation while retaining their
  input policies, raw report fields and checked totals.

- Consolidate node-status cache recovery and portable Governance continuation
  checks, preserving freshness policies, typed errors and continuation schemas.

- Reuse validated reward-checkpoint summaries in SNS diffs while retaining raw
  diagnostics for invalid checkpoints. Derive new/missing neuron flags from rows.
- Compare archive manifests through the shared streaming canonical-JSON matcher.

- Consolidate live/cached NNS proposal views and strict principal evidence checks,
  preserving pagination policies, report fields and family-specific errors.
- Complete bounded-read error handling for the selected Host 0.22 API.

- **Breaking:** SNS neuron/proposal refreshes share `SnsSnapshotRefreshRequest`,
  `SnsSnapshotRefreshReport` and `sns_snapshot_refresh_report_text`. Update imports
  and JSON consumers to `collection` and `row_count`; report schemas remain 1.
- **Breaking:** Native Governance builders return `NnsGovernanceError` directly;
  remove matches and conversions for the deleted `NnsGovernanceHostError` wrapper.
- Share live/cached SNS neuron report assembly, retaining owner scope, provenance
  and complete-snapshot totals before view limits.

- **Breaking:** NNS neuron/proposal refreshes share `NnsGovernanceRefreshReport`
  and `nns_governance_refresh_report_text`. JSON uses `collection` and `row_count`,
  with explicit publication and point-in-time assurance; report schemas remain 1.
- **Breaking:** `NnsNodeListRequest` composes `inventory: NnsInventoryListRequest`
  with view filters. Update struct literals and inventory-field access.

- **Breaking:** Registry inventory refreshes share `NnsInventoryRefreshReport`
  and `nns_inventory_refresh_report_text`. Update family imports and count fields
  to `collection` and `row_count`; provider Governance identity remains explicit.
- Share live/cached SNS proposal view projection and fix mixed-case live text
  searches. Simplify inventory cache plumbing to the concrete shared requests.

- **Breaking:** Targeted SNS requests compose `SnsLookupRequest`; update
  constructors and access shared fields through `lookup`.
- **Breaking:** Topology source capabilities take the public read/refresh
  requests directly. Refresh settings compose a `read` scope; custom adapters
  must update signatures and field access.
- **Breaking:** NNS ballots share `NnsGovernanceVote`, retaining unknown vote
  codes in text and JSON. Contradictory cached classifications refuse; preserve
  old evidence and explicitly refresh affected proposal snapshots.

- **Breaking:** Dashboard requests share `IcDashboardRequest` for endpoint and
  caller clock. Parameterized requests compose it as `dashboard`; node-status
  snapshot and boundary-node data-center builders take it directly. Update Rust
  constructors, struct literals and field access; CLI and report JSON are unchanged.
- Share NNS neuron/proposal snapshot locking, attempt sequencing and publication,
  preserving collection-specific paging, stored evidence and finalization failures.

- Fix Governance smoke-runner compilation against IC Build 0.10 and Host FS
  0.22, preserving release-mode Wasm selection and publication permissions.
  [#68](https://github.com/dragginzgame/ic-query/issues/68).

Detailed release notes: [docs/changelog/0.59.md](docs/changelog/0.59.md)

## [0.58.0] - 2026-10-16

- Delegate release cache and formatter preparation to Shared Tooling's pinned
  preflight helper. Keep source admission, explicit offline validation and
  interrupted-release recovery in Query; reject tool-catalog changes during setup.

- Reuse Host's private fixture-directory allocator across cache tests, including
  physical temporary paths and explicit missing-child cases.

- **Breaking:** Local SNS reward-checkpoint reads now accept regular files up to
  64 MiB. Larger files refuse before parsing; invalid JSON retains its source path.
  Invalid UTF-8 is reported as a JSON parse failure.

- **Breaking:** Align the public certificate parse-error payload with Host 0.20.
  Consumers naming Host types must align their dependency.
- **Breaking:** CMC and ICRC certificate evidence uses the shared strict decoder,
  with an 8 MiB combined byte budget and a 64-level nesting limit. Original
  certificate evidence and application-specific leaf validation are preserved.
- Share Candid query mechanics across ICRC and SNS while retaining source policy,
  typed failures and diagnostic labels. Reuse Host's SHA-256 parser and formatter
  for catalog and Registry archive digests; stored JSON remains unchanged.

- Adapt text renderers to Host 0.20's borrowed escaping results, allocating owned
  text only where Query's existing report helpers or table rows require it.

- Use canonical ICRC-3 wire types from the existing ledger-types dependency,
  preserving archive callback validation and raw report JSON. Simplify live
  SNS/ICRC source futures without changing their caller-runtime contract.

- Fix manifest dependency-table ordering that blocked release formatting checks.
  Formatting-hook failures now include their retained formatter diagnostics.
- Align the Governance harness's Host process dependency with IC Build 0.7,
  fixing its compilation while preserving TERM grace and bounded reaping.

Detailed release notes: [docs/changelog/0.58.md](docs/changelog/0.58.md)

## [0.57.0] - 2026-10-15

- **Breaking:** Adopts Host 0.19. The concrete parse error exposed by
  `CertificateVerificationError::Decode` now comes from Host 0.19; consumers
  naming that type must align their Host dependency.
- Uses the published standalone Testkit Candid decoder, preserving Query's
  work quotas, transport limits and original reply evidence.
  [#58 — Adopt bounded Candid reply decoding](https://github.com/dragginzgame/ic-query/issues/58).
- Adopts the latest Shared Tooling 0.15.0 snapshot and its evidence collector.
  Failed tool selections retain safe local diagnostics without admitting
  untrusted archive payloads. Common tool pins are unchanged.
- Aligns the Governance development harness with IC Build 0.6 and Host 0.19,
  preserving its TERM grace and bounded reaping. Temporary fixture roots use
  physical paths for Host's stricter parent admission.
- Removes the final Python tooling invocation and corrects setup requirements.

Detailed release notes: [docs/changelog/0.57.md](docs/changelog/0.57.md)

## [0.56.0] - 2026-10-15

- **Breaking:** Certificate decoding uses Host's strict owned decoder. The public
  `CertificateVerificationError::Decode` payload is now Host's `ParseError`;
  update code that depends on the concrete payload type. Budgets, authentication
  order and original evidence remain explicit.
  [#61 — Adopt Host certificate decoding](https://github.com/dragginzgame/ic-query/issues/61).
- Adopts Shared Tooling 0.15.0 documentation checkers, replacing private engines
  while retaining Query's reviewed debt baseline and source selection.
  [#55 — Adopt shared rustdoc debt checker](https://github.com/dragginzgame/ic-query/issues/55),
  [#57 — Adopt shared type-layout checker](https://github.com/dragginzgame/ic-query/issues/57).

- Uses IC Build's bounded probe resolver to admit Governance Wasm outputs and
  reject changed build inputs before publication, preserving metadata and receipts.
  [#53 — Finish bounded IC Build adoption](https://github.com/dragginzgame/ic-query/issues/53).

- Compiles Testkit canister tests before starting PocketIC and selects explicit
  hard/idle lifetimes, preventing cold CI builds from expiring an unused server.

Detailed release notes: [docs/changelog/0.56.md](docs/changelog/0.56.md)

## [0.55.9] - 2026-10-15

- Keeps Query-specific Testkit setup in the canister guide, preserving the reviewed
  Shared Tooling setup snapshot and its integrity check.

- Fixes standalone certified Subnet Catalog builds by enabling the shared Registry
  certificate-evidence validators with their owning feature.

- Uses Host's shared Unicode table renderer and terminal escaping, preserving
  plain-table layout while making bidi controls and Unicode line separators visible.
  Raw report and JSON values are unchanged.
- Simplifies lossless ICRC transaction JSON serialization and SNS enrichment,
  preserving wire records, complete coverage checks and typed query failures.
- Shares cache age/recovery policy lookup and removes redundant NNS/SNS cache-root
  setup, preserving command behavior and cache policy.
- ICRC account history shares pagination bookkeeping with SNS while retaining
  its own cursor validation, exhaustion rules and failure evidence.
- Shares certificate hex-shape checks, simplifies subaccount normalization and
  avoids repeated proposal-query normalization without changing report evidence.
- Prepares node filters once and shares topology refresh projections.
- Capability probes classify missing methods from typed replica rejection codes;
  diagnostic text alone no longer marks a query as unsupported.

- Adopts Testkit 0.44 for native HTTP fixtures and deterministic execution of the
  compiled Governance probe. Server preparation is explicit; CI runs the new
  canister checks alongside deployment smoke.

```bash
make install-testkit
make testkit-check canister-test
```

Detailed release notes: [docs/changelog/0.55.md](docs/changelog/0.55.md)

## [0.55.8] - 2026-10-14

- Refresh-attempt cache reads reject duplicate top-level fields and share bounded
  JSON admission without constructing an intermediate payload tree.
- Governance probe builds use Host's bounded Wasm metadata encoder, preserving
  metadata bytes and section order while enforcing the complete artifact limit.

Detailed release notes: [docs/changelog/0.55.md](docs/changelog/0.55.md)

## [0.55.7] - 2026-10-14

- Adopts Shared Tooling 0.12.3, preserving common tool pins.
- Publication retains source-bound upload receipts, exact arguments and command
  output. Interrupted attempts block another invocation until their evidence and
  admission locks are reconciled; existing library-first recovery is preserved.
- The Governance development probe shares source inventory and its named-example
  build recipe with IC Build 0.2.7, retaining fingerprint bytes and process deadlines.
- Shares NNS/SNS proposal ordering mechanics, preserving missing-value placement,
  direction, ID tiebreaks and report output.
- Simplifies NNS list request construction and shares quantity-text finishing,
  preserving command defaults, cursor semantics, rounding and unit labels.

Detailed release notes: [docs/changelog/0.55.md](docs/changelog/0.55.md)

## [0.55.6] - 2026-10-14

- Adopts Shared Tooling 0.11.4, including common fixture completion/retention
  handling and exact workspace-version rendering. Existing tool pins are preserved.
- Failed and incomplete fixture checks preserve diagnostics and original failure
  status, including cleanup and reporting failures.
  [#49](https://github.com/dragginzgame/ic-query/issues/49).
- Release metadata rendering reuses the shared pure helper while retaining exact
  saved-source, index and interruption checks.
  [#50](https://github.com/dragginzgame/ic-query/issues/50).

Detailed release notes: [docs/changelog/0.55.md](docs/changelog/0.55.md)

## [0.55.5] - 2026-10-13

- Adopts Shared Tooling 0.11.0. CI inspection rejects conflicting selectors and
  invalid run identities; host/IC setup shares managed tool-selection admission.
  Existing tool pins and Query release adapters are preserved.

- Expose opt-in bounded certificate/witness and ICRC-3 tip authentication with
  typed failures and caller-selected budgets. Share witness commitment checks
  with existing certified reports; callers retain trust-root and payment policy.
- Expose opt-in bounded generic ledger-block admission and contiguous chain
  verification. Admitted immutable values cannot change before hashing; ledger
  transaction interpretation and spending authority remain consumer-owned.
  Documents evidence budgets, returned fields and typed failure variants.

- Consolidates NNS topology read-option parsing, preserving command syntax,
  defaults, cache selection and report output.
- Simplifies SNS command request construction, preserving output, clock and
  cache-root selection and live versus cache-only behavior.
- Shares SNS catalog-refresh endpoint/output parsing and simplifies NNS
  inventory request setup, preserving command defaults and error ordering.
- Removes redundant CLI setup adapters and shares SNS snapshot paths, preserving
  command behavior and cache layout. [#51](https://github.com/dragginzgame/ic-query/issues/51).

Detailed release notes: [docs/changelog/0.55.md](docs/changelog/0.55.md)

## [0.55.4] - 2026-10-13

- Adopts Shared Tooling 0.10.2. Common tool catalogs belong to the reviewed
  snapshot; host, IC and Rust exceptions must be selected independently.
  Existing tool versions and compiler policy are preserved.
  [#48](https://github.com/dragginzgame/ic-query/issues/48),
  [Shared #153](https://github.com/dragginzgame/shared-tooling/issues/153).
- Cargo CI cache keys ignore unrelated literal catalog edits while retaining
  Cargo selections and build identities. [Shared #154](https://github.com/dragginzgame/shared-tooling/issues/154).
- Selects Host artifacts, filesystem and process 0.17.2; Query's consumed APIs
  remain unchanged.
- LOC reports reject CR/LF directory paths before normalization, and release-lock
  refusals preserve filesystem diagnostics and status.
  [Shared #151](https://github.com/dragginzgame/shared-tooling/issues/151),
  [#152](https://github.com/dragginzgame/shared-tooling/issues/152).

Detailed release notes: [docs/changelog/0.55.md](docs/changelog/0.55.md)

## [0.55.3] - 2026-10-13

- Shares NNS Subnet refresh parsing and ICRC account target parsing, preserving
  command syntax, defaults, validation, cache policy and output selection.
- Shares arbitrary-precision decimal carry and validation for token and cycle
  text formatting, preserving rounding and invalid-input handling.

- Shares IC endpoint/output parsing and Registry error-code labels, preserving
  endpoint defaults, report output and typed transport errors.

Detailed release notes: [docs/changelog/0.55.md](docs/changelog/0.55.md)

## [0.55.2] - 2026-10-13

- Adopts Shared Tooling 0.9.15. Tool setup preserves lock-acquisition diagnostics
  and Cargo source-download failure statuses; CI inspection refuses ambiguous
  implicit branch selections.
  [Shared #145](https://github.com/dragginzgame/shared-tooling/issues/145),
  [#146](https://github.com/dragginzgame/shared-tooling/issues/146),
  [#148](https://github.com/dragginzgame/shared-tooling/issues/148).
- CI inspection recovers exact-attempt failed-job logs when failed-step logs are
  empty. Maintenance refuses CR/LF directory paths before probing or writing state.
  [Shared #149](https://github.com/dragginzgame/shared-tooling/issues/149),
  [#150](https://github.com/dragginzgame/shared-tooling/issues/150).
- Selects Host artifacts, filesystem and process 0.17.0. Query's consumed APIs
  remain unchanged; refresh existing pinned Cargo tools explicitly after adopting
  the changed Shared build policy. Prior tools and evidence remain retained.

- Shares arbitrary-precision decimal validation and comparison across report
  families, preserving raw values, existing errors, bounds and cursor policies.

- Consolidates CLI refresh and cache-status parsing within NNS and SNS,
  preserving command defaults, validation and collection behavior.
- Shares SNS cache identity-error mapping and bounds retained refresh-attempt
  matches without changing ambiguity, parsing or read-budget errors.

Detailed release notes: [docs/changelog/0.55.md](docs/changelog/0.55.md)

## [0.55.1] - 2026-10-13

- Adopts Shared Tooling 0.9.10. Rust-tool fixtures work from the consumer's
  compiler context and reuse existing Git history without creating commits.
  [Shared #143](https://github.com/dragginzgame/shared-tooling/issues/143),
  [#144](https://github.com/dragginzgame/shared-tooling/issues/144).
- Cargo-tool setup rejects non-boolean offline configuration while retaining
  explicit environment overrides. Refresh the existing pinned Cargo tools
  explicitly after adopting the new build policy; previous builds remain retained.
  [Shared #140](https://github.com/dragginzgame/shared-tooling/issues/140).

- Maintainer reports distinguish symbol deletions from moves and renames and
  include scoped Rust, type and documentation counts, including zeroes.
  [Shared #141](https://github.com/dragginzgame/shared-tooling/issues/141).

- Consolidates internal CLI option parsing, Dashboard ledger URL construction
  and SNS text formatting, and removes redundant cache feature/Serde attributes.
  CLI behavior, report formats and library APIs are unchanged.

Detailed release notes: [docs/changelog/0.55.md](docs/changelog/0.55.md)

## [0.55.0] - 2026-10-12

- Restricts native test networking to native targets so the Governance Wasm
  probe compiles without Mio. Feature checks inspect production dependency
  boundaries separately from test runtimes and admit individual host features.
- **Breaking:** live collection and cache reads that may refresh now return
  futures on the caller's runtime. Await host operations; synchronous wrappers
  and library runtime error variants are removed. Pure projections, rendering
  and explicit cache-only reads remain synchronous.
- Custom host sources require `Sync` and return `Send` futures; progress sinks
  require `Send`. Construct stateful live adapters with `::default()` and reuse
  them within a collection. Agents retain their admitted endpoint/response
  policy, and Dashboard requests reuse a bounded HTTP client.
- The CLI drives async dispatch with one runtime per command. Command syntax,
  report/cache JSON shapes, schema 1, deadlines and refresh recovery are preserved.

- Adopts Shared Tooling 0.9.5. Host/IC setup shares lock cleanup, releases shared
  locks when checkout cleanup fails, and preserves original installation errors
  and failed candidates. Installers reject CR/LF consumer paths before setup;
  evidence checks reject NUL bytes before hashing. Asset setup retries transient
  download failures within bounded attempts/time; snapshot/recovery identities
  require complete Git IDs. Native acceptance remains
  tracked in [#44](https://github.com/dragginzgame/ic-query/issues/44).
- Selects Host artifacts, filesystem and process 0.15.1.
- Release preparation preserves reviewed README/library examples and treats
  their freshness as advisory, avoiding rejection of already-updated examples.
  [Shared #100](https://github.com/dragginzgame/shared-tooling/issues/100).

Detailed release notes: [docs/changelog/0.55.md](docs/changelog/0.55.md)

## [0.54.15] - 2026-10-12

- Adopts Shared Tooling 0.9.0. Snapshot callers share validated
  records and reject NUL bytes; syntax inspection remains separate from integrity.
- Adds a read-only, online Cargo-tool freshness
  check. It compares common pins, checkout receipts and user-installed packages
  with non-yanked stable releases, retaining registry evidence without installing
  tools or changing selections.
- Formatting admits the selected Rust catalog consistently with setup. Tool
  diagnostics refuse redirected files, snapshot replacement preserves intact
  destinations on copy failure, and successful PCRE2 checks stay quiet.
- The workspace LOC reporter also emits JSON crate rows and numeric totals,
  preserving the existing human-readable report.

```bash
make check-cargo-tool-versions
bash scripts/dev/cloc.sh --json .
bash scripts/ci/verify-shared-tooling-snapshot.sh --read-manifest .shared-tooling.snapshot
```

## [0.54.14] - 2026-10-12

- Adopts Shared Tooling 0.8.1: malformed snapshot fields are rejected consistently,
  while filenames retain literal spaces. Tool-installation failures identify
  changed build inputs and retain both observations for diagnosis.
  [Shared #133](https://github.com/dragginzgame/shared-tooling/issues/133),
  [Shared #134](https://github.com/dragginzgame/shared-tooling/issues/134).

## [0.54.13] - 2026-10-12

- Adopts Shared Tooling 0.8.0. Snapshot refresh derives companions from recorded
  entrypoints and automatically retires unchanged owned files outside that set,
  preserving local edits and caller evidence.
  [Shared #131](https://github.com/dragginzgame/shared-tooling/issues/131).
- Selects Host artifacts, filesystem and process 0.14.5. This package-metadata
  release preserves library source, dependency contracts and supported builds.

## [0.54.12] - 2026-10-12

- Adopts Shared Tooling 0.6.6. Staged formatting admits prepared tool receipts
  against the staged pins and compiler, preserving unrelated edits. The reviewed
  hook can now be activated per clone. [#44](https://github.com/dragginzgame/ic-query/issues/44).
- CI checks prepared manifests, lockfile, toolchain, pins and snapshot between
  stages. Release validation retains its initial checkpoint through nested CI;
  changed inputs stop later stages and retain diagnostics.
  [Shared #129](https://github.com/dragginzgame/shared-tooling/issues/129).
- Prepares cargo-sort-derives 0.14.1, preserving literal `$crate` paths during
  derive sorting. Existing tool payloads remain retained.
  [Shared #113](https://github.com/dragginzgame/shared-tooling/issues/113).
- Selects Host artifacts, filesystem and process 0.14.4, with unchanged library
  source and dependency contracts.

```bash
make install-hooks
bash scripts/ci/run-validation-targets.sh --input Cargo.toml --input Cargo.lock fmt-check
```

## [0.54.11] - 2026-10-12

- Adopts Shared Tooling 0.6.4 through one Make entrypoint, consolidating common
  setup, formatting, snapshot checks and release routing while preserving Query's
  admission checks. [#44](https://github.com/dragginzgame/ic-query/issues/44).
- Uses a fixture-owned executable for completion checks, removing a Linux-only
  path assumption that blocked Intel macOS CI.
  [#45](https://github.com/dragginzgame/ic-query/issues/45).
- Selects Host artifacts, filesystem and process 0.14.2, with unchanged library
  APIs and Query behavior.

## [0.54.10] - 2026-10-11

- Aligns release documentation with the delivered shared engineering baseline
  and Query's explicit release and validation exceptions.
  [#47](https://github.com/dragginzgame/ic-query/issues/47).

## [0.54.9] - 2026-10-11

- Adopts Shared Tooling 0.6.3: normal release targets restart superseded local
  preparation from a clean descendant commit with fresh validation, retaining
  the original plan and Query validation receipt.
  [Shared #126](https://github.com/dragginzgame/shared-tooling/issues/126).
- Adopts Host 0.14.0 and explicitly admits successful process completion in the
  Governance smoke harness, preserving failure diagnostics and cleanup.
  [#46](https://github.com/dragginzgame/ic-query/issues/46).

## [0.54.8] - 2026-10-11

- Adopts Shared Tooling 0.6.2: Cargo setup stops on lock, validation and activation
  failures, preserving the original error and build evidence. Run
  `make install-tools`, then offline `make tools-check`.
  [Shared #120](https://github.com/dragginzgame/shared-tooling/issues/120).
- Shared Cargo fixtures accept padded macOS build counts and exercise that
  boundary on every host. [Shared #124](https://github.com/dragginzgame/shared-tooling/issues/124).
- Release source-conflict errors identify the saved plan, phase and commits,
  and link maintainer recovery guidance while retaining validation evidence.
  [Shared #126](https://github.com/dragginzgame/shared-tooling/issues/126).
- Uses Host artifacts, filesystem and process 0.13.5 with unchanged Query cache
  behavior.

## [0.54.7] - 2026-10-11

- Adopts Shared Tooling 0.6.0: Cargo-built tools reuse verified shared selections
  across checkouts. Run `make install-tools`, then offline `make tools-check`.
  Previous installations and build evidence remain retained.
- Audit and Machete setup, checks and execution use the shared Cargo owner,
  removing local installation/probe recipes and avoiding repeated Cargo installs.
  [Shared #123](https://github.com/dragginzgame/shared-tooling/issues/123).
- Direct Machete execution passes its metadata option without an extra Cargo
  subcommand token, fixing dependency validation during release checks.
- CI retains local and referenced shared Cargo evidence, including safe local
  evidence when store admission fails, through one shared selection contract.
  [Shared #121](https://github.com/dragginzgame/shared-tooling/issues/121).

## [0.54.6] - 2026-10-11

- Consolidates synchronous and asynchronous cache refresh-lock completion,
  preserving lock lifetime, cancellation cleanup and the original query failure.
- Simplifies the development Governance smoke runner and preserves the original
  typed Host execution error, exit status and captured diagnostics.
- Consumer tooling anchors temporary paths before directory changes, preserving
  diagnostics and cleanup with relative or symlinked `TMPDIR` selections.
  [#45](https://github.com/dragginzgame/ic-query/issues/45).
- Selects Host artifacts, filesystem and process 0.13.1 with unchanged library
  APIs and behavior.
- Adopts Shared Tooling 0.5.0: host and IC binaries reuse verified per-user
  bundles across checkouts; existing command paths and previous bundles remain.
  Run `make install-host-tools install-ic-tools` to prepare the selection.
  Common Rust tools use one local selection, removing duplicate formatter setup.
  CI retains referenced shared bundles and local Rust builds and receipts.
  Local evidence survives refused shared-store collection.
  [#44](https://github.com/dragginzgame/ic-query/issues/44).
- Repeated tool setup skips matching Cargo Audit and Cargo Machete installations,
  avoiding Cargo's unnecessary force-overwrite advice.
  [Shared #123](https://github.com/dragginzgame/shared-tooling/issues/123).

## [0.54.5] - 2026-10-11

- Adopts Shared Tooling 0.3.11: Cargo tool setup consistently
  uses the selected consumer's toolchain context, validation retains the same
  diagnostics it reports live, and release-command fixtures support relative
  temporary paths. [#39](https://github.com/dragginzgame/ic-query/issues/39).
- Fixture completion checks reject contradictory retained payloads and failed
  cleanup on Bash 3.2, preserving refusal diagnostics and evidence.
  [#42](https://github.com/dragginzgame/ic-query/issues/42).
- Consumer CI, release/recovery and publication fixtures explicitly fail
  mandatory comparisons, including contradictory release-plan observations.
  [#42](https://github.com/dragginzgame/ic-query/issues/42).

## [0.54.4] - 2026-10-10

- Release preparation stops when version or source observations fail, even if
  they print the expected value. Original failure status and diagnostics survive;
  subsequent preparation and release effects are refused.
  [#41](https://github.com/dragginzgame/ic-query/issues/41).
- Adopts Shared Tooling 0.3.7. Cargo tool setup and offline checks can select
  exact published versions directly from an explicit consumer lockfile.
  [Shared #96](https://github.com/dragginzgame/shared-tooling/issues/96).
- Shared fixtures explicitly reject failed assertions on Bash 3.2. Cargo tool
  setup refuses an absent host identity and non-executable candidates before
  activation. [Shared #107](https://github.com/dragginzgame/shared-tooling/issues/107).
- Selects Host 0.12.5: invalid publication paths containing NUL fail before
  creating parent directories. Existing errors and valid-path behavior remain
  unchanged. [Host #54](https://github.com/dragginzgame/ic-host-tooling/issues/54).

```bash
bash scripts/dev/install-rust-tools.sh --consumer /path/to/consumer \
  --package ic-testkit --lockfile Cargo.lock --bin ic-testkit-server --profile release
```

- Keep only the newest CI run per workflow and branch or PR, cancelling older
  queued and running checks while retaining the existing host matrix and gates
  ([Shared #108](https://github.com/dragginzgame/shared-tooling/issues/108)).

## [0.54.3] - 2026-10-10

- Adopts Shared Tooling 0.3.5 and its reviewed Binaryen 133 tool pins. Prepare
  the selection with `make install-ic-tools`; previous tool bundles are retained.
  Query's canister builds remain unoptimized.
  [Shared #102](https://github.com/dragginzgame/shared-tooling/issues/102).
- Selects Host artifacts, filesystem and process 0.12.4, with unchanged library
  APIs and stored contracts. [Host #53](https://github.com/dragginzgame/ic-host-tooling/issues/53).

## [0.54.2] - 2026-10-10

- Adopts Shared Tooling 0.3.3: parallel Cargo tooling preserves Make's jobserver,
  and validation rejects malformed nesting depth and premature success while
  retaining failure evidence. [Shared #99](https://github.com/dragginzgame/shared-tooling/issues/99),
  [Shared #104](https://github.com/dragginzgame/shared-tooling/issues/104).
- Query's fixture and diagnostic checks require complete execution before
  successful cleanup, preventing Bash 3.2 errors from appearing to pass and
  deleting evidence. [#40](https://github.com/dragginzgame/ic-query/issues/40).

## [0.54.1] - 2026-10-10

- Adopts Shared Tooling 0.3.1: complete tool setup refuses unsupported IC
  platforms or unavailable Rust/Cargo toolchains before downloading tools.
  Host-tool failures identify the selected tool, version, path and repair command.
  [#39](https://github.com/dragginzgame/ic-query/issues/39).
  [Shared #101](https://github.com/dragginzgame/shared-tooling/issues/101).

```bash
bash scripts/dev/install-ic-tools.sh --preflight
bash scripts/dev/install-rust-tools.sh --preflight
```

## [0.54.0] - 2026-10-10

Detailed release notes: [docs/changelog/0.54.md](docs/changelog/0.54.md)

- Adopts Shared Tooling 0.3.0: standard setup and offline checks cover the complete
  host, IC and Cargo toolsets, then Query's formatter and audit tools in order.
  Removes `install-dev` and optional host-tool flags; prepare a Rust toolchain
  first, then use the common commands below. Release preflight retains narrow
  formatter preparation. [#39](https://github.com/dragginzgame/ic-query/issues/39).
- CI inspection reports unavailable failed-step logs and preserves partial
  evidence. [Shared #97](https://github.com/dragginzgame/shared-tooling/issues/97).
- Selects published Host 0.12.0, retaining the current cache, artifact and process
  APIs. Its release adopts the same complete tooling setup contract.
  [Host #47](https://github.com/dragginzgame/ic-host-tooling/issues/47).

```bash
make install-tools
make tools-check
make dependency-tools-check
```

## [0.53.0] - 2026-10-10

Detailed release notes: [docs/changelog/0.53.md](docs/changelog/0.53.md)

- Adopts Shared Tooling 0.2.13 and removes Query's duplicated Make execution
  guard. Shared admission refuses unsafe modes even when caller flags are
  replaced, preserving normal parallel and recursive commands.
  [Shared #30](https://github.com/dragginzgame/shared-tooling/issues/30).
- Snapshot verification refuses line breaks in supplied and resolved directory
  paths before they can select another checkout, preserving ordinary aliases.
  [Shared #95](https://github.com/dragginzgame/shared-tooling/issues/95).
- Selects Host 0.11. Governance smoke uses its current process API and retains
  typed cleanup failures separately from the original command failure. Durable
  smoke-file publication refuses directory-required
  targets before writing, preserving existing bytes and typed failure evidence.
  [Host #44](https://github.com/dragginzgame/ic-host-tooling/issues/44).
  [#38](https://github.com/dragginzgame/ic-query/issues/38).
- Formatting requires the exact prepared local cargo-sort installation. Release
  preflight prepares and checks it before validation; ordinary CI checks it early
  and refuses missing selections without installing or falling back to global tools.
  [#35](https://github.com/dragginzgame/ic-query/issues/35).
  [Shared #96](https://github.com/dragginzgame/shared-tooling/issues/96).
- Release guard fixtures include formatter preparation and offline admission,
  preventing missing setup helpers from blocking release validation.

```bash
make install-format-tools
make format-tools-check
```

## [0.52.6] - 2026-10-10

Detailed release notes: [docs/changelog/0.52.md](docs/changelog/0.52.md)

- Adopts Shared Tooling 0.2.10. Offline snapshot verification reports its recorded
  upstream version and exact commit, and rejects malformed version annotations.
- Formatting prints a compact success line or a failure summary pointing to
  complete retained diagnostics, also preserved in failed CI artifacts.
  [#35](https://github.com/dragginzgame/ic-query/issues/35).
- The shared registry helper supports optional exact package metadata with
  bounded, retained responses; publication keeps its existing observation policy.

```bash
bash scripts/ci/check-crates-io-version.sh --metadata /tmp/ic-query-registry-observation ic-query 0.52.5
```

## [0.52.5] - 2026-10-09

Detailed release notes: [docs/changelog/0.52.md](docs/changelog/0.52.md)

- Adopts Host 0.10.1 and its unified durable writer in the Governance smoke
  runner. File-publication failures retain their publication phase and cleanup
  evidence; cache contracts and stored formats remain unchanged.
  [#37](https://github.com/dragginzgame/ic-query/issues/37).
- Durable smoke-file publication completes parent-directory sync when another
  creator wins the directory creation race.
  [Host #43](https://github.com/dragginzgame/ic-host-tooling/issues/43).

## [0.52.4] - 2026-10-09

Detailed release notes: [docs/changelog/0.52.md](docs/changelog/0.52.md)

- Adopts Shared Tooling 0.2.8, including isolated Make admission and release
  fixture routing. Retains Query's stronger execution guard for overridden flags.
- Repairs GNU Make 3.81 parsing while preserving release cache preparation,
  checkout-local helper selection and offline formatting.
  [#35](https://github.com/dragginzgame/ic-query/issues/35).
- Selects published Host 0.9.7 with the existing cache and process contracts.
- Development setup installs cargo-sort through the shared local installer;
  formatting prefers that selection over stale local or global copies.
  [#36](https://github.com/dragginzgame/ic-query/issues/36).

## [0.52.3] - 2026-10-09

Detailed release notes: [docs/changelog/0.52.md](docs/changelog/0.52.md)

- Adopts Shared Tooling 0.2.6. Shared test exports now require their declared
  helpers before changing consumer files; Query retains its existing selection.
  [Shared #73](https://github.com/dragginzgame/shared-tooling/issues/73).
- Shares release and formatting Make recipes, preserving execution guards,
  local helper routing and dependency preparation. Formatting admission and
  execution use the same selectable Cargo executable.
  [#35](https://github.com/dragginzgame/ic-query/issues/35).
- Selects published Host 0.9.4 with the existing cache and process contracts.
- Resolves node-status targets without buffering all identifiers, retaining
  exact matches, unique prefixes and ordered ambiguity details.
- Package and publication clean-source failures now identify staged, unstaged
  and untracked paths without changing the checkout or index.

```bash
make fmt-check FORMAT_CARGO=cargo
```

## [0.52.2] - 2026-10-09

Detailed release notes: [docs/changelog/0.52.md](docs/changelog/0.52.md)

- Runs native CI once per atomic branch/tag release, preserving all Linux and
  macOS checks, MSRV and canister integration on the branch. Manual CI dispatch
  can qualify a selected branch or tag across all nine jobs.
  [#34](https://github.com/dragginzgame/ic-query/issues/34).
- Adopts reviewed Shared Tooling 0.2.3 snapshot-selection and CI queue guidance.
- Streams probe source and lockfile hashing instead of buffering complete
  files, preserving the fingerprint encoding.

```bash
gh workflow run ci.yml --repo dragginzgame/ic-query --ref v0.52.2
```

## [0.52.1] - 2026-10-09

Detailed release notes: [docs/changelog/0.52.md](docs/changelog/0.52.md)

- Adopts Shared Tooling 0.2.2. IC tool setup and offline checks process every
  selected pin, including a final row without a trailing newline.
  [Shared #87](https://github.com/dragginzgame/shared-tooling/issues/87).
- Selects published Host 0.9.2, retaining cache publication, private reply
  evidence and command cleanup behavior.
- Reduces smoke-runner receipt memory by moving admitted reports and streaming
  JSON into private atomic publication, preserving raw numbers and formatting.

## [0.52.0] - 2026-10-09

Detailed release notes: [docs/changelog/0.52.md](docs/changelog/0.52.md)

- Adopts Shared Tooling 0.2.0 and removes PocketIC from the common developer
  tool bundle. Run explicit `make install-ic-tools` before offline tool checks;
  prior six-tool bundles and evidence remain intact. Query's ICP-managed smoke
  runner continues to use the remaining five tools.
  [#33](https://github.com/dragginzgame/ic-query/issues/33),
  [Shared #76](https://github.com/dragginzgame/shared-tooling/issues/76).
- Dependency checks require one fully validated
  exception document, preventing malformed JSON streams from suppressing pinning
  failures. Valid exception catalogs retain their behavior.
  [Shared #86](https://github.com/dragginzgame/shared-tooling/issues/86).
- Selects published Host 0.9.0 with the existing cache and smoke-runner contracts.
- Builds smoke-probe metadata from one bounded, structurally inspected Wasm
  read, removing the second read and redundant digest comparison.

## [0.51.1] - 2026-10-09

Detailed release notes: [docs/changelog/0.51.md](docs/changelog/0.51.md)

- Adopts Shared Tooling 0.1.36. Cargo-tool setup and offline checks reject
  multi-document receipts, recheck installation paths after builds, and retain
  Cargo's original failure status and build evidence.
  [Shared #65](https://github.com/dragginzgame/shared-tooling/issues/65).
- Selects published Host 0.8.9 with the existing library and smoke-runner contracts.
- Prepares locked dependencies before canister build, bundle and smoke runner
  compilation. Explicit Cargo offline settings remain authoritative; failed
  preparation stops before runner or deployment work.

## [0.51.0] - 2026-10-09

Detailed release notes: [docs/changelog/0.51.md](docs/changelog/0.51.md)

- Replaces the Governance smoke harness with one development-only Rust runner.
  Direct callers must use `governance_smoke`; Python and the separate process
  and artifact helpers are retired. Receipts and original Candid replies remain
  preserved, with the existing cleanup timing and network trust policy.
  [#29](https://github.com/dragginzgame/ic-query/issues/29).
- Bounds smoke Candid decoding work, type tables and headers in addition to
  reply size, retaining exact rejected reply evidence and raw JSON numbers.
  [#30](https://github.com/dragginzgame/ic-query/issues/30).
- Announces automatic inventory and Subnet Catalog refreshes from the library's
  actual cache policy, including invalid and selected stale-cache recovery.
  Cache hits and cache-only or filesystem-authority refusals remain silent.
  [#28](https://github.com/dragginzgame/ic-query/issues/28).
- Prepares locked dependencies during standard release preflight so newly
  selected crates can download before offline validation. Explicit Cargo offline
  settings remain authoritative; failed preparation stops the release.
  [Shared #84](https://github.com/dragginzgame/shared-tooling/issues/84).
- Selects published Host 0.8.8, retaining explicit command deadlines and cleanup.
- Removes the unused consumer fleet reporter; run fleet inventories in Shared
  Tooling. Local setup/check commands and `make cloc` remain available.
  [#31](https://github.com/dragginzgame/ic-query/issues/31).
- Adopts Shared Tooling 0.1.35, including the shared dependency-preparation
  policy and optional installation of selected Cargo binaries/examples with
  explicit profiles, offline receipt checks and retained failed builds.
  IC-tool setup reuses verified bundles across
  comment-only or reordered pin files, preserving installed pins and receipts.
  Changed tool, version, host or archive-checksum records still require explicit
  setup; offline checks still authenticate installed files and versions.
  [Shared #79](https://github.com/dragginzgame/shared-tooling/issues/79).

```bash
bash scripts/dev/install-rust-tools.sh --consumer "$PWD" \
  --package cargo-sort --version 2.1.4 --bin cargo-sort --profile release
```

```bash
cargo run -p ic-query-cli --example governance_smoke --locked --offline -- local
```

## [0.50.3] - 2026-10-08

Detailed release notes: [docs/changelog/0.50.md](docs/changelog/0.50.md)

- Selects published Host 0.8.4 while retaining the optional library features and
  development-only process helper.
- Refreshes Shared Tooling 0.1.30's setup guidance. Runtime helpers and Query's
  tool-installation workflow remain unchanged.
- Preserves native CI runs for each pushed commit when a later release arrives;
  newer revisions still supersede earlier PR checks.
  [#27](https://github.com/dragginzgame/ic-query/issues/27).

## [0.50.2] - 2026-10-08

Detailed release notes: [docs/changelog/0.50.md](docs/changelog/0.50.md)

- Names staged, unstaged and untracked files when release preflight refuses
  changed source, and distinguishes failed Git observations. Preserves source
  and index bytes without automatic repair.
  [#26](https://github.com/dragginzgame/ic-query/issues/26).
- Adopts Shared Tooling 0.1.29, including the shared source check and reviewed
  PocketIC 16.1.0 tool pins.

```bash
bash scripts/ci/check-release-source.sh
```

## [0.50.1] - 2026-10-08

Detailed release notes: [docs/changelog/0.50.md](docs/changelog/0.50.md)

- Selects IC Host Tooling 0.8.2 and moves smoke-command IO and process-group
  cleanup to its shared runner. Preserves five-second TERM grace, bounded
  reaping, failed-command evidence and successful background-network handoff.
  [Host #5](https://github.com/dragginzgame/ic-host-tooling/issues/5).
- Adopts Shared Tooling 0.1.28, including simulation-only release fixtures,
  safe local tracking-ref reconciliation and literal installer-path handling.
  Keeps the maintenance catalog available without activating a schedule.
  [#25](https://github.com/dragginzgame/ic-query/issues/25).
- The development process helper launches the harness and its fixture suite;
  the optional maintenance runner can check its prerequisites:

```bash
cargo run -p ic-query-cli --example governance_process --locked --offline -- \
  python python3 -m unittest discover -s scripts/canister -p 'test_*.py'
bash scripts/dev/run-maintenance.sh --check "$PROJECTS_ROOT" "$MAINTENANCE_STATE"
```

## [0.50.0] - 2026-10-08

Detailed release notes: [docs/changelog/0.50.md](docs/changelog/0.50.md)

- Uses `ic-agent` for Governance smoke calls and certified module/metadata reads.
  Removes ICP response envelopes and hexadecimal Candid decoding, and drops the
  unused `ic-host-tools` dependency. AGENTS.md now prefers direct agent APIs.
- Hard-cuts schema-1 smoke receipts: each report references its retained binary
  Candid reply file instead of embedding an ICP response. Module evidence records
  one probe principal and plain before/after SHA-256 hashes. Receipt consumers
  must update and new runs require fresh evidence
  directories; existing evidence is retained without migration.
  CI uploads the reply files alongside receipts, including on failure.
- Selects published IC Host Tooling 0.7.1 for the retained artifact and filesystem
  APIs, preserving the existing feature profiles.
- The development helper exposes direct agent reads and probe calls:

```bash
cargo run -p ic-query-cli --example governance_artifact --locked --offline -- \
  report mainnet-smoke https://icp-api.io/ "$PROBE_CANISTER" economics "$REPLY_FILE"
```

## [0.49.1] - 2026-10-08

Detailed release notes: [docs/changelog/0.49.md](docs/changelog/0.49.md)

- Preserves smoke-command timeouts and interruptions when process-group cleanup
  is refused, retaining cleanup diagnostics in failed receipts. Escalation holds
  the leader PID until group signalling finishes; successful background startup
  still hands its lifetime to network cleanup.
  [#23](https://github.com/dragginzgame/ic-query/issues/23).
- Selects published Host 0.5.2 for the three existing dependencies, retaining
  the current feature profiles and artifact/cache contracts.

## [0.49.0] - 2026-10-08

Detailed release notes: [docs/changelog/0.49.md](docs/changelog/0.49.md)

- Hard-cuts the public refresh-attempt Rust contract: `status` now uses
  `CacheRefreshAttemptStatus`, and `validate_snapshot_refresh_attempt` returns
  `Result<(), String>`. Callers must use the enum field directly. Schema-1 JSON
  labels remain unchanged; unsupported labels now produce parsing errors.
  [#22](https://github.com/dragginzgame/ic-query/issues/22).
- Adopts Shared Tooling 0.1.23's release runner and governance snapshot.
  Query retains direct, maintainer-run releases and refuses unqualified PR
  delivery before its adapter changes release state. Shared runner fixtures
  explicitly select direct delivery, including under an enclosing PR selection.
  Final checks reject changed release payloads and tag objects; completed resume
  reconciles local and remote release identities without repeating effects.
  [#20](https://github.com/dragginzgame/ic-query/issues/20).
- Selects published IC Host Tooling 0.5.1 for retained cache, archive and
  Governance artifact APIs. The update preserves dependency profiles and retained
  APIs; owned process-group integration remains separate work.
- Keeps the original Governance smoke failure when network cleanup or receipt
  publication also fails, and retains available command diagnostics in failed
  receipts. Host process adoption still needs a successful background-runtime
  handoff contract. [Host #5](https://github.com/dragginzgame/ic-host-tooling/issues/5).

## [0.48.1] - 2026-10-07

Detailed release notes: [docs/changelog/0.48.md](docs/changelog/0.48.md)

- Selects IC Host Tooling 0.4.6, repairing managed-cache and Governance helper
  compilation on Intel and Apple Silicon macOS. The filesystem dependency now
  requires at least 0.4.5 so downstream builds receive the Darwin repair.
  [#19](https://github.com/dragginzgame/ic-query/issues/19).

## [0.48.0] - 2026-10-07

Detailed release notes: [docs/changelog/0.48.md](docs/changelog/0.48.md)

- Uses IC Host Tooling's descriptor-based durable writer for managed caches,
  removing the local staging and publication pipeline. Cache confinement,
  serialization budgets and schema-1 bytes remain unchanged.
  [#19](https://github.com/dragginzgame/ic-query/issues/19).
- Hard-cuts the public cache publication error contract: `PublishManagedFile`
  replaces `SyncTemp` and `Replace`, identifies whether the new output is visible,
  and retains staging cleanup failures. Library callers must update error matches;
  visible output requires reconciliation before retrying.
- Adopts Shared Tooling 0.1.20's contribution rules: an explicitly requested PR
  includes its topic branch, scoped commits and branch push. Releases remain
  maintainer-run under the local policy.
  [#18](https://github.com/dragginzgame/ic-query/issues/18).

## [0.47.11] - 2026-10-07

Detailed release notes: [docs/changelog/0.47.md](docs/changelog/0.47.md)

- Keeps Governance smoke-harness identities and ICP settings outside disposable
  build/runtime state. Home selection is scoped to ICP children, preserves an
  explicit persistent `ICP_HOME`, and rejects homes inside output directories.
  [#16](https://github.com/dragginzgame/ic-query/issues/16).
- Adopts Shared Tooling 0.1.19: Rust-tool setup rejects redirected installation
  paths before execution, release notes preserve historical end-of-file bytes,
  and validation retains one combined log of the current attempt's failures.
  [#17](https://github.com/dragginzgame/ic-query/issues/17),
  [#3](https://github.com/dragginzgame/ic-query/issues/3).
- Updates the selected IC Host artifacts, filesystem and response helpers to
  0.4.2, retaining Query's current APIs and dependency profiles.

## [0.47.10] - 2026-10-07

Detailed release notes: [docs/changelog/0.47.md](docs/changelog/0.47.md)

- Adopts Shared Tooling 0.1.18. Validation rejects Make options and assignments
  before running any target; LOC reports exclude both selected build-directory
  aliases and their physical paths.
  [#3](https://github.com/dragginzgame/ic-query/issues/3).
- Adds explicit repository-local setup and offline checks for the optional
  shared Cargo-tool set. The normal development setup retains its current tools.

```bash
make install-rust-tools
make rust-tools-check
```

- Adopts published IC Host Tooling 0.4.1 for bounded cache streams, export-path
  resolution, archive hashing and the Governance development helper. Query uses
  retained APIs; cache confinement, byte limits, archive identities and the
  response-only dependency profile remain unchanged.

## [0.47.9] - 2026-10-07

Detailed release notes: [docs/changelog/0.47.md](docs/changelog/0.47.md)

- Adopts Shared Tooling 0.1.16 and its canonical release-runner fixture,
  removing repeated local runner scenarios while retaining Query's metadata,
  staging and recovery checks.
  [#3](https://github.com/dragginzgame/ic-query/issues/3).
- Adds optional retained validation logs and timings, preserving Make failure
  status. LOC reports accept a selected Cargo manifest; tooling inventory
  recognizes custom snapshot manifests and SSH source identities.
  [#15](https://github.com/dragginzgame/ic-query/issues/15).

```bash
make cloc CLOC_MANIFEST=Cargo.toml
perl scripts/dev/cloc-tooling.pl --json --snapshot-root .shared-tooling.snapshot "$PWD"
VALIDATION_LOG_DIR="$PWD/target/validation-runs" make ci
```

## [0.47.8] - 2026-10-07

Detailed release notes: [docs/changelog/0.47.md](docs/changelog/0.47.md)

- Adopts Shared Tooling 0.1.15's common Make commands, removing duplicate setup
  recipes and provisioning pinned local cloc with the host set. Adds workspace
  Rust LOC and sibling tooling inventory commands.
  [#15](https://github.com/dragginzgame/ic-query/issues/15).

```bash
make install-host-tools host-tools-check
make cloc
make cloc-tooling
```

- Uses published IC Host Tooling 0.3.3 for export-path resolution and archive
  hashing, removing the private resolver and hashing writer. Retains export alias
  protection, symlink limits, canonical bytes and confined atomic cache publication.
  [#13](https://github.com/dragginzgame/ic-query/issues/13),
  [#14](https://github.com/dragginzgame/ic-query/issues/14).
- Uses the response-only tools profile for the Governance development helper,
  excluding Candid extraction and process execution from its normal dependency graph.
- Fixes Make-mode admission on macOS's GNU Make 3.81, where short execution
  flags may follow long options. Keeps inherited selections and parallel execution.
  [#3](https://github.com/dragginzgame/ic-query/issues/3).

## [0.47.7] - 2026-10-07

Detailed release notes: [docs/changelog/0.47.md](docs/changelog/0.47.md)

- Uses IC Host Tooling 0.3.1's smaller artifact crate for bounded cache streams
  and exact canonical JSON comparison.
  Production cache features exclude artifact parsers, process execution and
  ordinary-path filesystem helpers; cache bytes, limits and atomic replacement
  remain unchanged. Cache reads allocate fallibly rather than preallocating
  from file metadata. Governance development helpers adopt the split crates.
- Removes the remaining archive byte-limit writer and duplicate canonical JSON
  hashing. Governance receipts use shared atomic filesystem publication with
  owner-only permissions, file and parent-directory synchronization. Receipt
  publication streams stdin without buffering another complete copy.
- Adopts Shared Tooling 0.1.14: release attempts stay bound to their recorded
  destination, and snapshot verification hashes files independently of the
  inspected checksum helper. Adds exact-commit CI inspection across workflows
  and keeps the shared guides unchanged through reviewed exports.
  [#3](https://github.com/dragginzgame/ic-query/issues/3).
- Rejects Make's ignore-errors, dry-run, touch and question modes before recipes
  can run, preventing false validation success while preserving normal nested
  commands and parallel execution. Direct release and validation scripts also
  reject non-executing modes before changing recovery state or retained logs.
- Excludes Cargo's selected build directory from LOC reports, including custom
  directories inside a package.

```bash
printf '%s\n' '{"schema_version":1,"status":"running"}' | (
  umask 077
  cargo run -p ic-query-cli --example governance_artifact --locked --offline -- \
    write-receipt /tmp/ic-query-receipt.json
)

bash scripts/dev/gh-ci.sh --commit HEAD --all-workflows --limit 100
```

## [0.47.6] - 2026-10-06

Detailed release notes: [docs/changelog/0.47.md](docs/changelog/0.47.md)

- Uses Shared Tooling's TOML-aware workspace version reader for version queries,
  changelog checks, release preflight and publication/tag admission. Valid comments
  are accepted; failed or malformed observations stop before release effects.
  Metadata transformations and retained-source checks remain local.
- Adopts Shared Tooling 0.1.11 and pinned Cargo sorting in `fmt`/`fmt-check`.
  Both check prepared formatter tools offline before use; explicit development
  setup installs the same cargo-sort 2.1.4 pin. Sorting preserves dependency selections.
  [#10](https://github.com/dragginzgame/ic-query/issues/10).
- Retains failed CI, release/publication fixture and feature/rustdoc diagnostic
  directories, reporting their paths. Hosted CI uploads failed validation evidence
  for 30 days on each supported host.
- Keeps passing and ignored Rust tests out of CI error labels while preserving
  failure context. Release changelog comparisons retain exact version ordering
  for large components. The shared repair/reporting policy replaces local wording.

```bash
PATH="$PWD/.tools/host/bin:$PATH" bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml
make format-tools-check
make fmt
make fmt-check
```

## [0.47.5] - 2026-10-06

Detailed release notes: [docs/changelog/0.47.md](docs/changelog/0.47.md)

- Makes local builds, tests, linting, rustdoc and package validation explicitly
  locked/offline, reporting missing cached dependencies without downloading them
  or changing the selected lockfile. Dependency setup remains explicit.
- Preserves Cargo's failure status and diagnostics in feature-boundary checks,
  stopping before later checks after a failed compilation, API test or tree read.
- Refreshes the committed Shared Tooling 0.1.8 snapshot with explicit Bash 3.2 failure
  guards, shared release-command checks, lockfile rewriting and RustSec preparation.
  Failed dependency checks retain preparation evidence and admitted database identity.
- Enforces workspace version and dependency inheritance through the shared
  declaration checker in the existing CI/release pin gate.
  [#9](https://github.com/dragginzgame/ic-query/issues/9).
- Installs pinned ripgrep 15.2.0 with PCRE2 through the shared host toolset,
  replacing the separate Cargo installation and verifying its archive offline.
- Adopts IC Host Tools 0.2.0 for bounded JSON cache writing and byte counting,
  removing duplicate writers while preserving cache bytes, limits and atomic
  replacement. Pure-library and canister builds retain their dependency boundary.
  [Shared adoption](https://github.com/dragginzgame/ic-host-tools/issues/5).

```bash
bash scripts/dev/install-host-tools.sh --with-ripgrep --check
bash scripts/ci/check-release-commands.sh "$PWD"
PATH="$PWD/.tools/host/bin:$PATH" bash scripts/ci/check-dependency-pins.sh --cargo-inheritance
perl scripts/ci/rewrite-local-lock-versions.pl Cargo.lock 0.47.4 0.47.5 \
  ic-query ic-query-cli > /tmp/ic-query-0.47.5.candidate.lock
# Explicit local advisory checkout and a new destination whose parent exists:
bash scripts/ci/prepare-rustsec-db.sh local /path/to/advisory-db /tmp/new-rustsec-attempt
```

## [0.47.4] - 2026-10-06

Detailed release notes: [docs/changelog/0.47.md](docs/changelog/0.47.md)

- Adopts the reviewed Shared Tooling 0.1.7 snapshot. Publication now stops when
  the registry cannot establish whether the exact version exists; only a confirmed
  absent version admits publication. Retains library-first ordering and registry
  index readiness checks. [#8](https://github.com/dragginzgame/ic-query/issues/8).
- Shares portable digest generation between checksum verification and local IC
  tool receipts, with failed hash commands and unrepresentable filenames rejected.
- Adds the shared local-link checker to CI for current guides and contracts,
  stopping the gate when a referenced local file is missing.

```bash
bash scripts/ci/verify-file-checksum.sh --print sha256 Cargo.lock
bash scripts/ci/check-crates-io-version.sh ic-query "$(make -s version)"
make doc-links-check
```

## [0.47.3] - 2026-10-06

Detailed release notes: [docs/changelog/0.47.md](docs/changelog/0.47.md)

- Groups public API tests by feature configuration in the feature-boundary gate,
  reducing repeated Cargo invocations while preserving the existing coverage.
  Dependency-tree inspection now explicitly uses the selected lockfile offline.
- Runs Governance receipt and artifact-helper unit suites once per native host
  in CI, retaining the separate live smoke and bundle jobs. Package-content
  inspection now uses locked/offline dependencies too.

## [0.47.2] - 2026-10-06

Detailed release notes: [docs/changelog/0.47.md](docs/changelog/0.47.md)

- Captures Governance artifact-helper diagnostics in exceptions and failed
  receipts, so expected rejection tests no longer produce misleading CI error
  highlights. Actual failures retain their diagnostic and raw response evidence.

- Adopts the reviewed Shared Tooling 0.1.6 snapshot for explicit repository-local
  jq/yq and IC tool setup, including Quill. CI and canister commands select the
  verified local tools; ordinary checks never install missing tools. Failed and
  previous toolsets remain retained. ICP CLI stays at 1.6.0.
  [#5](https://github.com/dragginzgame/ic-query/issues/5).
- Removes the duplicate regex Actions guard in favor of the existing structured
  dependency-pin gate, and packages each crate once while preserving Cargo's
  network retries and original failure status.
  [#6](https://github.com/dragginzgame/ic-query/issues/6),
  [#7](https://github.com/dragginzgame/ic-query/issues/7).

- Consolidates LOC reporting and annotated-tag verification through the reviewed
  shared helpers. Consumer CI retains setup wiring and native tool verification;
  installer regression suites remain upstream. Governance module verification
  reuses bounded Wasm inspection and hashing through `ic-host-tools`. The smoke
  harness uses the shared toolset's version policy and reuses Cargo's selected
  artifact directory.

```bash
make install-tools
make tools-check
export PATH="$PWD/.tools/host/bin:$PWD/.tools/ic/bin:$PATH"
quill --version
CARGO_NET_OFFLINE=true bash scripts/dev/cloc.sh "$PWD"
```

## [0.47.1] - 2026-10-06

Detailed release notes: [docs/changelog/0.47.md](docs/changelog/0.47.md)

- Adopts Shared Tooling 0.1.5: confines logger checkout identity to its invocation
  and adds offline dependency-pin checks to CI/release validation. Development
  setup installs a checksum-verified parser for each supported native host.
  [Shared Tooling #6](https://github.com/dragginzgame/shared-tooling/issues/6),
  [#7](https://github.com/dragginzgame/shared-tooling/issues/7).
- Configures native Governance smoke and bundle CI on Linux and both macOS
  architectures with verified ICP CLI 1.6.0 archives and executable version
  checks. Retains host-specific evidence; new native qualification is pending.
- Uses a physical temporary root in NNS public API fixtures, avoiding macOS
  `/tmp` symlink rejection while preserving managed-cache confinement checks.
- Adopts `ic-host-tools` 0.1.9 in a development-only Governance artifact helper for
  bounded Wasm inspection and ICP response admission. Candid decoding uses its
  canonical Rust decoder and preserves report text without numeric conversion.

```bash
make install-dev
make dependency-pins-check
cargo run -p ic-query-cli --example governance_artifact --locked --offline -- \
  inspect-wasm target/canister-smoke/governance_probe.wasm
```

## [0.47.0] - 2026-10-06

Detailed release notes: [docs/changelog/0.47.md](docs/changelog/0.47.md)

- Replaces the separate bump/stage/commit/push interface with one reviewed
  release workflow. Maintainers use `make release-patch`, `make release-minor`,
  or `make release-major`; retry reconciles the saved release, including after
  committed fixes, then validates the requested next increment. Preserves
  dependency selection, stages the selected release notes, retains artifacts
  and evidence, and atomically pushes only the selected branch and tag.
  [#2](https://github.com/dragginzgame/ic-query/issues/2),
  [#3](https://github.com/dragginzgame/ic-query/issues/3).
- Breaking maintainer workflow change: separate bump/stage/commit/push Make
  targets are retired, and publication requires the current version's annotated
  tag at HEAD. CLI and Rust APIs are unchanged. Adds native macOS CI coverage
  configuration; qualification remains pending its workflow runs.
- Adopts Shared Tooling 0.1.4 staging and logging checks: rejects conflicting
  pending notes before validation, checks the exact staged release payload, and
  retains raw failed-validation logs across retries and logging failures.
- Publication and release fixtures isolate inherited Make overrides and logger
  checkout identity so validation uses each fixture's selected version and gate.

## [0.46.x] - 2026-10-04 - Cache-only NNS analytics

Detailed release notes: [docs/changelog/0.46.md](docs/changelog/0.46.md)

- `0.46.5` simplifies CI, release, and publication fixtures by comparing
  complete command traces directly. Removes redundant trace arrays and Bash
  features unavailable in the bundled macOS shell. Checks retain command
  ordering, failure propagation, library-first publication, retry safety, and
  release guards; failed CI sequences now check every preceding command.
  The second-pass cache-write limit fixture now uses managed publication to
  avoid permission failures caused by a group-writable shell umask.
  Production release flow, CLI behavior, Rust APIs, and schema identifiers are
  unchanged. Native macOS qualification remains pending.

- `0.46.4` simplifies exact cycle/token formatting, sharing cycle units and
  removing temporary fraction/padding buffers. Dashboard validation derives
  duplicate rejection from existing ordering and UTC-day binding instead of
  redundant identity sets; independent identity checks, bounds, and checked
  totals remain enforced. Dashboard reward/release pages and token values use
  builder-validated request limits after checking the source's exact query
  echo, removing repeated request checks. Exact rounding, arbitrary precision,
  and raw JSON are preserved. Public documentation checks retain existing
  artifacts and still reject incomplete diagnostics. Agent guidance pins a
  reviewed Shared Tooling revision, including focused validation, artifact
  preservation, and required macOS support. Public Rust APIs, CLI grammar,
  reports, and cache shapes are unchanged; schemas remain `1`.

- `0.46.3` fixes a panic on malformed Unicode ICRC subaccounts and rejects
  signed byte pairs that were accidentally accepted as hexadecimal. ICRC
  subaccounts and SNS neuron IDs use the existing shared hex decoder. ICRC
  retains mixed-case and surrounding-whitespace normalization; SNS IDs remain
  strictly lowercase and 32 bytes. Public Rust APIs, CLI grammar, successful
  reports, and cache shapes are unchanged; schemas remain `1`.

- `0.46.2` simplifies SNS reward checkpoint completion bookkeeping and removes
  repeat checks after ending evidence equals validated starting evidence.
  Strict pagination, full bracket comparisons, completion-time checks, and
  restored-report validation remain enforced. Proposal text and SNS neuron
  sorting avoid temporary string keys; neuron value sorts share their ordering
  rule. Live and restored neuron evidence share permission and pending
  disbursement checks, with more specific malformed-checkpoint diagnostics.
  Public Rust APIs, CLI grammar, successful reports, and cache shapes are
  unchanged; schemas remain `1`.

- `0.46.1` simplifies NNS analytics construction while retaining validation of
  caller-supplied state, rows, and restored reports. Proposal queries and
  refreshes now share one page-size validator and limit, also used by the CLI.
  Removes both analytics `InvalidReport` variants,
  `NnsProposalHostError::InvalidRefreshPageSize`, and
  `NNS_PROPOSAL_REFRESH_MAX_PAGE_SIZE` as Rust API hard cuts; use the existing
  wrapped `NnsProposalError::InvalidLimit` and `NNS_PROPOSAL_MAX_PAGE_SIZE`.
  CLI grammar, accepted limits, report JSON, and cache shapes are unchanged;
  schemas remain `1`.

- `0.46.0` adds cache-only NNS proposal activity and public-neuron distribution
  commands using the existing pure library reports. Proposal activity supports
  a half-open creation-time window; both commands support text and raw JSON.
  Neither command fetches or refreshes data.
- NNS proposal and neuron snapshots now require the actual final
  `collection_state`, bound to their stored identity, provenance, timestamps,
  and completeness. This is a schema-1 cache hard cut: explicitly refresh
  existing snapshots before using cache-backed reads. Adds host error variants
  for activity, distribution, and missing neuron caches; downstream exhaustive
  matches need updating. Existing pure analytics report shapes are unchanged.

```bash
icq nns proposal refresh
icq nns proposal activity --from 1700000000 --until 1700100000 --json
icq nns neuron refresh
icq nns neuron distribution --json
```

## [0.45.x] - 2026-10-04 - Cache and decoder bounds, validation, and cleanup

Detailed release notes: [docs/changelog/0.45.md](docs/changelog/0.45.md)

- `0.45.14` rewrites the README introduction around plain-language use cases,
  a first session, a short IC glossary, and an approachable explanation of
  evidence provenance while retaining the complete technical reference.
  Refreshes the documentation audit against the current command tree, library
  feature boundary, cache policy, and release history; corrects managed-cache
  permission guidance and stale release labels. CLI grammar, Rust APIs,
  serialized shapes, and schema identifiers are unchanged.

- `0.45.13` rejects proposal ID zero in NNS disk snapshots, matching live
  collection and analytics checks; invalid caches remain visible in local
  status and list reports. NNS collectors derive their next state from admitted
  pages without repeating restored-state validation. Neuron reports derive
  cache origin from their cache path instead of storing a second internal flag.
  Public Rust APIs, CLI grammar, cache identities, and serialized shapes are
  unchanged; schemas remain `1`.

- `0.45.12` gives patch/minor/major releases one sequential recipe and CI
  ownership of the target-version changelog check. Release staging follows
  workspace-owned version metadata; publication retains library-first ordering,
  registry visibility checks, and retry-safe skips. Replaces release recipe
  text assertions with failure/ordering fixtures and verifies publication
  rejects dirty trees and missing or stale tags before reaching Cargo. ICRC
  cursor normalization uses validated decimal text directly, and attempt reads
  validate without discarding a normalized copy. SNS neuron pages use strict
  ordering to reject duplicate ids, and ICRC history reuses disjoint-range
  evidence instead of tracking resident ids again. Public Rust APIs, CLI grammar,
  cache identities, and serialized shapes are unchanged; schemas remain `1`.

- `0.45.11` rejects complete-cache row counts that exceed the declared page
  capacity and custom ICRC completion results beyond the requested page cap.
  SNS discovery records its actual inventory width; contradictory cached
  counts are rejected and visibly refreshed by its existing list policy.
  Removes redundant SNS collection-wide duplicate tracking while preserving
  boundary overlaps, first-observed values, exhaustion, and failure recovery.
  SNS Root health reuses shared plain-decimal conversion, preserving arbitrary
  precision and zero-balance classification. Public Rust APIs, CLI grammar,
  cache identities, and serialized shapes are unchanged; schemas remain `1`.

- `0.45.10` fixes live ICRC history failures caused by grouped Candid numbers
  and emits plain decimal strings for ICRC balances, allowances, token metadata,
  block histories, and archive ranges, including SNS token metadata. Validates
  blocks and archive ranges against the reported log length while preserving
  valid empty pages and unknown custom-source lengths. Refresh locks use one
  cleanup owner through creation, writing, and sync. Account-history pages
  share cursor and ordering validation, reject invalid pages before updating
  collection progress, and no longer need final sorting/deduplication.
  Corrects cache permission guidance for supported `0755`/`0644` paths. Public
  Rust APIs, CLI grammar, and serialized shapes are unchanged; schemas remain `1`.

- `0.45.9` bounds ICRC archive following to the requested page, 100 callbacks,
  and a 64 MiB Candid decoding budget, rejecting invalid or duplicate block
  identities. Cache publication, including Subnet Catalog, enforces its readers'
  size ceilings and preserves existing snapshots on oversize failure. SNS refreshes validate
  pagination cursors and boundaries; failed lock acquisition cleans up after
  directory-sync failure. Adds `IcrcError::InvalidTransactionPage` and
  `CacheFileError::WriteLimitExceeded`, requiring downstream exhaustive-match
  updates. CLI grammar and serialized shapes are unchanged; schemas remain `1`.

- `0.45.8` gives native NNS cache refreshes one completion decision through
  their portable collectors and simplifies SNS proposal and cached-neuron
  report construction. Live and local SNS lookup share root-principal parsing.
  Page caps, ordered rows, complete-only publication, failed-refresh evidence,
  cached provenance, and view options are preserved. Public Rust APIs, CLI
  grammar, cache identities, and serialized shapes are unchanged;
  schemas remain `1`.

- `0.45.7` rejects invalid provenance in caller-owned Dashboard node-status
  snapshots, including certification and point-in-time claims, through the
  same observation validation used by cache reads. Replaces proposal and neuron
  collection status types with `nns::governance::NnsGovernanceCollectionStatus`
  as a Rust API hard cut. Simplifies node uniqueness validation and topology
  kind/capacity aggregation while preserving collection exhaustion rules,
  ordering, unknown counts, and failed-refresh recovery. CLI grammar, cache
  identities, serialized shapes, and status labels are unchanged;
  schemas remain `1`.

- `0.45.6` accepts readable managed cache directories/files (`0755`/`0644`),
  fixing failures on conventional user cache trees. Group/world-writable paths
  and symlinks remain rejected; new directories/files retain `0700`/`0600`.
  Simplifies CloudEngine exact-node and provider validation, removing
  temporary row storage and a redundant provider identity set. Dashboard
  Subnet/provider status views build detailed rows only for selected groups,
  preserving snapshot-wide totals, assignment comparisons, ordering, and
  source/cache validation. Consolidates Dashboard page validation and topology
  component reads; topology versions projects provenance directly while
  preserving source order and failures. Public Rust APIs, CLI grammar, cache
  identities, and serialized shapes are unchanged; schemas remain `1`.

- `0.45.5` removes redundant SNS collection-state wrappers and forwarding
  modules while preserving page validation, deduplication, cursor handling,
  and failed-refresh behavior. Consolidates NNS provider aggregation into one
  entry per provider and gives summary construction ownership of join counts.
  Registration evidence, per-operator capacity, report ordering, and component
  provenance remain intact. Public Rust APIs, CLI grammar, cache identities,
  and serialized shapes are unchanged; schemas remain `1`.

- `0.45.4` removes `UncertifiedCatalogCollection::new` and its
  `with_registry_evidence` method as a Rust API hard cut; custom sources must
  construct explicit routing authority and Registry record evidence.
  Simplifies SNS identity ownership and catalog validation, preserving original
  ids, provenance, unknown lifecycle codes, and lifecycle evidence checks.
  Removes redundant snapshot copies and publication-time validation in ICRC
  account history, NNS neurons, and Dashboard node status; source and disk
  validation and failed-refresh preservation remain enforced. NNS proposal
  status reuses one attempt observation; listing retains best-effort handling.
  Fixes the schema guard to reject multi-digit nonconforming versions.
  CLI grammar, cache identities, and serialized shapes are unchanged;
  schemas remain `1`.

- `0.45.3` bounds SNS collection discovery to 1,024 candidates and 16,384
  directory entries per scan, with a shared 1 GiB read allowance for cache
  lists and numeric lookups. Exceeding a bound returns an error rather than a
  partial or missing result. Numeric cache reads reject replacement snapshots
  claiming a different SNS id. Consolidates attempt reads and removes redundant
  construction and forwarding machinery. Adds `CacheFileError::ScanLimitExceeded`,
  requiring downstream exhaustive-match updates; CLI grammar and serialized
  shapes are unchanged, and schemas remain `1`.

- `0.45.2` bounds local cache-status inspection and checks snapshot replacement
  state under the refresh lock. SNS status reuses one refresh-attempt observation;
  proposal reports and cache lookup shed redundant intermediate models. ICRC
  account-history timestamps use the caller's clock plus elapsed collection
  time, and cache reads reject reversed intervals. Simplifies collection state
  and transaction-id validation. Public Rust signatures, CLI grammar, serialized
  shapes, and schema identifiers are unchanged.

- `0.45.1` fixes canonical UTC timestamp validation across Governance and
  shared report/cache consumers. Previously accepted noncanonical padding or
  extra signs now follow existing validation or stale-data policies. Proposal
  cache reads use the loader's missing-file result directly, preserving typed
  content and filesystem errors. Consolidates cache refresh and storage
  mechanics while retaining family recovery rules and atomic publication.
  Public Rust signatures, CLI grammar, and serialized shapes are unchanged;
  schemas remain `1` and Rust 1.91.0 remains supported.

- `0.45.0` bounds JSON cache reads, local inventory traversal, and Candid
  decoding work. Repairs invalid UTF-8 only under the selected invalid-content
  policy and prevents refresh-lock cleanup from deleting a replacement lock
  after a directory sync failure. ICRC-3 archive following queries the supplied
  callback method name. Rejects malformed Governance endpoints, invalid or
  reversed collection timestamps, and inconsistent complete-collection counts.
  Public error enums gain variants requiring downstream exhaustive-match
  updates; schemas remain `1` and Rust 1.91.0 remains supported.
  Handles closed stdout consistently for text, JSON, help, and version output.
  Reduces cache serialization, JSON scanning, and retained Registry-history
  copying; an isolated JSON-loading benchmark improved by 21–26%.
  Documents 87 more public API items in the latest audit cleanup and checks
  documentation debt by item identity.

## [0.44.x] - 2026-10-02 - Governance canister tooling and dependency refresh

Detailed release notes: [docs/changelog/0.44.md](docs/changelog/0.44.md)

- `0.44.2` rejects refresh export paths that alias the managed cache or refresh
  lock, fixing dry runs that could overwrite a Subnet Catalog while reporting
  no cache write. Protects Subnet Catalog and shared NNS cache refreshes against
  path, symlink, and Unix hard-link aliases; distinct exports remain supported.
  Adds bounded cross-process Registry history reuse for live Subnet Catalog
  loads and refreshes, with explicit opt-in for caller-owned sources. Restored
  transcripts preserve endpoint isolation, fresh record reads, and agreement;
  local cache inventory exposes the transcript and its writer lock.
  A separate-process mainnet trial avoided all 154 cold history queries and
  refreshed in 20.979 s versus a 92.236 s cold acquisition.
  Release/version-bump workflows retain Cargo build artifacts for subsequent
  builds by removing the automatic post-bump `cargo clean`.

- `0.44.1` updates development and CI to Rust 1.99.0, fixes Clippy findings,
  and adds no-default library linting to the CI gate. Refreshes locked
  dependencies while retaining the declared Rust 1.91.0 minimum. Records
  Canic's Subnet Catalog feedback and the remaining cross-process history
  reuse opportunity.

- `0.44.0` updates the Governance canister smoke harness and its checksum-pinned
  CI tool to ICP CLI 1.6.0. The local NNS runtime remains pinned to 16.0.0;
  production `icq` behavior is unchanged.
- Refreshes locked dependencies without changing direct library requirements
  or the Rust 1.91.0 minimum.
- Measures a fresh two-endpoint Subnet Catalog acquisition at 88.199 s and
  312 explicit queries in one development-build run. The in-memory history
  checkpoint does not carry across separate CLI processes; see the
  [measurement notes](docs/design/subnet-catalog-acquisition-performance.md).

## [0.43.x] - 2026-09-15 - Governance canister testing and Subnet Catalog performance

Detailed release notes: [docs/changelog/0.43.md](docs/changelog/0.43.md)

- `0.43.1` overlaps Subnet Catalog agreement collection across at most two endpoints.
  Adds a reusable live source with endpoint-isolated history checkpoints,
  structured acquisition progress, and bounded retries for selected transient
  transport failures. Exact version/content agreement, assurance floors,
  atomic publication, cancellation cleanup, and cache-hit authority remain
  enforced. See [detailed notes](docs/changelog/0.43.md) and
  [measurement and integration guidance](docs/design/subnet-catalog-acquisition-performance.md).
- In a single development-build measurement at Registry version 64108, cold
  two-endpoint acquisition improved from 158.961 s to 91.008 s. A subsequent
  refresh with retained history took 18.181 s and avoided 154 history queries;
  immediate cache reuse preserved identical snapshot authority.
```bash
cargo run -p ic-query-cli \
  --example subnet_catalog_timing -- /tmp/icq-catalog-timing
```

- `0.43.0` adds a deployable Governance probe, pinned local NNS smoke tests,
  deployed metadata and module-hash verification, and schema-1 execution
  receipts. ICP CLI 1.5.0 builds environment-specific bundles, and a separate
  CI job retains local test evidence and artifacts for 30 days. Mainnet
  verification requires an explicitly supplied, already deployed probe.
  Receipts are saved incrementally with atomic replacement, and interrupted
  or timed-out local startup attempts trigger process and network cleanup.
- Updates development and CI to Rust 1.98.1 and refreshes locked dependencies
  while preserving the Rust 1.91.0 minimum. Production `icq` commands and
  report contracts retain their existing behavior.

- Boxes detailed Subnet Catalog failures to reduce result sizes under Rust
  1.98.1. Explicit detailed-error return types now use `Box`; all typed
  provenance and the original source errors are retained.
- Fetches a fresh, shallow RustSec database with visible Git progress and a
  slow-transfer cutoff before dependency auditing; failed downloads stop the
  check and remove the partial database.

```bash
make canister-build
make canister-smoke
make canister-bundle
```

## [0.42.x] - 2026-08-21 - Stable Subnet Catalog authority identity

Detailed release notes: [docs/changelog/0.42.md](docs/changelog/0.42.md)

- `0.42.0` hard-cuts the Subnet Catalog load authority API so
  `CatalogSnapshotAuthorityEvidence` contains only Registry version, catalog
  digest, assurance, and canonical source endpoints. `snapshot_authority()`
  derives that stable value exclusively from `ValidatedSubnetCatalog`, while
  `CatalogLoadOutcome::path` and `CatalogLoadOutcome::disposition` continue to
  report transient acquisition/cache provenance. The removed
  `CatalogAuthorityEvidence` and `authority_evidence()` names are not retained
  as compatibility aliases. The certified-cache outcome follows the same
  naming boundary with `snapshot_authority()` and diagnostic
  `cache_evidence()`. Read policies, atomic refresh, cache and failure behavior,
  schemas, network calls, and CLI surfaces are unchanged.

## [0.41.x] - 2026-08-21 - Mainnet Subnet Catalog routing authority

Detailed release notes: [docs/changelog/0.41.md](docs/changelog/0.41.md)

- `0.41.2` adds portable public constants for the Subnet Catalog's Registry
  keys and typed constructors for Subnet-list, legacy-routing, modern routing-
  shard, and Subnet-record subjects. A companion constructor builds ordinary
  pinned-value evidence with explicit requested/returned versions, Registry
  timestamp, endpoint, and inline/chunked representation while fixing the
  individual-read assurance to `UncertifiedQuery`. Downstream fixtures and
  custom sources no longer need to spell or assemble Registry keys manually.
  The helpers align with the existing fail-closed evidence rules; they do not
  relax catalog validation or fabricate records. No schema shape, routing-
  source rule, cache format, network call, or CLI surface changes.

- `0.41.1` fixes mainnet Subnet Catalog routing by reconstructing the complete
  pinned `canister_ranges_*` family used by current DFINITY Registry clients.
  The live collector requires nonempty modern authority and never reads the
  retired monolithic `routing_table`, so an empty family fails closed and a
  frozen deleted-Subnet route cannot create false catalog inconsistency.
  Complete bounded delta pagination reconstructs the pinned family; missing,
  malformed, shard-boundary-violating, overlapping, contradictory, or unknown-
  Subnet evidence fails closed, and large values retain bounded hash-verified
  chunk retrieval. Diagnostic replay may inspect pre-shard legacy history;
  certified promotion permits legacy routing only under the caller's explicit
  `AllowHistoricalTarget` policy, while `RequireLatestObserved` requires modern
  shards.
  Schema-1 catalog and report provenance is hard-cut in place to retain routing
  source plus every value's requested/returned versions, key/schema/subject,
  timestamp, endpoint, assurance, and transport representation. Existing
  detailed failures now retain the returned individual value version, failing
  endpoint and assurance, and all completed record evidence. Exact typed
  subjects and per-endpoint record completeness are validated, including the
  lower bound encoded by each shard key. No CLI surface is added. Detailed
  release notes include the downstream fixture, projection, custom-source, and
  cache-refresh adoption contract; ordinary load, refresh, and resolution call
  sites need no routing adapter, while direct schema literals and older
  schema-1 caches require the documented hard-cut update.

- `0.41.0` adds detailed synchronous, asynchronous, and caller-supplied-source
  Subnet Catalog load APIs under `subnet-catalog-host`. Typed failures retain
  requested authority policy, exact load stage and failure-side cache
  disposition, the pinned Registry version and offending Registry/Subnet/
  routing/endpoint/field subject when known, stable code/category,
  retryability including typed `Unknown`, and the original host error. Existing
  simple load APIs now map the same implementation back to their prior source
  errors. Managed-cache creation also tolerates concurrent creators without
  weakening symlink, file-type, or permission validation. No CLI, cache schema,
  network-call bound, or persistence behavior changes.

## [0.40.x] - 2026-08-15 - Portable NNS public-neuron distribution analytics

Detailed release notes: [docs/changelog/0.40.md](docs/changelog/0.40.md)

- `0.40.1` hard-cuts the portable proposal and public-neuron continuation
  states' cumulative row counters from target-width `usize` to fixed-width
  `u64`. Schema-1 continuations now keep the same numeric contract when native
  hosts exchange state with 32-bit Wasm consumers. Release documentation also
  records the dates of the tagged 0.38.0–0.40.0 releases and explicitly notes
  that the repository has no retained receipt for the 0.38 mainnet canister
  smoke requirement.

- `0.40.0` adds a pure portable distribution builder and retained-report
  validator over caller-supplied, API-exhausted public NNS neuron collections.
  Schema-1 reports retain collection and transport provenance, the false
  point-in-time guarantee, and row retrieval-time range; emit canonical raw-
  code state, optional visibility, and optional neuron-type counts with
  effective stake; and preserve reported/unreported coverage for optional
  staked maturity and current voting-power totals. Known-neuron metadata and
  Neurons' Fund join-timestamp presence remain factual counts. No crate,
  feature, dependency, network call, cache, stable-memory policy, or CLI
  surface is added.

## [0.39.x] - 2026-08-15 - Portable NNS proposal activity analytics

Detailed release notes: [docs/changelog/0.39.md](docs/changelog/0.39.md)

- `0.39.0` adds a pure portable activity builder over caller-supplied,
  API-exhausted NNS proposal collections. Schema-1 reports retain collection
  provenance and the false point-in-time guarantee, apply an optional
  half-open proposal-time window, and emit deterministic raw-code topic,
  decision-status, reward-status, and UTC-day counts. Partial collections and
  malformed retained rows fail with typed errors. Retained or deserialized
  reports can be passed to `validate_nns_proposal_activity_report` to recheck
  identity, source provenance, window accounting, included ranges, assurance,
  and canonical count tables; freshly built reports use the same validation
  path. No feature, dependency, schema version, cache, network call,
  persistence policy, or CLI surface is added.

## [0.38.x] - 2026-08-14 - Canister-native NNS Governance reporting

Detailed release notes: [docs/changelog/0.38.md](docs/changelog/0.38.md)

- `0.38.4` adds portable, serializable schema-1 continuation states for
  complete NNS proposal and public-neuron walks. Each advance validates the
  retained identity, concrete collector provenance, family cursor, lifecycle,
  and cumulative accounting before making exactly one bounded call, then
  returns that page with the next state. Explicit page ceilings distinguish
  incomplete progress from observed API exhaustion. Native proposal and
  neuron cache refreshes now use the same continuation engines; stable-memory
  layout, scheduling, retries, and atomic publication remain caller-owned.

- `0.38.3` adds canister-native bounded NNS neuron index pages and exact public
  neuron detail through the existing `canister` feature and
  `CanisterNnsSource`. Native, canister, and custom sources now share one async
  source, row conversion, validation, report assembly, and tagged Governance
  provenance contract. Complete neuron refresh and filesystem caches remain
  native-only; persistence and multi-page orchestration remain caller-owned.

- `0.38.2` adds canister-native bounded NNS proposal list and exact detail
  reports through the existing `canister` feature and `CanisterNnsSource`.
  Each report makes one replicated inter-canister call, shares validation and
  row projection with the native adapter, enforces the existing 8 MiB response
  ceiling, and records tagged transport provenance. Complete proposal refresh
  and persistence remain native and caller-owned respectively.

- `0.38.1` makes minor and major release staging include generated README and
  library-usage dependency versions. Release commits reject unstaged,
  untracked, and unexpected staged files, create a tag only after the commit
  leaves a clean tree, and retry one transient post-gate `cargo clean` failure.

- `0.38.0` adds one `canister` runtime feature inside `ic-query`, giving Rust
  canisters async access to the four bounded direct NNS Governance reports
  through replicated inter-canister calls and the same validation and assembly
  path as the native adapter. Direct Governance requests and schema-1 reports
  now use tagged transport-aware provenance; the canister adapter records the
  executing collector principal, bounds raw responses before decoding, and
  owns no scheduling, retries, or persistence. The workspace remains two
  crates, and `ic-query-cli` retains no feature matrix.

- `0.38.0` gives `cargo audit` a fresh disposable RustSec checkout on every
  dependency-gate run. Stale files in its shared advisory cache can no longer
  create duplicate-ID parse failures, and the exact temporary checkout is
  removed after both successful and failed checks.

## [0.37.x] - 2026-08-09 - ICRC account and holder index reporting

Detailed release notes: [docs/changelog/0.37.md](docs/changelog/0.37.md)

- `0.37.1` makes cache-backed SNS proposal list and detail reports enforce the
  mainnet-only contract before reading local snapshots. SNS neuron and proposal
  cache text now shares one list/status format: proposal lists expose complete
  snapshot evidence, missing numeric caches retain refresh guidance, refresh-
  attempt paths are not duplicated, and attempt details form a separate visual
  section. NNS neuron snapshot identity failures and malformed attempt sidecars
  are reported through distinct typed errors.

- `0.37.1` consolidates shared Dashboard request, principal, provenance, and
  text helpers; NNS Governance refresh-attempt lifecycle handling; snapshot
  identity loading; and SNS cache lookup, report, and rendering boundaries.
  Redundant dispatch wrappers, leaf modules, and historical-form tests are
  removed. CLI commands and options, JSON report fields, persisted schema-1
  contracts, cache paths, and network-call bounds are unchanged.

- `0.37.0` adds one-request official Dashboard ICRC `account list`, exact
  `account info`, and `holder list` reports. Pages accept at most 100 rows,
  expose reusable opaque forward/backward cursors without following them, and
  preserve raw indexed balances, transactions, update evidence, fee-collector
  details, holder percentages, and USD values. Upstream nanosecond creation
  times are exposed as Unix seconds plus a lossless subsecond-nanosecond
  remainder. Reports are off-chain, uncertified, uncached, and not complete or
  point-in-time ledger snapshots.
- Release bumps now update the `README.md` and library-usage dependency
  examples automatically after the clean-tree CI gate, so maintainers no
  longer need to pre-edit every documented minor version.

```bash
icq icrc analytics account list mxzaz-hqaaa-aaaar-qaada-cai --limit 25
icq icrc analytics account info mxzaz-hqaaa-aaaar-qaada-cai <account-id> --json
icq icrc analytics holder list mxzaz-hqaaa-aaaar-qaada-cai --sort-by=-balance
```

## [0.36.x] - 2026-08-09 - Node-provider reward reporting

Detailed release notes: [docs/changelog/0.36.md](docs/changelog/0.36.md)

- `0.36.0` adds live official Dashboard node-provider reward `info`, one-page
  `list`, and bounded aggregate `history` reports below
  `nns node-provider reward`. Records preserve raw ICP e8s, Unix-second reward
  and history timestamps, optional proposal/Registry/XDR evidence, raw
  Dashboard update time, and mode-specific JSON details. List never
  auto-pages and explicitly reports that adjacent upstream offset pages may
  overlap even with a pinned reward-index ceiling. All three operations are
  off-chain, uncertified, uncached, and make exactly one request.

```bash
icq nns node-provider reward info 7562 --json
icq nns node-provider reward list --limit 25
icq nns node-provider reward history --start 1752537600 --end 1784073600 --step 86400
```

## [0.35.x] - 2026-08-08 - CloudEngine Type4 node reporting

Detailed release notes: [docs/changelog/0.35.md](docs/changelog/0.35.md)

- `0.35.0` adds `cloud-engine node list` and `cloud-engine node info`. The
  list makes one bounded official Dashboard request with an explicit `Type4`
  reward filter and all four current status filters, optionally restricted to
  one node provider. Exact info makes one stable node-id lookup and requires
  the returned node to be Type4. Both preserve raw status, assignment,
  provider, operator, version, location, address, and hardware evidence under
  off-chain Dashboard provenance.
- The public Dashboard endpoint constant is hard-cut from
  `DEFAULT_CLOUD_ENGINE_PROVIDER_SOURCE_ENDPOINT` to
  `DEFAULT_CLOUD_ENGINE_DASHBOARD_SOURCE_ENDPOINT`, reflecting its shared use
  by provider and Type4 node reports. No compatibility alias is retained.

```bash
icq cloud-engine node list --node-provider-id <node-provider-id>
icq cloud-engine node info <node-id> --json
```

## [0.34.x] - 2026-08-08 - CloudEngine provider reporting

Detailed release notes: [docs/changelog/0.34.md](docs/changelog/0.34.md)

- `0.34.0` adds `cloud-engine provider list` and `cloud-engine provider info`.
  The list makes one bounded official Dashboard request for the complete node-
  provider resource, validates all rows, and retains providers carrying
  explicit CloudEngine counts or locations. Exact info preserves the full raw
  provider record and explicitly reports whether CloudEngine evidence is
  present. These off-chain reports are live, uncached, uncertified, and not
  point-in-time guaranteed.

```bash
icq cloud-engine provider list
icq cloud-engine provider info <node-provider-id> --json
```

## [0.33.x] - 2026-08-08 - Certified API boundary-node reporting

Detailed release notes: [docs/changelog/0.33.md](docs/changelog/0.33.md)

- `0.33.0` adds `ic api-boundary-node list`. One response-bounded mainnet
  `read_state` request authenticates the complete certified
  `api_boundary_nodes` subtree and reports node principals, domains, optional
  IPv4 addresses, required IPv6 addresses, and one shared certificate time.
  This is configured API boundary-node identity, not operational health,
  reachability, location, ownership, or HTTP-gateway inventory.

```bash
icq ic api-boundary-node list
icq ic api-boundary-node list --json
```

## [0.32.x] - 2026-08-08 - Bounded replica release reporting

Detailed release notes: [docs/changelog/0.32.md](docs/changelog/0.32.md)

- `0.32.0` adds one-request official Dashboard replica-version list and exact
  info reports. List pages default to 50 rows, cap at 100, expose explicit
  offset and proposal-index ceilings, and never auto-page; info preserves the
  full raw release summary and Dashboard-recorded Subnet rollout proposals.
  Reports remain off-chain, uncertified, uncached, and do not claim which
  binary a Subnet currently runs.

```bash
icq ic replica-version list --limit 25
icq ic replica-version list --offset 25 --max-proposal-index 438 --json
icq ic replica-version info e3d101b22ae3fa02aca737f9fb96cc6c4ca83ac3
```

## [0.31.x] - 2026-08-08 - CloudEngine control-plane reporting

Detailed release notes: [docs/changelog/0.31.md](docs/changelog/0.31.md)

- `0.31.1` hard-cuts the remaining active pre-1.0 schema identifiers to `1`
  across Registry-version and certified-delta reports, certified Registry
  archive manifests, ordinary Subnet Catalog caches, and Subnet Catalog
  list/info/refresh reports. Current shapes are unchanged. Content carrying
  another identifier is rejected without compatibility readers, migration, or
  automatic rewriting and must be explicitly refreshed or bootstrapped. CI now
  rejects active Rust schema constants above `1`.

- `0.31.1` adds `cloud-engine list`. It reads the complete Registry-classified
  CloudEngine Subnet inventory from the existing Subnet Catalog cache policy,
  then makes one exact public control-plane operator-binding query per row, up
  to 100. Registry and control-plane authority remain separate, and an absent
  binding remains distinct from a per-row lookup failure. Operator details stay
  in `cloud-engine info`.

```bash
icq --network ic cloud-engine list
icq --network ic cloud-engine list --json
```

- `0.31.1` no longer requires maintainers to replace a matching detailed
  changelog heading's `Unreleased` suffix before running a version bump. The
  release gate still requires the target version in the committed root ledger
  and detailed changelog, plus matching dependency-example minor versions.

- `0.31.0` adds the top-level, mainnet-only `cloud-engine` family. `info`
  resolves one Subnet and, when an operator binding is returned, fetches four
  public details in a five-call bounded report; `prices` fetches the network
  fee and at most 1,000 public marketplace rows in exactly two calls. Both
  reports are live and uncached, preserve raw JSON values and explicit
  endpoint/query provenance, and state that ordinary sequential query
  responses are neither certified nor one point-in-time snapshot. The pure
  library surface is available without host dependencies, while
  `cloud-engine-host` adds the focused native adapter and is included by
  `host`.

```bash
icq --network ic cloud-engine info 2nl67-oqoc5-cmocj-otlhq-kr2kr-53hov-drrds-7ihcs-fhomv-2eyvu-6qe
icq --network ic cloud-engine prices --json
```

## [0.30.x] - 2026-08-05 - Certified Registry evidence

Detailed release notes: [docs/changelog/0.30.md](docs/changelog/0.30.md)

- `0.30.26` consolidates canonical compact-JSON byte counting and comparison,
  archive digest writers, and serialization-to-IO error conversion across the
  certified Registry archive and certified Subnet Catalog cache. Confined
  cache reads and directory discovery/removal now have separate internal
  owners, inherited feature gates no longer repeat broader host features, and
  cache status rejects ambiguous headers containing both supported schema
  version fields. The Registry replay tests are organized by subsystem. Public
  Rust and CLI APIs, cache schemas and paths, dependencies, and network
  behavior are unchanged.

- `0.30.25` adds the focused `certified-subnet-catalog-host` feature for
  certified Registry archive/replay and archive-bound Subnet Catalog authority
  without the broader NNS host surface or ic-query's direct Dashboard Reqwest
  edge. Certified cache access is hard-cut to one explicit-policy
  `load_nns_certified_subnet_catalog` flow returning an authority-bearing load
  outcome; the four 0.30.24 load/publication functions and publication request
  are removed without aliases. Shared managed-file reads are bounded, atomic
  JSON cache publication streams after serialization preflight, and refresh
  locks and attempt sidecars reject content above 64 KiB and 1 MiB. Cache
  schemas, paths, network behavior, and CLI surface are unchanged.

- `0.30.24` finishes the certified Subnet Catalog cache boundary with
  explicitly named local missing and missing-or-invalid publication policies,
  observable cache dispositions, and compact persistable authority evidence.
  Recovery reuses only the caller-supplied authenticated archive: it does not
  hide a network call, refresh stale archive evidence, replace filesystem
  failures, or change the schema-1 cache. Cache-only load and force publication
  remain distinct operations.

- `0.30.23` adds a separate versioned cache envelope for certified Subnet
  Catalog projections. Publication first qualifies a fully authenticated
  Registry archive, then writes the canonical envelope atomically under its own
  confined lock and caller-selected byte ceiling. Cache-only loading is local,
  never repairs or refreshes, and returns authority only after the complete
  envelope exactly matches a fresh projection from the supplied authenticated
  archive. Cached `Certified` text therefore cannot authenticate itself.

- `0.30.22` adds explicit bounded cleanup for unreferenced certified Registry
  archive objects. Cleanup holds the archive lock, fully reauthenticates the
  retained manifest and objects, scans only the exact flat `objects/`
  directory, and applies caller-selected scan/count/byte ceilings before the
  first deletion. Referenced objects are never selected, successful removals
  synchronize their parent directory, and partial filesystem failures report
  exact completed counts. Cleanup is local-only and never automatic; no CLI,
  schema, path, refresh, or read-through behavior changes. Release bump targets
  now retain Cargo build artifacts when the changelog, clean-worktree, CI, or
  version-update path fails and run `cargo clean` only after a successful bump.

- `0.30.21` adds explicit live incremental refresh for an existing certified
  Registry archive. Refresh rejects non-mainnet and missing archives before
  source work, holds the archive lock across bounded local reauthentication,
  collection, and atomic publication, and reserves cumulative replay and
  storage capacity before every call. It publishes one complete successor
  segment, including an authenticated unchanged-version observation, while
  preserving the prior manifest on failure. No read-through policy, cleanup,
  default path, migration, or CLI surface is added.

- `0.30.20` hard-cuts certified Registry archives to segmented manifest schema
  2 and adds local resumable publication. Each batch records its segment and
  target; a completed archive can retain either a higher exact target or a
  fresh authenticated unchanged-version observation. Resume reauthenticates
  the complete prior archive under explicit cumulative limits, rewrites no
  historical objects, makes no source call, and preserves the prior manifest
  on failure. Schema-1 archives require an explicit new force bootstrap; no
  migration, fallback, default path, live incremental coordinator, or CLI
  surface is added.

- `0.30.19` adds an explicit lock-guarded live force bootstrap for the
  certified Registry archive. Callers supply the source/time/replay policy,
  confined roots, storage ceilings, and lock staleness; non-mainnet fails
  before filesystem or source work. Every bounded batch is locally
  reauthenticated before durable admission, and the manifest is replaced only
  after exact-target completion, preserving any prior complete archive on
  failure. This always starts at version zero and adds no read-through policy,
  incremental refresh, cleanup, default path, or CLI surface.

- `0.30.18` hard-cuts certified Subnet Catalog projection to require an explicit
  `NnsCertifiedSubnetCatalogProjectionRequest` with caller-owned identity,
  future-skew, observation-time, maximum certificate-age, and pinned-version
  policy. Promotion now fails before catalog projection when evidence is too
  old or a later archive batch observed a newer Registry version under the
  require-latest policy. Successful authority retains exact age and version
  evidence through `NnsCertifiedSubnetCatalogFreshness`. There is no default
  age, source call, cache behavior, or CLI change.

- `0.30.17` hard-cuts the Subnet Catalog to schema 3 and enables certified
  catalog authority only from a fully reauthenticated
  `NnsAuthenticatedRegistryArchive`. The new archive-bound projection checks
  manifest/replay identity, version, schema, accounting, endpoints,
  certificate-time bounds, root key, evidence-chain digest, and complete-state
  digest before reusing the existing canonical catalog validation path. Its
  result keeps the `ValidatedSubnetCatalog` attached to the qualifying archive;
  serialized raw catalogs still cannot self-assert `Certified`. Schema-2
  catalogs and the earlier authenticated-session projection API are removed
  without migration or aliases. Existing ordinary schema-2 catalog caches are
  invalid: cache-only reads fail locally, while an already-authorized
  refresh-capable read policy makes a live Registry query and replaces the
  cache with schema 3. The archive-bound projection itself adds no cache
  publication, refresh policy, network call, or CLI surface.

- `0.30.16` adds explicit confined publication and bounded restoration for the
  certified Registry archive. A streaming publisher writes each canonical
  report once as an owner-only content-addressed object, syncs files and newly
  created directories, and atomically replaces the manifest only after complete
  authenticated replay. Loading bounds and hashes one object at a time,
  reauthenticates all retained evidence locally, and requires an exact recomputed
  manifest. Missing, oversized, noncanonical, unconfined, symlinked, or tampered
  content fails closed; failed final publication preserves the prior manifest.
  There is no default path, refresh policy, network call, CLI, or certified
  catalog promotion.

- `0.30.15` adds a versioned, bounded certified Registry archive-manifest
  contract under `nns-host`. Its builder accepts only locally authenticated
  retained batches, streams canonical report JSON through SHA-256 and byte
  accounting without another encoded copy, enforces explicit batch/per-report/
  total ceilings before replay publication, and emits a manifest only after
  exact-target completion. Pure validation rejects identity, schema, ordering,
  continuity, digest, endpoint, and accounting inconsistencies. Manifests remain
  untrusted indexes until every report is reauthenticated and replayed; this
  adds no filesystem storage, cache, network call, CLI, or certified catalog
  assurance.

- `0.30.14` makes the shared `patch`, `minor`, and `major` release gate run
  `cargo clean` after CI exits, including failed CI runs, while preserving the
  gate's original status if cleanup itself fails. CI guards now use exact,
  named, trapped temporary paths; feature-boundary logs share one owned
  directory and are removed together on success or failure.

- `0.30.13` adds a bounded in-memory replay builder that accepts only sealed,
  locally reauthenticated Registry delta batches. It starts at version zero,
  preserves the existing exact-target, cumulative-limit, provenance, and
  atomic-application contracts, exposes read-only progress, and returns the
  same sealed authenticated replay-session type as live bootstrap only after
  complete reconstruction. It performs no network or filesystem IO and adds
  no archive format, cache, CLI, or certified catalog assurance.

- `0.30.12` adds local reauthentication for retained schema-3 certified
  Registry delta reports. The operation validates the report, verifies its raw
  certificate and mixed-tree commitment against the built-in mainnet root key,
  decodes the committed delta, and compares every version, mutation,
  precondition, and chunk reference before returning a sealed borrowed
  capability. The source endpoint is validated but never called. This adds no
  archive, cache, replay restoration, CLI, or certified catalog assurance.

- `0.30.11` hard-cuts certified Registry delta reports to schema 3 and
  retains each unique hash-verified large-value chunk once in canonical digest
  order. The pure validator now re-hashes every retained chunk, requires the
  table to match the exact referenced digest set, recomputes retained-byte
  accounting, and reconstructs every chunked mutation from its ordered
  evidence. Live calls and ceilings are unchanged. Replay evidence-chain
  digests consequently commit to the stronger report; state digests are
  unchanged. This adds no cache, CLI, or certified catalog assurance.

- `0.30.10` hard-cuts the built-in certified Registry bootstrap to return a
  sealed `NnsAuthenticatedRegistryReplaySession`. Only ic-query's fixed live
  path, which verifies every batch against the mainnet root key, can construct
  it; custom sources and manual replay continue to return ordinary sessions.
  An authenticated catalog-projection wrapper composes the existing projection
  without copying rows or provenance. Callers now use `.replay_session()` to
  inspect the live-bootstrap result or `.into_replay_session()` to explicitly
  discard the capability. This remains in-memory evidence and does not enable
  cached or validated certified catalog assurance.

- `0.30.9` adds a pure, cacheless Subnet Catalog projection from a completed
  exact-target Registry replay session. It decodes the replayed Subnet list,
  routing table, and every referenced Subnet record, then uses the same
  canonical ordering, IC-native classification policy, and routing validation
  as the live catalog. The projection borrows its replay session so version and
  provenance commitments remain attached. Incomplete state and missing,
  malformed, or structurally invalid records fail with typed errors. This does
  not serialize a mirror, publish a catalog, or enable certified assurance.

- `0.30.8` extends exact-target replay sessions with compact provenance needed
  before any certified projection. Each admitted validated report advances a
  domain-separated SHA-256 evidence chain without buffering another full
  report; sessions also retain distinct source endpoint strings in canonical
  order and certificate-time bounds. A separate canonical digest of keys,
  values, mutation positions, and timestamps appears only after the pinned
  target is complete. One public schema constant versions both commitments.
  Failed batches publish none of this candidate provenance. The commitments
  are not serialized, cached, or standalone authentication, and certified
  Subnet Catalog assurance remains unsupported.

- `0.30.7` aligns certified replay with two additional committed Registry
  history rules proven by a complete bounded mainnet reconstruction. Repeated
  keys inside one atomic version are preserved and applied in their stable,
  canonical key order; retained value content on a committed delete remains
  raw evidence but is ignored when rebuilding current state. Decreasing key
  order still fails closed. A cacheless probe reconstructed exact Registry
  target `62948` in 75 batches and 77 calls using 63,814,080 encoded response
  bytes and 22,345,176 bytes of current state; these measurements are evidence,
  not new defaults.

- `0.30.6` adds a bounded diagnostic Registry bootstrap probe. It shares the
  complete bootstrap's pre-call reservation and validation loop but returns a
  typed `Complete` or `CapacityReached` outcome with the accumulated session,
  allowing current-history sizing without another unreserved call. A zero-call
  budget returns explicit empty progress. The complete bootstrap remains
  complete-only and converts capacity exhaustion to its existing typed error;
  probe state is explicitly incomplete, uncached, and not catalog authority.
  Certified replay now also preserves historical non-delete mutations whose
  empty legacy protobuf value is absent on the wire, matching the official
  Registry transport's empty-inline-value interpretation.

- `0.30.5` adds an explicit caller-runtime async Registry bootstrap from
  version zero. Before every built-in source call it reserves capacity for one
  certified query, up to 64 chunk queries, and up to 40 MiB of encoded
  responses; insufficient remaining batch, call, or byte capacity stops before
  the call. The first response pins the exact target and only a complete
  session is returned. Limits have no defaults, custom sources remain
  responsible for their internal work, and no cache, CLI, or certified catalog
  publication is added.

- `0.30.4` adds a pure exact-target Registry replay session. Its first valid
  certified batch pins the selected Registry version; later batches must still
  certify that target and may not move it when the live Registry advances.
  Explicit ceilings bound admitted Registry versions, batches, reported query
  calls, encoded response bytes, and reconstructed state. Root-key changes,
  cumulative-limit failures, and completed-session reuse fail atomically. The
  session performs no source calls and does not yet publish certified catalog
  authority.

- `0.30.3` adds a pure in-memory Registry replay API that applies exactly one
  validated certified delta batch after the state's current version. It uses
  the IC's committed-changelog replacement/delete semantics, requires explicit
  caller ceilings for live entries and combined raw key/value bytes, and
  publishes the candidate state only after the whole batch succeeds. The state
  begins at version zero and carries current values and mutation positions, but
  is not serialized authority evidence. No network loop, cache, CLI, catalog
  projection, or assurance promotion is added.

- `0.30.2` hard-cuts certified Registry delta reports to schema 2 and completes
  certified large-value chunk references under fixed call, chunk, per-value,
  total-value, response-byte, and agent-body ceilings. Reports preserve the
  original absent/inline/chunked encoding, ordered SHA-256 references,
  reconstructed value bytes, and separate certified/chunk/total accounting;
  the pure validator recomputes those invariants. Repeated digests reuse one
  verified chunk response. The shared exact-version Registry value path now
  applies the same bounded hash-verified reconstruction. This adds no cache,
  replay loop, CLI surface, or certified Subnet Catalog claim.

- `0.30.1` adds a mainnet-only caller-runtime async library operation for one
  certified `get_certified_changes_since` batch. It authenticates the shared
  latest-version and delta witness, requires contiguous eight-byte version
  labels, decodes native atomic mutations, preserves raw evidence and exact
  accounting, and fails closed on malformed, oversized, unknown, conflicting,
  or incomplete content. A pure validator recomputes structural report
  invariants for trusted custom sources; cryptographic authentication remains
  the source's responsibility. The operation is one-call, uncached,
  non-paginating, has no CLI surface, and does not promote Subnet Catalog
  assurance.

- `0.30.0` hard-cuts the existing NNS Registry version report and source
  contract to one authenticated `get_certified_latest_version` call. The host
  verifies the mainnet-root-key certificate, Registry canister authority,
  certified-data commitment, bounded protobuf mixed hash tree, canonical
  unsigned-LEB128 version leaf, and certificate time before returning version
  2 evidence. JSON preserves the certificate, witness, root-key digest, time,
  and byte counts; text stays compact. This certifies only the latest version,
  not ordinary Subnet Catalog `get_value` reads. Command grammar and caching
  are unchanged; `nns-host` now has a direct CBOR dependency.

## [0.29.x] - 2026-08-05 - Subnet Catalog authority and embedder hardening

Detailed release notes: [docs/changelog/0.29.md](docs/changelog/0.29.md)

- `0.29.8` completes the family-level library feature split with `nns-host`
  for the complete NNS governance, Registry inventory, component-cache, and
  topology surface, plus `cmc-host` for certified Cycles Minting Canister
  reports. `nns-host` includes `nns-topology-host`; `host` remains the complete
  convenience union. APIs, cache schemas and paths, calls, CLI behavior, and
  default features are unchanged.

- `0.29.7` adds the independent `sns-host` library feature for native SNS
  discovery, targeted reports, complete proposal/neuron caches, reward
  checkpoints, and local reward diffs. It excludes Dashboard, Registry, NNS,
  system-canister, and native ICRC host adapters and has no direct ic-query
  Reqwest, Prost, CBOR, or SHA-256 edge; Reqwest, CBOR, and cryptographic
  packages remain transitive through `ic-agent`. APIs, schemas, paths, calls,
  and CLI behavior are unchanged.

- `0.29.6` adds the independent `icrc-host` library feature for native ICRC
  ledger/index queries, certified-tip verification, and complete
  account-history caches. It excludes Dashboard, Registry, NNS, and SNS host
  adapters and has no direct ic-query Reqwest or Prost edge; Reqwest remains
  transitive through `ic-agent`. APIs, schemas, paths, calls, and CLI behavior
  are unchanged.

- `0.29.5` adds the independent `dashboard-host` library feature for official
  Dashboard REST reports and the confined observed node-status cache. It
  excludes `ic-agent`, Registry protobufs, CBOR certification, and native
  NNS/SNS/ICRC host adapters while `host` remains the complete union. Report
  and cache schemas, paths, live calls, and CLI behavior are unchanged.

- `0.29.4` adds the focused `nns-topology-host` library feature as a strict
  superset of `subnet-catalog-host`. It exposes the exact-version joined NNS
  Subnet/node/operator/provider live and cache API without enabling ic-query's
  direct Dashboard Reqwest or CBOR certification edges; those packages may
  remain transitive through `ic-agent`. Broader independently cached topology
  summaries remain under `host`. Rust report shapes, cache schemas and paths,
  live call behavior, and CLI grammar are unchanged.

- `0.29.3` closes the authority-bearing route API by returning the matched
  validated Subnet classification with each Canister route. Catalog loads can
  require a minimum assurance, reject known-insufficient refresh selections
  before collection, and emit compact persistable authority evidence; a
  convenience constructor covers explicit missing/invalid/stale refresh.
  Defaults, cache schemas and paths, CLI grammar, and ordinary one-endpoint
  call behavior are unchanged. Release automation now runs its complete CI
  gate once before changing version files and does not repeat it after commit
  and tag creation.

- `0.29.2` hard-cuts the Subnet Catalog host API and version-2 cache/report
  evidence to caller-runtime async load/refresh operations with synchronous
  adapters, exact Registry query-call counts, and explicit one-endpoint or
  bounded two-to-three-endpoint source selection. Agreement requires the same
  Registry version and canonical Registry payload from distinct hostnames;
  mismatch never falls back to one endpoint. CLI grammar remains unchanged
  and continues to select one endpoint. Version-1 catalog evidence is not
  migrated; authorized invalid-content read-through replaces it, while
  cache-only callers must refresh explicitly.

- `0.29.1` hard-cuts every managed cache family to capability-rooted load,
  discovery, refresh-lock, and atomic-publication operations. On Unix it
  rejects symlinks, path escapes, nonregular managed files, permissive
  directories, and files not using mode `0600`; new directories and files use
  `0700` and `0600`. Cache paths, schemas, network behavior, and CLI grammar are
  unchanged. Older permissive cache trees are not migrated or repaired
  automatically and must be removed or secured before use.

- `0.29.0` hard-cuts the Subnet Catalog Rust and persisted-report contracts to
  separate raw JSON from validated evidence, label current single-endpoint
  Registry collection `uncertified_query`, retain raw Registry Subnet types,
  bind routes to exact provenance and a canonical digest, and make every cache
  refresh policy and resulting disposition explicit. It also adds a
  caller-runtime async fetch, structured library error classification, and
  refresh/list/info provenance without changing CLI grammar or live call
  counts. Existing catalog caches use the replaced version-1 shape: authorized
  read-through repairs them, while cache-only callers must explicitly refresh.

## [0.28.x] - 2026-08-04 - observed IC node and Subnet status

Detailed release notes: [docs/changelog/0.28.md](docs/changelog/0.28.md)

- `0.28.5` makes `HostCacheError` the single public owner of generic JSON
  cache failures for observed node status and SNS reporting. Family errors
  retain only missing-cache guidance and semantic/identity failures; cache
  schemas, refresh policy, commands, and report output are unchanged.

- `0.28.4` consolidates official Dashboard adapter invariants without changing
  reports: canister page cursors now have one normalization path for public
  builders and direct live-source calls, shared principal validation lives at
  the common source boundary, and bounded metric/ICRC series use one inclusive
  observation-count rule. Commands, JSON, text, calls, and caches are
  unchanged.

- `0.28.3` consolidates the observed-status projection internals without
  changing reports: Subnet aggregates are constructed once before view
  filtering, node/Subnet/provider selection and non-up evidence reuse shared
  helpers, and the redundant status-only counting flow is removed. Every
  bounded Dashboard time-series request also shares one collection-time bound.
  Commands, JSON, text, calls, and cache behavior are unchanged.

- `0.28.2` adds typed less/equal/greater comparisons of each provider's
  unassigned versus assigned up and conservative non-up node counts. Reports
  include all-provider comparison totals and compact text labels, while the
  projection now constructs each provider aggregate once; calls and cache
  identity are unchanged.

- `0.28.1` separates status-report provenance and summaries from their tables
  with a blank line. It also rejects empty live/custom node snapshots,
  noncanonical cached row order, and semantically invalid rows supplied to pure
  projections; existing read-through policies visibly replace recoverably
  invalid caches.

- `0.28.0` adds node-, Subnet-, and node-provider operational views over one
  bounded official Dashboard `/nodes` response. The views share a 60-second
  atomic cache, preserve raw status and assignment evidence, expose explicit
  uncertified/default-scope provenance, and make no per-row follow-up calls.
  Subnet rows keep down-only and conservative non-up fault-distance evidence
  separate, while provider rows retain status-by-assignment comparisons;
  missing, invalid, or stale cache content visibly refreshes.

- The component-cache consistency command and corresponding public topology
  report API are hard-cut from `health` to `check`, avoiding confusion with
  observed machine status. There is no compatibility command, alias, wrapper,
  or cache migration.

```bash
icq nns node status
icq nns subnet status tdb26 --all
icq nns node-provider status --json
icq nns topology check
```

## [0.27.x] - 2026-08-04 - bounded official ICRC analytics

Detailed release notes: [docs/changelog/0.27.md](docs/changelog/0.27.md)

- `0.27.5` makes every live base endpoint credential-, query-, and
  fragment-free before transport construction, disables redirects for official
  Dashboard requests, and caps every native `ic-agent` response at 8 MiB.
  Valid endpoints, call counts, report schemas, and cache behavior are
  unchanged.

- `0.27.4` gives every official Dashboard live adapter one shared 8 MiB
  response-body ceiling. Declared and streamed body sizes are checked before
  JSON decoding, and oversized or incomplete bodies now return distinct
  typed host errors without changing live-only or cache behavior.

- `0.27.3` adds a one-request bounded token-value series with a 24-hour,
  1,000-row default and a hard 90-day, 1,000-row ceiling. Reports preserve
  nullable raw price and 24-hour-volume fields, each external provider and
  URL, exact Dashboard provenance, and explicit possible truncation when the
  requested limit is reached.

```bash
icq icrc analytics token-values mxzaz-hqaaa-aaaar-qaada-cai --limit 100 --json
```

- `0.27.2` adds one-request indexed account and transaction counts alongside
  holder count. All three scalar endpoints now share one typed count kind,
  request, report, source method, builder, renderer, live URL path, and CLI
  dispatch flow; none requests rows, follows a cursor, or creates a cache.

```bash
icq icrc analytics account count mxzaz-hqaaa-aaaar-qaada-cai
icq icrc analytics transaction count mxzaz-hqaaa-aaaar-qaada-cai --json
```

- `0.27.1` adds a one-request official ICRC holder count without requesting or
  enumerating holder rows. Total-supply and holder-count reports now share one
  public analytics ledger target, one CLI target parser, one `LiveIcSource`
  capability, and the existing non-certified Dashboard provenance contract.

```bash
icq icrc analytics holder count mxzaz-hqaaa-aaaar-qaada-cai
```

- `0.27.0` adds a live-only, one-request historical total-supply series for
  one ledger indexed by the official IC Dashboard ICRC analytics API. Public
  requests, reports, renderers, fixture-source builders, and the live adapter
  preserve exact bounds, raw base-unit strings, and explicit non-certified
  Dashboard provenance. Daily queries default to 30 days and all queries are
  capped at 1,000 requested and returned observations; the operation performs
  no enumeration, pagination, follow-up calls, or caching.

```bash
icq icrc analytics total-supply mxzaz-hqaaa-aaaar-qaada-cai
```

## [0.26.x] - 2026-08-03 - SNS maturity reward evidence

Detailed release notes: [docs/changelog/0.26.md](docs/changelog/0.26.md)

- `0.26.21` makes `cache status` distinguish generic header integrity from
  cache age and shows each canonical family's automatic, explicit, or
  missing-only invalid-content recovery policy. The bounded local inventory
  still performs no family-specific semantic validation, full history scan,
  network call, or mutation. Maintained user, library, adapter, and roadmap
  documentation is refreshed against the current CLI and schema contracts.

```bash
icq cache status
```

- `0.26.20` extends recoverable invalid-content refresh from the SNS catalog to
  bounded Subnet and NNS inventory caches and to explicit topology and ICRC
  account-history read-through APIs. Validated replacements remain atomic;
  failed refreshes preserve the original file, cache-only and status operations
  stay local and strict, NNS inventory custom sources are validated before
  publication, and complete Governance histories retain explicit recovery.

- `0.26.19` adds typed native SNS proposal actions and votes, lifecycle-aware
  SNS catalog rows and filtering, terminal-aware metadata health, and shared
  human-readable cycle and byte quantities. `sns list` also atomically repairs
  malformed, incompatible, or invalid joined catalogs. Raw codes, unknown
  evidence, canonical JSON labels, and exact JSON quantities remain lossless.

```bash
icq sns list --all
```

- `0.26.18` replaces duplicated NNS and SNS report provenance strings with
  shared `ReportDataSource` and `ReportResultScope` classifications. Live,
  cache, bounded-live, and complete-cache assembly, validation, rendering, and
  JSON now consume one typed vocabulary without changing output labels.

- `0.26.17` replaces free-form NNS neuron state, visibility, type, and
  recent-ballot vote labels with typed native classifications. Unknown numeric
  codes and omitted optional values remain distinct and lossless; live rows,
  custom sources, and caches now enforce agreement between raw codes and their
  classifications without changing JSON or CLI labels.

- `0.26.17` adds explicit `reported_zero`, `reported_nonzero`, and
  `unavailable` cycle-balance evidence to SNS Root canister reports. A failed
  health query now preserves the discovered inventory with a typed query gap
  instead of discarding it or conflating unavailable balances with zero.

- `0.26.17` hard-cuts every remaining current schema identifier back to `1`,
  including cache-status reports, refresh locks, SNS list/neuron reports and
  neuron snapshots, and NNS topology health. There are no version-2 readers or
  migrations; refresh version-2 SNS neuron snapshots, and verify any lingering
  version-2 refresh lock before removing it manually.

- `0.26.16` replaces free-form NNS proposal topic and ballot-vote labels with
  `NnsProposalTopic` and `NnsProposalVote`. Projection, topic filtering,
  sorting, rendering, and caches now use typed native classifications, and the
  redundant numeric-to-label module is removed without changing JSON or CLI
  labels.

- `0.26.15` replaces free-form NNS proposal decision-status and reward-status
  labels with `NnsProposalStatus` and `NnsProposalRewardStatus`. Native
  projection, filters, sorting, rendering, and caches now share one code/label
  vocabulary while existing JSON and CLI labels remain unchanged.

- `0.26.14` completes `SnsCanisterMethod` coverage for all sixteen fixed
  SNS-native method names used by the live adapters. Discovery, metadata,
  parameters, proposals, neurons, and reward collection now share the same
  typed vocabulary as metrics, Root, swap, and upgrade; the ICRC-106 ledger
  method remains protocol-owned and raw.

- `0.26.13` replaces eighteen free-form native SNS method fields across
  metrics, Root inventory and health, swap, upgrade, and partial-query gaps
  with `SnsCanisterMethod`. Live calls, source validation, reports, gaps, and
  rendering now share the exact native method labels while existing JSON and
  CLI output remain unchanged.

- `0.26.12` replaces free-form SNS metrics and Root-health call-type strings
  with `SnsCanisterCallType`. Source validation, live conversion, reports, and
  rendering now share one typed invocation vocabulary while existing JSON and
  CLI labels remain unchanged.

- `0.26.11` replaces free-form NNS topology-gap classifications and duplicate
  ICRC/SNS ledger-metadata value types with typed domain enums. Gap construction,
  native metadata conversion, ordering, and rendering now consume closed
  vocabularies while existing JSON and CLI labels remain unchanged.

- `0.26.10` replaces free-form ICRC capability status and NNS node Subnet-kind
  strings with typed domain enums. Capability diagnostics, Registry
  projection, node filters, topology counts, and text rendering now consume
  typed classifications while existing JSON, cache, and CLI labels remain
  unchanged.

- `0.26.9` replaces the free-form SNS proposal `decision_state` string with
  `SnsProposalDecisionState`. Live projection, cached filtering, and lifecycle
  sorting now share the typed state while existing cache JSON and CLI labels
  remain unchanged.

- `0.26.8` replaces free-form component-topology assessment, capacity, and
  provider status strings with three public domain-specific enums. Existing
  JSON labels and CLI text are unchanged; typed construction removes invalid
  states and string-based sort fallbacks.

- `0.26.7` consolidates NNS, SNS, and ICRC complete-collection evidence into
  one public `CacheCollectionCompleteness` DTO and validator. Serialized cache
  shapes remain unchanged; the duplicate ICRC-specific type and status
  constant are removed.

- `0.26.6` replaces free-form NNS, SNS, and ICRC refresh-attempt lifecycle
  fields with `CacheRefreshAttemptStatus::{Running, Complete, Failed}`. Writers
  now accept only those states, while existing sidecar/report JSON and CLI text
  labels remain unchanged.

- `0.26.5` replaces the remaining free-form complete-cache validation strings
  in NNS neuron/proposal, SNS neuron/proposal, and ICRC account-history
  summaries with one shared `CacheValidationStatus::{Valid, Invalid}` Rust
  enum. Existing `ok`/`invalid` JSON labels and CLI text are unchanged.

- `0.26.4` replaces free-form global cache and refresh-lock status strings with
  typed Rust enums while preserving the existing JSON labels and CLI output.
  Cache discovery, generic-header parsing, and lock projection now have
  separate internal owners; cache and network behavior is unchanged.

- `0.26.3` extends local-only `cache status` with active, stale, and invalid
  refresh-lock evidence. Refresh locks now record the stale threshold chosen by
  their owner, and competing refreshes honor that recorded policy. NNS
  Registry leaf and Subnet caches now share the `nns/<network>/...` layout used
  by the other domain-scoped caches; typed loaders neither migrate nor reuse
  the replaced paths.

- `0.26.2` adds local-only `cache status`, which inventories known complete
  caches across networks, reports file age and size, and distinguishes managed
  `fresh`/`stale` policies from readable `unmanaged` caches and invalid files.
  It never refreshes or removes cache state.

- `sns list` now reuses one atomic joined discovery catalog for one hour,
  avoiding a repeated metadata fan-out on consecutive calls. `sns refresh`
  forces an explicit replacement; targeted SNS commands retain their bounded
  targeted discovery calls instead of refreshing the all-SNS catalog.

- NNS topology health schema 2 no longer treats sources without an age policy
  as healthy freshness evidence. It reports their count explicitly and marks
  the cache-freshness check non-OK until every source age is assessable.

```bash
icq cache status
icq cache status --json
icq sns refresh
icq sns list
```

- `0.26.1` replaces the abbreviated `sns params` command with
  `sns parameters` as a pre-1.0 hard cut with no alias. Nested Canister,
  neuron, proposal, and reward commands retain the consistent
  family-operation-identifiers grammar.

- Command namespaces without a selected operation now render the same complete
  local help as their explicit `help` subcommand. Every CLI `Commands` section
  is alphabetized, including the generated `help` entry.

```bash
icq sns
icq sns info 1
icq sns parameters 1
icq sns canister list 1
icq sns neuron list 1
icq sns proposal info 1 387
icq sns reward
icq sns reward checkpoint 1 --json
```

- `0.26.0` adds exact live SNS neuron detail and API-exhausted reward
  checkpoints without enlarging the fixed-size neuron list/cache schema. The
  reports preserve native neuron identifiers, permissions, followees,
  maturity state, pending disbursement accounts, complete Governance brackets,
  strict pagination evidence, collection bounds, and typed policy findings.

- It also adds a pure no-host checkpoint validator and local-only reward diff.
  Loaded files are treated as untrusted evidence; allocation is valid only
  when stable target/event evidence, policy observations, non-negative joined
  deltas, and native distributed maturity reconcile exactly. Zero distribution
  and failed invariants remain typed outcomes.

```bash
icq sns neuron info 1 000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f --json
icq sns reward checkpoint 1 --max-pages 10 --json
icq sns reward diff before-checkpoint.json after-checkpoint.json --json
```

## [0.25.x] - 2026-08-01 - Fuller fixed-size SNS neuron evidence

Detailed release notes: [docs/changelog/0.25.md](docs/changelog/0.25.md)

- `0.25.34` separates official Dashboard canister, metric, and bounded
  network-resource request contracts behind the existing IC model facade.
  Public Rust paths, request fields, and behavior are unchanged.

- `0.25.33` separates NNS Governance economics, cached-metrics, and
  reward/modulation text rendering behind the existing facade. Public
  functions and exact human-readable output are unchanged.

- `0.25.32` separates NNS Governance economics, cached-metrics, and
  reward/modulation model families behind the existing explicit facade.
  Public Rust paths, native response shapes, and behavior are unchanged.

- `0.25.31` separates NNS neuron source-independent pagination, provenance,
  and report assembly from the built-in Governance wire adapter. Public Rust
  paths, source calls, cache contracts, and behavior are unchanged.

- `0.25.30` separates ICRC request and report contracts into account,
  indexed account-history, and ledger-wide owners behind the existing explicit
  facades. Public Rust paths, JSON contracts, and behavior are unchanged.

- `0.25.29` separates the NNS proposal model into request, serialized
  report/row, and filter/sort vocabulary owners behind the existing facade.
  Public Rust paths, JSON contracts, and behavior are unchanged.

- `0.25.28` gives SNS proposal and neuron pagination one resolved refresh
  context for request, source, target, attempt, progress, page-cap, incomplete,
  and exhaustion policy. Family-specific cursors, validation, and source calls
  remain separate; refresh behavior is unchanged.

- `0.25.27` centralizes the resolved SNS refresh-attempt lifecycle and reuses
  the shared complete paged-collection result for proposal and neuron
  refreshes, removing duplicate locked adapters and intermediate DTOs.
  Refresh behavior is unchanged.

- `0.25.26` centralizes strict SNS cache loading by stable list id and Root
  canister principal, removing duplicate proposal/neuron lookup flows and the
  redundant neuron lookup-error module. Typed lookup behavior is unchanged.

- `0.25.25` drives SNS cache-list and cache-status reports directly from the
  collection storage marker, removing the duplicate proposal/neuron status
  adapters and remaining list callbacks. Report behavior is unchanged.

- `0.25.24` centralizes complete SNS cache loading and validation behind the
  collection storage contract, deleting the proposal/neuron load modules and
  loader plumbing. Cache errors, reports, and behavior are unchanged.

- `0.25.23` makes each SNS collection marker own its cache schema, field
  allowlist, and missing-cache error, replacing the enum selector and duplicate
  proposal/neuron scan modules. Cache and report behavior are unchanged.

- `0.25.22` replaces duplicated proposal and neuron cache-summary modules and
  their projection macro with one typed SNS snapshot-summary owner. Public
  APIs, cache evidence, report output, and behavior are unchanged.

- `0.25.21` centralizes SNS proposal and neuron cache-list and cache-status
  report assembly, and removes their family-local path forwarders. Public APIs,
  cache identities and schemas, report output, and behavior are unchanged.

- `0.25.20` removes the remaining test-only Subnet duration/error surface,
  compact-renderer visibility bridge, NNS CLI value aliases, and single-use
  test adapters. Tests now use the supported shared duration parser, real host
  errors, command values, and observable report text directly. Production
  behavior is unchanged.

- `0.25.19` removes the final shared test-only Clap error augmenter and three
  System help forwarders. Tests now retain Clap’s native diagnostics without
  cloning and rendering complete commands on parse failures, while System help
  assertions render production builders directly. Production behavior is
  unchanged.

- `0.25.18` removes the ICRC test-support facade and its 13 type-specific
  parser adapters plus 18 help forwarders. ICRC tests now use one generic
  production-Clap harness and render command builders directly; production
  behavior is unchanged.

- `0.25.17` removes SNS option parsers and 22 help-rendering functions that
  existed only for tests. Focused SNS tests now share production-Clap harnesses
  and render the supported command builders directly; production behavior is
  unchanged.

- `0.25.16` removes the remaining test-only helper macro, option parsers, and
  help forwarders from the generic NNS Registry-leaf command families. Node,
  node-provider, node-operator, and data-center tests now exercise production
  Clap builders and option projections directly; production behavior is
  unchanged.

- `0.25.15` removes test-only option parsers and help-rendering modules from
  the NNS Subnet and Topology command owners. Focused tests reuse the shared
  NNS production-Clap harness and render supported command builders directly.
  Production grammar, parsing, dispatch, reports, and source behavior are
  unchanged.

- `0.25.14` removes test-only option parsers and help forwarders from the NNS
  neuron, Governance, and Registry command owners. Focused NNS tests now share
  one production-Clap parsing harness and render supported command builders
  directly. Production grammar, parsing, dispatch, reports, and source behavior
  are unchanged.

- `0.25.13` removes single-use configurable NNS proposal command builders and
  routes proposal option and help tests directly through the supported Clap
  commands. Production grammar, parsing, dispatch, reports, and source
  behavior are unchanged.

- `0.25.12` consolidates repeated Dashboard CLI test parsing for canister,
  metric, and network-resource options behind one IC-family harness. Production
  command parsing, help, dispatch, reports, and source behavior are unchanged.

- `0.25.11` centralizes ledger id, source endpoint, and output selection across
  live ICRC balance, allowance, ledger-transaction, and archive CLI option
  models. Command grammar, parsing behavior, dispatch requests, reports, and
  source behavior are unchanged.

- `0.25.10` separates ICRC account-history cache identity and strict storage,
  refresh and atomic publication, local list/status projection, and attempt
  evidence behind the existing public facade. Public APIs, cache paths and
  schemas, refresh policies, reports, and source behavior are unchanged.

- `0.25.9` removes duplicated test-only parsers from every ICRC option model
  and routes focused command tests through one shared Clap parsing helper.
  Production option parsing, CLI behavior, public APIs, reports, and source
  behavior are unchanged.

- `0.25.8` separates ICRC ledger-wide and account/account-history Clap command
  construction behind the existing command facade. CLI grammar and help,
  option parsing, dispatch, public Rust APIs, reports, and source behavior are
  unchanged.

- `0.25.7` centralizes optional ICRC subaccount normalization across account
  reports and complete-history caches and reuses one typed principal converter
  for live and cached transaction targets. Accepted inputs, errors, cache
  identities, public APIs, reports, and source behavior are unchanged.

- `0.25.6` centralizes common SNS custom-source validation for capability
  errors, exact evidence fields, and canonical principal text. Capability-
  specific invariants, validation errors, public APIs, reports, and source
  behavior are unchanged.

- `0.25.5` centralizes canonical lowercase-hex checks and corrects the focused
  Subnet feature documentation and gate to distinguish direct optional
  dependencies from packages retained transitively through `ic-agent`. It also
  simplifies shared internal feature gates by relying on `host` enabling
  `subnet-catalog-host`. Validation errors, public features and APIs, report
  output, and source behavior are unchanged.

- `0.25.4` separates live ICRC token/account queries, ledger-history and
  archive traversal, and capability probing behind the existing fetch facade.
  Public APIs, report output, source calls, and verification behavior are
  unchanged.

- `0.25.3` separates official Dashboard canister, metrics, and network-resource
  CLI parsing, dispatch, option models, and focused tests behind the existing
  `ic` command facade. CLI behavior, output, source calls, and public Rust APIs
  are unchanged.

- `0.25.2` separates NNS neuron cache models, paths, attempt evidence, page
  collection, publication, refresh orchestration, and cached report projection
  behind the existing public facade. Public APIs, CLI behavior, cache paths and
  schemas, validation, refresh behavior, and network calls are unchanged.

- `0.25.1` corrects schema-version documentation and consolidates shared
  proposal ordering, scalar text formatting, and small NNS leaf command
  adapters. Public APIs, CLI behavior, report JSON, cache schemas, sorting,
  rendering, and network behavior are unchanged.

- `0.25.0` expands every SNS neuron list and complete snapshot row with the
  fixed-size native Governance fields already returned by `list_neurons`:
  source NNS neuron id, auto-stake maturity, aging timestamp, raw dissolve
  state, voting-power percentage multiplier, vesting period, and neuron fees.
  It adds no query, pagination, fanout, or implicit cache behavior.

- This is a pre-1.0 report/cache hard cut. SNS neuron report and cache schemas
  advance to version 2; version-1 neuron caches are rejected rather than
  migrated or read through a compatibility branch and must be explicitly
  refreshed.

- Live custom-source rows, refresh pages, and loaded snapshots now share
  canonical lowercase-id, derived timestamp, uniqueness, and limit validation.
  Cache schema identity is checked before a changed row shape is decoded, so
  stale schemas return the typed unsupported-schema error.

## [0.24.x] - 2026-08-01 - Bounded SNS governance metrics

Detailed release notes: [docs/changelog/0.24.md](docs/changelog/0.24.md)

- `0.24.0` adds `sns metrics` for one SNS resolved by list id or Root
  principal. It uses the official Governance `get_metrics` composite query and
  preserves raw treasury ledger/accounts, current and original e8s amounts,
  per-treasury cached timestamps, voting-power metrics, proposal-window counts,
  genesis time, and latest SNS-ledger block time.
- The proposal-count window defaults to 30 days and is capped at 365 days.
  Including targeted discovery, the command makes three client requests; the
  Governance composite query performs its own bounded latest-block lookup.
  `ic-query` does not enumerate transactions, fan out, create a cache, or claim
  that differently timestamped metrics form one point-in-time snapshot.

```bash
icq sns metrics 1
icq sns metrics 23ten-uaaaa-aaaaq-aabia-cai --window 90d --json
```

## [0.23.x] - 2026-08-01 - Bounded SNS completeness

Detailed release notes: [docs/changelog/0.23.md](docs/changelog/0.23.md)

- `0.23.0` adds `sns swap` for one SNS resolved by list id or Root principal.
  It makes exactly three bounded native queries, retains component failures as
  typed gaps, and never calls `get_state`, enumerates participants, or creates
  a cache.
- Adds `sns upgrade` using Governance `get_running_sns_version` and SNS-W
  `get_next_sns_version`. It preserves all six Wasm hashes and pending-upgrade
  state, distinguishes no blessed successor from a query failure, and makes at
  most four live calls including targeted discovery.
- Direct SNS lookup now enriches only the selected SNS; unknown lookup requests
  no metadata, while `sns list` enriches the complete inventory. Sequential
  swap and upgrade evidence explicitly has no point-in-time guarantee.
- Hardens custom-source evidence and gives swap and upgrade reports focused
  public source traits, DTOs, builders, and text renderers. Metadata text must
  be trimmed and non-empty when present, and payload fields cannot coexist with
  a metadata error.
- As a pre-1.0 Rust API hard cut, replaces `SnsListSource` and
  `MainnetSnsList` with `SnsDiscoverySource`, `MainnetSnsInventory`,
  `MainnetSnsCanisters`, and `MainnetSnsMetadata`. No compatibility alias or
  fallback discovery flow remains.

```bash
icq sns swap 1
icq sns swap 23ten-uaaaa-aaaaq-aabia-cai --json
icq sns upgrade 1
icq sns upgrade 23ten-uaaaa-aaaaq-aabia-cai --json
```

## [0.22.x] - 2026-08-01 - Structural consolidation

Detailed release notes: [docs/changelog/0.22.md](docs/changelog/0.22.md)

- `0.22.7` replaces the mixed Dashboard model with explicit request/query,
  serialized report/row, host source-data, and host error owners behind one
  facade. Existing `ic_query::ic::*` paths, feature availability, constructors,
  fields, derives, report JSON, typed errors, and source contracts are
  unchanged. This completes the planned 0.22 cohesive-module boundary work.

- `0.22.6` separates Dashboard source traits, request normalization, untrusted
  source validation, and report projection into canister, metric, and
  network-resource owners behind one shared provenance facade. Existing public
  trait paths, typed errors, validation and canonical ordering, report fields,
  and custom-source behavior are unchanged.

- `0.22.5` separates the official Dashboard live adapter into canister,
  metric, and network-resource owners behind one shared HTTP transport facade.
  URL construction, wire decoding, and their focused tests now live with the
  capability that owns them. Existing public source traits and builders,
  endpoints, validation order, report output, request bounds, and network-call
  counts are unchanged.

- `0.22.4` separates the former monolithic ICRC text renderer into account,
  ledger-history, and ledger metadata/evidence owners behind one explicit
  internal facade. The facade retains only shared table-section and alignment
  mechanics. Existing public renderer paths, text output, report JSON, CLI
  behavior, feature availability, cache behavior, and network calls are
  unchanged.

- `0.22.3` separates ICRC request contracts and constructors from serialized
  report/row contracts behind one explicit internal facade. Existing public
  `ic_query::icrc::*` paths, feature availability, type fields and derives,
  constructors, report JSON, cache behavior, and network calls are unchanged.

- `0.22.2` separates ICRC account-history live orchestration, arbitrary-size
  cursor validation, complete collection state, generic ICRC-index decoding,
  and deployed ICP-index decoding into cohesive internal owners. The generic
  and ICP wire contracts remain deliberately distinct while sharing only their
  request envelope and protocol-neutral projection helpers. Public Rust paths,
  report JSON, cache schemas, pagination and index-discovery semantics, and
  network calls are unchanged.

- `0.22.1` replaces staged CLI parsing with one composed Clap command tree and
  typed family/leaf dispatch. Clap now owns nested help, propagated version,
  required-subcommand diagnostics, and top-level network validation; reduced
  passthrough grammars, raw argument scans, and the hidden network-forwarding
  option are removed. Native nested `icq help <path>` is supported, while the
  old leaf-trailing positional shortcut (for example, `icq sns list help`) is
  removed as a pre-1.0 hard cut; use `-h`, `--help`, or
  `icq help <path>`. Command names, report options and output, network
  authority, cache behavior, and live calls are unchanged.

  ```bash
  icq help nns topology
  ```

- `0.22.0` establishes the ordered structural-consolidation design for one
  composed Clap grammar, shared NNS Registry inventory orchestration, shared
  snapshot lifecycle mechanics, and cohesive module boundaries. The first
  slice replaces the NNS neuron wildcard export with an explicit current API
  list, table-drives repeated topology non-mainnet tests, makes published README
  documentation links resolve through GitHub, and adds a focused
  `subnet-catalog-host` feature without enabling `ic-query`'s direct optional
  Dashboard Reqwest or CBOR dependencies. Those packages may remain transitive
  through `ic-agent`. The full `host` feature remains a superset. Registry node,
  provider, operator, and data-center reports now share one cache-missing
  refresh driver and one exact-or-unique-prefix resolver, plus common
  network/source-request/fetch/write orchestration. NNS Governance proposal and
  neuron snapshots also share one attempt-sidecar construction, validation,
  status, and failed-progress owner.
  SNS proposal and neuron snapshots now share target lookup, lock acquisition,
  complete-cache provenance, atomic publication, and attempt finalization while
  retaining their distinct cursors and row validation. Public Rust paths, CLI
  behavior, report JSON, cache schemas, refresh policy, and network calls are
  unchanged.

## [0.21.x] - 2026-08-01 - Certified Cycle Minting Canister reporting

Detailed release notes: [docs/changelog/0.21.md](docs/changelog/0.21.md)

- `0.21.1` hard-cuts every report-producing CLI command from
  `--format <text|json>` to text-by-default output with `--json` selecting raw,
  script-friendly JSON. The shared flag and output-selection path now covers
  all IC Dashboard, ICRC, NNS, SNS, and system-canister report leaves and
  aligns the spelling with Canic. The removed `--format` form has no alias or
  compatibility parser. Clap now also validates the finite mainnet-only
  `--network ic` contract before staged command dispatch, so help-like option
  values cannot bypass an invalid network or an unsupported family option.
  Report JSON, text rendering, valid mainnet requests, cache identity and
  contents, and the public Rust API are unchanged.

  ```bash
  icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai --json
  icq nns topology summary --json
  icq system cycles --json
  ```

- `0.21.0` adds bounded live `system xdr` and `system cycles` reports backed
  by one native Cycle Minting Canister
  `get_icp_xdr_conversion_rate` query. The host adapter authenticates the
  application-level certificate for the mainnet CMC, verifies the
  certified-data hash-tree commitment and native rate leaf, and preserves the
  raw permyriad rate, market timestamp, endpoint, collection provenance, and
  certificate evidence. The cycles view derives cycles per ICP exactly from
  the certified rate and the IC protocol constant of one trillion cycles per
  XDR. These live-only reports reject non-mainnet networks before agent
  construction, do not scrape uncertified CMC metrics, and never enumerate or
  create a cache. Shared canister certified-data authentication is reused by
  CMC and ICRC tip verification without changing the ICRC report contract.

  ```bash
  icq system xdr
  icq system cycles --format json
  ```

## [0.20.x] - 2026-07-31 - Bounded Dashboard network metrics

Detailed release notes: [docs/changelog/0.20.md](docs/changelog/0.20.md)

- `0.20.2` adds a one-request, explicitly bounded Dashboard v3 daily network
  activity report. It defaults to seven days, caps requests at one year and
  responses at 366 rows, and preserves raw daily average/maximum query,
  update, total transaction, and block-rate strings. Rows are canonically
  ordered and retain exact query, endpoint, retrieval, and non-certified
  provenance. The report is live-only and never paginates, fans out, fills
  missing days, or creates a cache. This is a pre-1.0 Rust-API hard cut for
  custom `IcNetworkSource` implementations, which must implement the new
  daily-statistics capability.

  ```bash
  icq ic network daily-stats
  icq ic network daily-stats \
    --start 1784937600 --end 1785542400 --format json
  ```

- `0.20.1` adds the finite official Dashboard v4 boundary-node data-center
  report. It makes exactly one request, preserves raw data-center ids, names,
  owners, regions, coordinates, and node-count strings, retains zero-node
  locations, and derives canonical row ordering plus checked data-center and
  node totals. The public library adds typed request/report/source DTOs,
  `IcNetworkSource` on the existing `LiveIcSource`, custom-source validation,
  and text rendering. The report is explicitly off-chain, non-certified,
  live-only, and never paginates, follows up, or creates a cache.

  ```bash
  icq ic network boundary-node-data-centers
  icq ic network boundary-node-data-centers --format json
  ```

- `0.20.0` adds one-request bounded time-series reports for nine official
  Dashboard network metrics: instruction and message execution rates, cycle
  burn, block rate, node and Subnet counts, registered canister counts, total
  energy-consumption rate, and boundary-node count. Queries default to the
  preceding hour at a five-minute step and are capped at 1,000 observations
  per returned series. Reports preserve raw value strings, explicit query
  bounds, endpoint/retrieval provenance, and the Dashboard's non-certified,
  no-point-in-time guarantees. The public library adds typed metric
  request/report/source DTOs, `IcMetricSource` on `LiveIcSource`, custom-source
  validation, and text rendering. Metrics are live-only and never fan out,
  paginate, or create a cache. The shared invalid-source error prefix is
  generalized so metric failures are not mislabeled as canister failures.

  ```bash
  icq ic metrics instruction-rate
  icq ic metrics cycle-burn-rate --start 1700000000 --end 1700003600 --step 300
  icq ic metrics ic-node-count --format json
  ```

## [0.19.x] - 2026-07-31 - Bounded Dashboard canister discovery

Detailed release notes: [docs/changelog/0.19.md](docs/changelog/0.19.md)

- `0.19.1` consolidates repeated Dashboard source and report provenance into
  the existing `IcSourceRequest` and one flattened
  `IcDashboardReportProvenance`, and shares collection option parsing,
  transport execution, and text provenance rendering. Report JSON, CLI
  grammar and output, validation, live request counts, and cache behavior are
  unchanged. This is a breaking pre-1.0 Rust-API hard cut: Dashboard report
  provenance is now accessed through `.provenance`, and source-data DTOs echo
  the source request through `.source`; the replaced fields are removed
  without aliases.

- `0.19.0` adds filtered canister counts and one-page canister discovery
  through the official Dashboard v4 API. Count fetches no rows; page is fixed
  to canister-id order, defaults to 50 rows, is capped at 100, and never follows
  a cursor or writes a cache automatically. Typed library requests, reports,
  capability traits, custom-source validation, and renderers preserve raw
  Dashboard values and explicit off-chain provenance.

  ```bash
  icq ic canister count --has-name true
  icq ic canister page --query ledger --limit 25 --format json
  icq ic canister page --after ryjl3-tyaaa-aaaaa-aaaba-cai --limit 25
  ```

## [0.18.x] - 2026-07-30 - Official Dashboard canister reporting

Detailed release notes: [docs/changelog/0.18.md](docs/changelog/0.18.md)

- `0.18.1` consolidates live HTTP(S) endpoint parsing across `ic-agent` and
  official Dashboard sources, while retaining Dashboard-specific base-URL
  validation and typed errors. Top-level CLI error handling now owns usage
  exit codes and broken-pipe detection consistently for every command family.
  The README and supporting guides are reorganized around authority,
  collection mode, cache behavior, the current command hierarchy, and the
  distinction between CLI topology diagnostics and the exact-version joined
  topology library API. Command behavior, report JSON, cache schemas, and the
  public Rust API are unchanged.

- `0.18.0` adds the official Dashboard authority family and a bounded live
  canister detail report. Text and JSON preserve raw canister classification,
  name, controllers, Subnet, language, module hash, Dashboard update time, and
  nullable proposal-linked upgrade history alongside endpoint and retrieval
  provenance. Reports explicitly state that Dashboard evidence is not
  certified and is not one point-in-time snapshot. The public library exposes
  `LiveIcSource`, `IcCanisterSource`, typed source/request/report/error DTOs,
  custom-source construction, validation, and rendering. This report is
  live-only and does not add a cache.

  ```bash
  icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai
  icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai --format json
  ```

## [0.17.x] - 2026-07-30 - SNS Root inventory and health

Detailed release notes: [docs/changelog/0.17.md](docs/changelog/0.17.md)

- `0.17.1` hardens live and custom-source boundaries. Malformed or non-HTTP(S)
  endpoints now return typed agent-build errors instead of reaching
  `ic-agent` parser panics, and the global mainnet option is forwarded
  consistently to every NNS and SNS command family. ICRC account-transaction
  pages, SNS discovery and Root inventory, and NNS Registry version results
  now validate returned identity, provenance, limits, ordering, and authority
  claims before projection. Duplicate SNS health summaries no longer select
  status by response order. Complete NNS Governance, SNS Governance, and ICRC
  index caches reject unknown or duplicate top-level fields and impossible
  point-in-time claims. This is a pre-1.0 Rust-API and cache-validation hard
  cut: new typed error variants are added, previously tolerated invalid custom
  results and extra cache fields are rejected, and no compatibility path is
  retained.

  ```bash
  icq --network ic nns governance economics
  icq --network ic nns neuron list --limit 25
  icq --network ic sns canister list 1
  ```

- `0.17.0` adds `SnsCanisterSource` on the existing `LiveSnsSource` and a
  typed live SNS Root inventory and operational-health report. Root's
  `list_sns_canisters` query remains the membership authority; the health
  ingress always sends `update_canister_list = false`. Reports preserve native
  roles and status, raw module hashes and operational values, canonical
  ordering, explicit typed relation gaps, and the lack of a point-in-time
  guarantee. The shared `SnsSourceRequest` now carries network identity, a
  breaking pre-1.0 hard cut that lets direct built-in SNS source calls reject
  non-mainnet networks before constructing an agent. The report is live-only
  and does not alter existing caches.

  ```bash
  icq sns canister list 1
  icq sns canister list 23ten-uaaaa-aaaaq-aabia-cai --format json
  ```

## [0.16.x] - 2026-07-30 - Native NNS Governance reports

Detailed release notes: [docs/changelog/0.16.md](docs/changelog/0.16.md)

- `0.16.0` adds live native NNS Governance economics, cached-metrics, latest
  reward-event, and maturity-modulation reports. The library exposes one
  focused `NnsGovernanceSource` capability on `LiveNnsSource`, typed public
  payloads and errors, shared flattened query provenance, and text renderers.
  JSON preserves current Governance fields and raw numeric values, including
  named `key`/`value` projections for unlabeled Candid metric buckets. Every
  entry point rejects non-mainnet networks before a source or agent call.
  These bounded point-value reports are explicitly live-only and do not read
  or write the proposal or neuron caches.

  ```bash
  icq nns governance economics
  icq nns governance metrics --format json
  icq nns governance reward-event
  icq nns governance maturity-modulation
  ```

## [0.15.x] - 2026-07-30 - Public NNS neuron reporting

Detailed release notes: [docs/changelog/0.15.md](docs/changelog/0.15.md)

- `0.15.1` consolidates the identical proposal and neuron collection
  contracts into shared `NnsGovernanceRefreshRequest`,
  `NnsGovernanceCacheRequest`, and `NnsGovernanceRefreshAttemptStatus` types,
  and exposes one typed `NnsGovernanceQueryError` instead of copying its
  transport variants and mapping flow into each report family. Shared
  Governance cache provenance and validation are also defined once, and
  proposal cache JSON/operation failures use the shared `HostCacheError`.
  This is a breaking Rust-API hard cut: the replaced family-specific request,
  attempt, transport-error, and duplicated proposal cache-error variants are
  removed without aliases. CLI behavior, report JSON, and cache schemas are
  unchanged.

- `0.15.0` adds native public NNS neuron list and detail queries, complete
  atomic refreshes, cache-preferred reads, and cache-only status reporting.
  Reports preserve raw Governance neuron state, visibility, type, vote,
  stake, staked maturity, voting-power, known-neuron, ballot, endpoint, and
  collection provenance. Complete collections validate canonical neuron-id
  pagination and API exhaustion while explicitly recording
  `point_in_time_guaranteed: false` because Governance exposes no stable
  collection version. The existing `LiveNnsSource` gains the focused
  `NnsNeuronSource` capability, and proposal and neuron calls now share one
  internal Governance transport.

  ```bash
  icq nns neuron list --limit 25
  icq nns neuron info 123456789 --verbose
  icq nns neuron refresh --page-size 300
  icq nns neuron cache status --format json
  ```

## [0.14.x] - 2026-07-30 - Complete snapshots and shared cache contracts

Detailed release notes: [docs/changelog/0.14.md](docs/changelog/0.14.md)

- `0.14.1` moves every CLI cache from repository-local `.icq` state to one
  user-level cache root resolved from `ICQ_CACHE_ROOT`,
  `$XDG_CACHE_HOME/ic-query`, or `$HOME/.cache/ic-query`. Library cache
  requests now accept the actual cache root and consistently use `cache_root`
  terminology. The patch also consolidates repeated ICRC ledger requests, NNS
  inventory requests, SNS cache inspection DTOs, JSON cache error mapping,
  refresh policies, and attempt progress. Account-history collection reduces
  full-history memory overhead, requires custom sources to preserve an
  explicitly requested index, and retains the resolved index in failed
  collection evidence. This is a breaking CLI-storage and Rust-API hard cut:
  former project discovery, `ICQ_ICP_ROOT`, family-specific shared DTOs, old
  field and builder names, compatibility aliases, and cache migration are not
  retained. CLI command grammar and report schemas are unchanged.

- `0.14.0` replaces the live-only plural account-history command with explicit
  live-page, complete-refresh, cache-only list, and cache-status operations.
  Complete refreshes resolve and verify one index, exhaust backward pagination,
  validate canonical unique transaction ids and stable pagination evidence,
  and atomically publish one account snapshot. Failed or capped refreshes
  preserve the last complete cache and record attempt progress. This is a
  breaking CLI and library hard cut with no aliases for the replaced plural
  command or public page types. Complete snapshots explicitly report that the
  index API provides no point-in-time guarantee.

  ```bash
  icq icrc account transaction page mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa --limit 25
  icq icrc account transaction refresh mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa
  icq icrc account transaction list mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa --sort oldest --limit 100
  icq icrc account transaction cache status mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa
  ```

## [0.13.x] - 2026-07-29 - Canonical query command hierarchy

Detailed release notes: [docs/changelog/0.13.md](docs/changelog/0.13.md)

- `0.13.2` authenticates live ICRC-3 tip certificates against the IC root of
  trust and requested ledger canister, enforces certificate freshness, and
  validates that the returned hash-tree digest and required tip leaves match
  the certified data. The capability probe now reports a tip certificate as
  available only after the same verification. CLI and JSON shapes are
  unchanged.

- `0.13.1` adds typed, live ICRC index account-history queries. The command
  discovers the index through ICRC-106 unless one is supplied, verifies the
  index reports the requested ledger, preserves ledger/index/account/endpoint
  provenance and the full typed transaction payload in JSON, and exposes an
  exclusive backward-pagination cursor. It supports both generic index-ng and
  the deployed ICP index interface. Account-history caching remains a roadmap
  item.

  ```bash
  icq icrc account transactions mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa --limit 25
  icq icrc account transactions ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa --index-canister-id qhbym-qaaaa-aaaaa-aaafq-cai --format json
  ```

- `0.13.0` makes the roadmap command hierarchy current: SNS governance uses
  singular `proposal` and `neuron` families with explicit operations, while
  generic ICRC queries are separated into ledger-wide and account-scoped
  families. This is a breaking CLI hard cut; the replaced direct and plural
  forms have no aliases. The public NNS proposal unsupported-network error now
  retains the rejected network identity. Report models, JSON shapes, and cache
  schemas are unchanged.

  ```bash
  icq sns proposal list 1
  icq sns proposal info 1 387
  icq sns neuron list 1
  icq icrc ledger transactions ryjl3-tyaaa-aaaaa-aaaba-cai
  icq icrc account balance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa
  ```

## [0.12.x] - 2026-07-29 - Canonical NNS reporting adapter

Detailed release notes: [docs/changelog/0.12.md](docs/changelog/0.12.md)

- `0.12.3` centralizes the built-in mainnet network policy used by NNS, SNS,
  topology, and subnet-cache adapters so unsupported-network rejection cannot
  drift between report families.

- `0.12.2` rejects non-mainnet global network identities before NNS or SNS
  family dispatch and makes the current mainnet-only target contract explicit
  in top-level help and documentation.

- `0.12.1` adds a living 1.0 roadmap that tracks reporting coverage, caching
  and follow-up-query policy, prioritized workstreams, and the completion bar.
  CLI help now identifies live, cache-backed, cache-preferred, cache-only,
  forced-refresh, and view-dependent collection behavior. Global `--network`
  is honored by NNS proposals and rejected for ICRC commands instead of being
  silently ignored. `make install` now replaces an existing local `icq`
  binary.

  ```bash
  icq --network ic nns proposal list
  ```

- `0.12.0` replaces nine family-specific live NNS host adapters with
  `ic_query::nns::LiveNnsSource` and replaces four duplicate source request
  DTOs with
  `ic_query::nns::NnsSourceRequest`. Direct live Registry, subnet-catalog,
  inventory, proposal, and topology capability calls enforce their network
  contract consistently. This is a breaking library hard cut with no aliases;
  report schemas, cache formats, CLI behavior, and JSON output are unchanged.

## [0.11.x] - 2026-07-29 - Exact-version NNS topology

Detailed release notes: [docs/changelog/0.11.md](docs/changelog/0.11.md)

- `0.11.4` centralizes NNS node, data-center, node-operator, and node-provider
  cache-path, refresh-lock-path, network-validation, and typed cache-load
  wrappers. Public paths, errors, cache schemas, and refresh behavior are
  unchanged.

- `0.11.3` centralizes the shared NNS node, data-center, node-operator, and
  node-provider refresh-report and text projections. Public report fields,
  JSON and text output, cache behavior, and CLI behavior are unchanged.

- `0.11.2` consolidates duplicated SNS neuron and proposal cache error mapping,
  summary projection, and refresh-attempt persistence while preserving public
  APIs, cache and sidecar schemas, errors, and CLI behavior. Automatic
  missing-cache refreshes for the four NNS inventory families now use their
  existing refresh-request constructors instead of repeating defaults.

- `0.11.1` consolidates Registry inventory source requests, exact relation
  resolution, host cache errors, freshness/provenance projection, and SNS
  refresh-attempt reads. All public NNS Registry inventory live sources now
  reject non-mainnet requests before agent construction. This is a breaking
  library hard cut: the five family-specific inventory source-request types
  become `ic_query::nns::NnsInventorySourceRequest`, and component cache
  failures are exposed through `HostCacheError` and `CacheFileError`. Report
  schemas, cache formats, and CLI behavior are unchanged.

- `0.11.0` adds an exact-Registry-version `NnsSubnetTopologyReport` with raw
  Subnet kinds, canonical per-Subnet node-provider counts, strict relation and
  count validation, atomic joined caching, and distinct load, refresh,
  refresh-missing, and refresh-stale library APIs. This is a breaking library
  release because `NnsTopologyProvidersReport` gains the required
  `registry_versions` field; aggregate provider reports now retain every
  component Registry version instead of dropping their source provenance. The
  public live source rejects non-mainnet requests before agent construction,
  and required node and node-operator `key_not_present` responses become
  relation-specific typed errors while preserving other Registry failures.
  The native live-call stack also updates to `ic-agent` 0.49.2.

## [0.10.x] - 2026-07-12 - Canonical library and process boundaries

Detailed release notes: [docs/changelog/0.10.md](docs/changelog/0.10.md)

- `0.10.4` centralizes hosted and local CI on one sequential gate, separates
  CI, publication, and release-script regressions, and makes workspace
  publishing resumable across crates.io indexing delays. Publishing now
  requires the release tag at `HEAD`, package-list failures propagate, and
  pre-bump cleanliness is checked before CI starts.

  ```bash
  make ci-scripts-check
  make publish-guards-check
  ```

- `0.10.3` makes the public-rustdoc debt check independent of forced Cargo
  color in hosted CI, covers that environment in the local release guards,
  reruns the complete CI gate on the release commit before pushing it, and
  fully verifies the CLI package against an unpublished same-workspace library
  version instead of requiring that version to exist on crates.io first. The
  push gate also rejects a current-version tag that does not point to `HEAD`.

- `0.10.2` fixes CI setup by installing the pinned `ripgrep`, `cargo-audit`,
  and `cargo-machete` development tools through one shared target. The library
  process-boundary check now reports how to install its missing `rg`
  prerequisite instead of failing opaquely.

  ```bash
  make install-dev
  ```

- `0.10.1` removes the remaining stdout/stderr ownership from `ic-query`.
  Paged refreshes expose structured `QueryProgressEvent` callbacks while the
  `ic-query-cli` crate owns terminal detection, same-line progress, and
  missing-cache notices. It also shares the duplicated SNS cache identity
  model and validation, centralizes complete-snapshot path discovery, closes
  the 0.10 documentation drift, and adds CI guards for the process boundary
  and the existing public-rustdoc backlog.

- `0.10.0` makes each NNS and SNS family root its only public library path.
  Redundant nested `report` modules are private, topology requests, reports,
  builders, sources, constants, and renderers now live at
  `ic_query::nns::topology`, and the node-provider endpoint constant is named
  `DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT`. This is a hard cut with no old
  path re-exports, constant alias, deprecated wrapper, or compatibility shim.

## [0.9.x] - 2026-07-11 - Library and CLI ownership cut

Detailed release notes: [docs/changelog/0.9.md](docs/changelog/0.9.md)

- `0.9.0` removes the library `cli` feature and moves all Clap definitions,
  command parsing and dispatch, process output, project-context discovery, and
  CLI-only errors into `ic-query-cli`. `ic-query` now exposes only reusable
  duration helpers, requests, report models, source adapters, cache/refresh
  mechanics, builders, and renderers through its default and `host` surfaces.
  This is a hard cut: downstream crates must use the typed library API or the
  `ic-query-cli` executable crate; no compatibility feature, command aliases,
  re-export shims, or deprecated wrappers are retained.

## [0.8.x] - 2026-07-11 - Hard cache and runtime cut

Detailed release notes: [docs/changelog/0.8.md](docs/changelog/0.8.md)

- `0.8.3` reorganizes the generic ICRC implementation around explicit command
  dispatch, option parsing, report assembly, source adaptation, live fetching,
  public contracts, source data, error, and subaccount-validation boundaries.
  The cleanup preserves the existing CLI, public Rust API, JSON reports, and
  schema versions, and removes the remaining production parser panic.

- `0.8.2` makes numeric SNS proposal and neuron cache lookup read lightweight
  snapshot headers before loading only the matching complete snapshot, and
  rejects duplicate cache ids instead of selecting one by filesystem order.
  It also adds exhaustive current-contract coverage for snapshot completeness,
  routing-range order, terminal control escaping, and failed atomic-write
  cleanup.

- `0.8.1` restores the detailed 0.8 release ledger omitted from the 0.8.0
  commit and makes release automation fail closed: target-version notes and
  the complete CI gate must pass before metadata changes, package failures are
  preserved, untracked files fail the clean-tree check, and release steps stay
  sequential. It also completes the hard cut by removing positional version
  shortcuts and redundant public request/status aliases, requiring stable SNS
  row identifiers, validating persisted lock, attempt, and snapshot invariants,
  resetting every current report schema to version 1, preserving raw ICRC
  metadata in JSON while escaping terminal controls in text, and rejecting
  ambiguous routing catalogs.

- `0.8.0` hardens live host execution, complete snapshot publication, and cache
  operations. Live builders no longer panic when called from an existing
  Tokio runtime; paged refreshes reject stalled pages and invalid page sizes;
  failed proposal refreshes preserve progress; stale locks require explicit
  operator cleanup; CLI parse errors retain their diagnostics; and malformed
  attempt sidecars remain visible. This is an intentional breaking release:
  NNS/SNS complete snapshot schema is reset to version 1 and requires logical
  identity fields. Snapshot files outside the current shape and removed
  refresh-lock field names are rejected, with no compatibility aliases or
  automatic migrations.

## [0.6.x] - 2026-07-01 - Public source adapters

Detailed patch breakdown: [docs/changelog/0.6.md](docs/changelog/0.6.md)

- `0.6.9` fixes documentation drift after the source-adapter pass. The README
  now describes `host` as the feature for live calls, cache-backed builders,
  refresh helpers, and custom source adapters, and the library guide now leads
  with the 0.6 public source-adapter model instead of old 0.5-era boundary
  wording.

- `0.6.8` adds a compile-tested downstream source-adapter example to the
  library guide. The example shows how host users can implement a custom
  `NnsRegistrySource` for fixture, mirror, or proxy-backed report assembly
  without enabling `cli` or routing through the built-in live adapter.

- `0.6.7` completes the NNS topology source-adapter path. Host users can now
  implement `NnsTopologyRefreshSource` and pass it to topology refresh without
  routing through the built-in live topology adapter, while the default refresh
  builder now uses the same source boundary internally.

- `0.6.6` opens the NNS topology read-report source-adapter path. Host users
  can now implement `NnsTopologySource` and pass it to topology summary,
  coverage, versions, health, gaps, capacity, regions, and providers builders
  without routing through the built-in live topology adapter.

- `0.6.5` completes the SNS source-adapter path for neuron reports. Host users
  can now implement `SnsNeuronsSource` and pass it to SNS neuron list builders
  and complete neuron-cache refresh without routing through the built-in live
  SNS adapter.

- `0.6.4` extends the SNS source-adapter path to governance proposals. Host
  users can now implement `SnsProposalSource` and `SnsProposalsSource` and
  pass them to SNS proposal detail/list builders and complete proposal-cache
  refresh without routing through the built-in live SNS adapter.

- `0.6.3` opens the first SNS source-adapter path. Host users can now
  implement `SnsListSource`, `SnsTokenSource`, and `SnsParamsSource` and pass
  them to SNS list, info, token, and governance-parameter builders without
  routing through the built-in live SNS adapter.

- `0.6.2` opens the NNS proposal source-adapter path. Host users can now
  implement `NnsProposalSource` and pass it to proposal list/detail builders
  and complete-cache refresh without exposing private governance wire DTOs or
  routing through the built-in live NNS governance adapter.

- `0.6.1` extends the public source-adapter line to NNS inventory reports.
  Host users can now implement custom node, data-center, node-provider, and
  node-operator sources and pass them to the matching list/info builders and
  refresh wrappers without routing through the built-in live NNS registry
  adapters.

- `0.6.0` starts the public source-adapter line. Generic ICRC host users can
  now implement `IcrcSource` and call `build_icrc_*_report_with_source`
  builders directly, and subnet catalog host users can implement
  `SubnetCatalogSource` for cache refresh, load-or-refresh, and report
  builders. NNS registry version host users can also implement
  `NnsRegistrySource` and call `build_nns_registry_version_report_with_source`
  without routing through the built-in live NNS registry adapter.

## [0.5.x] - 2026-06-25 - Library boundary cleanup

Detailed patch breakdown: [docs/changelog/0.5.md](docs/changelog/0.5.md)

- `0.5.24` closes the 0.5 downstream-library documentation pass. The library
  usage guide now includes compile-tested SNS proposal, neuron, and
  cache-status examples for host users that do not enable `cli`, and it
  clarifies that custom source traits remain internal until a later public
  source-adapter design.

- `0.5.23` tightens the final library-boundary cleanup for the 0.5 line.
  CLI-only stdout/project-root helpers now compile only with `cli`, CLI
  duration parsing is no longer included in host-only downstream builds, and
  the generic NNS leaf refresh constructor is limited to CLI dispatch paths
  while host refresh accessors remain available.

- `0.5.22` refreshes downstream library documentation. The README now keeps a
  shorter feature-boundary summary and links to a dedicated library usage guide
  with canic-style migration notes, host/no-default dependency guidance, and
  examples for replacing `icq` process shell-outs with public request
  constructors and report builders. A downstream-usage integration test now
  keeps those example patterns compiling under no-default and host-only builds.

- `0.5.21` completes the remaining small NNS request constructor pass.
  Registry-version and topology read/refresh requests now have public
  constructors, topology refresh has a dry-run setter, and registry/topology CLI
  dispatch uses those constructors.

- `0.5.20` improves subnet catalog request ergonomics for downstream library
  users. Cache, list, info, and refresh requests now have public constructors
  and builder-style setters for filters, range display, forced resolution,
  dry-run, and output controls; subnet CLI and topology adapters use the same
  constructors.

- `0.5.19` improves NNS inventory request ergonomics for downstream library
  users. Node, data-center, node-provider, and node-operator list/info/refresh
  requests now have public constructors; node list requests also have
  builder-style filter setters, and the CLI construction paths use the public
  constructors.

- `0.5.18` improves NNS proposal request ergonomics for downstream library
  users. NNS proposal list/detail requests now have public constructors and
  builder-style setters for filters, sort controls, verbosity, and ballot
  display, and the CLI dispatch path now uses those library constructors.

- `0.5.17` improves SNS request ergonomics for downstream library users. SNS
  list, lookup, proposal list/detail, and neuron list requests now have public
  constructors and builder-style setters for optional view and cache controls.

- `0.5.16` improves the generic ICRC native library path. Downstream crates
  now get public constructors for every generic ICRC request DTO, a public
  default ICRC source endpoint constant, and host-without-CLI smoke coverage
  for the live ICRC report builder exports.

- `0.5.15` opens the SNS native host API without requiring `cli`.
  Downstream crates can now call SNS list/info/token/params, proposal
  list/detail, and neuron report builders directly, inspect SNS proposal and
  neuron caches, resolve their cache/refresh sidecar paths, refresh complete
  SNS proposal and neuron snapshots, and render SNS neuron/proposal cache and
  refresh reports from the library.

- `0.5.14` opens the NNS proposal native host API without requiring `cli`.
  Downstream crates can now inspect complete NNS proposal caches, build
  list/detail reports from a complete cache, call the live NNS proposal report
  builders explicitly, refresh the complete proposal cache, resolve proposal
  cache paths, and render cache/refresh reports directly from the library.

- `0.5.13` improves the native NNS inventory library path for downstream
  crates. Node, data-center, node-provider, and node-operator cache request
  types now have public constructors, and their host APIs expose cache and
  refresh-lock path helpers plus refresh-lock defaults without requiring
  `cli`. The public API smoke tests now seed cached inventory reports and
  exercise the same cache-backed list/info builders that the CLI uses.

- `0.5.12` opens subnet catalog list/info/refresh request, report, row,
  builder, refresh, and text-rendering APIs under `features = ["host"]`
  without requiring `cli`. Native downstream crates can now build and render
  the same cache-backed subnet catalog reports used by `icq nns subnet`
  without spawning the `icq` executable.

- `0.5.11` opens NNS topology request/report/row DTOs and text renderers
  under `--no-default-features`, and exposes topology cache-backed builders
  and refresh execution to `host` users without requiring `cli`. Command
  parsing and dispatch remain behind `cli`. Downstream crates can now build,
  construct, deserialize, and render topology summary, coverage, versions,
  health, gaps, capacity, regions, providers, and refresh reports without
  spawning `icq`.

- `0.5.10` opens NNS data-center, node-provider, and node-operator list/info
  request, report, row, and text-rendering DTOs under `--no-default-features`.
  Downstream crates can now construct and render the cached NNS inventory leaf
  reports without enabling native live-call, cache, refresh, or CLI
  dependencies. This slice also exposes the subnet-catalog host cache request,
  cached load, load-or-refresh, cache path helpers, and default mainnet
  endpoint under `features = ["host"]` so native tools can replace
  `icq nns subnet info` shell-outs with direct library calls.

- `0.5.9` opens NNS node list/info request, report, row, filter, constant,
  and text-rendering DTOs under `--no-default-features`. Downstream crates can
  now construct and render NNS node reports without enabling native live-call,
  cache, or CLI dependencies.

- `0.5.8` opens NNS proposal list/detail request, report, row, filter, sort,
  tally, ballot, and text-rendering DTOs under `--no-default-features`.
  Downstream crates can now construct and render NNS proposal reports without
  enabling native live-call, cache, or CLI dependencies.

- `0.5.7` tightens the generic ICRC no-default library boundary. The public
  API smoke test now constructs and renders token, balance, allowance, index,
  transaction, block-type, archive, tip-certificate, and capability reports
  without enabling native live-call or CLI dependencies.

- `0.5.6` opens SNS governance-parameter report DTOs and text rendering under
  `--no-default-features`, along with SNS proposal list/detail request,
  report, row, filter, sort, and text-rendering DTOs. Downstream crates can
  now construct and render SNS params and proposal reports without enabling
  native live-call, cache, or CLI dependencies. The make-driven Cargo checks
  also disable HTTP multiplexing, use a higher network retry count, and retry
  package verification to make CI less sensitive to transient crates.io HTTP/2
  failures.

- `0.5.5` opens more pure SNS library surface under `--no-default-features`.
  Downstream crates can now construct and render SNS info and token report
  DTOs without enabling native live-call, cache, or CLI dependencies. The SNS
  no-default public API smoke test now covers list, info, and token reports.

- `0.5.4` tightens the feature-boundary guard after the no-default library
  surfaces were opened. The CI script now separately asserts that pure
  no-default builds avoid CLI/live-call dependencies and that `host` without
  `cli` still avoids `clap`; the README and crate docs also clarify that
  no-default is a host/CLI dependency boundary, not a `no_std` promise.

- `0.5.3` opens the pure subnet-catalog model, JSON, and resolver API under
  `--no-default-features` so downstream crates can validate catalog snapshots
  and resolve subnet/canister principals without host cache refresh or CLI
  dependencies. The feature-boundary check now runs the ICRC, NNS, SNS, and
  subnet-catalog public API smoke tests under no-default builds.

- `0.5.2` opens the first pure no-default NNS and SNS library surfaces after
  the crate split. `ic_query::nns::registry` now exposes registry-version
  request/report/text DTOs without `host`, and `ic_query::sns` now exposes SNS
  list request/report/text DTOs without `host`; live builders, cache IO, and
  CLI dispatch remain behind host/CLI features. This slice also makes
  `ic-query --features host --no-default-features` compile without the CLI
  feature, and the feature-boundary CI check now covers that host-only build
  plus the no-default public API smoke tests.

- `0.5.1` corrects release documentation after the library-boundary cleanup
  was published as `0.5.0`. The detailed changelog now has a `0.5.x` ledger,
  the `0.5.0` notes live in the matching release line, and the README library
  dependency example points at the `0.5` release series. This slice also adds
  a CI guard for the library feature boundary so default/no-default
  `ic-query` builds stay free of CLI parsing and native live-call dependencies.

- `0.5.0` tightens the library and CLI package boundary after the initial
  split. The top-level process runner now lives in `ic-query-cli`, while
  `ic-query` keeps only family-level CLI adapters behind the `cli` feature for
  the wrapper. This slice also changes the `ic-query` default feature set to
  empty so plain library dependencies do not pull CLI parsing or native
  live-call adapters.

## [0.4.x] - 2026-06-25 - Library and CLI package split

Detailed patch breakdown: [docs/changelog/0.4.md](docs/changelog/0.4.md)

- `0.4.0` splits the project into a virtual workspace with
  `crates/ic-query` as the reusable library package and `crates/ic-query-cli`
  as the package that installs the existing `icq` binary. This slice adds
  public report facades for generic ICRC reports, NNS registry version reports,
  and deployed SNS list reports so downstream crates can start using typed
  request/report APIs without invoking CLI argument parsing. The library also
  gates CLI and live host-call dependencies behind features so
  `ic-query --no-default-features` compiles for native and
  `wasm32-unknown-unknown` targets without pulling `clap`, `ic-agent`, Tokio,
  or `futures`.

  ```bash
  cargo install ic-query-cli
  ```

## [0.3.x] - 2026-06-24 - Generic ICRC ledger queries

Detailed patch breakdown: [docs/changelog/0.3.md](docs/changelog/0.3.md)

- `0.3.9` adds `--follow-archives` to `icq icrc transactions`, allowing the
  bounded live transaction query to follow returned ICRC-3 archive callbacks.
  Followed archive blocks and archive follow errors are reported separately so
  ledger-returned blocks, callback ranges, and archive fetch results remain
  script-friendly. This slice also cleans up the generic ICRC implementation by
  deduplicating common command option wiring, ICRC-3 block/range row
  conversion, and text table rendering helpers without changing CLI behavior,
  report schemas, or output semantics.

  ```bash
  icq icrc transactions mxzaz-hqaaa-aaaar-qaada-cai --start 0 --limit 1 --follow-archives
  icq icrc transactions mxzaz-hqaaa-aaaar-qaada-cai --start 0 --limit 1 --follow-archives --format json
  ```

- `0.3.8` adds `icq icrc capabilities <ledger-canister-id>` for live generic
  ICRC endpoint probing. The report keeps each probed method independent so
  unsupported optional endpoints appear as `unsupported` rows instead of
  failing the whole command.

  ```bash
  icq icrc capabilities mxzaz-hqaaa-aaaar-qaada-cai
  icq icrc capabilities mxzaz-hqaaa-aaaar-qaada-cai --format json
  ```

- `0.3.7` adds live generic ICRC-3 tip certificate inspection through
  `icrc3_get_tip_certificate`. Text output shows certificate and hash tree byte
  counts plus truncated hex previews when present; JSON keeps
  `certificate_present` plus optional full certificate and hash tree hex
  strings and byte counts.

  ```bash
  icq icrc tip-certificate mxzaz-hqaaa-aaaar-qaada-cai
  icq icrc tip-certificate mxzaz-hqaaa-aaaar-qaada-cai --format json
  ```

- `0.3.6` adds live generic ICRC-3 ledger discovery for supported block types
  and archive ranges. Both reports are live-only, include the queried source
  endpoint, and keep archive range bounds as string fields in JSON.

  ```bash
  icq icrc block-types ryjl3-tyaaa-aaaaa-aaaba-cai
  icq icrc archives ryjl3-tyaaa-aaaaa-aaaba-cai --from qaa6y-5yaaa-aaaaa-aaafa-cai --format json
  ```

- `0.3.5` adds live generic ICRC transaction history pages through the
  ledger's `icrc3_get_blocks` endpoint. Text output shows compact block
  summaries and archive callback ranges; JSON keeps raw ICRC-3 block values
  and block/log indexes as string fields.

  ```bash
  icq icrc transactions ryjl3-tyaaa-aaaaa-aaaba-cai
  icq icrc transactions ryjl3-tyaaa-aaaaa-aaaba-cai --start 100 --limit 50 --format json
  ```

- `0.3.4` cleans up the generic ICRC index slice by sharing ICRC/SNS token
  metadata text formatting, centralizing generic ICRC live query setup, using
  the shared ICRC-106 index-error formatter from SNS token reports, and
  replacing the remaining plain test `unwrap` without changing CLI behavior,
  report schemas, cache paths, or output.

- `0.3.3` adds live ICRC-106 index discovery by ledger canister id, returning
  the configured index canister when available or the ledger-reported index
  discovery error as text/JSON fields.

  ```bash
  icq icrc index ryjl3-tyaaa-aaaaa-aaaba-cai
  icq icrc index ryjl3-tyaaa-aaaaa-aaaba-cai --format json
  ```

- `0.3.2` adds live ICRC-2 allowance queries by ledger canister id, owner
  principal, and spender principal, including optional owner and spender
  subaccounts plus text/JSON reports that keep raw allowance base units and
  expiration nanoseconds script-friendly.

  ```bash
  icq icrc allowance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa aaaaa-aa
  icq icrc allowance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa aaaaa-aa --owner-subaccount 0000000000000000000000000000000000000000000000000000000000000000 --spender-subaccount 0000000000000000000000000000000000000000000000000000000000000000
  ```

- `0.3.1` consolidates duplicated generic ICRC and SNS ledger-token live
  plumbing behind a shared ICRC ledger helper for wire types, token metadata
  calls, metadata conversion, and ledger query error mapping without changing
  CLI behavior, report schemas, cache paths, or output.

- `0.3.0` adds live generic ICRC ledger token metadata and account balance
  queries by ledger canister id, including text/JSON reports that show the
  queried source endpoint and preserve raw base-unit token amounts in JSON.

  ```bash
  icq icrc token ryjl3-tyaaa-aaaaa-aaaba-cai
  icq icrc balance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa --subaccount 0000000000000000000000000000000000000000000000000000000000000000
  ```

## [0.2.x] - 2026-06-16 - SNS proposal detail expansion

Detailed patch breakdown: [docs/changelog/0.2.md](docs/changelog/0.2.md)

- `0.2.42` adds cache-compatible NNS and SNS proposal text search with
  `--query <text>`, reports the selected `query_filter` in proposal list
  text/JSON output, and makes NNS proposal list reports expose `result_scope`
  so bounded live views are distinguishable from complete-cache views.

  ```bash
  icq nns proposal list --query subnet
  icq sns proposals 1 --query treasury
  ```

- `0.2.41` adds proposal list filters for SNS reward eligibility and
  NNS/SNS proposers while keeping them cache-compatible where complete
  snapshots are available.

  ```bash
  icq nns proposal list --proposer 123456789
  icq sns proposals 1 --eligible yes
  icq sns proposals 1 --proposer 00010203
  ```

- `0.2.40` adds NNS and SNS proposal sorting by latest tally timestamp, plus
  SNS proposal sorting by action id, reward eligibility, and reward-event end
  timestamp, while reusing the existing live and complete-cache proposal view
  sorters.

  ```bash
  icq nns proposal list --sort tally-time
  icq sns proposals 1 --sort action-id
  icq sns proposals 1 --sort tally-time
  icq sns proposals 1 --sort eligible
  icq sns proposals 1 --sort reward-end
  ```

- `0.2.39` adds NNS proposal sorting by reward status, voting deadline, and
  total potential voting power while reusing the existing live and
  complete-cache proposal view sorter.

  ```bash
  icq nns proposal list --sort reward-status
  icq nns proposal list --sort deadline
  icq nns proposal list --sort voting-power
  ```

- `0.2.38` adds cache-backed SNS proposal topic sorting with
  `icq sns proposals <id|root-principal> --sort topic`, defaulting to
  ascending topic-label order and applying the sort before `--limit`.

  ```bash
  icq sns proposals 1 --sort topic
  ```

- `0.2.37` makes SNS proposal `--topic <topic>` filters cache-compatible by
  preserving proposal topic labels in complete snapshots, refreshing legacy
  proposal snapshots that lack topic labels before topic-filtered views, and
  allowing decided-status filters to combine with topic filters through the
  complete cache path.

  ```bash
  icq sns proposals 1 --status decided --topic governance
  ```

- `0.2.36` makes SNS proposal `--status adopted|rejected` filters
  cache-compatible for topic-free list views by preserving raw SNS governance
  status codes in proposal rows, refreshing legacy proposal snapshots that
  lack those codes before applying final-status filters, and bumping SNS
  proposal JSON report schemas for the new optional raw `status` field.

- `0.2.35` deduplicates SNS cache inspection plumbing by sharing invalid-cache
  summary fields, valid cache-id lookup, and cache-error text rendering across
  SNS neuron and proposal cache reports without changing CLI behavior, cache
  paths, report schemas, or output semantics.

- `0.2.34` hardens complete snapshot cache inspection so NNS proposal, SNS
  proposal, and SNS neuron cache list/status reports surface malformed,
  unsupported, or identity-mismatched local cache files as invalid local rows
  while keeping normal cache-backed reads strict and live-free cache status
  behavior unchanged.

- `0.2.33` makes crate packaging more intentional by excluding internal
  workflow, agent, governance, toolchain, and dev-only script files from the
  published tarball, and adds a CI guard for package contents without changing
  CLI behavior.

- `0.2.32` validates complete snapshot cache identity fields when present,
  rejecting caches whose recorded domain, entity, collection, or scope does not
  match the logical cache key while preserving compatibility with older caches
  that do not yet contain identity fields, adds NNS/SNS family-level mismatch
  coverage, adds a CI guard against tag-pinned GitHub Actions, standardizes
  README command fences, and removes stale clippy allow-list entries.

- `0.2.31` fixes release documentation drift, clarifies NNS proposal cache
  status wording, documents corrupted refresh-lock handling, and adds a
  changelog/version consistency check to the local and CI gate, pins CI actions
  to exact revisions, documents clap required-value invariants, and adds
  logical identity fields to newly written snapshot caches without changing CLI
  behavior.

- `0.2.30` continues internal cleanup by splitting NNS proposal report assembly,
  labels, and source modules into focused owners, moving shared cache policy
  tests under the cache-file module, tightening clap/parser coverage, and
  preserving CLI behavior, cache paths, report schemas, and output.

- `0.2.29` centralizes shared cache-file, clock, and runtime error formatting
  through typed `thiserror` errors, splits NNS proposal snapshot refresh into
  orchestration, page collection, and publish modules, deduplicates NNS
  proposal cache option parsing, and normalizes shared cache/snapshot helper
  docs without changing CLI behavior, cache paths, report schemas, or output.

- `0.2.28` adds explicit complete NNS proposal snapshot refresh and cache
  inspection commands under `icq nns proposal refresh` and
  `icq nns proposal cache list|status`, reusing the shared snapshot cache
  lock, progress, attempt sidecar, and complete-only publish flow while
  keeping normal `icq nns proposal list` as a bounded live query.

  ```bash
  icq nns proposal refresh
  icq nns proposal refresh --max-pages 5
  icq nns proposal cache status
  ```

- `0.2.27` moves NNS proposal queries under grouped commands
  `icq nns proposal list` and `icq nns proposal info <proposal-id>`, removes
  the ungrouped `icq nns proposals` and `icq nns proposal <proposal-id>` forms,
  and updates help, README examples, and parser coverage to enforce the grouped
  surface.

  ```bash
  icq nns proposal list --reward-status settled
  icq nns proposal info 132411 --ballots
  icq nns proposal info 132411 --verbose
  ```

- `0.2.26` adds `icq nns proposal <proposal-id> --ballots`,
  `icq nns proposal <proposal-id> --verbose`, and
  `icq nns proposals --reward-status <status>`, including deterministic NNS
  ballot rows in JSON, compact-by-default NNS proposal summaries, and
  API-backed NNS proposal reward-status filtering.

  ```bash
  icq nns proposal 132411 --ballots
  icq nns proposal 132411 --verbose
  icq nns proposals --reward-status settled
  ```

- `0.2.25` adds direct live NNS governance proposal list and detail queries,
  including status/topic filters, local proposal sorting, sort direction
  controls, verbose list details, and text/JSON reports that expose the
  selected view options.

  ```bash
  icq nns proposals --status open
  icq nns proposals --topic governance
  icq nns proposals --sort title --asc
  icq nns proposal 132411 --format json
  ```

- `0.2.24` adds local SNS proposal list sorting by proposer neuron id and
  reward event round, reusing the existing proposal view sorter for bounded
  live rows and complete proposal snapshots before limit truncation.

  ```bash
  icq sns proposals 1 --sort proposer
  icq sns proposals 1 --sort reward-round
  ```

- `0.2.23` adds local SNS proposal list sorting by status, ballot count, and
  reject cost, and centralizes SNS proposal sort direction policy in the
  report sort model so command parsing and report assembly share the same
  API-order versus local-sort behavior.

  ```bash
  icq sns proposals 1 --sort status
  icq sns proposals 1 --sort ballots
  icq sns proposals 1 --sort reject-cost
  ```

- `0.2.22` makes SNS proposal sort direction defaults match the selected
  sort: title/action default ascending, id/tally/timestamp sorts default
  descending, and explicit `--asc`/`--desc` is rejected for API-order views.

  ```bash
  icq sns proposals 1 --sort title
  icq sns proposals 1 --sort title --desc
  icq sns proposals 1 --sort total-votes
  ```

- `0.2.21` adds local SNS proposal list sorting by proposal title, action,
  yes tally, no tally, and total vote tally, reusing the shared proposal view
  sorter for live rows and complete proposal snapshots while keeping cache
  identity, cache paths, and JSON row fields unchanged.

  ```bash
  icq sns proposals 1 --sort title --asc
  icq sns proposals 1 --sort action
  icq sns proposals 1 --sort total-votes
  ```

- `0.2.20` shares SNS neuron/proposal refresh-attempt metadata and progress
  DTO plumbing, snapshot cache path construction, and cache-list lookup flow
  plus snapshot scan/load helpers while keeping family-specific storage error
  mapping, report DTOs, cache paths, report schemas, text output, and JSON
  fields unchanged, and documents the section-style type doc rule for scoped
  public helper types.

- `0.2.19` centralizes shared SNS neuron/proposal cache-status lookup flow
  behind associated-type cache-family traits, keeping family-specific storage
  and report DTOs separate, shares the identical refresh-attempt status DTO,
  and removes duplicated id/root cache-status branching without changing CLI
  behavior, cache paths, report schemas, text output, or JSON fields.

- `0.2.18` adds cache-backed `icq sns proposals --status decided` filtering
  for complete proposal snapshots, rejects combining that synthetic local
  status with topic filters, and documents that adopted/rejected still require
  live fallback because cached proposal rows do not carry the raw governance
  status enum.

  ```bash
  icq sns proposals 1 --status decided
  ```

- `0.2.17` adds `icq sns proposals --sort decided|executed|failed` for
  newest decision, execution, and failure proposal views, adds `--asc` and
  `--desc` direction controls for local proposal sorts, applies sorting
  through the shared proposal view layer for live rows and complete proposal
  snapshots, and keeps cache identity, cache paths, and JSON row fields
  unchanged.

- `0.2.16` moves changelog contribution rules out of the public changelog and
  into `AGENTS.md`, routes SNS nested `neurons`/`proposals` refresh/cache
  dispatch through shared clap helpers, shares cached lookup/cache command
  setup across SNS runners, and removes remaining SNS neuron/proposal cache
  re-export/timestamp helper shims without changing CLI behavior, cache
  behavior, report schemas, or output.

- `0.2.15` removes over-fragmented one-function and single-child modules
  across NNS topology, NNS leaf runtime/cache errors, SNS text/source/live
  helpers, and proposal cache reports, enforces the `module.rs` versus
  `module/mod.rs` layout rule across the touched tree, and shares SNS cache
  summary ordering plus root-principal parsing between neuron and proposal
  cache reports without changing CLI behavior, cache behavior, report schemas,
  or text output.

- `0.2.14` moves deployed SNS list sorting into the report view layer and
  leaves lookup focused on stable id assignment and input resolution, applies
  the code-hygiene module-header and module-granularity standard to SNS
  command/report modules and touched NNS topology modules, and normalizes
  touched SNS/NNS imports away from `super::super` paths without changing
  behavior.

- `0.2.13` centralizes SNS proposal and neuron row view transforms under the
  report view layer, removing cache-local filter/sort helpers without changing
  CLI behavior, cache behavior, or report schemas.

- `0.2.12` adds `icq sns proposals --sort api|id|created`, reports the
  selected proposal sort in text and JSON, and applies cache-backed proposal
  sorting before limit truncation without changing proposal cache identity.

  ```bash
  icq sns proposals 1 --sort created
  icq sns proposals 1 --sort id --limit 50
  ```

- `0.2.11` reports live-versus-cache provenance for SNS proposal list and
  detail reports, including cache path and completeness metadata when a
  complete local proposal snapshot is used.

- `0.2.10` lets `icq sns proposal` reuse an existing complete proposal
  snapshot for detail lookups before falling back to live governance reads.

  ```bash
  icq sns proposals refresh 1
  icq sns proposal 1 42
  ```

- `0.2.9` splits SNS proposal cache-backed report building into focused load,
  filter, report projection, collection fetch, progress, attempt, and state
  modules, splits proposal cache status report building, and moves proposal
  cache input lookup into storage without changing cache behavior or CLI
  behavior.

- `0.2.8` splits SNS proposal report DTOs and proposal cache refresh
  orchestration/storage/attempt handling into focused modules, aligning
  proposal and neuron cache structure and moving proposal cache discovery onto
  the shared deterministic snapshot scanner without changing report schemas,
  cache behavior, or CLI behavior.

- `0.2.7` splits SNS proposal cache internals into focused model, path,
  storage, attempt, collection, and report modules, and shares SNS cache-file
  error formatting between neuron and proposal caches without changing CLI
  behavior.

- `0.2.6` centralizes missing-cache load/refresh policy across subnet catalog,
  cached NNS component reports, and SNS proposal auto-cache creation without
  changing CLI behavior, and documents the explicit-refresh rule for SNS
  neuron complete snapshots.

- `0.2.5` makes normal `icq sns proposals` list views auto-create and reuse
  complete local proposal snapshots, adds manual proposal cache inspection and
  refresh commands, and splits related SNS source/live proposal modules.

  ```bash
  icq sns proposals 1
  icq sns proposals refresh 1
  icq sns proposals cache list
  icq sns proposals cache status 1
  ```

- `0.2.4` splits SNS source traits and live-source implementations into
  focused list, token, params, proposal, and neuron modules without changing
  CLI behavior.

- `0.2.3` splits SNS clap value-enum, report request, and report sort/filter
  model plumbing into focused modules while preserving existing list, neuron,
  and proposal option behavior.

  ```bash
  icq sns list --sort name
  icq sns neurons 1 --sort stake
  icq sns proposals 1 --topic governance
  ```

- `0.2.2` adds `icq sns proposals --topic <topic>` to filter bounded live SNS
  governance proposal listings by SNS topic, reports the selected filter, and
  tightens request-mapping coverage for unfiltered versus concrete topic
  selectors.

  ```bash
  icq sns proposals 1 --topic any
  icq sns proposals 1 --topic governance
  icq sns proposals 1 --topic treasury-asset-management --format json
  ```

- `0.2.1` centralizes cached NNS leaf cache errors and JSON cache helpers,
  removes duplicated cache-error macro plumbing, and tightens command/test
  module hygiene.

  ```bash
  icq nns node list
  icq nns node-provider refresh
  icq sns neurons 1 --owner 2vxsx-fae --sort api
  ```

- `0.2.0` adds direct SNS proposal ballot table output with compact neuron IDs
  by default and full IDs under `--verbose`.

  ```bash
  icq sns proposal 1 387 --ballots
  icq sns proposal 1 387 --ballots --verbose
  ```

## [0.1.x] - 2026-06-13 - Snapshot cache and SNS query growth

Detailed patch breakdown: [docs/changelog/0.1.md](docs/changelog/0.1.md)

- `0.1.49` simplifies topology read command runners and option tests, and
  splits NNS macro plumbing into focused modules.

- `0.1.48` centralizes CLI help/version argument collection, NNS/SNS clap
  usage-error mapping, cached NNS leaf runtime setup, and SNS neuron cache
  command setup.

- `0.1.47` centralizes compact-vs-verbose text/JSON writing for NNS list
  commands, passthrough subcommand-argument extraction for clap dispatch, and
  NNS node/subnet runtime cache-request setup.

- `0.1.46` centralizes SNS lookup-command runtime fields for info, token,
  params, proposal, proposals, neurons, and neuron refresh commands, plus
  shared CLI, SNS, and NNS command args/help/version handling.

- `0.1.45` removes remaining production wildcard imports and centralizes
  clap parse-to-usage, help/version handling, and NNS project-root usage-error
  handling for command dispatch and option parsers.

- `0.1.44` replaces wildcard module re-exports across SNS, cached NNS report
  roots, subnet catalog reports, and topology fixtures with explicit export
  lists.

- `0.1.43` adds a shared NNS leaf refresh-cache writer and migrates node,
  node-provider, node-operator, data-center, and topology cache-request
  adapters onto shared NNS leaf cache helpers.

- `0.1.42` extracts shared locked, paged snapshot refresh and
  attempt-lifecycle orchestration, centralizes SNS neuron attempt writers, and
  migrates SNS neuron complete-refresh paging onto the generic runners.

- `0.1.41` extracts shared snapshot JSON loading/writing, header validation,
  refresh-attempt, and full-collection path scanning helpers and migrates SNS
  neuron cache reads, writes, and attempts onto them.

- `0.1.40` extracts shared snapshot-cache key, path, envelope, completeness,
  and paged-collection state helpers and migrates SNS neuron complete snapshots
  onto them without changing cache JSON shape.

- `0.1.39` splits SNS report source, lookup, live fetch, text helpers, neuron
  models, neuron cache collection, and live proposal conversion helpers into
  focused modules.

- `0.1.38` splits NNS component text rendering plus topology text, relation,
  refresh, gap, and derived report helpers into focused modules.

- `0.1.37` splits NNS registry, node, and cached component report roots into
  focused build, source, model, text, and refresh modules.

- `0.1.36` splits shared NNS leaf command/option/runtime helpers plus NNS
  component command, runtime, spec, and report-adapter wiring into focused
  modules.

- `0.1.35` splits SNS params text, SNS neuron command plumbing, and NNS
  topology provider/command/option plumbing into focused modules.

- `0.1.34` splits live SNS Candid wire types and proposal text rendering into
  focused modules.

- `0.1.33` splits SNS text/build orchestration and NNS topology build/read
  orchestration into focused modules.

- `0.1.32` splits SNS neuron-cache storage, refresh, report, attempt, and
  collection handling into focused modules.

- `0.1.31` splits shared cache-file JSON, refresh-lock, and write helpers into
  focused modules.

- `0.1.30` splits subnet catalog list text rendering, classification model
  enums, and root catalog helpers into focused modules.

- `0.1.29` splits subnet catalog resolver and report model definitions into
  focused modules.

- `0.1.28` splits IC registry live-source, relation-inventory fetch
  orchestration, and public registry models into focused modules.

- `0.1.27` splits IC registry transport helpers, relation helpers, and
  relation inventory tests into focused modules.

- `0.1.26` splits IC registry protobuf wire types and registry domain
  projection mappers into focused modules.

- `0.1.25` continues NNS cleanup by splitting cached leaf report model roots
  plus topology runtime and request wiring into focused modules.

- `0.1.24` continues module cleanup by splitting subnet catalog tests, models,
  and text rendering plus NNS topology clap command construction into focused
  modules.

- `0.1.23` decomposes NNS topology report models, provider/summary assembly,
  and shared topology fixtures into focused modules.

- `0.1.22` splits cached NNS leaf report roots and subnet catalog host/report
  support while preserving existing cache and report behavior.

- `0.1.21` splits major NNS, SNS, registry, subnet catalog, and neuron-cache
  test and orchestration modules into focused files.

- `0.1.20` splits SNS report assembly, report-root orchestration, and SNS
  report tests by command family and cache behavior.

- `0.1.19` splits live SNS fetch, conversion, and report model wiring for
  deployed SNSes, tokens, governance parameters, proposals, and neurons.

- `0.1.18` splits the custom NNS node command and SNS command runtime, clap,
  and parser modules without changing command behavior.

- `0.1.17` deduplicates standard cached NNS leaf report adapters and reuses a
  shared mainnet-only network guard.

- `0.1.16` centralizes cached NNS leaf cache paths, refresh-lock paths, JSON
  cache error mapping, refresh text rendering, and shared JSON cache reports.

- `0.1.15` hardens cache writes, refresh locks, SNS neuron cache paths,
  command-output helpers, and panic-prone timestamp/subnet internals.

- `0.1.14` splits live SNS, NNS topology/subnet CLI, SNS command specs,
  cache-file support, subnet catalog request/report, and neuron-cache modules.

- `0.1.13` tightens shared NNS leaf helpers, JSON cache providers, topology
  dispatch, and SNS lookup result types.

- `0.1.12` splits IC registry relation inventory fetching and async
  live-source querying out of broader registry modules.

- `0.1.11` decomposes IC registry adapters, wire types, transport helpers,
  relation inventory helpers, domain projections, and annotations.

- `0.1.10` splits NNS component report text/model modules and NNS topology
  helper/rendering modules.

- `0.1.9` splits SNS command, report, cache, and live source modules and
  renames the live SNS source to match its broader role.

- `0.1.8` splits SNS neuron cache model/path helpers and live SNS Candid plus
  conversion helpers.

- `0.1.7` corrects release target boundaries, tightens package/publish
  requirements, and splits NNS topology aggregation modules.

- `0.1.6` hardens SNS neuron snapshot cache coverage, splits cache mechanics,
  deduplicates request construction, and normalizes Rust module layout.

- `0.1.5` splits SNS command dispatch and text rendering, deduplicates live SNS
  Candid queries, and reuses shared command helpers.

- `0.1.4` adds bounded SNS governance proposal listings and direct proposal
  detail lookup commands.

- `0.1.3` adds local SNS neuron snapshot cache inspection commands and fixes
  cache-backed neuron sorts to avoid unnecessary live fetches.

- `0.1.2` deduplicates SNS lookup plumbing, moves duration formatting into a
  helper, splits SNS report internals, and preserves failed refresh metadata.

- `0.1.1` adds SNS nervous system parameter lookup and shared token/e8s amount
  rendering.

- `0.1.0` introduces complete SNS neuron snapshots, cache-backed neuron sorts,
  refresh progress output, and Canic-style release helper commands.

## [0.0.x] - 2026-06-13 - Initial IC query extraction

Detailed patch breakdown: [docs/changelog/0.0.md](docs/changelog/0.0.md)

- `0.0.9` adds bounded SNS neuron listings, 0.1 snapshot-cache design docs,
  and stricter SNS lookup validation.

- `0.0.8` adds SNS token metadata lookup, stable SNS-W numeric ids, logo
  payload hiding, and long-value table fixes.

- `0.0.7` deduplicates cached NNS leaf command plumbing and topology read
  request construction.

- `0.0.6` deduplicates cached NNS leaf dispatch and splits topology read
  parsing plus report request/model code.

- `0.0.5` lets clap render top-level help and uses clap range validation for
  subnet list limits.

- `0.0.4` adds release helpers, SNS metadata fallback visibility, SNS list
  sorting, CLI smoke tests, cache-write hardening, and NNS/SNS query cleanup.

- `0.0.3` adds SNS list/info lookups, groups NNS commands under `src/nns/`,
  renames the local cache directory to `.icq/`, and updates README positioning.

- `0.0.1` creates the `icq` executable, extracts the former Canic NNS query
  surface, and adds read-only text/JSON metadata reports.
