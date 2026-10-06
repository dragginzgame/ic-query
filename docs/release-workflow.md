# Release workflow

The maintainer runs one of `make release-patch`, `make release-minor`, or
`make release-major`. Each uses the same complete `make ci` gate, then prepares
metadata and notes, stages the explicit release files, commits, creates an
annotated tag, and atomically pushes the selected branch and tag. Publication
remains the separate `make publish` command.

The runner and its version/changelog helpers are an immutable Shared Tooling
snapshot at
[`a37771f1b6b5fc9a88ed6ab3b705bdda35cd8fa3`](https://github.com/dragginzgame/shared-tooling/tree/a37771f1b6b5fc9a88ed6ab3b705bdda35cd8fa3) (0.1.6),
recorded in `.shared-tooling.snapshot`. See the reviewed
[common release contract](https://github.com/dragginzgame/shared-tooling/blob/a37771f1b6b5fc9a88ed6ab3b705bdda35cd8fa3/docs/releases.md).
`make shared-tooling-check` verifies the selected files' digests and executable
modes without a network request. This snapshot selects release mechanics,
publication tag checking, dependency-pin checking, repository-local tool setup
and LOC reporting. The engineering baseline remains the separate revision
pinned in `AGENTS.md`.
Update shared files through the upstream snapshot exporter, never local patches.
The exporter protects staged, unstaged, deleted and untracked destinations before
replacement. For an authorized refresh of an already-staged snapshot, export into
an isolated checkout, verify the existing snapshot, and reconcile only its known
shared bytes; preserve the consumer index and unrelated edits.

The publication tag adapter selects this workspace's package version and exact
HEAD commit, then delegates annotated-tag identity checks to the vendored
`scripts/ci/check-release-tag.sh`. Consumer fixtures retain clean-worktree,
selected-version, stale/missing/lightweight-tag and publication-order checks.

## Selections and preparation

The default destination is `RELEASE_REMOTE=origin`, `RELEASE_BRANCH=main`.
Override both explicitly when using another named remote or branch. The branch
must be checked out, and the remote must have exactly one push URL. Select only
one release target per Make invocation. The default Make target prints help.

Commit reviewed source and the numbered pending notes before invoking a release.
The initial preflight requires a clean worktree and an existing lockfile,
checks root and detailed candidate headings before validation or saved intent,
prepares its cache with `cargo fetch --locked --offline`, and rejects missing
dependencies without an online retry. Explicitly prepare missing cached inputs
under separate network authority before retrying; do not regenerate the lockfile.
Authorized dependency changes prepare every affected independent graph before
release validation. IC Query currently has one workspace and one root lockfile;
examples and the Governance probe share that graph. Verify it with
`cargo metadata --locked --offline --format-version 1` after preparation.
Release cache fetching preserves the prepared selection.

`make install-dev` prepares the checksum-verified jq/yq pair under
`.tools/host/bin`, alongside the existing development tools. `make dependency-pins-check` checks parsed Cargo/Action inputs
and tracked workspace lockfiles offline; the complete CI and release gate includes
it. The parser identities and native setup are documented in
[supported hosts](supported-hosts.md). This declaration check does not qualify
runtime behavior or replace locked compilation and tests.

Pending notes use one undated `## [X.Y.Z]` root heading, linked to the matching
`docs/changelog/<major>.<minor>.md` heading. Preserve historical minor-line
indexes. The selected release kind must compute the same candidate as the notes;
conflicting notes fail before metadata mutation. Preparation finalizes the
root and detailed headings with the saved UTC release date.

The consumer adapter changes only the workspace package version, its owned
`ic-query` dependency version, the two owned lockfile package versions, and
the README/library dependency examples. External dependency records and their
checksums remain byte-for-byte unchanged. The release file set is exactly:

- `Cargo.toml` and `Cargo.lock`;
- `README.md` and `docs/library-usage.md`;
- `CHANGELOG.md` and the selected minor-line detailed changelog.

One metadata helper owns both this file set and the canonical workspace version
used by Make, changelog checks, tag checks, and publication. The shared runner
owns release commit/tree/tag checks; consumer hooks check the prepared payload
and its validation evidence. Late hooks inspect committed files at the explicit
`RELEASE_COMMIT`, which may precede HEAD, and compare them with transformations
of the saved `RELEASE_SOURCE` inputs. Their file admission compares those two
commits; newer committed fixes are validated by their own next-release gate.
Publication separately requires the current
version's annotated tag at HEAD before contacting the registry.

Prepared checks compare every file with transformations of the saved validated
inputs, and check Cargo metadata with `--locked --offline`. An unrelated changed
path, untracked source, conflicting payload, or missing validation evidence stops
the workflow. Admission examines staged and unstaged paths separately. Before
commit, every declared index file must equal the validated prepared payload;
restoring a working file does not hide an unrelated staged edit. Build outputs
are retained; release targets never run cleanup.

## Evidence and recovery

Unique validation directories under Git's `release-state` directory retain the
selected source, previous/candidate versions, kind, UTC date, remote/branch,
before-files, complete validation log, and success marker. Failed attempts retain
their own logs; subsequent validation does not overwrite them. Shared schema-1
plans retain intent before metadata preparation and record the exact staged tree
and completed phases. Do not remove plans or evidence to obtain a clean retry.
CI uses the pinned `run-validation-targets.sh` with fail-fast ordering. Release
validation uses that same logger around the complete gate and retains unique raw
failure logs under its attempt directory's `validation-failures/`. Ordinary CI
retains failures in `target/validation-failures/`. Retries preserve earlier raw
logs; `latest.log` is a convenience copy. If the configured destination fails,
the logger preserves and reports its temporary logs instead of deleting them.

A normal target rerun first reconciles an unfinished release at its saved
candidate. An unchanged same-kind retry finishes only that release. After its
release commit exists, a newer descendant or different requested increment is
handled internally: finish the older release, then read the current manifest
version and run fresh preflight and complete validation for the next increment.
For example, after an interrupted `0.47.0` minor push and a committed callback
fix, `make release-patch` completes `0.47.0` and validates the fix for `0.47.1`.
Prepare matching numbered next-release notes with that fix before invoking the
command. Maintainers do not need to edit plans or reconstruct an old checkout.
`make release-resume VERSION=X.Y.Z` explicitly selects only retained intent and
starts no next increment. Source, metadata, destination, index/tree, and tag
conflicts stop recovery.
An occupied lock is never stolen; inspect its owner before any manual removal.
An interrupted preparation admits only files equal to their validated before
or expected after content, then finishes that same preparation. The canonical
package version is published last.

The shared runner pushes with `--no-follow-tags --atomic` and explicit refspecs
for the selected branch and exact annotated tag. It rejects unsupported atomic
push without a fallback. A lost push reply is reconciled against both remote
identities; an unavailable remote stops recovery, and matching identities finish
without replaying the push. Older recovery pushes that exact release commit,
preserving the existing tag; newer fixes are pushed only after their own release
validation. If the remote already contains the older release, its confirmed tip
is preserved while adding a missing tag. Diverged history stops recovery.
No force push or unrelated tag publication is used. Failed Git inventory stops
both release admission and publication prerequisites, even with empty output.

The former separate bump/stage/commit/push targets and local bump/commit scripts
are retired. Use the standard maintainer targets above. Agents prepare source and
run isolated fixture checks; they never invoke the real release workflow or its
version-changing adapter against this repository.

## Qualification

`make release-guards-check` uses isolated workspaces and file-backed Git/Cargo
stubs. It exercises all increments, identical phase order, dependency selection,
missing-cache and gate failures, metadata/payload conflicts, explicit staging,
atomic push scope, retained artifacts/logs, occupied locks, and recovery after
lost preparation/push replies. `make ci-scripts-check` checks the complete CI
target sequence; `make publish-guards-check` checks separate publication behavior.
These focused fixtures perform no real commits, tags, pushes or publication.
The real consumer Make callbacks also cover an interrupted minor release followed
by committed fixes and patch/minor/major or exact resume, fresh-gate failure and
retry, historical metadata binding, failed Git inventory, and history/tag/remote
conflicts. Old plans, tags, validated inputs and logs remain unchanged.
`scripts/ci/check-release-metadata.sh` supplements these stubs with a real Git
index and synthetic source tree derived from existing history, without new
commits. It exercises hidden staged changes, stale index payloads, conflicting
headings and the actual release adapter/logger across two failed Make gates.
It isolates inherited Make overrides and logger checkout identity. A distinct
parent checkout runs this fixture through the actual shared logger and rejects
gate execution in the parent, covering invocation from release validation.
The 0.1.5 logger keeps its temporary checkout/snapshot identity within its own
dispatch. Release selections, failure-log policy and nesting depth still reach
nested targets. Independent fixtures retain their own explicit selections.

Native CI qualification is configured for the [supported host matrix](supported-hosts.md),
including Apple's system Bash on both macOS architectures. A passing Linux
fixture, added matrix, or available installer is not native macOS qualification.
Record the matching native workflow run before claiming that qualification.

The upstream [0.1.6 CI run](https://github.com/dragginzgame/shared-tooling/actions/runs/37450707625)
for `a37771f1b6b5fc9a88ed6ab3b705bdda35cd8fa3` passed Linux and both macOS
architectures. Upstream checks do not qualify the consumer's parser setup or
release adapters; matching IC Query native runs remain required.
