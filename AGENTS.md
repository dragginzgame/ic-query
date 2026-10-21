# AGENTS.md

This file is normative for automated coding agents working in this repository.
Apply the shared baseline and the explicit local exceptions below; code and habit
do not override them.

## Shared baseline and local overlay

Apply the local [engineering baseline](DRAGGINZGAME.md) first.
[.shared-tooling.snapshot](.shared-tooling.snapshot) owns its reviewed revision
and selected tooling identities; this overlay no longer inherits the historical
upstream AGENTS.md as a separate policy baseline.

Preserve these standing maintainer exceptions and their local scope:

- Release execution, version bumps and release refs remain maintainer-only,
  as described below; contribution PR authority is separate.
- The Changelog section retains Query’s requested/active-release note cadence,
  selected-or-next-patch default and CLI examples, for one explicit release ledger.
- Public-metadata schemas remain `1`, with breaking shapes replaced through
  the local hard-cut contract. Identify explicit resets and coordinate consumers;
  no exception authorizes deleting caller evidence or ignoring external obligations.

Tool setup and local extension targets follow [local setup](docs/local-setup.md);
release adapters follow [the local release workflow](docs/release-workflow.md).
Consumer release fixtures select only `scripts/ci/test-release-runner.sh` with
simulated Git effects; native tracking, PR and owner-metadata fixtures are not
selected. The local [host matrix](docs/supported-hosts.md) owns qualification.
Policy adoption does not activate hooks, schedules, installers or release commands.

## Session Handoff

- At session start, read `README.md`, `CHANGELOG.md`, and relevant
  `docs/design/` files. Treat local docs plus the current worktree as the
  handoff; use old chat only to resolve ambiguity. Task-relevant read-only sibling
  inspection follows the shared baseline and explicit user exclusions; never
  edit outside this repository.

## Git And Release Boundaries

- Ordinary fixes and continuation prepare working-tree changes only. An explicit
  commit request authorizes a scoped local commit; an explicit PR request includes
  its topic branch, scoped commits, branch push and PR creation/update under
  [the contribution rules](rules/contributions.md). PR delivery does not authorize
  merging, direct integration-branch pushes or rewriting shared history.
- Automated agents must never run release/version-bump targets or scripts,
  create release tags, or push release refs, including `make release-patch`,
  `make release-minor`, `make release-major`, `make release-resume`, and the
  `release-prepare-version` adapter. Isolated fixtures may prepare Git indexes
  and trees and simulate release effects with command stubs; they must never
  create real commits, tags or pushes, or change this repository's package
  metadata or index.
- The maintainer handles version bumps, releases, release commits/tags and
  release pushes. Contribution authority does not override this release boundary.

## Pre-1.0 Compatibility

- Breaking public API or semantic changes require a minor release before 1.0.
  Do not fit an incompatible change into an assigned patch release; schema-1
  hard cuts do not waive release selection or consumer retirement obligations.
- Classify changes at Query's published consumer boundary. Contributor-only
  tooling, CI, formatting and PATH changes use patch notes when public APIs,
  product behavior and supported consumer build requirements remain unchanged;
  an upstream tooling minor does not automatically require a Query minor.
- Before `1.0.0`, every current emitted or persisted schema version must remain
  `1`, including report, cache, sidecar, and lock schemas. Breaking shape
  changes replace the version-1 contract in place; do not increment a schema
  to `2` or higher, retain an older reader, or add a migration. Treat any
  existing schema version above `1` as nonconforming debt to remove in a
  dedicated hard-cut change, not as precedent for another version bump. New
  versioned contracts start and remain at `1` until the package reaches 1.0;
  this rule governs contract/schema identifiers, not Cargo package releases.
- Before `1.0.0`, every breaking change is a hard cut. Do not preserve old
  behavior through compatibility shims, CLI or Serde aliases, deprecated
  wrappers or re-exports, dual parsers, fallback readers, legacy schema
  branches, or automatic migrations.
- Delete replaced surfaces and the redundant implementation they supported in
  the same change. Historical changelogs may describe removed behavior, but
  they do not justify keeping compatibility code.
- Do not add or retain anti-resurrection tests whose only purpose is proving
  that a removed command, field, schema, alias, wrapper, or behavior remains
  unavailable. Test the current supported contract and generic invalid input,
  not historical forms.
- Ordinary internal Rust type aliases that name a current shared type are not
  compatibility aliases. They remain subject to the normal ownership and
  redundancy rules in this file.

## Changelog

- Use root `CHANGELOG.md` as the concise release ledger. Update it only for
  requested release notes or active release slices. Keep entries factual and
  user-facing; include implementation detail only when it affects behavior,
  compatibility, release flow, or operations.
- Every changelog request prepares a numbered release entry and matching
  detailed heading, never a new `Unreleased` heading or suffix. Use the
  maintainer's explicit target version; otherwise default to the next patch
  after the current Cargo package version. Reuse an already prepared entry for
  that target rather than advancing the version on repeated changelog requests.
  This assigns the notes to a release only: do not change manifests, lockfiles,
  generated version metadata, or run version-bump or release commands.
- When the maintainer assigns a target package version or asks whether the
  current patch should ship as that version, treat it as an active release
  slice: prepare the matching root release-ledger entry and detailed changelog
  heading before handoff without waiting for a separate changelog request.
- Detailed patch breakdowns live in `docs/changelog/<major>.<minor>.md`; link
  the detailed file from the matching minor line in root `CHANGELOG.md` when
  present.
- New pending batches use one numbered, undated `## [X.Y.Z]` root heading
  and matching detailed heading. Release preparation finalizes the selected
  root heading with its UTC date. Preserve historical minor-line indexes.
- When a patch introduces any new CLI surface, include a fenced `bash` example
  in both root `CHANGELOG.md` and the matching detailed changelog file. "New
  CLI surface" includes commands, subcommands, options, option values, and new
  supported combinations of existing options. Use `bash`, not `sh`, for the
  fence language. The example must show the newly introduced surface directly.
  Do not add command examples for cleanup-only patches that do not change CLI
  behavior.

## Code Boundaries

- Ownership: CLI parsing and dispatch live under `crates/ic-query-cli/src/`.
  Report construction, host calls, cache reads, and text rendering belong in
  the relevant report module under `crates/ic-query/src/`; reusable cache
  mechanics belong in its `cache_file/` module; reusable formatting belongs
  in Host Text, while report-specific token amount and unit-selection policy
  stays in local helpers.
- Process arguments, stdout/stderr, terminal detection, and progress rendering
  belong exclusively to `ic-query-cli`. The reusable library may emit typed
  progress events, but it must never select or write to a process output sink.
- Keep NNS and SNS command families separate unless a helper is genuinely
  shared. Keep clap parsing separate from report building, and keep live host
  calls behind source traits or local helpers so tests can use fixtures.
- Text output is human-facing and may be compact or formatted; JSON output
  should preserve raw, script-friendly fields and avoid lossy display
  conversions.
- Separate a human-facing metadata, provenance, or summary preamble from a
  following table with one blank line. Treat later titled table blocks as
  separate visual sections too.
- Cache keys describe collected data, not views. Sorts, limits, and text
  verbosity are view options and must not change complete snapshot identity.

## Style

- Rust edition is 2024. Prefer existing local patterns over new frameworks or
  broad abstractions.
- Modules with child files use directory `mod.rs`; never keep both `foo.rs`
  and `foo/`, and do not use `#[path = "..."]`.
- Do not split code into one-function leaf modules unless that function owns a
  real boundary such as parsing, IO, cache policy, or a reusable public helper.
  Prefer grouping small related helpers in one owner module.
- More than three generic parameters on a function, struct, or impl is a design
  smell; prefer associated types, concrete provider traits, or smaller helpers
  unless the extra type parameters are clearly justified.
- Structs, enums, and traits that cross module boundaries, including
  `pub(crate)` and `pub(in ...)` items, use the repository section-style doc
  block: empty `///`, type name, empty `///`, description, closing empty `///`,
  then a blank line before attributes or the item.
- Prefer `#[expect(...)]` over `#[allow(...)]` for lint suppressions so stale
  suppressions are caught. Use `#[allow(...)]` only when `#[expect(...)]`
  generates false positives or otherwise cannot model the lint accurately.
- Keep imports at file top, changes scoped to the task, comments limited to
  intent/invariants/non-obvious behavior, and avoid module restructuring unless
  restructuring is the task.
- Every CLI `Commands` section must print subcommands alphabetically by command
  name, including Clap's generated `help` command. Enforce this through the
  shared command-tree boundary rather than relying on builder insertion order.
- Invoking a CLI command namespace without its next operation must print the
  same complete local help as its explicit `help` subcommand and exit
  successfully. This includes namespaces preceded by a valid scope argument;
  invalid values and incomplete leaf operations remain errors.

## Testing

- Follow the [shared validation cadence](DRAGGINZGAME.md#validation-and-evidence).
  The local targets and native matrix define Query's product qualification scope.
- Check for active builds before compilation or source edits. Do not change
  source under validation or compete for its build lock. Preserve build and
  evidence artifacts; do not run cleanup helpers that erase consumer outputs.
  Helpers may remove their own exact temporary files.
- Prepare the selected lockfile's offline cache explicitly before validation.
  Keep validation locked/offline; report missing dependencies rather than
  silently retrying online or changing versions.
- Keep unit tests next to the code. Prefer `tests.rs` or `tests/mod.rs` for
  large groups; small inline `mod tests { ... }` blocks are fine.
- Use fixture sources instead of live network calls in unit tests. Assert typed
  errors or observable behavior, not brittle full strings, unless exact CLI
  text is the contract.
- Report which checks passed and which checks were not run.

## Tooling, feedback and host support

Use the adopted [maintenance and feedback rules](rules/agent-maintenance.md),
[shared setup](docs/local-setup.md) and [host matrix](docs/supported-hosts.md).
Keep ordinary checks offline and installation explicit; authorised dependency
preparation follows [Cargo network policy](rules/cargo-dependencies.md#cargo-network-policy).
The snapshot owns shared tooling; never patch its exports in place. Native host
qualification remains distinct from source review or Linux-only evidence.

## Security And Network

- Consume Candid contract types, derives, encoders, bounded reply decoding,
  native Agent APIs, certificate/hash-tree types and standard ledger DTOs
  through the published `ic-platform` dependency. Host owns strict bounded CBOR decoding; Agent owns
  authentication. Keep endpoint/root-key admission, quotas, retry policy and
  error projection local. The `canister` adapter and typed Wasm probe use
  Platform's `cdk` feature and entrypoint generator; do not declare Candid, CDK
  or either macro package directly. Native Testkit fixtures use Platform for
  required external Candid protocols and Wire for owned RPCs. Preserve the published
  runtime semantics and upstream executor/ic0 identities.

- Prefer `ic-agent` for IC protocol calls whenever it supplies the required
  capability, including development and verification tooling. Use typed Candid
  arguments and direct reply bytes; do not route canister calls or certified
  state reads through ICP CLI JSON envelopes or hexadecimal response decoding.
  Keep ICP CLI at project, build, deployment and managed-runtime boundaries.
  Preserve explicit endpoints, local/mainnet root-key policy, deadlines and
  original reply evidence when replacing subprocess calls.
- This is a read-only metadata query CLI. Do not add mutation behavior unless
  the maintainer explicitly changes the project scope.
- Keep network endpoints explicit in command options and reports. Do not hide
  live network calls behind commands that look cache-only.

## Checklist

- Preserve dirty worktree state; do not revert user changes. Inspect relevant
  files before editing.
- Keep text output readable and JSON output stable/raw.
- Keep cache refresh behavior explicit: missing cache, refresh progress,
  partial failure, lock contention, invalid coordination files, and complete
  snapshots should be visible.
