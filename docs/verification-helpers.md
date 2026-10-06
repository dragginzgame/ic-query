# Focused verification helpers

These optional helpers share mechanical checks. Consumers retain document
selection, release validation gates, publication authority and registry polling
policy. They require Perl core modules or Bash 3.2+, as noted below, and run on
Linux and macOS. The portable regression suite includes offline fixtures;
native CI qualifies each supported host separately.

## Local documentation links

```sh
perl scripts/ci/check-documentation-links.pl --root /path/to/repo README.md docs/guide.md
```

Supply a root and at least one document. Relative document arguments resolve
from that root; absolute document arguments are also accepted. Relative links
resolve from their containing document. Absolute link paths retain filesystem
semantics: `/guide.md` means the host's `/guide.md`, not the repository root.
Use relative links for repository navigation. Targets outside the root are not
prohibited; this is an existence check, not a containment boundary.

The Perl checker handles ordinary inline links, images and reference definitions,
angle-bracket paths with spaces, percent-encoded filenames and optional titles.
It removes query/fragment suffixes before checking existence, skips URI schemes,
protocol-relative URLs and anchor-only links, and ignores backtick/tilde fenced
examples and inline backtick code. Both files and directories may be targets.
Missing input documents or local targets fail the command.

This deliberately follows IcyDB's lightweight checker. It does not validate
anchors, network availability, reference-label resolution or a complete Markdown
renderer grammar (for example, nested/escaped parentheses in bare destinations,
HTML links and indented code blocks). Use angle brackets around complex paths.
A passing check only covers the supported local references in the supplied
roster. Product-specific inventories, version examples and document selection
remain consumer-owned.

Shared Tooling's `make check-doc-links` selects root, audit, rule and guide
Markdown files and runs in CI and the release gate. Consumers choose their own
roster; snapshot-relative links must exist in the adopted file set.

## Standard release command adoption

```sh
bash scripts/ci/check-release-commands.sh /path/to/repo tool-versions.env
```

The root's Makefile is copied into a temporary checkout without Git metadata.
Additional arguments select reviewed, repository-relative files needed while
Make parses that copy. For example, Canic includes `tool-versions.env`; IcyDB
reads `scripts/ci/actionlint-checksums.tsv`. No files are implicitly discovered
or sourced from siblings. The copied Makefile must call
`scripts/ci/run-release.sh` relative to its fixture checkout (directly or through
Bash). That path contains a substitute that records exact arguments and returns
a chosen status. GNU Make and Bash are prerequisites.

The checker exercises patch/minor/major routing, resume with an explicit version,
explicit remote/branch propagation, runner success and failure propagation,
and rejection of every pair of different release targets before dispatch.
It clears inherited Make controls and validation logger identity so nested
validation cannot redirect or skip the checks. Success removes its temporary
fixture; failure prints and retains its path with separate command logs and
argument evidence.

Review the selected Makefile's parse-time expressions, includes, prerequisites
and recipes before invoking this helper. They must be read-only except for the
substituted runner. This is not a sandbox for arbitrary Makefiles: absolute
commands or unrelated prerequisites could still have effects. It verifies the
entry-point contract, not release execution, Git reconciliation, receipts,
publication or caches. Keep those consumer-specific tests alongside a thin
call to this helper, and keep the shared runner's own fixtures upstream.

Shared Tooling exercises its own Makefile in the portable suite; the focused
`make check-release-commands` target offers the same adoption check.

## Exact crates.io version observation

```sh
status=0
bash scripts/ci/check-crates-io-version.sh my-crate 0.1.2 || status=$?
case "$status" in
    0) echo 'Already present' ;;
    1) echo 'Absent; the caller may consider separately authorized publication' ;;
    2) echo 'Observation unavailable or invalid; stop' >&2; exit 2 ;;
esac
```

The Bash/curl helper accepts one crate name and one canonical stable `X.Y.Z`
version. Prerelease/build versions, leading-zero components, malformed names and
extra arguments are rejected. Its exit contract is **0 present (HTTP 200),
1 absent (HTTP 404), 2 unavailable or invalid**. Transport failure wins over any
HTTP-looking output; rate limits, authorization failures, server failures and
unexpected responses are unavailable. Diagnostics go to stderr; stdout is empty.
Call it in a conditional or capture its status as above when using `set -e`.
Never use a blanket nonzero test to decide that publication is safe.

One request targets the fixed crates.io API, with HTTPS-only redirects, a
10-second connection timeout and 30-second total timeout. It disables implicit
curl configuration, supplies no credentials, and neither polls nor retries.
It does not run Cargo, inspect dependencies, publish or clean artifacts.
A present version does not prove that a new consumer can resolve every dependency.
Post-publication waiting, time budgets and package ordering stay in the caller;
an inconclusive result must stop that flow rather than trigger publication.

## Snapshot adoption

After these files have a reviewed committed revision, adopt each desired helper
and this guide through the [snapshot workflow](consuming-snapshots.md). The
documentation, release-command and registry helpers have no dependencies on
other shared scripts. Their upstream fixtures
are `test-documentation-links.pl`, `test-release-commands.sh` and
`test-crates-io-version.sh`; the release fixture additionally uses the upstream
Makefile and is not a consumer's product test.

Replace only the corresponding local mechanical blocks. IcyDB retains its
persisted-inventory and version-example checks, release receipts/cache checks
and publication waiting. Canic and IC Timers can replace their duplicated
release entry-point smoke tests; IC Metrics keeps its publication checks.
Registry callers must explicitly handle all three statuses before deleting
their local observation implementation. Run each consumer's focused checks and
native host qualification, and distinguish that evidence from upstream fixtures.

## Portable file digests

```sh
bash scripts/ci/verify-file-checksum.sh --print sha256 artifact.wasm
bash scripts/ci/verify-file-checksum.sh sha256 EXPECTED_HEX artifact.wasm
```

`--print` emits only one lowercase SHA-256 or SHA-512 digest on successful stdout.
The existing three-argument comparison form retains its interface and silent
success. Both forms require a regular file (including a symlink resolving to a
regular file). Relative paths resolve from the caller's working directory.

The verifier owns backend selection: native `sha256sum`/`sha512sum` when available,
otherwise Perl's `shasum`. It hashes the file through stdin, so spaces, leading
dashes, backslashes and newlines in filenames do not become options or escaped
checksum records. File-read errors, backend failures and malformed output fail
without emitting a digest; a failing backend is never retried through another.
No network access or build is involved.

Shared Tooling uses this same owner for IC installer receipts and snapshot
manifest generation. Their existing manifest formats are unchanged. Digest
support for arbitrary filenames does not widen those formats: IC receipts still
use unescaped line records and reject backslashes/newlines. Failed receipt
traversal or hashing preserves the candidate and previous active toolset.

Consumers already vendoring `verify-file-checksum.sh` need no additional shared
file for digest generation. After refreshing its committed snapshot, replace
only the file-hash bodies in IcyDB/Canic Wasm reports with `--print`; keep report
schemas, subject selection and stream/tree identity encodings local.

## Isolated RustSec database preparation

```sh
# The parent directory exists; rustsec-attempt must be a new path.
bash scripts/ci/prepare-rustsec-db.sh online \
  https://github.com/RustSec/advisory-db.git "$work_dir/rustsec-attempt" || exit $?
cargo audit --no-fetch --db "$work_dir/rustsec-attempt/db"
```

Select exactly `online SOURCE DESTINATION` or `local SOURCE DESTINATION`.
Online mode requires an explicit HTTPS repository URL without embedded credentials,
query parameters or fragments; it creates a fresh shallow clone with Git's
low-speed connection protection and terminal prompts disabled. It resolves the
moving feed to the commit actually cloned. This records the observation, not a
pin for future refreshes. Git configuration and available credentials remain
host-owned; the helper does not install credentials or alter Git configuration.

Local mode takes an existing Git repository directory and selects its committed
HEAD before cloning. It never copies working-tree edits or untracked cache files.
It clones without hardlinks, dissociates borrowed objects, and checks out that
exact commit, retaining failure if it cannot obtain it. This allows the prepared
database to outlive the selected cache. Only the file transport is allowed in
local mode; there is no network fallback. Neither mode recurses into submodules.
Inherited repository-specific Git environment variables cannot redirect these
operations into another checkout.

The helper exclusively creates DESTINATION, rejecting occupied paths and symlinks.
Its parent must already exist. Within it, `db/` is the isolated checkout,
`prepare.log` records source/mode and Git diagnostics, and `revision` records the
verified full commit on success. Successful stdout contains that same commit.
Preparation failures retain the destination
and log, report its location, and omit the success revision. Invalid arguments or
an unavailable local directory fail before claiming a destination.

The caller owns this directory and its evidence lifetime on both success and
failure. Preserve failed attempts; choose a new destination when retrying. Do
not attach an unconditional cleanup trap that deletes the only failed evidence.
Retain `revision` beside the resulting audit evidence. Preparation success proves
isolation and source identity, not database schema validity, advisory freshness
or a passing security audit; Cargo audit remains the database reader.

Consumers must stop on preparation failure and invoke their audit with `--no-fetch`
against the prepared `db/`. Canic retains dependency fetching, lockfile selection
and its risk inventory; IC Query retains warning/exemption policy and subsequent
cargo-machete ordering. Neither adapter should let cargo-audit fetch again and
change the recorded database identity. The helper never invokes Cargo, selects
exemptions or changes a shared advisory cache.

Adopt `scripts/ci/prepare-rustsec-db.sh` and this guide from a reviewed committed
snapshot before replacing consumer preparation bodies. It requires Bash 3.2+
and Git, with no shared-script dependencies. `test-rustsec-db.sh` covers simulated
online/failure cases and real local isolation using existing Git history; it
neither creates commits nor performs a live vulnerability audit. Consumer tests
must retain policy/order checks and prove audit cannot run after preparation fails.
