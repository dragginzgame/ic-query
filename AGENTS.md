# AGENTS.md

This file is normative for automated coding agents working in this repository.
If code or habit conflicts with this file, this file wins.

## Session Handoff

- At session start, read `README.md`, `CHANGELOG.md`, and relevant
  `docs/design/` files. Treat local docs plus the current worktree as the
  handoff; use old chat only to resolve ambiguity. Read `../canic` only when
  asked, and never edit outside this repository.

## Git And Release Boundaries

- Automated agents must never run `git commit`, `git tag`, `git push`, or
  release/version-bump targets or scripts, including `make patch`,
  `make minor`, `make major`, `make release-patch`, `make release-minor`,
  `make release-major`, `make release-stage`, `make release-commit`, and
  `make release-push`.
- Prepare working-tree changes only; the maintainer handles commits, tags,
  version bumps, releases, and pushes.

## Changelog

- Use root `CHANGELOG.md` as the concise release ledger. Update it only for
  requested release notes or active release slices. Keep entries factual and
  user-facing; include implementation detail only when it affects behavior,
  compatibility, release flow, or operations.
- Detailed patch breakdowns live in `docs/changelog/<major>.<minor>.md`; link
  the detailed file from the matching minor line in root `CHANGELOG.md` when
  present.
- When a patch introduces any new CLI surface, include a fenced `bash` example
  in both root `CHANGELOG.md` and the matching detailed changelog file. "New
  CLI surface" includes commands, subcommands, options, option values, and new
  supported combinations of existing options. Use `bash`, not `sh`, for the
  fence language. The example must show the newly introduced surface directly.
  Do not add command examples for cleanup-only patches that do not change CLI
  behavior.

## Code Boundaries

- Ownership: CLI parsing and dispatch live under `src/*/mod.rs` and
  `src/cli/`; report construction, host calls, cache reads, and text rendering
  belong in the relevant report module; reusable cache mechanics belong in
  `src/cache_file.rs`; reusable formatting belongs in small shared modules
  such as `src/table.rs`, `src/duration.rs`, and token amount helpers.
- Keep NNS and SNS command families separate unless a helper is genuinely
  shared. Keep clap parsing separate from report building, and keep live host
  calls behind source traits or local helpers so tests can use fixtures.
- Text output is human-facing and may be compact or formatted; JSON output
  should preserve raw, script-friendly fields and avoid lossy display
  conversions.
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

## Testing

- Prefer targeted tests first; broaden when risk warrants it.
- Keep unit tests next to the code. Prefer `tests.rs` or `tests/mod.rs` for
  large groups; small inline `mod tests { ... }` blocks are fine.
- Use fixture sources instead of live network calls in unit tests. Assert typed
  errors or observable behavior, not brittle full strings, unless exact CLI
  text is the contract.
- Report which checks passed and which checks were not run.

## Security And Network

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
