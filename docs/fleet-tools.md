# Fleet reports and CI inspection

Shared Tooling owns these central reports. They inspect selected repositories;
they do not run sibling setup, build or release targets. Consumers retain their
own validation and issue resolution. See the [ownership boundary](../README.md#repository-fleet-ownership)
and [supported tool dependencies](supported-hosts.md#tool-specific-dependencies).

Prepare tools explicitly using [local setup](local-setup.md). Commands below run
from Shared Tooling unless another working directory is specified. Prepared
host tools in this checkout's `.tools/host/bin` take precedence over `PATH`.

## Cargo workspace LOC report

For one row per sibling root workspace:

```bash
make cloc
make cloc CLOC_PARENT=/path/to/projects
make cloc CLOC_ROOT=/path/to/repository CLOC_SUBCRATES=1
```

For a single checkout or an independent workspace:

```bash
bash scripts/dev/cloc.sh /path/to/repository
bash scripts/dev/cloc.sh --subcrates /path/to/projects
bash scripts/dev/cloc.sh --manifest testing/Cargo.toml /path/to/repository
bash scripts/dev/cloc.sh --json /path/to/repository
bash scripts/dev/cloc.sh --check-tools
```

The workspace report discovers members through locked, offline Cargo metadata.
It shows Library/app, Tests, Examples, Benchmarks, Build/tooling, Unclassified
and Total columns. Nested members, `target/`, Git internals and Cargo's configured
output directory are excluded from their containing package's counts. Each
admitted Rust source path counts once, including copies with identical bytes.

The unified [cloc.sh](../scripts/dev/cloc.sh) reports workspace summaries by
default; `--subcrates` adds individual crate rows without changing totals.
The target defaults to the current directory. A parent directory selects its
immediate repositories; a checkout selects its root workspace. `--json` selects
one workspace and always includes its crate rows.

The parent-directory report visits
immediate Git checkouts in name order, including linked worktrees and hidden
directories, skipping symlink aliases and non-repositories.
Each workspace checks its prerequisites. A repository without a root manifest shows `N/A`; a failed
count shows `ERROR`, retains other rows and makes the command fail. Totals sum
successful workspaces; failures
label the sum `TOTAL (partial)`. Independent excluded workspaces are not silently
combined with root counts. Select them explicitly with `--manifest` or the
consumer Make interface's `CLOC_MANIFEST` input.

### Source classification

Cargo target kinds identify library/executable, test, example, benchmark and
build-script roots. Static Rust token inspection follows ordinary module files
and separates test-only items, including inline `#[cfg(test)]` modules and test
functions. Nested comments and quoted/raw strings are opaque. `cfg(all(test,
...))` is test-only; `cfg(any(test, ...))` is not assumed test-only. This is a
source inventory, not compilation, macro expansion or a shipped-Wasm measurement.
Predicates disabled in both test and non-test modes are not evidence of tests.
Inner attributes and raw module identifiers participate in module discovery.

Supporting `tests/`, `tests.rs` and `*_tests.rs` paths identify test code.
Legacy `test/` and `test.rs` are counted as tests with hard-cut rename diagnostics.
Examples, benches and tooling paths supply corresponding categories when target
and module usage is unavailable. Actual usage takes precedence over a misleading
filename, with a diagnostic for conflicting conventions. Source not
reachable from a target and without a declared or conventional role remains
Unclassified; it is never silently counted as library code. No production
visibility change or broad source relocation is needed merely to count lines.

Declare exceptional roles in the owning package manifest, for example:

```toml
[package.metadata.shared-tooling.loc]
category = "tests" # Whole fixture-only package, including its helper source.
generated = ["src/generated"] # Existing file or directory, independent of role.

[package.metadata.shared-tooling.loc.paths]
"build.rs" = "build_tooling"
"examples" = "examples"
```

Accepted categories are `library_app`, `tests`, `examples`, `benchmarks`,
`build_tooling` and `unclassified`. Rules name package-relative files or
directories, not globs, and must match Rust source. Longest path rule wins over
the package role; declarations override path/target inference. Inline test-only
items still count as tests. Generated markers (`@generated` or `DO NOT EDIT`
near the file start) and declared generated paths set a separate flag; generated
source under Cargo output remains excluded.

Shared target source is counted once and flagged with all observed target
categories. Without an override or conventional path, the accounting owner is
the first applicable category in the table's order. That choice does not claim
exclusive use. A physical line containing both test and enclosing code is counted
once in the enclosing category and flagged. Inline partitions must reconcile with
cloc's original file total; otherwise the file remains Unclassified with a
diagnostic. Macro bodies are counted where written; unresolved module/include
paths and conditional attributes require review. Rust in documentation comments
is excluded by cloc, including doctests. Non-Rust assets are outside this report.
Malformed tokens in compile-fail fixtures retain their declared/path category
and a diagnostic; without that role they remain Unclassified. Input/read/tool
failures still fail the report rather than silently skipping files.

The inventory includes Cargo targets outside their member directory and their
resolved modules. Canonical paths are counted once; a source shared by packages
is attributed to the first admitting package in name order. Unreferenced source
outside member directories is not an automatic repository-wide Rust inventory.
Counts use copies of the exact bytes inspected, with `source_sha256` in each
file record. This prevents edits during cloc from changing classified counts;
it is not an atomic snapshot of a concurrently edited workspace. A nonempty
file omitted by cloc fails the report instead of silently contributing zero.
Module traversal queues file visits and releases each token tree before visiting
the next file. Inline partitioning reloads captured bytes one file at a time;
peak parser memory follows file size rather than the whole workspace's tokens.

JSON schema **2** contains `manifest`, `crates`, `totals`, `files` and
`diagnostics`. Crate rows have `crate` and `counts`; each count object has the six
category keys above plus `total`. File records retain category, counts, target
categories, `source_sha256`, `shared` and `generated` flags. Category counts sum exactly to the
all-source total. Schema 1's `runtime_loc`, percentages and test-function counts
are removed; update consumers rather than retaining aliases. Failure emits no
partial workspace JSON. Text uses `???` for unclassified LOC; diagnostic notes
remain in JSON. Use `cloc.sh --help` for commands and diagnostic guidance.

## Tooling adoption audit

Ask an agent to run [snapshot-drift](../tasks/snapshot-drift.md) for the selected
repositories, then [tooling-duplication](../tasks/tooling-duplication.md) for
local mechanisms and missed shared capabilities. These tasks combine deterministic
snapshot verification with caller, CI and release-policy review. LOC and version
numbers alone cannot establish adoption quality. Run on request; this does not
enable background monitoring or add deployment gates.

## Sibling GitHub dashboard

```bash
bash scripts/dev/github-siblings.sh
bash scripts/dev/github-siblings.sh --interval 120
bash scripts/dev/github-siblings.sh --once /path/to/projects
```

The dashboard inventories immediate Git checkouts with `AGENTS.md` and a GitHub
`origin`. It skips symlink aliases, deduplicates repositories and defaults to
the script checkout's parent. Restart it to discover new siblings. It needs an
already authenticated GitHub CLI (`gh auth login`), Git, jq, Perl core and IANA
timezone data.

Both **ISSUES** and **PRS** have **OVERALL** (`OPEN`, `CLOSED`) and **TODAY**
(`ADDED`, `CLOSED`) groups. Closed PRs include merges; open PRs include drafts.
Closed issues include duplicates and issues closed as not planned. These are
GitHub state counts, not proof of substantive fixes.

TODAY starts at **06:00 Europe/Paris**, including daylight-saving changes; before
06:00 it uses the previous day's cutoff. Added counts include creations in any
current state. Closed counts include currently closed items whose latest
closure or PR merge falls within that window. Reopened issues do not count as
currently closed. Complete totals use batched GraphQL requests; recent activity
uses pagination. Failed observations show `ERROR`, never zero, and totals become
`TOTAL (partial)`.

In a terminal the dashboard refreshes every 60 seconds; `q` quits and `r`
refreshes early. `--once` and redirected input/output produce one report and
fail on partial observations. Watch mode retries on its next cycle. Rows sort
by open issues descending, then open PRs descending, then repository name, with
errors last. Repositories with no open issues and no open PRs are grey; other
rows use the colour scale relative to the largest open-issue backlog. A repository
with only open PRs remains active. `NO_COLOR`, `TERM=dumb` and redirected output
disable colour. Repository labels include the organization only when the table
fits the known terminal width.

## GitHub Actions inspection

```bash
bash scripts/dev/gh-ci.sh --commit HEAD --all-workflows --limit 100
bash scripts/dev/gh-ci.sh --run 123456 --logs
bash scripts/dev/gh-ci.sh --failed --logs
```

The default workflow is `CI`. `--commit` resolves a local revision to a full
commit identity without an implicit branch filter; add `--branch` to restrict
it. `--all-workflows` removes the workflow filter. Lists are bounded by `--limit`,
not a complete CI verdict; inspect the relevant run explicitly. `--failed`
searches historical failures that may have been superseded by later successes.
`GH_REPO` selects another GitHub repository, but commit revisions still resolve
in the caller's local checkout. Detached HEAD needs an explicit selector.

Run IDs must be positive integers. `--run` excludes list/search filters;
`--list --logs` is rejected. `--logs` pins the repository, run and attempt before
fetching evidence. Empty failed-step logs for a completed failed/timed-out
attempt trigger a download of only that attempt's failed/timed-out job logs.
The original observation and recovered logs remain in the announced temporary
directory. API failures and empty logs for cancelled/unfinished attempts remain
failures; completed successful, neutral or skipped runs may have no failed logs.
The command never reruns CI or converts logs into a passing qualification.

For agent-driven inspection and separately authorized fixes, use the
[maintenance rules](../rules/agent-maintenance.md). Repeatable fleet tasks and
their optional schedule live in the [task catalog](../tasks/README.md).

## Stack dependency overview

The [README diagram](../README.md#toko-stack) and [manifest evidence](stack.md)
have a separate deterministic daily local refresher. It reads existing checkouts
and leaves generated documentation changes for the next commit; it does not
launch an agent, stage files or fetch upstream changes.
