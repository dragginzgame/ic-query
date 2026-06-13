# AGENTS.md

This file is normative for automated coding agents working in this repository.
If code or habit conflicts with this file, this file wins.

## Session Handoff

- At the start of a new session, read `README.md`, `CHANGELOG.md`, and any
  relevant files under `docs/design/`.
- Treat local docs and the current worktree as the handoff. Do not replay old
  chat history unless it is needed to resolve ambiguity.
- `../canic` may be read for reference when the maintainer asks, but never edit
  files outside this repository.

## Git And Release Boundaries

- Automated agents must never run `git commit`.
- Automated agents must never run `git tag`.
- Automated agents must never run `git push`.
- Automated agents must never run release targets or scripts that create
  commits, tags, pushes, or version bumps, including `make patch`,
  `make minor`, `make major`, `make release-patch`, `make release-minor`,
  `make release-major`, and `make release-push`.
- Prepare changes in the working tree only. The human maintainer handles
  commits, tags, version bumps, releases, and pushes.

## Changelog

- Use root `CHANGELOG.md` as the concise release ledger.
- Add or update changelog entries when the maintainer asks for release notes or
  when a change is clearly part of an active release slice.
- Keep changelog text factual and user-facing. Avoid implementation noise unless
  it affects behavior, compatibility, release flow, or operations.

## Ownership

- CLI parsing and command dispatch live under `src/*/mod.rs` and shared
  helpers in `src/cli/`.
- Report construction, host calls, cache reads, and text rendering belong in the
  relevant report module.
- Reusable cache mechanics belong in `src/cache_file.rs`.
- Reusable formatting belongs in small shared modules such as `src/table.rs`,
  `src/duration.rs`, and token amount helpers.
- NNS and SNS command families should stay separate unless a helper is genuinely
  shared.

## Layering

- Keep clap parsing separate from report-building logic.
- Keep live host calls behind source traits or local helper boundaries so tests
  can use fixtures.
- Text output is for humans and may be compact or formatted.
- JSON output should preserve raw, script-friendly fields and avoid lossy
  display conversions.
- Cache keys describe collected data, not views. Sorts, limits, and text
  verbosity are view options and must not change complete snapshot identity.

## Style

- Rust edition is 2024.
- Prefer existing local patterns over introducing new frameworks or broad
  abstractions.
- Keep imports at file top.
- Keep changes scoped to the requested task.
- Comment intent, invariants, and non-obvious behavior only.
- Avoid restructuring modules during feature work unless the restructuring is
  the task.

## Testing

- Prefer targeted tests first; broaden when risk warrants it.
- Unit tests should live next to the code they exercise.
- Use fixture sources for networked behavior instead of live network calls in
  unit tests.
- Assert typed errors or observable behavior, not brittle full error strings,
  unless the exact CLI text is the contract being tested.
- Report which checks passed and which checks were not run.

## Security And Network

- This is a read-only metadata query CLI. Do not add mutation behavior unless
  the maintainer explicitly changes the project scope.
- Keep network endpoints explicit in command options and reports.
- Do not hide live network calls behind commands that look cache-only.

## Checklist

- Preserve dirty worktree state and do not revert user changes.
- Inspect relevant files before editing.
- Keep text output readable and JSON output stable/raw.
- Keep cache refresh behavior explicit: missing cache, refresh progress,
  partial failure, stale locks, and complete snapshots should be visible.
- Leave commits, tags, version bumps, releases, and pushes to the maintainer.
