# AGENTS.md

This file is normative for automated coding agents working in this repository.
If code or habit conflicts with this file, this file wins.

## Shared Baseline And Local Overlay

- Adopt the [Shared Tooling engineering baseline](https://github.com/dragginzgame/shared-tooling/blob/5a65686d1da9e039b22b3e17694ea083d82cfa9e/AGENTS.md)
  at reviewed revision `5a65686d1da9e039b22b3e17694ea083d82cfa9e`.
  This file is the local overlay; a moving sibling checkout is not inherited
  automatically. Review a new revision before changing this reference.
- Retain these standing maintainer instructions as scoped exceptions:
  numbered changelog drafts use the explicitly selected version or the next
  patch, rather than an undecided `Draft`; repository-owned public-metadata
  schemas remain `1` and breaking shapes use the hard-cut policy below, rather
  than retaining a frozen older discriminator. The reasons are one numbered
  release ledger and one current pre-1.0 metadata contract without migrations.
  These exceptions do not authorize deleting caller-owned evidence or ignoring
  external consumers; identify required explicit resets and coordinated updates.
- Product authority, cache/network identities, supported features, numeric
  limits, formatting, and exact validation/release commands remain local.
  The stricter prohibition on agent-run release effects below remains in force.
- Read-only workspace version checking, release execution and lockfile
  transformation, publication tag and registry
  checking, dependency-pin and documentation-link checking, isolated RustSec
  preparation, formatter prerequisites, local tool setup, LOC reporting,
  exact-commit CI inspection and the standard Rust workspace layout adopt
  Shared Tooling at reviewed revision `b2646cde9abbc8861857a4379c683a0c19eba43e`; the selected
  immutable helpers are recorded in `.shared-tooling.snapshot`. This scoped
  tooling adoption does not upgrade the engineering baseline reference above.
  Apply the runner contract in `docs/releases.md` and local adapters described
  in `docs/release-workflow.md`, and dependency rules in
  `rules/dependency-pinning.md` with local setup in `docs/local-setup.md`,
  `docs/ic-tools.md`, registry observation and local link checks in
  `docs/verification-helpers.md`,
  and the consumer host matrix in `docs/supported-hosts.md`.
  Dependency preparation adopts the Cargo network policy in
  `rules/cargo-dependencies.md`: authorized dependency updates include their
  registry/Git access, and standard release/deployment preparation fetches the
  selected lock before offline validation. Explicit caller offline settings
  remain authoritative; the agent-run release prohibition below still applies.
  `DRAGGINZGAME.md` and linked snapshot guides accompany those contracts;
  they do not silently replace the explicitly
  pinned engineering baseline or activate unrelated hook/tooling adoption.
  The linked `tasks/` catalog and local schedule templates are available offline;
  their adoption does not activate scheduled execution. Consumer release tests
  select the simulation-only `scripts/ci/test-release-runner.sh`; native tracking,
  PR and owner metadata fixtures remain outside that selection.
  The root virtual workspace and packages under `crates/` follow
  `rules/rust-workspaces.md`; `docs/supported-hosts.md` remains the consumer-owned
  host matrix rather than a shared snapshot file.
- Contribution and PR authority adopts `rules/contributions.md` and the
  corresponding Scope and authorization guidance at that same reviewed revision.
  This scoped adoption replaces the older baseline's blanket agent-commit ban;
  the engineering baseline remains separately pinned. The local prohibition on
  agent-run release effects below remains stricter than the shared release rule.

## Session Handoff

- At session start, read `README.md`, `CHANGELOG.md`, and relevant
  `docs/design/` files. Treat local docs plus the current worktree as the
  handoff; use old chat only to resolve ambiguity. Read `../canic` only when
  asked, and never edit outside this repository.

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
  in small shared modules such as `table.rs`, `duration.rs`, and token amount
  helpers.
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

- Prefer targeted tests first; broaden when risk warrants it.
- Run focused checks automatically. Broad workspace validation, full CI, and
  release gates require an explicit request or their configured CI pipeline;
  ordinary continuation and readiness requests do not authorize those gates.
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

## Tooling And Feedback

- Keep wrappers small and effects explicit. Use the maintained implementation
  language for substantial tooling; do not add new Python tooling. Existing
  consumer tools are not permission to introduce another implementation.
- Keep CI/release shared tools as reviewed snapshots, never symlinks or mutable
  sibling dependencies. Fix shared tooling upstream only when authorized; do
  not patch a vendored copy in place.
- Apply the [shared repair and owning-repository feedback workflow](rules/agent-maintenance.md)
  and the Scope and authorization / Feedback and handoff sections in
  `DRAGGINZGAME.md` from the reviewed snapshot above. This scoped adoption carries
  the maintainer's standing issue-reporting authorization and replaces the local
  workflow duplication; the separately pinned engineering baseline and local
  release boundaries remain authoritative for other obligations.

## Host Support

- Preserve required macOS support for dependency setup, native tools, builds,
  tests, CI, and release tooling. Declare macOS versions, architectures, and
  prerequisites in a local host matrix; isolate GNU/BSD and process differences
  at their owning boundary. Portable scripts target Bash 3.2 unless an explicit
  support-matrix decision changes that requirement.
- Qualify affected workflows through native CI or recorded native execution.
  Linux passes and installer branches do not qualify macOS. Report missing
  host coverage and portability failures as gaps without weakening the support
  requirement or claiming unexecuted checks passed.

## Security And Network

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
  partial failure, stale locks, and complete snapshots should be visible.
