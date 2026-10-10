# Supported hosts

IC Query requires Linux and macOS support for native builds, tests, dependency
setup, and maintainer release tooling. The declared host matrix is:

| Host | Architecture | CI label | Native workflow |
| --- | --- | --- | --- |
| Ubuntu 24.04 | x86-64 | `ubuntu-24.04` | Complete CI, MSRV, release/publication fixtures, canister integration |
| macOS 15 Sequoia | Apple Silicon (ARM64) | `macos-15` | Complete CI, MSRV, release/publication fixtures, system Bash fixtures, canister integration |
| macOS 15 Sequoia | Intel (x86-64) | `macos-15-intel` | Same native checks as Apple Silicon |

The macOS labels and architectures follow
[GitHub's hosted-runner reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners).
These rows declare required support and configured coverage. Qualification
requires matching native CI results for the actual revision; until those runs
pass, macOS coverage remains an explicit gap. Other host versions are not yet
qualified by this matrix.

## Prerequisites

- Rustup with Rust `1.99.0`, rustfmt, Clippy, and `wasm32-unknown-unknown`, plus
  the declared MSRV `1.91.0` for its separate check. Installation must preserve
  the repository's selected `Cargo.lock`.
- Bash 3.2 or newer, GNU Make 3.81 or newer, Git, Perl with its core modules,
  Python 3 for the maintained documentation tools, and standard Unix
  utilities. macOS needs Xcode Command Line Tools for its compiler, Git and Make.
  Release/checksum helpers support both GNU `sha256sum` and macOS `shasum`.
- `make install-dev` explicitly installs the exact Cargo Audit and Cargo Machete
  versions declared in `Makefile`, cargo-sort 2.1.4 from `ci/tool-versions.env`,
  plus common jq 1.8.2, Mike Farah yq 4.47.2 and
  ripgrep 15.2.0 with PCRE2 and cloc 2.10 under `.tools/host/bin`. Host versions
  and payload digests have one owner in the immutable `ci/tool-versions.env` snapshot.
  Cargo-sort uses the shared selected-package installer and a versioned local
  directory; Cargo Audit and Cargo Machete retain their global Cargo destinations.
  `make install-format-tools` prepares only that selected formatter. Offline
  `make format-tools-check` admits its receipt and bytes before version probes;
  global or optional-bundle copies cannot substitute for a missing selection.
  `make host-tools-check` authenticates all payloads before version and PCRE2 checks;
  the complete CI/release gate includes that offline check and
  `make dependency-pins-check`. Installation is separate from ordinary validation.
- `make install-tools` prepares host tools followed by the complete IC set:
  Quill 0.5.4, ICP CLI 1.6.0, didc 0.6.2, ic-wasm 0.11.1 and
  wasm-opt 132 under `.tools/ic/bin`. IC versions and archive digests have one
  owner in `ci/ic-tools.tsv`; `make tools-check` verifies both sets offline.
  Make selects the local paths; interactive shells use
  `export PATH="$PWD/.tools/host/bin:$PWD/.tools/ic/bin:$PATH"`.
  See [local setup](local-setup.md) and [IC tools](ic-tools.md) for bootstrap
  packages, complete-set activation, locks and retained failed/previous sets.
  Shared 0.2.0 requires explicit setup to replace an active six-tool bundle;
  offline validation refuses it without conversion. Query uses ICP-managed
  networks and has no PocketIC runtime or Testkit setup requirement.
  curl, tar with xz/gzip support and a SHA-256 backend are required. dfx is
  excluded. Linux ARM64 has a host-tool mapping but no complete IC set and remains
  outside the qualified consumer matrix.
- Explicitly prepare the selected dependency cache before a gate. Hosted CI
  uses `cargo fetch --locked`, then runs the gate with `CARGO_NET_OFFLINE=true`.
  Standard local release entry points prepare the cache with `cargo fetch --locked`,
  preserving explicit Cargo offline settings and the selected versions. Standalone
  release adapters use `cargo fetch --locked --offline`.
  Local build, check, Clippy, test, MSRV, rustdoc and package validation also
  select locked/offline access explicitly. They report missing inputs even if
  the ambient Cargo setting permits downloads. Cargo Machete's metadata scan
  is offline; the separate RustSec database refresh remains a live Git fetch.
- Publication additionally requires separately authorized crates.io credentials.
  Release pushes require an explicitly selected Git remote and branch; neither
  dependency setup nor fixture testing supplies publication authority.

The [Governance canister smoke harness](canister-smoke.md) has additional ICP CLI
and local-network prerequisites. Its separate live integration job is configured
on all three hosts, explicitly preparing and checking the common IC set.
ICP CLI remains 1.6.0 with the same reviewed architecture-specific digests. Each
downloaded archive is verified before extraction and the executable version is checked before PATH admission. Native macOS canister-network
qualification requires passing smoke runs on this configuration; Rust
cross-compilation and the general macOS gate do not qualify it.

Tool installation explicitly selects `/bin/bash` and puts `/bin` first on PATH.
The macOS workflow also puts `/bin/bash` first on PATH before the complete gate,
so its shell helpers and their child fixtures use Apple's system Bash in one
gate execution. CI resolves the temporary directory to a physical path
before fixtures run, so macOS `/var` aliases do not violate managed-cache
confinement. Production symlink rejection is unchanged.
The general gate and MSRV checks run natively on each host.
Build and validation evidence remain consumer-owned; release flows do not clean
them automatically.

The prior Shared Tooling 0.1.6 snapshot passed its native provisioning CI on
[Linux and both macOS hosts](https://github.com/dragginzgame/shared-tooling/actions/runs/37450707625).
The initial 0.1.7 upstream CI [failed](https://github.com/dragginzgame/shared-tooling/actions/runs/37458968809).
The prior committed revision `9f8c7c768793f4ce8f25be9e88282c0f63a06e7f`
includes the [Bash 3.2 failure-handling fixes](https://github.com/dragginzgame/shared-tooling/issues/14)
for IC installer and release-tag guards. Its [CI run](https://github.com/dragginzgame/shared-tooling/actions/runs/37479591040)
passed Linux provisioning and lint/security jobs; native macOS qualification
failed on both architectures at the host-tool regression fixture stage, after
the IC installer fixtures passed. The logs do not identify the failing subcase.
Linux PAX-archive substitution reproduces a fixture failure when the corruption
test rebuilds a pinned archive: access-time metadata changes its digest even
though the extracted executable is identical. Restoring the original archive
bytes passes with Bash 5 and 3.2. This isolates a fixture defect for the
[upstream host-tool owner](https://github.com/dragginzgame/shared-tooling/issues/17);
it does not identify the exact native macOS failing command.
That revision did not qualify native macOS support.
The prior 0.1.8 revision `d957d1f8801885c5b69e4a9ef900155f5f2a8a9d`
retains those fixes and adds Cargo inheritance checks. Its
[native CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37484175750)
passed Linux and lint/security; both macOS jobs failed at the same host-tool
fixture stage. The prior IC Host Tools 0.2.0's
[CI](https://github.com/dragginzgame/ic-host-tools/actions/runs/37483358223)
passed Linux and MSRV; both macOS jobs failed at the host-tool installer fixture
stage. That dependency revision did not qualify native macOS support.
The prior Shared Tooling 0.1.9 revision
`b32d3038c850a7c53470c326b0f7f11263b31669` restores authenticated archive bytes
in its host-tool fixture and retains installation diagnostics. Its
[native CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37489483879)
passed Linux, both macOS architectures and lint/security, including formatter
admission, portable regression fixtures and pinned native IC executables.
The corrected host fixture also passes Linux PAX substitutions under Bash 5 and
3.2; those substitutions remain focused evidence rather than native qualification.
The prior 0.1.10 revision `21f3ec3dd97f2968c9f0b08924451bb2f71770d1`
retains the same host installers and formatter checker while adding upstream
fixture retention and unrelated compiler-cache/tag-maintenance support. Its
[native CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37491682760)
passed Linux and lint/security; both macOS jobs passed host/IC installer,
version-reader, formatter and hook fixtures, then failed the new retention test.
Retained diagnostics from both hosts show BSD `sed` joined its substitute launcher's shebang
to the next line, yielding an invalid `/bin/bashcase` interpreter. That test is
outside the selected snapshot, whose runtime helper bytes remain unchanged from
0.1.9. Complete native qualification of 0.1.10 remains outstanding under the
[upstream fixture-retention owner](https://github.com/dragginzgame/shared-tooling/issues/21).
The prior 0.1.11 revision `46c02774a8335cb3949d6f04284c4f53375353c1`
generates that launcher directly with `printf` and adds exact changelog comparisons,
corrected validation error labels and the shared repair/reporting workflow.
Focused retention, logger and changelog fixtures pass under Linux Bash 5 and 3.2.
Its [native CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37500153922)
passed Linux, both macOS architectures and lint/security.
The prior 0.1.13 revision `e378671d90afa237ff63a4b0e3b9551eb2c222b6`
includes captured release destinations, independent snapshot checksums,
exact-commit CI inspection and the standard Rust workspace guide. Its
[native CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37581058940)
passed Linux, both macOS architectures and lint/security.
The prior 0.1.14 revision `25e7ce83149e081e4dcc52c55c33724e44153f2a`
adds isolated Make execution admission and selected-target LOC exclusion. Its
[native CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37586649650)
passed Linux, both macOS architectures and lint/security.
The prior 0.1.15 revision `bfb50bd0884b5e6c5ee9592056531c6108f96d73`
adds the common Make include, pinned local cloc and sibling tooling inventory.
Its [native CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37593142226)
passed Linux, both macOS architectures and lint/security.
The prior 0.1.16 revision `b69507367d45e3db9543359e689e1fcba0467ff4`
adds retained validation timings, selected-manifest LOC and corrected snapshot
classification. Its [CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37598153506)
passed Linux and lint/security; both macOS jobs were cancelled, so exact-revision
macOS qualification remains incomplete upstream. The released consumer 0.47.9
adoption passed all three native [release-check jobs](https://github.com/dragginzgame/ic-query/actions/runs/37603344473).
Its [branch run](https://github.com/dragginzgame/ic-query/actions/runs/37603344957)
passed MSRV and canister smoke on all three hosts; the ARM checks job was
cancelled, so that branch run is not a completed green gate.
The prior 0.1.18 revision `a3430b34b32a60f3b245a2b4f7e2f5321556fe56`
adds validation-goal admission, physical-target LOC exclusion and an optional
local Rust-tool installer. Its [CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37604299590)
passed Linux, both macOS architectures and lint/security. Released consumer
0.47.10's [matching branch CI](https://github.com/dragginzgame/ic-query/actions/runs/37619655814)
has completed successfully with all nine check, MSRV and canister jobs passing
on the three supported hosts. Its [tag run](https://github.com/dragginzgame/ic-query/actions/runs/37619656675)
has completed successfully with all three native check jobs passing; MSRV and
canister jobs are skipped by the configured tag workflow.
The released 0.47.11 selection of 0.1.19, `a06e4719e3839b8eefcfb88ec8923aa88eb63ccc`,
adds Rust-tool path admission, release-note EOF preservation, combined failure
logs and portable temporary-root fixture handling. No matching upstream CI run
was available at adoption review. Its released consumer adoption now has matching
native qualification: the
[branch run](https://github.com/dragginzgame/ic-query/actions/runs/37636399405)
passed all nine checks, MSRV and canister jobs across Linux and both macOS hosts.
The [tag run](https://github.com/dragginzgame/ic-query/actions/runs/37636399265)
passed all three native checks and skips MSRV/canister under the configured policy.
The released 0.48.0 snapshot selects Shared Tooling 0.1.20,
`3ecc48e579f6cf6e6ab01a6645d8a250fc8c6934`. Its
[exact-source CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37641211708)
passed Linux, macOS Intel/ARM and lint/security. The selected runtime helpers
retain their 0.1.19 bytes; this refresh adds contribution-policy and linked
governance guidance. The existing engineering baseline remains separately pinned.
IC Query's adoption changes require their own matching native CI; upstream
qualification does not establish consumer execution or deployment compatibility.
The released 0.49.0 snapshot selects Shared Tooling 0.1.23,
`0ba0ad00ed94848e54ecc82629b6b7873b7284c0`. Its
[exact-source CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37746567888)
has passed Linux, Intel/Apple Silicon macOS and lint/security.
The prior 0.1.21 Intel job exceeded its 10-minute execution limit. The 0.1.22
owner workflow allows 25 minutes including setup and evidence upload, with a
separate 15-minute regression limit. It also normalizes temporary paths in the
owner LOC fixtures. The matching owner run qualifies those corrections; Query
retains its existing CI matrix and complete gate.
Query's stub-based adapter and real-index metadata checks pass locally on Linux.
The exact committed owner's LOC context fixture also passes locally with
enclosing configuration and physical/aliased trailing-slash temporary roots.
The canonical runner fixture selects direct delivery and keeps release effects
stubbed; the separate real-Git PR fixture remains upstream. The released consumer
CI results and outstanding macOS failure are recorded below; direct release
delivery remains selected. Pending 0.49.1 retains this snapshot while the
concurrent symbolic tracking-ref case in
[Shared #62](https://github.com/dragginzgame/shared-tooling/issues/62) remains open.
The prior IC Host Tooling 0.3.1 registry crates were reviewed at
`38a2a5127be064014e6d39d72d0300ffb2cf20be`. That exact published source passed
[Linux, both macOS architectures and MSRV CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37580017649).
IC Query 0.47.7's [native CI](https://github.com/dragginzgame/ic-query/actions/runs/37588946717)
failed on both macOS hosts at the consumer's Make-mode fixture: GNU Make 3.81 placed
`-i` after the first flag word, so the consumer parse guard admitted execution.
The released 0.47.8 correction reads all short invocation flags from `MFLAGS`;
Linux checks with GNU Make 3.81 and 4.3 do not replace native qualification.
The prior IC Host Tooling 0.3.2 registry crates were reviewed at
`c7c0d85765054909c05d86f6d3fd2c9965510335`, including published package provenance.
Its [native CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37589678525)
passed Linux and MSRV, but both macOS jobs failed the resolver's directory-traversal
test: `file/..` returned a resolved path where the test expected `NotADirectory`.
The prior IC Host Tooling 0.3.3 registry crates record published VCS revision
`3d18ca9a9ed0ac5935a16c5bac99694d8e9a7d0a`; their Rust sources match that commit.
Its [native CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37595113180)
passed Linux, both macOS architectures and MSRV. The corrected traversal fixture
compares each host's native canonicalization result, including missing-prefix
rewind, while production resolution is unchanged. This resolves the upstream
qualification failure reported in
[Host #1](https://github.com/dragginzgame/ic-host-tooling/issues/1).
The released 0.47.8 resolver, archive hashing and response-only adoption passed
all nine [consumer branch-CI jobs](https://github.com/dragginzgame/ic-query/actions/runs/37597823342),
including complete checks, MSRV and canister smoke on all three native hosts.
The prior IC Host Tooling 0.4.0 registry crates record published VCS revision
`6b171744def811882ba6c71d50135efa898302a9`; their packaged Rust sources match
that committed owner. Its [native CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37602699181)
passed Linux, macOS Intel/ARM and MSRV. Query uses retained APIs and does not
re-export the host crates.
The prior 0.4.1 registry crates record published VCS revision
`ce2dd57cedc5000b44bb6a9ff5194f7d65a42c38`; their packaged Rust sources match
the committed owner. Its [CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37614534197)
passed Linux and MSRV, but both macOS jobs stopped in the shared tool-command
fixture before Rust checks. The TMPDIR spelling defect and shared repair
are tracked in [Host #6](https://github.com/dragginzgame/ic-host-tooling/issues/6)
and [Shared Tooling #56](https://github.com/dragginzgame/shared-tooling/issues/56).
The released consumer 0.47.10 selects these packages; upstream 0.4.0 coverage
does not qualify 0.4.1. Consumer native CI remains required separately.
The released 0.47.11 selection of 0.4.2 registry crates records published VCS revision
`6501d0e9fa7ba0439ec7a4010ca7bf0205e1d712`; their packaged Rust sources and
original manifests match the committed owner. Its
[matching CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37624014360)
passed Linux and MSRV; both macOS jobs failed at `tooling-command-check` before
Rust qualification. The old fixture compares a physically normalized command
path with an expected `/var/folders/.../T//` spelling. Host's released source
still selects the older shared fixture; adoption of the committed 0.1.19 repair
is tracked in [Host #6](https://github.com/dragginzgame/ic-host-tooling/issues/6).
Native owner qualification and the consumer's adoption qualification remain
separate requirements.

The released 0.48.0 selection uses published Host 0.4.3 at
`644d49c096ae05c2e17e1b6aacf14770988c5cf6`; package sources and original manifests
match that commit. Its [exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37639415895)
passes Linux and MSRV, but both native macOS jobs fail compilation: the durable
writer passes `u32` permissions to Darwin's `u16` raw-mode API. Query's exact-source
[branch CI](https://github.com/dragginzgame/ic-query/actions/runs/37646652323) and
[tag CI](https://github.com/dragginzgame/ic-query/actions/runs/37646652181) pass their
Linux jobs and fail native macOS jobs at the same dependency compiler expression.
The tag workflow skips MSRV/canister by configuration; the branch failures cover
those native workflows too.

The released 0.48.1 selection uses published Host 0.4.6 at
`0fb05f9e18f032425188d68e1d69317a0f0127d5`; package provenance, Rust sources and
original manifests match that commit. Host 0.4.5 repairs the Darwin conversion
without truncating unvalidated permissions, closing
[Host #18](https://github.com/dragginzgame/ic-host-tooling/issues/18). Host 0.4.6
also corrects the non-UTF-8 filename fixture to follow independently observed
native filesystem admission, preserving typed failure and cleanup evidence;
production publication behavior remains unchanged.
[Exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37648086908)
passes Linux, both macOS architectures and MSRV. Released 0.48.1 requires
filesystem version 0.4.5 or newer for downstream builds and selects 0.4.6 in its
lockfile; its matching consumer native qualification is recorded below.

The released 0.49.0 selection uses published Host 0.5.1 at
`81f9809861159def2fd0987fcb7961cda4afd969`. All four owner crates are published
and non-yanked; Query's three selected lockfile checksums match the official
registry. Committed Host crate files are unchanged from 0.5.0; 0.5.1 adopts the
owner's release-tooling corrections. Its
[exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37750135927)
passes Linux, Intel/ARM macOS and Rust 1.88. The earlier 113 focused Linux tests
used Host 0.5.0 and do not qualify this lockfile selection. No process, gzip
or IC limit-report dependency profile was added.
Query's [0.49.0 tag CI](https://github.com/dragginzgame/ic-query/actions/runs/37757330449)
at `59bf55ac5725ce4d336e4955c74a89b5f1e4793c` passes Linux checks but fails
both native macOS checks in the escaped-descendant Python fixture: refused
process-group cleanup replaces its timeout. This is tracked in
[#23](https://github.com/dragginzgame/ic-query/issues/23), not a green consumer gate.

The 0.49.0 typed-attempt and private CLI cleanup passed 95 focused
library tests and 12 CLI leaf tests on Linux with Host 0.5.1, locked/offline
after explicit cache preparation. Strict Clippy for both affected packages and
the library's no-default-features check also pass. These local results cover the
changed ownership boundaries; they are not native macOS or full consumer CI proof.

The released 0.49.1 selection uses published Host 0.5.2 at
`c7014995bf0890c1df9cd9b9a6ec14ea70f98c6f`, with matching registry checksums
for the three existing dependencies. Their implementation files are unchanged
from 0.5.1; the feature profiles remain unchanged. The owner's
[exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37762087718)
passes Linux, Intel/ARM macOS and MSRV. This does not qualify Query's dirty graph.

The 0.49.1 smoke repair retains original command failures and separately records
group-signal, reaping and pipe cleanup errors. Escalation reserves the leader PID
until group signals finish, and reaping is bounded. No process dependency was
adopted: Host 0.5.2 supplies an explicit successful handoff, while full consumer
IO, deadline, interruption and receipt integration remains under
[Host #5](https://github.com/dragginzgame/ic-host-tooling/issues/5#issuecomment-6057100625).
Focused Linux validation passes 38 Python receipt/process tests, 55 library
cache tests with `host`, four Governance artifact-helper tests and strict Clippy
for that helper. Cargo validation uses the explicitly prepared selected lockfile
with `--locked --offline`. This includes background handoff, SIGINT/SIGTERM,
refused cleanup diagnostics and a TERM-ignoring descendant whose leader exits
before KILL escalation.
The focused Rust checks were repeated after the subsequent `zerocopy` 0.8.62
lockfile selection; the Python run precedes that dependency update.
Matching native consumer CI, fresh local MSRV, full local CI and live network
smoke execution were not run during that local preparation. Subsequently,
released Query `a05ec4d2a30569ed738363f60e3fa71678d84cb3` passed all nine
[exact-source branch jobs](https://github.com/dragginzgame/ic-query/actions/runs/37767619642)
(checks, MSRV and canister smoke on all three native hosts), plus the three
[matching tag checks](https://github.com/dragginzgame/ic-query/actions/runs/37767619965).
That qualifies the released 0.49.1 cleanup repair, separately from dirty 0.50.0.

The released 0.50.0 smoke hard cut uses `ic-agent` for certified state reads and
typed Candid update calls in the development helper. Host artifact and filesystem
dependencies select published 0.7.1 in the updated workspace; `ic-host-tools` is
removed. The helper explicitly limits response bodies, preserves binary replies
before admission, uses the
built-in mainnet root key, and fetches local root keys only over loopback HTTP.
ICP retains managed-runtime/deployment discovery and lifecycle ownership.
Focused Linux validation against the earlier 0.7.0 graph passes 55 cache-file
tests, six Rust helper tests, strict helper Clippy and all 38 Python receipt/process
tests after the receipt simplification. Cargo used the explicitly prepared
locked/offline cache. Host's retained artifact/filesystem implementations are
unchanged from 0.6.0, and its
[exact-source 0.7.0 CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37773664766)
now passes Linux, both macOS architectures and MSRV. This owner result does not
qualify Query's dirty graph or imply consumer process adoption.
The 0.7.1 Rust implementations are byte-identical to 0.7.0. Its
[exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37776708008)
passes Linux and MSRV; both macOS jobs remain queued at inspection. The selected
0.7.1 lockfile cache was explicitly prepared offline. Focused checks of that
selection pass 55 cache-file tests, six Rust helper tests, 15 Python receipt
tests and strict helper Clippy. A disposable path-Query consumer with ordinary
registry Host 0.7.1 dependencies compiles with exactly one artifact identity
and one filesystem identity. This is not published Query consumer evidence; consumers pinned to
Host 0.6 require a coordinated upgrade, tracked in
[Query #24](https://github.com/dragginzgame/ic-query/issues/24).
Query 0.50.0 retains its existing
TERM grace and bounded reaping by maintainer decision; Host's 0.7.1 process
owner could not preserve those requirements.
Full local CI, fresh MSRV, live canister smoke and macOS execution
have not been run for this dirty selection. Earlier consumer and owner runs do
not qualify the new helper protocol path. CI retains the binary reply files
alongside their receipts on both successful and failed smoke runs.

Released Query 0.50.0 `bd583e2b6baa7570be363b2948345c57582c3f31` passes Linux
checks, MSRV and live canister smoke in its
[exact-source branch run](https://github.com/dragginzgame/ic-query/actions/runs/37789118978).
All nine native checks, MSRV and live smoke jobs pass across Linux and both
macOS architectures. All three native checks pass in
the [matching tag run](https://github.com/dragginzgame/ic-query/actions/runs/37789118881).
Those results cover the release,
not the following working-tree integration.

The released 0.50.1 selection uses published Host 0.8.2 at
`92bd2fecc71124b562e227a32a67644e1e5e34b7`, including the development-only
process crate. The helper now uses Host's command communication and owned-group
cleanup with five seconds each for TERM grace and bounded reaping. Local socket
EOF requests cancellation; Python retains harness lifecycle and receipt policy.
Successful background startup explicitly hands off after output admission.
The selected locked/offline cache was explicitly prepared. Focused Linux checks
pass all 37 Python fixtures through the actual Host runner, 55 cache-file tests,
six artifact-helper tests, strict process-helper Clippy, manifest ordering,
formatting and changed-document link checks. Direct script bootstrap help also
passes. The canister-only normal dependency tree contains no Host, agent, Tokio
or Reqwest dependency. Both development helpers compile with actual Rust 1.91.0
against this selected graph, locked/offline. Full local CI, the complete MSRV
gate, live smoke and native macOS execution have not been run for this working tree.

Host 0.8.0 failed its cleanup-evidence fixture on both macOS architectures
because `/bin/true` was absent; 0.8.1 retained that fixture. Published 0.8.2
implements the portable `/bin/sh -c 'exit 0'` repair reported in
[Host #5](https://github.com/dragginzgame/ic-host-tooling/issues/5), without a
production runtime change. Its
[exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37801871391)
passes Linux, MSRV and both native macOS jobs. The selected graph passes
the focused consumer checks above. Owner qualification and matching native
Query process acceptance remain separate.

Shared Tooling 0.1.26 was reviewed at
`75a8a60f49cec11d3f6aecab5c977029c42cc549`, followed by 0.1.27 at
`b866d41041a1986eeec95bde9af4c6ba0853d2e3`. Those reviews retained 0.1.23:
the then-current shared runner suite unconditionally performed real Git release effects
in scratch repositories, contrary to the local simulation-only fixture rule.
[Shared #70](https://github.com/dragginzgame/shared-tooling/issues/70) requests a
separate admissible fixture selection without weakening the owner's native
tracking/lock tests. The reviewed exporter successfully verified both 59-file
exports, including restoration before a consumer commit; the new runner tests
were not executed. The 0.1.27 bootstrap-path fixture passes on Linux, covering
relative/absolute invocation, inherited CDPATH and newline checkout paths.
Its runner changes only path resolution and retains the incompatible native
tracking block. Shared's exact 0.1.27 owner CI was queued at inspection and does
not establish native qualification or consumer adoption.

Released Query 0.50.1 adopts committed Shared Tooling 0.1.28 at
`1872ed2c20f6c70689bb2249050b1d673c60bfa0`. Canonical export verifies all 73
selected files, including the linked task catalog and schedule templates; no
schedule is activated. The consumer runner fixture delegates only inert Git
hashing, and native tracking/race tests remain in the owner's separate suite.
The engineering baseline remains independently pinned. Focused Linux validation
passes the actual Query simulation/adapter/recovery and metadata fixtures, CI
script fixtures, canonical Rust-tool tests with substitute Cargo, installed
host/IC offline checks, dependency declarations, documentation links and snapshot
integrity. No real release effect or full local CI was executed.
[Exact-source Shared CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37799837183)
passes Linux and Apple Silicon regression and lint/security; Intel regression
was cancelled before execution. Query's matching
[branch](https://github.com/dragginzgame/ic-query/actions/runs/37808359391) and
[tag](https://github.com/dragginzgame/ic-query/actions/runs/37808359543) CI at
`cf26d09e3c0acb861ff7566331531b14e6a17dc3` remain queued at inspection.
Linux MSRV and live smoke pass; Linux checks are in progress and all macOS
jobs remain queued.
[Query #25](https://github.com/dragginzgame/ic-query/issues/25)
retains that acceptance work.

Released Query 0.50.2 selects Shared Tooling 0.1.29 at
`1a54fb625d6e47efa64c4384808ecbc87be84e7e`, including the shared read-only
source-admission helper in its 74-file snapshot and PocketIC 16.1.0 pins.
Initial adapter admission uses no metadata exceptions; saved-evidence recovery
retains its separate contract. The
[exact-source owner CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37806453080)
passes Linux regression and lint/security; both macOS regressions remain queued.
Native consumer qualification is not supplied by these owner results or the
earlier Query release. [Query #26](https://github.com/dragginzgame/ic-query/issues/26)
owns the new admission behavior's acceptance.
Focused Linux checks pass the consumer simulation/adapter/recovery fixtures,
actual Git source/index preservation cases, nested metadata/log fixtures, CI
script fixtures, snapshot integrity, ShellCheck, dependency declarations and
documentation links. Actual source checks also pass with Bash 3.2.57 on Linux.
PocketIC 16.1.0 and the complete selected IC set are authenticated and verified
offline after explicit installation; the previous bundle is retained. Full
local CI, real release execution and native macOS qualification were not run.

Released Query 0.50.1's branch macOS jobs were cancelled before execution when
the next release superseded that run. Its matching tag checks pass on Linux and
both macOS architectures. Those complete checks qualify the adopted release
fixtures and artifact/filesystem cases; native MSRV/live smoke remain gaps for
that source. Released
0.50.2 at `7f1866d8b5c6026371e23c303e02e37af3f07d01` passes Linux checks,
MSRV and live smoke in its
[branch run](https://github.com/dragginzgame/ic-query/actions/runs/37817468555),
and Linux checks in its
[tag run](https://github.com/dragginzgame/ic-query/actions/runs/37817469555).
Both runs have now completed successfully on all three required hosts. The
branch run passes all nine checks, MSRV and live-smoke jobs; the tag run passes
all three configured checks jobs.

Released 0.50.3 preserves the selected registry Host 0.8.4 graph at
`97187b2a46d6f8a6964224a36a133d858ef0d223`, with defaults disabled and process
support development-only. Its
[owner CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37818647474)
passes Linux, both macOS architectures and MSRV. The selected cache was
prepared explicitly after offline fetch reported the missing filesystem crate.
The 74-file Shared Tooling snapshot now records 0.1.30 at
`4e274a2219c0b0cc3af68ec65658b373253518fb`; only setup guidance changes within
that selection. The owner-only Cargo installation assessment and manual workflow
are not selected or executed here. Shared's
[matching CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37809818114)
passes Linux, both macOS architectures and lint/security.
Focused locked/offline Linux validation passes 55 cache-file tests, six artifact
helper tests, 37 smoke-command fixtures through the actual Host runner, and strict
process-helper Clippy. Snapshot integrity, dependency declarations, documentation
links and existing CI script fixtures also pass. Cache preparation and checks
preserve the maintainer's selected lockfile bytes. Full local CI, complete MSRV,
live IC smoke and native macOS execution for this selection were not run.
The CI policy groups pushed commits by ref and source SHA, preserving
their queued/running qualification across later pushes. PR revisions retain
automatic cancellation. Released source
`edd68020d9ef8e76a241e89daf77e51155a92f57` passes all nine native checks,
MSRV and live-smoke jobs in its
[branch run](https://github.com/dragginzgame/ic-query/actions/runs/37823182436),
and all three configured native checks in its
[tag run](https://github.com/dragginzgame/ic-query/actions/runs/37823183374).
The preceding 0.50.2 Intel checks continued across the 0.50.3 push; its Apple
Silicon checks started afterward and passed. Both source runs completed,
qualifying consecutive-push preservation under
[#27](https://github.com/dragginzgame/ic-query/issues/27).

Pending 0.50.4 adopts Shared Tooling 0.1.31 at
`9af82393c620e486578febed74a648523725c234` with the same 74-file selection.
The installer and its canonical pin parser now compare validated complete
records for reuse. The owner IC-installer fixture passes on Linux Bash 5 and
genuine Bash 3.2.57, including substituted Linux, Intel Darwin and ARM Darwin
tool branches, damaged payloads, invalid or changed records, unchanged caller
and installation provenance, and failed-candidate retention. Those substituted
branches do not qualify native macOS execution. The owner fixtures are exercised
from an isolated exact-revision checkout rather than added to consumer CI.
The actual installed Linux six-tool bundle passes both offline checking and
explicit setup with comment-only/reordered pins and curl blocked: zero
downloads, no new bundle, unchanged active link, caller pins, receipts, Git index
and Cargo lockfile. A changed Apple Silicon archive digest refuses offline
reuse. Snapshot integrity, dependency declarations and documentation links pass.
The exact shared
[CI run](https://github.com/dragginzgame/shared-tooling/actions/runs/37891841317)
is in progress when inspected. This pending Query selection has no native CI
result yet; full local CI, Rust compilation and live smoke were not run for this
installer-only change.

A fresh ordinary-registry consumer selects non-yanked Query 0.50.1 plus Host
artifact/filesystem 0.8.2, with exactly one identity for each Host package and
no path overrides or registry patches. Complete locked/offline metadata and
compilation pass on Linux. This verifies published-manifest convergence;
coordinated downstream adoption remains consumer-owned under
[Query #24](https://github.com/dragginzgame/ic-query/issues/24).

The pending 0.51.0 hard cut replaces the Python smoke harness and its two Rust
bridges with one development-only Rust runner against the selected Host 0.8.6
registry packages. POSIX signal handling records the first SIGINT/SIGTERM;
Host preserves five-second TERM grace and bounded reaping. This source needs
matching native Linux and both macOS architecture checks, MSRV compilation and
live canister smoke. Prior released-source runs do not qualify this working tree.
Focused Linux checks pass 23 runner fixtures, 105 inventory/catalog tests,
65 NNS/CloudEngine CLI unit tests and the binary authority-refusal fixture.
The catalog-only feature selection compiles, strict runner/CLI Clippy passes,
and the runner compiles on actual Rust 1.91.0, all locked/offline against the
explicitly prepared 0.8.6 graph. Formatting, section-style type docs and changed
local document links pass. The compatible Shared 0.1.31 notes previously prepared
for 0.50.4 are folded into this minor slice, preserving the existing staged work.
No full local CI, fresh native macOS or live integration result is claimed here.

The same pending 0.51.0 slice now selects Host 0.8.8 and the 73-file Shared
Tooling 0.1.34 snapshot at `3d33cd250fcae7dbe5cabe44b2abd6b2c91a1822`.
The unused fleet reporter is retired. Standard release entry points prepare the
locked cache before offline validation; standalone adapters remain offline,
explicit Cargo offline settings are preserved, and saved-evidence admission
does not fetch against interrupted metadata. Focused substituted cache and
real-Git metadata/logger fixtures pass on Linux Bash 5 and genuine Bash 3.2.57.
The selected graph is explicitly prepared with `cargo fetch --locked`;
23 smoke-runner and 55 cache-file fixtures pass locked/offline against Host 0.8.8. Consumer
tool-command fixtures, local `make cloc`, snapshot integrity, ShellCheck and
changed documentation links pass. Matching native macOS, live smoke and full
consumer CI remain unexecuted for these working-tree changes. The local source
commit `0c47009fe6ef16980858ebef0ab4316638fe3960` is not pushed when inspected;
the remote default branch remains at released 0.50.3.

That pending slice now refreshes the same 73-file selection to Shared Tooling
0.1.35 at `be550afa57fe9e16872e5110b5cd69c24b4fa9e8`, including the committed
Cargo network-preparation contract and selected registry binary/example installer.
Fixed-bundle and selected-tool fixtures pass on Linux Bash 5 and genuine Bash
3.2.57 with substitute Cargo, covering receipt/byte admission, offline reuse,
failure retention and concurrent-install refusal. Query's release-cache fixtures
also pass. This does not qualify actual registry installation of a selected tool
or native macOS execution; upstream source CI was queued when inspected.

Released 0.51.0 is pushed at `26da02ccbea1bf5d6d84483b1879535f9bf98708`.
Its matching [branch CI](https://github.com/dragginzgame/ic-query/actions/runs/37908616344)
and [tag CI](https://github.com/dragginzgame/ic-query/actions/runs/37908616373)
remain queued when inspected; native qualification for the hard cut is pending.
Released 0.51.1 selects Shared Tooling 0.1.36 and Host 0.8.9.
Installer and consumer tool-command fixtures pass on Linux Bash 5 and genuine
Bash 3.2.57, including canister cache preparation before all three runner actions,
offline/network refusal and unchanged lockfiles. Against the explicitly prepared
Host 0.8.9 graph, 23 smoke-runner and 55 cache-file tests pass locked/offline.
Snapshot integrity, ShellCheck, changed document links and the current-version
changelog check pass. The source is now published at
`5c71413237247823c21fec11ba91c74cff705544`; matching
[branch CI](https://github.com/dragginzgame/ic-query/actions/runs/37913796835)
and [tag CI](https://github.com/dragginzgame/ic-query/actions/runs/37913796105)
remain queued when inspected. Full local CI, actual deployment and native macOS
execution were not run for this slice. Cache-preparation native acceptance stays
with [#32](https://github.com/dragginzgame/ic-query/issues/32).

Before the 0.52.0 hard cut, the pending batch adopted Shared Tooling 0.1.38 at
`926a20606591214ab29faa236b0b584e4857439e`, excluding the sibling's dirty
version edit and retaining the same 73 selected files. The dependency-pinning
checker requires one fully validated exception document. Canonical regression
fixtures pass on Linux Bash 5 and genuine Bash 3.2.57, including multi-document
refusal and unchanged exception, manifest, lock and index evidence. Query's
actual declaration/inheritance check, snapshot integrity, ShellCheck and changed
document links pass. Query has no exception catalog. Formatting-hook changes
in Shared 0.1.37 are outside the selected executable helpers; refreshed guidance
does not activate hooks. The selected
[Shared source CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37912382208)
is queued; no full local CI or native macOS execution qualifies this pending
adoption yet.

The pending 0.52.0 slice retains the maintainer's Host 0.8.10 lockfile
selection. The released Host source at
`e944f114f7542d27ead5df8996d1b2df04e06c31` changes shared tooling, with no library
source changes from 0.8.9. After explicit locked cache preparation, 23
smoke-runner and 55 cache-file tests pass on Linux, locked/offline against the
published 0.8.10 crates. Host's matching
[source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37915010103)
is queued when inspected. These focused checks do not qualify native macOS or
full consumer CI for the dirty selection.

Pending 0.52.0 now adopts Shared Tooling 0.2.0 at
`8140e3dd1b44409d682c721889ab702f438c6a17` through the canonical exporter with
the same 73 selected files. PocketIC has no runtime callers in Query; the retired
checkers were not selected. The current five-tool pins and installer replace
PocketIC provisioning together, without adding a Testkit server selection.
The initial Linux offline check refused the existing six-tool bundle before
activation. Explicit installation and offline verification of the actual
five-tool bundle pass; the previous pins and receipt match byte-for-byte and
every old receipt-covered payload still verifies. Evidence is retained under
`/tmp/ic-query-shared-020-evidence.Rr7v4O/`. Canonical installer fixtures pass on
Linux Bash 5 and genuine Bash 3.2.57, including refusal and retained evidence
through failed setup and successful activation. Query's tool-command fixtures
and actual five-tool offline check also pass under both shells. Snapshot
verification, ShellCheck and the current-version changelog check pass.
The [Shared source CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37916384666)
is queued when inspected. Native macOS setup/check and actual canister startup
were not run for this dirty adoption; they remain consumer qualification gaps
tracked in [#33](https://github.com/dragginzgame/ic-query/issues/33).

The pending 0.52.0 batch now completes the maintainer's Host 0.9 requirements
with published 0.9.0, reviewed at
`715854b47b888eefb6fc0fca76d9c99f55099150`. Its Rust library sources are unchanged
from 0.8.10; the owner adopts the same Shared 0.2.0 setup hard cut. An initial
locked check refused the in-progress manifest/lock mismatch before compilation;
the explicit dependency update and cache preparation precede all new validation.
The smoke build now uses one bounded, structurally inspected Wasm read before
metadata publication, retaining the same original bytes through that boundary.
Against the coherent 0.9.0 graph, 24 smoke-runner tests (including malformed and
oversized Wasm admission), 55 cache-file tests and strict CLI/example test Clippy
pass on Linux, locked/offline. An earlier isolated cleanup check also passed
against the prior coherent 0.8.10 selection; it did not qualify the edited root
manifest. Formatting and changed documentation links pass. The exact
[Host source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37917141870)
is queued when inspected. Full consumer CI, native macOS and live canister
startup were not run for these dirty changes.

Released Query 0.52.0 is now published at
`af02d9fc8f98bc4a7f54a7993c602634294bea3a`. Its matching
[branch CI](https://github.com/dragginzgame/ic-query/actions/runs/37919404601)
and [tag CI](https://github.com/dragginzgame/ic-query/actions/runs/37919405178)
remain queued when inspected. The five-tool hard cut is delivered; native
acceptance remains open in [#33](https://github.com/dragginzgame/ic-query/issues/33).
The exact 0.51.0 and 0.51.1 issue-fix runs also remain queued; their delivered
source is not yet qualified by passing native execution.

Pending 0.52.1 adopts committed Shared Tooling 0.2.2 at
`ee48bb37c98c771e77b92fd891f0757d8c1c8b99` through the canonical exporter,
retaining 73 selected files. The IC installer admits every validated pin row,
including a final row without a newline, and retains the original pin bytes.
Canonical IC installer fixtures pass on Linux Bash 5 and genuine Bash 3.2.57,
including installation of the final tool and refusal of a wrong final
executable version even when its checksum receipt matches. Bundle-retention
fixtures also pass.
Query's consumer tool-command fixtures, actual installed five-tool offline
check, snapshot integrity, ShellCheck, dependency declarations, changed document
links and current-version changelog check pass. Shared's separate CI binary
installer publication fix is outside Query's executable selection.

The same pending slice preserves the maintainer-selected published Host 0.9.2
graph, reviewed at `c5decaefd17809829bfa969966729d672f609c49`. Host simplifies
durable publication options and closes captured pipes at EOF; Query's public
API, file permissions, evidence publication and TERM/reaping policy are retained.
Host 0.9.2's library sources are unchanged from 0.9.1; its release adopts the
same Shared Tooling fixes and preserves each pushed source's CI qualification.
The smoke runner moves admitted reports into receipts and streams pretty JSON
through a fixed-size buffer before durable publication, retaining the final
newline. Its receipt test covers buffer-spanning data, raw numbers beyond
`u64`, owner-only permissions and exact formatting. Focused example/test
Clippy passes with warnings denied on Linux.
After explicit locked cache preparation, 24 smoke-runner and 55 cache-file tests
pass on Linux, locked/offline. Matching
[Shared source CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37918955655)
and [Host source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37924013633)
are queued when inspected. No full local CI, native macOS or actual canister
startup was run for this dirty adoption.

Released Query 0.52.1 at `8568128a996e246e0b134b6a0254588f3d01ca12`
delivers Shared 0.2.2, Host 0.9.2 and streamed private receipts. Its exact
[branch CI](https://github.com/dragginzgame/ic-query/actions/runs/37926517640)
and [tag CI](https://github.com/dragginzgame/ic-query/actions/runs/37926517149)
are queued when inspected; native acceptance remains unqualified.

Pending 0.52.2 adopts committed Shared Tooling 0.2.3 at
`ac4549c5ebde497f7db0da5d05d32835112e51de`, keeping the same 73-file selection
and engineering baseline. Host remains selected at 0.9.2. The optional
installer suite is not selected; its new companion declaration does not add
another consumer setup path. Snapshot integrity and changed documentation
links pass. The exact [Shared source CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37925303425)
is queued when inspected.

The pending workflow fix for [#34](https://github.com/dragginzgame/ic-query/issues/34)
runs automatic native qualification on main pushes and PRs, eliminating the
duplicate tag complete-CI matrix. All nine jobs remain selected, with the same
three hosts, gate commands, per-source concurrency and retained failure evidence.
Explicit dispatch runs all nine at a selected branch or tag for qualification
outside main; annotated release/publication tag guards remain unchanged.
Actionlint and ShellCheck pass. Event-selection and consumer CI-script fixtures
pass on Linux Bash 5 and genuine Bash 3.2.57, including refusal of duplicate
tag checks, missing gates, restricted dispatch, reduced hosts and checkout
overrides. No full local CI, native macOS, live canister startup or workflow
dispatch was run for these working-tree changes.

The same pending 0.52.2 slice streams probe source contents into the existing
fingerprint and uses Host's stream hasher for Cargo.lock. The source roster,
path-component ordering, NUL framing and schema-1 build metadata remain intact.
An independently checked fixed digest covers a file larger than the read buffer,
path identity, excluded documentation and refusal of missing required inputs.
After explicit locked/offline cache preparation, all 25 smoke-runner tests and
focused example/test Clippy with warnings denied pass on Linux. Formatting,
changed documentation links and the current-version changelog check pass.
This adds local fingerprint evidence; native macOS and actual probe build/smoke
qualification remain separate and were not run.

Released Query 0.52.2 at `5b6b6d6c63346d357d715da5f7a8d04db15467e2`
delivers the CI event selection and streamed fingerprint cleanup. Its exact
[main CI](https://github.com/dragginzgame/ic-query/actions/runs/37936963838)
has passed complete checks, MSRV and live canister smoke on Linux. All six
macOS jobs remain queued when inspected. The source has one nine-job branch
run and no duplicate tag run; native macOS acceptance and manual dispatch
execution remain separate gaps.

The initial pending 0.52.3 refresh reviewed committed Shared Tooling 0.2.5 at
`04e07b4bf54e7aeb03eb7804a845cee27b7305df` through the canonical exporter,
retaining 73 selected files and the separately pinned engineering baseline.
The selected Rust-tool fixture declares its installer/checksum companions;
the incomplete selection is refused before consumer writes. Snapshot integrity,
ShellCheck, changed documentation links, current-version changelog admission
and consumer tool-command fixtures pass. Rust-tool setup/check/failure fixtures
and consumer CI-script fixtures pass on Linux Bash 5 and genuine Bash 3.2.57
with substituted Cargo and network commands. The hook installer and pre-commit
hook remain outside Query's selection; actual hook configuration is untouched.

Exact [Shared source CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37937705371)
passes Linux portable regression and lint/security; both macOS lanes are queued.
Newer dirty upstream changes are excluded. The initial tooling-only refresh
preserved the maintainer's incoming Host 0.9.3 lockfile selection before Rust
validation. No full local CI, native macOS or live canister startup was run
for the dirty adoption.

The same pending 0.52.3 slice now reviews published Host 0.9.3 at
`545e7236b91d84e190c80931b784f72cc4fafb11`; its library sources are unchanged
from 0.9.2. After explicit locked cache preparation, 26 smoke-runner and 55
cache-file tests pass locked/offline on Linux. The smoke runner selects its
unique deployed probe through an iterator without collecting a match list;
fixtures retain unrelated canisters, duplicate/missing refusal and principal
admission. Focused example/test Clippy passes with warnings denied on Linux.
Exact [Host source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37938930037)
passes native Linux and MSRV; both macOS lanes are queued when inspected.

The pending target-resolution cleanup passes 10 node-status tests without host
features and 21 with `host`, plus strict Clippy for the library and tests without
host features, on Linux with locked/offline dependencies. Rust formatting and
the changelog check pass. These local checks do not qualify native macOS or a
complete consumer CI gate.

Pending 0.52.3 now selects reviewed Shared Tooling 0.2.6 at
`ce13a5314916891fd239d9b199b4a91b04775054`, with 75 canonical snapshot files.
The shared release and formatting Make includes replace local recipes under
[#35](https://github.com/dragginzgame/ic-query/issues/35). Consumer CI-script,
release-cache and metadata fixtures pass on Linux with Bash 5 and genuine Bash
3.2, including nested Make, inert external-runner isolation, exact destinations,
cache selection, formatter admission/order and command-line policy overrides.
Actual check-only formatting, snapshot verification, ShellCheck, changelog and
documentation-link checks pass. Formatting preserves manifests, lockfiles and
the Git index; the shared hook remains outside Query's selection and was not
activated or executed. These checks preserve Query's stronger local guard;
Shared's generic Make admission and fixture-isolation repairs remain upstream.

The maintainer-selected Host 0.9.4 artifacts/filesystem/process packages map to
published source `4e3daebd5df07c6449279535668436024a45c02b`; their library sources
are unchanged from 0.9.3. After explicit locked cache preparation, 26 smoke-runner
and 55 cache-file tests pass locked/offline on Linux. Exact
[Shared source CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37944389294)
passes Linux portable regression and lint/security; both macOS lanes are queued.
[Host source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37947548868)
is queued when inspected. Query's current committed source
`ec0af4f140220de2da2534d3be2ae170fff5f370` has no hosted run when inspected;
the additional dirty adoption changes have no remote qualification. No full
consumer CI, live canister deployment, native macOS or release was run locally.

The reported `.git/release-state/0.52.3.verify.muLRod` attempt retained Host
0.9.3 inputs at 17:01 CEST; the lockfile changed to 0.9.4 at 17:04, before
packaging refused dirty source at 17:10. The subsequent Shared 0.2.6 adoption
was written after that failure. Package/publication admission now delegates to
the selected shared source checker, reporting each staged, unstaged and
untracked path without changing the index. Consumer CI-script and publication
guard fixtures pass on Linux with Bash 5 and genuine Bash 3.2, including real-Git
dirty-lock refusal before Cargo, hidden staged changes, unusual untracked names,
failed status observations and source/index preservation. ShellCheck, changelog
and documentation-link checks pass. Original release logs remain intact; no
complete CI gate or release was rerun, and native macOS remains unqualified.

Pending 0.52.4 selects published Shared Tooling 0.2.8 at
`b2646cde9abbc8861857a4379c683a0c19eba43e` (76 canonical files) and Host
artifacts/filesystem/process 0.9.7 at
`ca62e661918db2f4320743b9042a4993a5fff2aa`. These three libraries are unchanged
from 0.9.4. The released consumer Makefile's combined target-specific
`override export` syntax fails GNU Make 3.81 parsing; portable exported policies
now retain forced release cache preparation and offline/no-auto-install
formatting under inherited and command-line caller settings. Snapshot routing
is bound before includes load. The shared probe's command-line `MAKEFLAGS`
override gap remains in 0.2.8; Query retains its stronger guard and reports
the executed failure in [Shared #30](https://github.com/dragginzgame/shared-tooling/issues/30#issuecomment-6084528903).

Consumer CI-script, release-cache, metadata, publication and actual release-command
fixtures pass on Linux with GNU Make 4.3/Bash 5 and genuine GNU Make 3.81/Bash
3.2. Actual check-only formatting passes on both profiles. Snapshot integrity,
ShellCheck, documentation links, current committed-version changelog admission
and whitespace checks pass. After explicit locked cache preparation, 26 smoke
and 55 host-feature cache tests pass locked/offline. Package manifests and the
real Git index remain unchanged; no release commands or shared hooks ran.
Evidence is retained in `/tmp/ic-query-0524-*.log`; these Linux compatibility
checks do not qualify native macOS or the complete consumer gate.

Exact [Query 0.52.3 CI](https://github.com/dragginzgame/ic-query/actions/runs/37953720710)
passes all three Linux jobs; its six macOS jobs remain queued when inspected.
Exact [Shared 0.2.8 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37955525946)
passes Linux portable regression and lint/security, with both macOS jobs queued.
Exact [Host 0.9.7 CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37958032736)
passes Linux native and MSRV checks, with both macOS jobs queued. Pending
consumer 0.52.4 changes have no remote qualification.

The same pending slice repairs
[#36](https://github.com/dragginzgame/ic-query/issues/36): explicit developer
setup prepares only cargo-sort through the selected-package installer, while
formatter targets prefer its versioned directory. The actual-Make regression
fails against the previous global recipe after successful setup, then passes
with the repair on both Linux Make/Bash profiles. It uses the reviewed installer
and checksum helper with substitute Cargo/rustc effects and receipts; conflicting
local/global copies remain unchanged, and a failed install retains its candidate
and stops before Audit/Machete setup. An initial draft changed the unrelated
host PATH prefix; that failure is retained separately from the final passes.
The final PATH binding applies only to formatter targets. Consumer CI-script
and shared Rust installer suites, release/cache routing, check-only formatting,
ShellCheck and snapshot checks pass on both profiles where applicable. After
explicit locked cache preparation, 26 smoke and 55 host-feature cache tests pass
against Host 0.9.7. Evidence: `/tmp/ic-query-0524-issue36-*.log`,
`/tmp/ic-query-0524-rust-tools-*.log` and `/tmp/ic-query-0524-host097-*.log`.
No actual tool installation, full gate or native macOS execution was performed;
the prepared consumer changes remain uncommitted.

The maintainer delivered 0.52.4 at
`1a2bd9ea19531c724ca7844c59b0d25c8e03e3b0`.
[Exact source CI](https://github.com/dragginzgame/ic-query/actions/runs/37961527140)
passes Linux checks, MSRV and canister integration; its six macOS jobs remain
queued when inspected. The initial pending 0.52.5 pass completes the maintainer-selected Host
artifacts/filesystem/process 0.10.0 adoption from published source
`98562bea26a98993d93b80ed908bea4876c32a91`. Receipt replacement uses the unified
pathname writer with explicit owner-only options, and the smoke runner propagates
byte-publication errors intact. The reusable library's descriptor-relative writer
and public cache error projection retain their contracts; schema-1 evidence
requires no migration or reset.

After explicit locked cache preparation, 26 smoke-runner and 55 host-feature
cache tests pass locked/offline on Linux. Focused smoke-example Clippy with
warnings denied passes. Existing fixtures now assert producer error identity,
pre-publication create-only refusal and preservation of original bytes. Evidence:
`/tmp/ic-query-0525-host010-{smoke,cache,clippy}.log`. Exact
[Host 0.10.0 CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37961457616)
passes Linux native and MSRV jobs; both macOS jobs are queued. Pending consumer
source has no remote qualification. No full consumer gate, live deployment,
native macOS execution, release, publication or sibling edit was performed.
That initial pass excluded the then-unpublished parent-sync changes.

Pending 0.52.5 now selects published Host 0.10.1 at
`c7bdc3d4e1c658957202eebd76bff2c51e22f645`, including
[Host #43](https://github.com/dragginzgame/ic-host-tooling/issues/43).
Pathname publication independently syncs a newly observed missing directory
and its parent link even when another creator wins its creation. Existing parent
directories and descriptor-relative cache publication retain their behavior.
After explicit locked cache preparation, all 26 smoke-runner and 55 host-feature
cache tests pass locked/offline on Linux, and focused smoke-example Clippy passes
with warnings denied. Evidence: `/tmp/ic-query-0525-host0101-{smoke,cache,clippy}.log`.
Exact [Host 0.10.1 CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37964131536)
passes Linux native and MSRV jobs; both macOS jobs remain queued. Pending consumer
source remains uncommitted and has no remote qualification. No full consumer gate,
native macOS execution, deployment or release was performed for this update.

The maintainer delivered 0.52.5 at
`52e924412562273e19cc29e2f92aff556391e86b`.
[Exact-source CI](https://github.com/dragginzgame/ic-query/actions/runs/37968228194)
passes all nine jobs: complete checks, MSRV and live canister integration on
Linux, Intel macOS and Apple Silicon. This qualifies the delivered Host 0.10
migration and 0.10.1 selection, formatter setup, five-tool setup and smoke/cache
changes tracked in issues #28–33, #36 and #37; those issues are closed.
The manual-dispatch acceptance path in
[#34](https://github.com/dragginzgame/ic-query/issues/34) remains unexecuted.

Pending 0.52.6 selects Shared Tooling 0.2.10 at
`43a0dc46cdc3c77e70a68e192561642ed50a3e0f` through the canonical exporter,
selecting 77 files with the new formatting reporter. An isolated clean consumer
export was reconciled against the verified pending 0.2.9 files, preserving the
real index and caller-owned lockfile changes. Formatting reports compact success
and retains full failure stdout/stderr with its original status; CI uploads
`RUNNER_TEMP/formatting.*` alongside existing validation evidence.
Consumer CI-script, release-cache, metadata and simulated release-command
fixtures pass on Linux with GNU Make 4.3/Bash 5 and GNU Make 3.81/Bash 3.2.
Actual check-only formatting passes on both profiles. The maintained registry
observation fixture, Actionlint and ShellCheck pass. After explicit selected-cache
preparation, locked/offline workspace metadata passes. Evidence:
`/tmp/ic-query-0526-shared0210-*.log`. Query retains its stronger Make execution
guard; the shared probe is unchanged. The existing caller lockfile selection
was not changed by this adoption.
[Exact Shared source CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38033580922)
passes Linux portable regression and lint/security; both macOS jobs are running.
These local consumer changes have no remote or native macOS qualification.
No broad consumer gate or release was run for this update.

The maintainer delivered 0.52.6 at
`9442a7ab7c5b7c9915d1789c8f4305ca02591980`.
[Exact-source CI](https://github.com/dragginzgame/ic-query/actions/runs/38035646441)
passes all nine jobs across Linux, Apple Silicon and Intel macOS. The delivered
formatting adoption has complete native evidence under
[#35](https://github.com/dragginzgame/ic-query/issues/35).

The initial review, originally planned for 0.52.7, selected Shared Tooling 0.2.11 at
`83efac446348dea024798a331d77933b24b429dc` through the canonical exporter,
retaining 77 files. Its committed version is 0.2.11 despite the 0.2.10 commit
message. Shared's independent Make flag probes replace Query's supplementary
parser; overwritten invocation evidence is refused before recipes. CI-script,
release-cache, metadata, publication and simulated release-command fixtures pass
on Linux with GNU Make 4.3/Bash 5 and GNU Make 3.81/Bash 3.2. Actual check-only
formatting passes on both profiles after explicit selected-cache preparation.
Snapshot integrity and ShellCheck pass; package manifests, lockfile and real
index remain unchanged. Evidence: `/tmp/ic-query-0527-*.log`.
[Exact Shared source CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38034912323)
passes all four jobs, including portable regression on both native macOS hosts.
That producer qualification does not qualify these uncommitted consumer changes;
no broad consumer gate, native consumer execution or release was run locally.

That review next selected committed Shared Tooling 0.2.12 at
`a8ba9b461b831846eacf64452e6ddcd2acd000f1` through the canonical exporter,
retaining 77 files. Its committed version is 0.2.12 despite the 0.2.11 commit
message. Snapshot verification and refresh reject LF/CR in supplied and physical
directory paths before trimming can select another checkout, including aliases
and custom-manifest parents. Existing artifacts remain intact; ordinary spaces,
relative paths and physical aliases remain supported under `CDPATH`.
[Shared #95](https://github.com/dragginzgame/shared-tooling/issues/95).
The upstream snapshot-distribution fixture and actual consumer integrity check
pass on Linux Bash 5 and genuine Bash 3.2; the fixture remains upstream rather
than widening consumer CI. ShellCheck passes, and refresh preserves the already
selected Host lockfile and real Git index. Evidence:
`/tmp/ic-query-0527-shared0212-*.log`.
[Exact Shared source CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38037750017)
passes Linux portable regression and lint/security; both native macOS jobs are
queued. These pending consumer changes have no native qualification; no full
consumer gate or release was run.

The same initial preparation selected published Host
artifacts/filesystem/process 0.10.2, whose package VCS metadata matches release
`6b755763aca71b7ea8c3de40edf032093dfdc62c`. Only those three package versions
and checksums change in the lockfile. Pathname writers refuse directory-required
targets before normalization or publication under
[Host #44](https://github.com/dragginzgame/ic-host-tooling/issues/44).
The actual smoke receipt regression checks typed pre-publication failure,
preserved original bytes and unchanged directory entries for trailing `/`, `/.`
and repeated suffixes. After explicit locked cache preparation, 26 smoke-runner
and 55 host-feature cache tests pass locked/offline on Linux, along with strict
smoke-example Clippy. Logs: `/tmp/ic-query-0527-host0102-*.log`.
[Exact Host release CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38037252544)
passes Linux native and MSRV checks; Intel macOS is running and Apple Silicon
remains queued. This dependency
adoption has no native consumer qualification; no full local gate, release or
live deployment was run.

The maintainer reassigned the pending batch to the **0.53.0 hard cut**. It selects
Shared Tooling 0.2.13 at `5864f468d39f8f9d1bd26fca1afe0e20f25f1b5e` through the
canonical exporter, retaining 77 files and the separately pinned baseline.
Published Host artifacts/filesystem/process 0.11.0 all record release source
`1d768c80a5bb87e3330a6b7bacfdfc543063968f` in their package VCS metadata; registry
entries are unyanked. The incoming manifest's 0.11 requirements remain intact,
and only the three Host versions/checksums change in the selected lockfile.
Package release versions remain 0.52.6 for the maintainer's version transaction.

Smoke commands use current Host output policies and retain typed cleanup causes
separately from the original failure. Captured parsing bytes, deadlines, TERM
grace, bounded reaping and handoff policy retain their consumer ownership.
After explicit locked cache preparation, all 26 smoke tests, 55 host-feature
cache tests and strict smoke-example Clippy pass locked/offline on Linux.
[#38](https://github.com/dragginzgame/ic-query/issues/38).

Formatter admission now requires the exact selected local receipt and bytes;
another installation cannot satisfy it. This checkout initially refused before
formatting because that selected installation was absent. Explicit
`make install-format-tools` prepared it through the canonical installer.
Release fixtures prove setup/check ordering after source admission and fetching,
setup/admission failure, explicit offline refusal, source changes during setup,
reuse, preservation of prior selections and lock bytes, and saved-evidence
recovery without setup/fetch against partial metadata. The actual consumer CI
fixture refuses a missing selection before any Cargo dispatch under parallel
Make, and formatting also refuses a changed selection without fallback.
CI-script, selected-tool and release-cache fixtures pass on Linux with GNU Make
4.3/Bash 5 and GNU Make 3.81/Bash 3.2. Real-Git metadata fixtures and simulated
release-command routing pass on both profiles. Evidence:
`/tmp/ic-query-0530-*.log`. The initially refused formatter admission identified
a missing local preparation input, not a source failure; final checks use the
explicitly prepared selection. Current 0.52.6 changelog admission passes; the
0.53.0 HEAD-bound check refuses the uncommitted new detailed file as expected.
No broad consumer gate or real release was run.
[Shared #96](https://github.com/dragginzgame/shared-tooling/issues/96).

[Exact Shared 0.2.13 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38039035514)
passes Linux portable regression, lint/security and Apple Silicon regression;
Intel regression also passes, completing all four producer jobs.
[Exact Host 0.11 release CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38040092044)
passes Linux native/MSRV and Apple Silicon; Intel macOS remains running.
These producer observations and Linux compatibility profiles do not qualify the
uncommitted 0.53.0 consumer changes. Native consumer macOS acceptance remains a
gap, alongside the complete consumer gate.

## Tool-specific dependencies

| Workflow | Explicit prerequisites |
| --- | --- |
| Host setup and offline verification | Bash, curl for setup, tar/gzip for ripgrep, Perl, SHA-256 backend, reviewed `ci/tool-versions.env` |
| IC setup and offline verification | Bash, curl for setup, tar with xz/gzip support, Perl, SHA-256 backend, reviewed `ci/ic-tools.tsv` |
| Dependency declaration checks | Git, local jq/yq, Cargo for workspace discovery |
| Workspace version queries, changelog defaults and release/publication version admission | Prepared Cargo toolchain, local jq/yq; offline manifest validation without dependency resolution |
| Formatting and its offline checks | Prepared rustfmt and exact cargo-sort 2.1.4 from `ci/tool-versions.env`; installed explicitly by `install-dev` |
| Optional shared Cargo-tool set | Prepared Cargo toolchain and native compilation prerequisites; explicit `install-rust-tools` / offline `rust-tools-check`, using the three pins in `ci/tool-versions.env` |
| Local documentation links | Perl core modules; current guide and contract roster selected by Make |
| RustSec preparation and auditing | Bash, Git, explicit HTTPS advisory source, Cargo Audit; failed databases and preparation logs retained |
| Focused CI script fixtures | Bash, Make, Git, Python 3, Perl and ordinary utilities; Cargo/network effects use stubs |
| Governance runner and receipt tests | Selected Rust toolchain and locked/offline dependency cache, POSIX process groups |
| Complete gate | Declared Rust toolchain, local jq/yq/ripgrep/cloc set, Cargo Audit/Machete and ordinary utilities |
| Governance integration | Verified local IC set, Wasm Rust target and explicit local-runtime network access |

The Governance harness selects a persistent `ICP_HOME` only for ICP children,
using the same override on Linux and macOS. Identities, settings and launcher
packages share that home; build receipts and local runtime output retain their
separate owners. It refuses identity homes beneath standard/configured Cargo
output or harness runtime state, without moving existing keys. See the
[home selection and recovery procedure](canister-smoke.md#identity-storage-and-cleanup).
Released 0.47.11 passed all nine [exact-source branch CI jobs](https://github.com/dragginzgame/ic-query/actions/runs/37636399405),
including complete checks, MSRV and canister smoke on Linux and both macOS
architectures. That qualifies its persistent identity-home and Shared Tooling
0.1.19 adoptions. Released 0.48.1's repaired Host selection now qualifies the
0.48 descriptor publication and Shared Tooling 0.1.20 adoption. Its
[exact-source branch CI](https://github.com/dragginzgame/ic-query/actions/runs/37652231894)
has passed all nine checks, MSRV and canister smoke jobs across the
three supported hosts. All three native checks also passed in the matching
[tag run](https://github.com/dragginzgame/ic-query/actions/runs/37652231990). The
[Apple Silicon MSRV job](https://github.com/dragginzgame/ic-query/actions/runs/37652231894/job/112898231130)
failed before running any steps because GitHub could not acquire a runner after
five attempts; its annotation identifies ARM runner capacity constraints.
This was unavailable native execution, not an observed compiler failure. The
maintainer requested a fix and the failed-job retry passed for the same
release SHA in its [MSRV job](https://github.com/dragginzgame/ic-query/actions/runs/37652231894/job/112933559853).
This completes released 0.48.1's native branch qualification. Released 0.49.0's
[exact-source branch CI](https://github.com/dragginzgame/ic-query/actions/runs/37757330459)
at `59bf55ac5725ce4d336e4955c74a89b5f1e4793c` passes MSRV and live canister
smoke on all three supported hosts, as well as Linux checks. Both native macOS
checks fail the Python escaped-descendant timeout fixture under
[#23](https://github.com/dragginzgame/ic-query/issues/23). Those passing jobs do
not establish a green complete consumer gate. The prepared 0.49.1 repair and Host
0.5.2 selection still need their own matching native consumer CI.

Installer implementation regression suites are owned by Shared Tooling. This
consumer runs the reviewed canonical Rust-tool fixture with substitute Cargo;
the wider owner suites remain upstream. The consumer gate also exercises Make
ordering, explicit pin selection, local PATH and failure propagation, verifies
the immutable snapshot and installed host tools, and requires actual setup
qualification in the configured native CI jobs.

The native `checks` matrix runs the Rust Governance runner fixtures through
`ci-scripts-check` and `test`, which selects all targets including examples.
Those fixtures cover receipt admission and process cleanup with the prepared
locked/offline cache. The separate `canister` matrix owns live local-network
smoke execution and bundle construction on each declared host.
Failed `checks` jobs upload their job-owned temporary validation directories
and complete validation logs as `validation-failures-<host>` artifacts for
30 days, including hidden fixture metadata. Successful helper invocations remove
their own temporary directories; failed local checks print retained paths too.

## Development LOC reports

The immutable `make/tools.mk` include owns setup, offline verification and LOC
commands. The maintained workspace reporter needs Cargo, cloc, jq, Bash and
ordinary Unix utilities. Prepare cloc with the pinned host set, then report this
workspace from the checkout root:

```bash
make install-host-tools host-tools-check
make cloc
make cloc CLOC_MANIFEST=Cargo.toml
```

The reporter selects Cargo workspace members and uses the same Rust file lists
for LOC and test-attribute counts. Nested members count once, and a final `TOTAL`
row sums the member rows. The default root is the caller's current directory;
pass an explicit root when invoking the script from elsewhere. Historical measurements retain their original
method identity; compare them only after checking file selection and counting
rules. This command discovers workspace metadata and counts files; it does not
compile tests or run a broad gate.

Fleet tooling reports run centrally in Shared Tooling. Query selects only its
local workspace reporter and the shared setup/check and verification companions.

IC Query's formatting gate now requires the same reviewed cargo-sort pin used
by explicit development setup. `fmt-check` checks dependency order before Rust
formatting and preserves source, the Git index, lockfiles and unrelated edits.

The shared Cargo-tool set is optional for Query. `make install-rust-tools`
prepares cargo-sort, cargo-sort-derives and candid-extractor under `.tools/rust`;
`make rust-tools-check` verifies that complete local set offline. Query does not
need the latter two executables for its current workflows, so `install-tools`
and `tools-check` retain their existing selections. `install-dev` selects only
cargo-sort through the shared package installer, with a release-profile binary
under `.tools/rust/cargo-sort-<pin>-bin-cargo-sort-release/installed/bin`.
Formatter targets prepend that directory to the shared Make include's paths,
so an older optional bundle or global copy cannot shadow successful setup.
Those existing copies remain intact. Cargo Audit and Cargo Machete keep their
pinned global Cargo installations. Installer substitution tests establish
dispatch and failure handling, not native compilation of these executables.
