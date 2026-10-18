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
  and standard Unix
  utilities. macOS needs Xcode Command Line Tools for its compiler, Git and Make.
  Release/checksum helpers support both GNU `sha256sum` and macOS `shasum`.
- `make install-dependency-tools` explicitly installs the exact Cargo Audit and Cargo Machete
  versions declared in `Makefile`. `make install-host-tools` prepares common
  jq 1.8.2, Mike Farah yq 4.47.2, ripgrep 15.2.0 with PCRE2 and cloc 2.10
  under `.tools/host/bin`. Host versions
  and payload digests have one owner in the immutable `ci/tool-versions.env` snapshot.
  Host and IC paths reference verified bundles in the per-user store described
  in [shared executable storage](local-setup.md#shared-executable-storage).
  Use the same `SHARED_TOOLING_STORE` setting for setup, offline checks and
  failure collection. Previous physical checkout-local bundles remain retained.
  Cargo-sort 2.1.4 and cargo-sort-derives use the shared exact-slot
  installer and `.tools/rust/common/bin`; Cargo Audit and Cargo Machete use the
  same shared build-identity owner through their consumer-pinned binary selections.
  `make install-rust-tools` prepares the common selection and
  `make install-dependency-tools` prepares the extensions. Dependency execution
  uses their returned checked paths; old global installs remain retained.
  Cargo tool builds use the prepared compiler and the
  [standard tool environment](local-setup.md#standard-cargo-tool-build-environment).
  Preserve the managed source cache for offline checks and reuse.
  Offline `make format-tools-check` admits its receipts and bytes before version probes;
  global or optional-bundle copies cannot substitute for a missing selection.
  `make host-tools-check` authenticates all payloads before version and PCRE2 checks;
  the complete CI/release gate includes that offline check and
  `make dependency-pins-check`. Installation is separate from ordinary validation.
- `make install-tools` first admits the complete IC platform/pins and probes the
  selected Rust/Cargo toolchain, with Rustup auto-installation disabled. This
  read-only preflight runs before downloads and creates no tool/build directories.
  It then prepares host tools followed by the complete IC set:
  Quill, ICP CLI, didc, ic-wasm and wasm-opt under `.tools/ic/bin`, followed by
  common cargo-sort and cargo-sort-derives. The selected IC versions, host mappings
  and archive digests are owned by the [IC tool catalog](../ci/ic-tools.tsv).
  Query does not select candid-extractor. Query's Audit/Machete owners follow
  through the ordered local extension lists. `make tools-check` verifies all
  those selections offline; neither command prepares a Rust toolchain.
  Make selects the local paths; interactive shells use
  `export PATH="$PWD/.tools/host/bin:$PWD/.tools/ic/bin:$PWD/.tools/rust/common/bin:$PATH"`.
  See [local setup](local-setup.md) and [IC tools](ic-tools.md) for bootstrap
  packages, complete-set activation, locks and retained failed/previous sets.
  Shared 0.2.0 requires explicit setup to replace an active six-tool bundle;
  offline validation refuses it without conversion. Query uses ICP-managed
  networks for deployment smoke; the separate deterministic canister tests require
  `make install-testkit testkit-check` as described in the canister guide.
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
The smoke job uses the ICP CLI version and host-specific digest from the same
[IC tool catalog](../ci/ic-tools.tsv). Each downloaded archive is verified before
extraction and the executable version is checked before PATH admission. Native
macOS canister-network qualification requires passing smoke runs on this
configuration; Rust cross-compilation and the general macOS gate do not qualify it.

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

## Pending 0.54.0 toolset qualification

Query adopts committed Shared Tooling 0.3.0
`88a73139a0f083344c41a6f6f4b5c3a8aca7dc1d`, retaining 77 selected files and
the separate engineering baseline. Actual Linux setup prepares all common sets
and Query's selected formatter/Audit/Machete owners. Subsequent explicitly offline
reuse and offline checks pass under parallel GNU Make 4.3 and 3.81/Bash 3.2.
Earlier installed toolsets, selected formatter receipts and build evidence remain
available. Evidence: `/tmp/ic-query-shared030-setup.log`,
`/tmp/ic-query-shared030-setup-reuse-verified.log`,
`/tmp/ic-query-shared030-setup-reuse-portable.log`, and
`/tmp/ic-query-shared030-tools-check-*.log`.

Consumer script fixtures pass on both profiles, exercising complete common and
product ordering, exact pin/owner overrides, setup/check refusal at each step and
formatter failure retention. Simulated release guards pass, including retained
and descendant recovery plus real-Git source/index/metadata preservation. Shared
CI-log fixtures pass on both profiles. Evidence:
`/tmp/ic-query-shared030-ci-scripts-*-verified.log`,
`/tmp/ic-query-shared030-release-guards-*.log`, and
`/tmp/ic-query-shared030-gh-fixtures*.log`. No real release effects occur.

The owned Cargo setup recipes preserve Make's jobserver. The immutable shared
formatter recipes still emit closed-descriptor warnings on GNU Make 4.3 parallel
checks, although checks succeed; this reproduced producer gap is tracked in
[Shared #99](https://github.com/dragginzgame/shared-tooling/issues/99), with a
separate effect-free common-recipe reproduction retained at
`/tmp/ic-query-shared030-jobserver-reproduction.log`.

Full pending-source consumer CI and native macOS execution have not run. At the
recorded inspection, [Shared 0.3.0 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38044218125)
passed Linux portable regression and lint/security, with both native macOS jobs
queued. [Released Query 0.53.0 CI](https://github.com/dragginzgame/ic-query/actions/runs/38044026763)
passed its three Linux jobs, with all six macOS jobs queued. Those observations
do not qualify these working-tree changes. Native complete setup, reuse, failure
retention and full consumer delivery remain required through the owning workflow.

## Pending 0.54.1 setup qualification

The working tree adopts committed Shared Tooling 0.3.1
`fa452afaa5012866eb1c20820dfa8038c106e7ec` through the canonical exporter,
retaining 77 files. Focused consumer CI-script and selected Cargo-tool fixtures
pass on Linux with Bash 5/GNU Make 4.3 and genuine Bash 3.2.57/GNU Make 3.81.
Parallel setup admits both read-only preflights before installation and stops
before any common/product setup when either fails. The producer's actual
tool-command, host/IC setup and failed-fixture retention suites pass on both
profiles against that isolated committed source, including unsupported platforms,
missing/unavailable Rust commands, authenticated diagnostics and retained evidence.
Logs: `/tmp/ic-query-shared031-ci-{modern,portable}.log`,
`/tmp/ic-query-shared031-rust-fixtures-{modern,portable}.log` and
`/tmp/ic-query-shared031-test-*-{modern,portable}.log`.

Actual read-only preflight, explicitly offline complete setup reuse and parallel
offline checks pass on Linux with both Make profiles. Global Cargo setup needed
filesystem access to the existing Cargo installation receipt outside the agent
sandbox; the initial sandbox refusal and successful authorized reuse logs are
retained at `/tmp/ic-query-shared031-setup-reuse-*.log`. Check logs are
`/tmp/ic-query-shared031-tools-check-*.log`. Package manifest, lockfile and real
index hashes are unchanged. Shared formatter jobserver warnings still reproduce;
[Shared #99](https://github.com/dragginzgame/shared-tooling/issues/99) remains open.

[Released Query 0.54.0 CI](https://github.com/dragginzgame/ic-query/actions/runs/38048774180)
passes checks, MSRV and canister integration on Linux; all six macOS jobs are
queued at this inspection. [Shared 0.3.1 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38049622600)
passes lint/security, with Linux portable regression running and both macOS jobs
queued. These runs do not qualify the pending consumer snapshot. No broad local
consumer gate or native macOS execution was run for 0.54.1; required native
acceptance remains outstanding.

## Pending 0.54.2 tooling qualification

Query adopts committed Shared Tooling 0.3.3
`d63f0cfaba8ab2961d6012064adbf051c1898bc1` through the canonical exporter,
with 78 files including the new advisory README catalog task. The separately
pinned engineering baseline and scheduling authority are unchanged. Shared
validation-runner qualification passes against the isolated committed source on
Linux Bash 5/Make 4.3 and genuine Bash 3.2.57/Make 3.81; retained logs are
`/tmp/ic-query-shared033-runner-review-{modern,portable}.log`.

Consumer handler injection checks pass on both profiles for nounset, failed
commands, explicit nonzero exits, premature zero exits and completed success or
failure. Failed evidence and status survive; only completed success cleans up.
The actual consumer CI-script, complete simulated release and publication suites
pass on both profiles, including normal diagnostic checks with substitute Cargo,
real-Git source/index/metadata preservation, nested failure retention and saved
recovery. Evidence: `/tmp/ic-query-0542-completion-*.log`,
`/tmp/ic-query-0542-ci-*-final.log`, `/tmp/ic-query-0542-release-*.log` and
`/tmp/ic-query-0542-publish-*.log`.

The producer's actual tool-command and formatting-include fixtures pass on both
profiles, covering real descriptor inheritance and unsafe-mode refusal with
substituted effects. Selected Cargo-tool fixtures also pass. Actual parallel
offline `tools-check` passes on both Make profiles without jobserver warnings;
actual parallel `fmt-check` passes on Linux. Logs:
`/tmp/ic-query-0542-test-{tool-commands,make-format}-*.log`,
`/tmp/ic-query-0542-rust-*.log`, `/tmp/ic-query-0542-tools-*.log` and
`/tmp/ic-query-0542-format.log`. Package manifest, incoming Host 0.12.2 lockfile
and real index hashes are preserved. All copied Shared files retain their
reviewed producer bytes. [#40](https://github.com/dragginzgame/ic-query/issues/40).

No full consumer CI, real release, native macOS execution or broad feature/API
matrix was run for this working-tree batch. Shared 0.3.3 producer CI passes Linux
portable regression and lint/security, with both macOS jobs queued. Released
Query 0.54.1 CI passes all three Linux jobs, with six macOS jobs queued. Those
observations do not qualify the uncommitted 0.54.2 consumer changes.

## Pending 0.54.3 tooling qualification

Query adopts committed Shared Tooling 0.3.5
`a744d7f1990b9e1451ef45cd6d495de00a141cd3` through the canonical exporter,
retaining 78 selected files and the separate engineering baseline. The producer's
validation-runner fixture remains outside that selection. Its complete exact-source
regression passes on Linux Bash 5/Make 4.3 and genuine Bash 3.2.57/Make 3.81 with
inherited parent log, failure-log and GitHub summary destinations. Parent sentinel
bytes and directory contents remain unchanged while the fixture's own nested
logging and failure cases execute. Evidence:
`/tmp/ic-query-0543-runner-{modern,portable}.log` and the retained
`/tmp/ic-query-0543-runner-{modern,portable}.*/` directories.
[Shared #105](https://github.com/dragginzgame/shared-tooling/issues/105).

The reviewed common matrix now selects Binaryen 133. Actual Linux installation,
parallel offline IC checks and download-disabled setup reuse pass on both Make
profiles. The previous 132 bundle's files, pins and receipt remain intact. The
shared exact-source offline Wasm optimization/execution smoke passes with the
prepared tools and pinned Node 24.21.0: its unoptimized input and `-O3`, `-Os`,
`-Oz` variants retain the tested branch, integer-boundary and IC-style reply
behavior. This is a shared fixture, not a deployed Query canister. Query's
product build flow remains unoptimized. Evidence:
`/tmp/ic-query-0543-ic-{setup,check-*,reuse-*}.log`,
`/tmp/ic-query-0543-old-ic-{before,after}.log` and
`/tmp/ic-query-0543-wasm-opt.log`; retained modules and qualification metadata
live under `/tmp/ic-query-shared035-review.JRaWTs/binaryen133/`.
[Shared #102](https://github.com/dragginzgame/shared-tooling/issues/102).

The incoming Host artifacts/filesystem/process 0.12.4 lock selection is preserved.
All three registry entries are unyanked, their checksums match the lock and
packaged VCS metadata identifies released source
`5400f159474cebac1ec7ae7c8763abfd258bde03`. Library source is unchanged from
0.12.3. After explicit locked fetching, all 26 Governance smoke tests and 55
host-feature cache tests pass locked/offline on Linux. Consumer CI-script fixtures
also pass on both Bash/Make profiles. Evidence:
`/tmp/ic-query-0543-{fetch,smoke-tests,cache-tests,ci-*}.log` and
`/tmp/ic-query-host0124-registry-{artifacts,fs,process}/`.

No full consumer CI, native macOS execution, actual deployment or release ran
for this working-tree batch. Package manifest, incoming lockfile and real index
hashes are preserved. Native macOS common-tool setup, reuse and consumer checks
remain required; Linux and Bash 3.2 results do not qualify those hosts. Query's
separate optimized product/startup acceptance would be required before introducing
an optimization policy.

## Pending 0.55.6 tooling qualification

Prepared Shared Tooling 0.11.4 at
`031d214ed044705b989d43e2379f30e6a3d44749` selects 84 roots and 95 payloads,
including the canonical fixture lifecycle and pure workspace-version renderer.
Full common tool pin records match the delivered 0.11.0 selection. Exact
[producer CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38429576527)
passes Linux and native macOS Intel/Apple Silicon.

Query's actual caller-trap integration and Cargo-tool fixtures pass on modern
Bash and genuine Bash 3.2.57. Real-index metadata fixtures cover each mixed
before/prepared file, changed retained inputs, hidden staged changes and failed
validation-log preservation. The simulated release guards, canonical renderer
cases, offline tool admission, snapshot integrity and declaration checks pass.
Evidence: `/tmp/ic-query-issues-0556/`. Source changes are uncommitted; these
local checks do not establish native consumer qualification or a remote gate.

Delivered Query 0.55.5 `08896ad2b184dd74d541a3632eee513466f1cc0f` passes its
[Linux complete CI job](https://github.com/dragginzgame/ic-query/actions/runs/38439995507/job/115444079845).
Both native macOS architectures fail host/MSRV/smoke compilation in Host's
retained-staging permission conversion. [Query #52](https://github.com/dragginzgame/ic-query/issues/52)
tracks adoption after the [Host #84](https://github.com/dragginzgame/ic-host-tooling/issues/84)
repair is published and qualified. Incoming Host 0.17.7 remains selected and
contains that defect; no native success, downgrade or downstream patch is claimed.
Full local CI, release execution and native consumer qualification were not run.

## 0.55.5 tooling preparation evidence

Prepared Shared Tooling 0.11.0 at
`a15ff323449f4b6190f647aec0de43cbcfb314ba` retains 81 roots and 91 payloads.
Nine selected payloads change; full host/Cargo and IC pin records are unchanged.
The existing Query selection has no removed CI-installer wrapper callers or roots.
The new optional Cargo release-metadata helper and dirty fixture-lifecycle draft
are outside this selection; Query's existing release adapter remains in place.

The exact-source GitHub inspection, host-tool and IC-tool fixtures pass on modern
Bash and genuine Bash 3.2.57. Host/IC suites exercise storage and evidence
companions as well. Offline tool admission, sourced-helper ShellCheck, dependency
pins, snapshot integrity and documentation links pass. The format gate exposed
preexisting dependency ordering in the committed library manifest; reordering its
four entries preserves all parsed TOML values. The same gate also required
ordering the existing ledger-evidence module after leb128; declarations and
feature conditions are unchanged. The corrected source passes formatting.
Evidence: `/tmp/ic-query-shared011-0555/`.

Exact [Shared 0.11.0 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38415610560)
is queued at inspection. Local shell fixtures, including Bash 3.2 on Linux, do
not establish native macOS producer or consumer qualification. Full local CI,
release execution and native consumer checks were not run. Package versions,
lock selection, compiler/tool pins and the real Git index remain unchanged.

## Pending 0.55.4 tooling qualification

Initial preparation selected public Shared 0.10.0
`ed9c26f015691859f67b50b512a94709dcdc0c6c`, with 81 roots/91 payloads.
Common host/Cargo versions and all IC version/digest records match the prior
snapshot. Their redundant explicit roots are removed; required installer
companions retain both immutable catalogs. No exception, tool upgrade or
application dependency change is introduced. Query's Make root omits its trailing
slash so all catalog callers and fixture assertions use the same path.

Evidence is retained in `/tmp/ic-query-shared010-0554/`. Explicit locked source
preparation and selected-tool setup pass, followed by offline tools/formatting
admission. Seven pinned producer suites pass on modern Bash and genuine Bash
3.2.57: command/default selection, formatting, Make consumption, host/IC setup,
Cargo freshness and snapshot distribution. The host/IC suites include compact
and full evidence-action cases against synthetic owned installations.

Query's CI-script fixtures pass on both shells, covering selected catalogs,
formatting, extension tools and retained evidence. Actual consumer hook, receipt,
prepared-tool reuse and selection-preservation fixtures pass on both shells.
Consumer release-cache, adapter and runner fixtures pass with simulated Git
changes; no real commits, tags or pushes were created. The initial consumer
fixture exposed a double slash in the snapshot catalog path; its refusal is
retained separately from the passing rerun after the Make root fix. ShellCheck
with sourced helpers, snapshot integrity and complete pin-record comparison pass.

Exact [Shared 0.10.0 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38382161082)
passes Linux, Apple Silicon and lint/security; Intel is in progress at inspection.
The released [Query 0.55.3 CI](https://github.com/dragginzgame/ic-query/actions/runs/38394745228)
passes Linux MSRV and canister checks; Linux checks are in progress and macOS
jobs are queued. Neither qualifies this uncommitted consumer adoption. Full
local CI, real release execution and native macOS consumer qualification were
not run. Delivery and final-source all-host acceptance remain open in
[#48](https://github.com/dragginzgame/ic-query/issues/48).


The previous 0.55.4 preparation selected Shared 0.10.2
`626fc2bd3125195520fb870b92abfec11b01454d`, retaining 81 roots/91 payloads.
Only the selected Cargo cache-key helper, Rust-tool fixture and snapshot-drift
rules change from 0.10.0; the unselected GitHub sibling dashboard is not adopted.
Literal-catalog cache keys ignore unrelated pin/comment/order edits while
preserving Cargo/build identities and conservative nonliteral handling. Installed
tool build identities and Query's CI cache policy remain unchanged.

The incoming Host artifacts/filesystem/process 0.17.2 lock selection is preserved.
All three registry checksums and packaged VCS identities match published source
`e3afa6ddf022309e7ba97f6cefb312c48738db4e`; their implementation sources are
unchanged from 0.17.1. Evidence: `/tmp/ic-query-shared0102-host0172-0554/`.
After explicit locked source preparation, 58 cache-file and 27 Governance runner
tests pass locked/offline. Strict host-library Clippy, complete offline tool/
format admission, pins, type-docs and sourced-helper ShellCheck pass. The expanded
Rust-tool/cache-key fixtures pass from the prepared consumer context on modern
Bash and genuine Bash 3.2.57, including guarded Git effects and staged inputs.

Exact [Host 0.17.2 CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38395332181)
passes Linux, Intel and Apple Silicon native gates, MSRV and lint/security.
Exact [Shared 0.10.2 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38405254734)
has Linux in progress and both macOS lanes queued at inspection. These producer
results do not qualify this uncommitted Query selection. Full local CI, native
macOS consumer checks and release execution were not run. Package manifests,
the incoming lockfile, pins/toolchain and real index are unchanged by this batch.

## Pending 0.55.2 qualification

Initial preparation selected Shared 0.9.13
`35cad65677ff989b5fb24a88d947fda37d2366e8`, retaining 83 roots/91 payloads.
That initial adoption excluded dirty upstream 0.9.14 changes. Host artifacts, filesystem
and process 0.16.0 matched the selected registry checksums; their Rust sources were
unchanged from 0.15.1. The incoming manifest/lock selection, including Deranged
0.5.9, remains intact. Query does not select the changed `ic-host-tools` defaults.

Evidence is retained in `/tmp/ic-query-adopt-0552/`. Locked source preparation,
explicit setup of the existing common/extension tools, and offline complete
Host/IC/Rust tool and formatting admission pass. Consumer-root Rust fixtures,
canonical lock diagnostic/status/retention checks, and pinned-source CI selection
fixtures pass on modern Bash and genuine Bash 3.2.57. Actual Query formatting
hook, receipt admission, prepared-tool reuse and selection preservation pass on
both shells. With the selected Host graph, 58 cache-file tests and 27 Governance
runner tests pass locked/offline; library and runner Clippy pass with warnings
denied. Snapshot, dependency pins, changed-script ShellCheck and documentation
links pass. Protected manifest/lock/pin/toolchain/index hashes are unchanged.

Exact [Shared 0.9.13 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38368099898)
passes Linux portable regression and lint/security; both macOS lanes are running
at inspection. Exact [Host 0.16.0 CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38368805239)
passes Linux native, MSRV and Apple Silicon native checks; Intel macOS is queued.
These owner observations do not qualify this uncommitted Query selection.
No full local CI, release execution or native macOS consumer run was performed.
Native consumer acceptance remains pending in the
[Shared fleet tracker](https://github.com/dragginzgame/shared-tooling/issues/98).

The additional decimal-mechanics cleanup is qualified in
`/tmp/ic-query-decimal-0552/`. CloudEngine, Dashboard/ICRC and SNS owner tests
pass, including arbitrary-size cursors and their leading-zero normalization.
Two ICRC history loopback fixtures initially lacked socket permission; their
successful owner rerun and original refusals are retained separately. Each of
the four affected host features and the pure library builds independently;
strict library Clippy, formatting and documentation checks pass. No full local
CI or native macOS run was performed for this additional prepared change.

The CLI option consolidation is qualified in `/tmp/ic-query-options-0552/`:
all 53 NNS and 36 SNS command tests pass locked/offline. Strict CLI Clippy,
including test targets, and formatting/documentation checks pass. Manifest,
lock, toolchain, tool-pin, snapshot and real-index hashes match the incoming
prepared selection. Full local CI and native macOS execution were not run.


The current prepared selection is Shared 0.9.15
`a9a8ea5ddea0db279d8c139ef5a951474c50f742` (83 roots/91 payloads),
canonically refreshed from a clean pinned checkout. Host artifacts/filesystem/
process 0.17.0 are already selected by the incoming manifest and lockfile; all
three registry checksums and packaged VCS identities match released Host source
`93a0ecc2afb7d4b8d8a252a21ed10d7e58b35901`. Dirty sibling changes are excluded.
No dependency, tool-pin, package-version or real-index changes were made.

Additional evidence is retained in `/tmp/ic-query-sns-shared0915-0552/`.
After explicit locked source preparation, 204 SNS report tests, 58 cache-file
and 27 Governance runner tests pass locked/offline. The extended status fixture
proves ambiguity handling plus later read-budget and parse-error precedence;
its final owner rerun passes after removing a redundant test clone identified
by Clippy. Strict library/test Clippy passes. Shared CI-log recovery and
maintenance-path fixtures pass on modern Bash and genuine Bash 3.2.57, using
stubbed GitHub and Codex calls. Offline tool/formatting admission, snapshot,
dependency-pin, type-doc and changed-script ShellCheck checks pass. The standalone
SNS host feature check and 307 local documentation links pass. Manifest, lock,
toolchain, pin and real-index hashes match the incoming prepared worktree.

Exact Shared 0.9.15 and Host 0.17.0 CI runs remain queued at inspection:
[Shared](https://github.com/dragginzgame/shared-tooling/actions/runs/38376977855),
[Host](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38376461607).
The released Query 0.55.1 run is also queued. These runs do not qualify the
uncommitted 0.55.2 source. Full local CI, release execution and native macOS
consumer qualification were not run for this additional batch.

## Released 0.55.1 Shared 0.9.10 qualification

Earlier preparation selected committed Shared 0.9.9
`da4977129b9e2d3a62178fba8925026f9e42daa6`, retaining 82 roots and deriving
90 payloads. The Rust fixture isolates its compiler context, borrows existing
history and guards forbidden Git effects; it runs from the consumer root.
The changed Cargo build-policy helper requires explicit preparation of existing
common and Query extension tools before offline admission. Tool versions,
application dependencies and native host requirements remain unchanged.
Evidence is retained in `/tmp/ic-query-shared099.xIBPNF/`.
Consumer-root Rust fixtures pass on modern Bash and genuine Bash 3.2.57,
including guarded Git effects, staged pins/compiler, cache keys and receipts.
Explicit common/extension setup and offline tools/formatting admission pass.
Query's CI-script target passes, including 27 Governance fixtures; real hook,
receipt, prepared-tool reuse and partial-selection preservation pass on both
shells. Initial sandboxed hook attempts could not acquire a shared-store lock;
their logs remain separate from the successful runs with setup permissions.
Snapshot integrity, dependency pins, ShellCheck, links and unchanged protected
manifest/lock/pin/toolchain/index hashes pass. No full local CI, release execution
or native macOS run was performed for this prepared batch.
The 0.9.10 delta adds reporting policy only; selected executable bytes are
unchanged from 0.9.9. Snapshot and documentation checks pass, with evidence in
`/tmp/ic-query-shared0910-review/`. Exact [Shared 0.9.9 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38305700979)
passes Linux and both macOS producer lanes. Query 0.55.1 is now delivered at
`839d43c972f136feeaebec9fb837ee63bf829a6d`, selecting Shared 0.9.10
`7e55adecf6f0b196c050f104b37f8c69b6bf434e`, 83 roots/91 payloads, and Host 0.15.1.
Its [exact consumer CI](https://github.com/dragginzgame/ic-query/actions/runs/38368542552)
passes Linux complete checks, MSRV and canister integration, plus Apple Silicon
MSRV. Other required macOS lanes remain queued at inspection; acceptance stays
pending in the [Shared fleet tracker](https://github.com/dragginzgame/shared-tooling/issues/98).

The slice's earlier prepared internal cleanup was qualified on Linux in
`/tmp/ic-query-cleanup-0551/`. NNS/SNS parsing, SNS reports, Dashboard URL and
loopback transport, managed cache writers and cache-header/status tests pass.
Independent Dashboard, SNS, ICRC and Subnet Catalog feature builds pass, as do
pure native and Wasm canister builds. Library and CLI Clippy pass with warnings
denied; formatting, source/process boundary, snapshot, type docs and local links
pass. Cargo Machete reports no unused dependencies. The selected manifests,
lockfile, tool pins, toolchain and Git index remain unchanged. Full local CI and
native macOS execution were not run for these prepared changes.

## Released 0.55.0 async host API qualification

Host capabilities and collectors now return caller-runtime futures. Object-safe
sources require `Sync` and return boxed `Send` futures; progress sinks require
`Send`. Pure projections, rendering and explicit cache-only IO stay synchronous.
The CLI owns its current-thread runtime. Stateful CloudEngine, ICRC, NNS
Governance and SNS adapters retain an endpoint/response-bound agent across a
collection; Dashboard sources retain a bounded HTTP client. Runtime wrappers
and their runtime failure branches are removed, with schema-1 cache/report
formats and lock/attempt recovery preserved.

Focused Linux evidence is retained in `/tmp/ic-query-055-async/`: 882 affected
host library tests pass, with 48 unchanged text tests filtered. All 171 CLI unit
tests, 45 CLI smoke tests, 27 Governance process fixtures and 133 downstream/API
tests pass. Library/test Clippy and all CLI-target Clippy pass with warnings
denied. Rust 1.91 native CLI/host checks and pure/canister library Wasm checks
pass locked/offline. Explicit fetching admitted the selected graph; the task
added Tokio test macros. Separate incoming AWS/Syn updates were preserved, with
the initial and current metadata retained independently.

Canonical Shared 0.9.5
`b1630b89482cc6071fc928a86962d79c9c11fdb7` retains the selected 82 roots/89
payloads. Real curl loopback retry fixtures pass on modern Bash and captured
Bash 3.2.57, including transient recovery, exhausted/permanent errors and
checksum refusal. Snapshot, shell and local-link admission pass. These checks
do not install tools, run a release or qualify native macOS. The optional
producer artifact-discovery helper is outside Query's selected closure.

The earlier focused evidence above qualified prepared source. The maintainer
delivered 0.55.0 at `6aa75c20d213cda9cd592c5722d0a0707eab6cdc`, retaining
Shared 0.9.5 and selecting Host artifacts/filesystem/process 0.15.1. Its
[exact CI](https://github.com/dragginzgame/ic-query/actions/runs/38290525080)
passes all nine complete-check, MSRV and canister jobs across Linux and both
macOS architectures. Direct macOS logs confirm actual formatting-hook, receipt,
prepared-tool reuse and selection-preservation fixtures, completing
[#44](https://github.com/dragginzgame/ic-query/issues/44) for this released source.
These results do not qualify the prepared 0.55.1 snapshot.

## Prior prepared Shared 0.9.2 and Host 0.14.10 qualification

The unshipped 0.54.16 tooling notes were folded into the accepted 0.55.0 slice;
the original qualification below retains its original source/evidence identity.

Canonical refresh selects Shared 0.9.2
`c118062a070a2e238f1eb3989b6fd700f369ce1e`, retaining 82 entrypoints and
89 payloads. Exact-source host/IC lifecycle fixtures pass on modern Bash at
0.9.1 `e8a9ea6b660f216166eb00d0af6a32462bdbda4a` and genuine Bash 3.2.57
at finalized 0.9.2, including checkout cleanup failure, shared-lock release,
original error propagation, retained candidates and retry/reuse. The lifecycle
owner is unchanged in 0.9.2; its new checksum NUL and installer CR/LF admission
fixtures pass on both Bash profiles. Complete offline tool admission, snapshot,
shell, notes and local-link checks pass. The initial old-shell PATH was absent
and fell back to modern Bash; final Bash 3.2 runs use a captured binary with
recorded version and checksum. Failed raw-index preservation evidence remains
retained; staged entries are unchanged from HEAD and final manifest/lock/index
hashes match the recaptured inputs. Evidence:
`/tmp/ic-query-shared091-adoption.LeIVWn/`.

The incoming Host 0.14.10 selection changes only three Host versions/checksums
in Query's lock. Explicit locked fetching, complete locked/offline metadata and
registry/archive/VCS/source admission pass for release
`886a8df0424d548e08126ef8dfab889e6d8e8071`. The only library-source change
isolates a producer unit test; runtime source, dependency declarations and the
declared Rust 1.88.0 floor remain unchanged. No runtime tests or native MSRV
build were repeated for this delta. Existing manifest, lock and real index
hashes are preserved. Evidence: `/tmp/ic-query-05416-host01410.mVBzK9/`.
[Exact producer CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38264225245)
is unfinished; [Host #70](https://github.com/dragginzgame/ic-host-tooling/issues/70)
retains native producer-test acceptance.

The earlier delivered Host 0.14.9 selection was admitted through explicit locked
fetching and complete offline locked metadata. All three non-yanked registry
checksums match the cached archives; their VCS records identify released Host
`bea0f3e764915bff5b8ff0c61ed36dd5596f3f43`. Packaged library sources and
dependency declarations match 0.14.5, retaining declared Rust 1.88.0. This is
archive/source/graph qualification, not repeated runtime tests or a new native
MSRV build. Query's declared Rust floor remains 1.91.0.

Delivered Query 0.54.15 `9974ef56f525f7c6cfc22d14f1673baa4c562f33`
records Shared 0.9.0. Its [matching CI](https://github.com/dragginzgame/ic-query/actions/runs/38249683614)
fails Linux checks during the Quill download with HTTP 500, before compilation;
the complete failed log is retained in `/tmp/ic-query-05416-inspection/`.
The canonical asset-download owner remains single-attempt in 0.9.2;
[Shared #138](https://github.com/dragginzgame/shared-tooling/issues/138)
owns the proposed bounded transport retry. All three canister and MSRV jobs now
pass; the two native check jobs remain queued. These results do not complete the
remaining checks/native criteria in
[#44](https://github.com/dragginzgame/ic-query/issues/44),
[#45](https://github.com/dragginzgame/ic-query/issues/45) and
[#46](https://github.com/dragginzgame/ic-query/issues/46).

## Delivered 0.54.15 Shared 0.9.0 qualification

Canonical refresh retains 82 entrypoints and 89 payloads at finalized Shared
0.9.0 `aec7a149f581a42b9531fc747b46024fc0314fcf`. Query's formatting
fixture uses the trusted shared reader against its copied manifest before
projecting file records. Syntax admission rejects NUL and malformed late input
without emitting partial records, preserving order and literal filename spaces;
payload integrity and committed-source ownership remain separate checks.

Reader boundary tests and actual Query formatting/hook/reuse pass on modern
Bash/Make and genuine Bash 3.2.57/GNU Make 3.81 against implementation commit
`019ce8236cc7cee14ea5540d468b939a2053ef30`. The final commit changes only
`VERSION`, retaining identical helper code. Final 0.9.0 integrity and record
output checks pass on both profiles. ShellCheck, syntax, notes, links and captured
non-lock input hashes pass. Final preservation checks detected an external Host
lock change to 0.14.9; the original failed logs remain retained and the incoming
lock is preserved. Repeated actual hook/reuse checks pass on both profiles with
the current 0.9.0 snapshot and incoming lock; `incoming-preservation.log` confirms
all newly captured inputs remain unchanged. This does not qualify the Host
library update or relabel the earlier failed preservation checks.
Evidence:
`/tmp/ic-query-shared090-adoption.htYCwG/`.

The superseded implementation CI failed on an HTTP 504 during IC tool download;
its complete log is retained. This is transport evidence, not a reader failure.
[Final source CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38246537047)
passes lint/security, with Linux regression running and native jobs queued at
inspection. Full local CI, library tests, release gates and native consumer
execution were not run. Pending consumer native acceptance remains under
[#44](https://github.com/dragginzgame/ic-query/issues/44).

## Prior 0.54.15 Shared 0.8.4 qualification

Canonical refresh retains all 82 entrypoints and 89 payloads at Shared 0.8.4
`dca2bdd018afa6be598f03bb3cce1bab79c66973`, matching upstream main at
review. Only the selected LOC reporter and snapshot guidance change for this
delta; no files or pins are added or retired.

The exact-source LOC fixtures pass with modern Bash and genuine Bash 3.2.57,
covering JSON/text agreement, independent workspace selection, generated-output
exclusion and original failure status without partial JSON. The actual Query
workspace JSON/text counts agree under locked/offline metadata. Snapshot,
ShellCheck, syntax, notes and local links pass; incoming Host lock, manifests,
local policy, Makefile and real index retain captured hashes. Evidence:
`/tmp/ic-query-shared084-adoption.z1WKRO/`.

Earlier 0.8.2/0.8.3 evidence below retains its original identity and was not
repeated for unchanged boundaries. No full local CI, library tests, release gate
or native consumer execution ran.
[Exact Shared CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38243170578)
passes lint/security, with Linux regression running and both macOS jobs queued
at inspection. Matching consumer native qualification remains open under
[#44](https://github.com/dragginzgame/ic-query/issues/44).

## Prior 0.54.15 Shared 0.8.3 qualification

Canonical refresh retains all 82 entrypoints and 89 payloads at Shared 0.8.3
`8cca7e67b03ce6e701b6e7dcec2e9753eb98337e`, matching upstream main at
review. No files are added or retired. Interrupted payload copies preserve the
original bytes, manifest and fixture index; same-source retry completes on
modern Bash and genuine Bash 3.2.57. New payload scratch names are ignored at
any depth, preserving existing orphaned artifacts.

The updated canonical Cargo fixtures pass on modern Bash/Make and genuine
Bash 3.2.57/GNU Make 3.81, including redirected observation refusal, unchanged
outside evidence/prior selection and retained archive links. Query's actual
formatting admission passes with distinct host/Rust catalogs on both profiles;
complete offline tool admission passes and successful PCRE2 checks remain quiet.
Snapshot, ShellCheck, syntax, notes, links and captured manifests, incoming Host
lock, policy, Makefile and real index pass preservation checks. Evidence:
`/tmp/ic-query-shared083-adoption.y9nLnP/`.

The 0.8.2 freshness feature and its qualified wrapper remain in this slice;
unchanged checks below were not repeated for the 0.8.3 delta. Full local CI,
release gates, library tests and native consumer execution were not run.
Matching remote consumer native acceptance remains outstanding under
[#44](https://github.com/dragginzgame/ic-query/issues/44); upstream native
qualification is tracked by its
[exact-source CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38238988308).

## Prior 0.54.15 Shared 0.8.2 qualification

Canonical refresh selects clean committed Shared 0.8.2
`aaf75a37717357dd62f4b0b15001eaa3ba1629cf`, with 82 entrypoints and
89 payloads. The three additions are the Cargo-tool freshness checker, its task
guide and offline substitute-registry fixture. The explicit online Make target
delegates to the shared Perl owner; CI registers only its offline fixture.
No pins or product behavior change, and the incoming Host 0.14.7 lock is preserved.

The fixture passes on modern Bash and genuine Bash 3.2.57. Make wrapper argument
preservation and failure propagation pass with modern Make and GNU Make 3.81.
The actual online target observes 17 common/user packages as current and retains
registry responses under `/tmp/ic-query-shared082-adoption.VftHrc/`. These are
receipt-version observations, not executable byte or build admission; Query's
Audit/Machete setup and offline checks retain their own scope. Snapshot, Perl,
ShellCheck, notes, local links and index/manifest/lock preservation checks pass.
The initial dry-run guard refusal remains retained; wrapper qualification uses
substituted command effects with real Make execution.

At source review, upstream main still selected 0.8.1 and no matching 0.8.2 CI run
was available. The snapshot is from the exact clean committed checkout; this is
not upstream publication or native qualification. Full local CI, library tests,
release gates and native execution were not run for this tooling-only delta.
Matching native acceptance remains outstanding under
[#44](https://github.com/dragginzgame/ic-query/issues/44).

## Delivered 0.54.14 Shared 0.8.1 qualification

Released Query `08182547d9d93680f6ab38e9ba6981acb091716f` matches remote
main and selects the snapshot reviewed below. Its
[exact-source CI](https://github.com/dragginzgame/ic-query/actions/runs/38236368356)
passes Linux MSRV/canister integration, with Linux checks running and all native
jobs queued at inspection. No additional current-source failure is observed.

Canonical refresh retains the same 79 entrypoints and 86 payloads at Shared
0.8.1 `c97f9f9c7b7c706fffdcfc5925cf58759b58093f`. No files are added or
retired. Both refresh and verification reject 12 malformed header/file/root
records on modern Bash and genuine Bash 3.2.57; verification also admits
filenames with leading, internal and trailing literal spaces.

The canonical Cargo fixtures pass on both Bash/Make profiles, exercising
compiler, native and policy identity changes during substitute installation.
They confirm original/observed evidence, refusal before activation, preservation
of the prior selection and inclusion of both observations in the standard
evidence archive. Complete offline tool admission and Query's actual formatting
hook/reuse pass, the latter with genuine Bash 3.2.57/GNU Make 3.81. ShellCheck,
syntax, notes, links and captured manifest/lock/index/payload hashes pass.
Evidence: `/tmp/ic-query-shared081-adoption.W0i3GW/`.

[Exact Shared CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38231862048)
passes Linux regression and lint/security; both macOS jobs are queued at
inspection. No full local CI, release gate, library tests or native consumer
execution ran for this tooling-only update. Matching pending-source consumer
native acceptance remains outstanding under
[#44](https://github.com/dragginzgame/ic-query/issues/44).

## Delivered 0.54.13 Shared 0.8.0 qualification

Released Query `ea3d23391d4ae6936b2b04f9295ae4b0c1760f22` matches remote
main and selects the snapshot reviewed below. Its
[exact-source CI](https://github.com/dragginzgame/ic-query/actions/runs/38232523331)
passes Linux MSRV and canister integration, with Linux checks running and native
jobs queued at inspection. Delivery does not complete native acceptance.

The canonical exporter refreshes the same 79 entrypoints and 86 payloads to
Shared 0.8.0 `045a6f22a9de497bfb3af90cb2571957384806f2`. Preview changes
two shared guides and apply records the new version/revision; no Query payload
is removed. Current refresh automatically retires unchanged previously owned
files outside the selected closure, after source, mode, index and path checks.
Query's direct caller roots and local policy remain intact.

Disposable consumer checks pass with modern Bash and genuine Bash 3.2.57:
recorded-root reuse, non-mutating preview, automatic source-proven retirement,
refusal to remove edited input, retained caller evidence and repeat refresh.
All retirement is confined to those fixtures. Explicit common-tool setup now
passes for the current native environment; actual Query formatting/hook/reuse
checks pass on modern Bash/Make and genuine Bash 3.2.57/GNU Make 3.81.
Audit/Machete setup also prepares their unchanged versions for the current native
environment; complete offline `make tools-check` passes. Its initial receipt
refusal, previous failed attempts and old tool payloads remain retained. Snapshot,
ShellCheck, notes, links and preservation checks pass. Evidence:
`/tmp/ic-query-shared080-adoption-path`.

[Exact Shared 0.8.0 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38229670802)
passes lint/security, with Linux regression running and both macOS jobs queued
at inspection. No full local CI, release gate or native consumer execution ran.
The local pending selection has no matching remote consumer CI; native acceptance
remains under [#44](https://github.com/dragginzgame/ic-query/issues/44).

## Prior 0.54.13 entrypoint implementation review

The canonical exporter selects committed Shared revision
`8e8cc21265272e0234326c53a8c7bf9e7d8d77df` (recorded version 0.6.6),
implementing the upcoming entrypoint-selection contract. The reviewed 79 roots
include Query's directly invoked/copied helpers, selected fixtures and offline
guides; seven internal companions are derived. The complete 86-file payload
roster is retained. This repository has one shared ownership manifest; local
adapters and the host matrix remain consumer-owned.

Canonical preview leaves consumer inputs unchanged; apply preserves the incoming
lock, manifests, local policy and real index. Snapshot integrity, ShellCheck,
syntax checks and local documentation links pass. Disposable consumers exercise
recorded-root reuse, non-mutating preview, source-proven exact retirement,
missing-selection refusal and generic invalid roots with modern Bash and
genuine Bash 3.2.57. No actual Query payload is retired.

Query's actual formatting hook and prepared-tool reuse pass on the modern profile.
The portable fixture formats successfully, then its final offline check refuses
a prepared receipt after this machine's native package inventory changes.
Explicit common setup subsequently refuses an occupied new shared installation
slot; old payloads and that lock remain intact. Retained evidence, including the
native inventory comparison and failed fixture, is recorded through
`/tmp/ic-query-shared070-adoption-path`. That review left current-environment
portable hook qualification outstanding; the 0.8.0 checks above resolve it.

[Exact Shared CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38227589866)
has passed lint/security, with Linux regression running and both native jobs
queued at inspection. Full local CI, release gates and native consumer execution
were not run; this local adoption does not close native acceptance under
[#44](https://github.com/dragginzgame/ic-query/issues/44) or
[Shared #131](https://github.com/dragginzgame/shared-tooling/issues/131).

## Pending 0.54.13 Host 0.14.5 qualification

The incoming lock selects the three published Host 0.14.5 packages from
`bb8f1e71f3d4599618d0f4084d27fa637408f1fb`. Their unyanked registry entries,
archive checksums and VCS identities match. Library sources are byte-identical
to 0.14.4; the upstream delta changes only release metadata. Explicit locked
fetching and complete locked/offline metadata pass, preserving the incoming
lock, manifests and real index. Library tests and unchanged tooling fixtures
were not repeated for this metadata-only update. Evidence is retained under
`/tmp/ic-query-05413-host0145-path`.

No full local CI, release gate or native execution ran for this selection.
Matching consumer native acceptance remains required; upstream CI does not
qualify the uncommitted consumer lock.

## Delivered 0.54.12 Shared 0.6.6 and Host 0.14.4 qualification

Delivered Query `b7402d412aff8318c47d4e14e90026f539f23af8` selects the
snapshot and dependency records reviewed below. Its
[exact-source CI](https://github.com/dragginzgame/ic-query/actions/runs/38225862489)
is unfinished at inspection: Linux MSRV and canister integration pass, Linux
checks are running and native jobs are queued. The preceding
[0.54.11 run](https://github.com/dragginzgame/ic-query/actions/runs/38216728698)
passed ARM checks and canister integration, but cancelled both Intel jobs.
Consumer acceptance remains open under
[#44](https://github.com/dragginzgame/ic-query/issues/44),
[#45](https://github.com/dragginzgame/ic-query/issues/45) and
[#46](https://github.com/dragginzgame/ic-query/issues/46).

The follow-up canonically selects the same 86 files from Shared 0.6.6
`a15eb9d406d287697df9bc8fa96db8cda6721a71`. Because the reviewed 0.6.5
batch was still uncommitted, the canonical exporter ran in an isolated consumer;
the verified existing selection was reconciled only to that export's known shared
bytes. Existing local adapters, fixtures, notes and the real index are preserved.

Explicit `make install-tools` prepares cargo-sort-derives 0.14.1; complete offline
tool admission passes. All three previous Cargo-tool payload hashes remain
unchanged. Actual Query formatting/hook and current physical tool reuse pass on
modern Bash/Make and genuine Bash 3.2.57/GNU Make 3.81 on Linux. A separate stdin
probe preserves literal `$crate` in direct/conditional derives, then compiles and
runs those exported macros from another crate. This is a sorter/downstream probe,
not staged derive-sorter qualification: Query keeps cargo-sort/rustfmt and has no
derive-sorter workaround markers to remove. Its Rust 1.99 development toolchain
satisfies the formatter's 1.88 build floor; the library MSRV remains 1.91.

The three published Host 0.14.4 archives match their selected checksums and identify
`f93cd8854d500616a73eda713d215a20ff0617c7`. Their library source matches 0.14.3.
The lock change is restricted to those three package records; other package
versions and dependency edges remain intact. Explicit locked fetching and complete
offline metadata pass. Library tests and unchanged CI/release simulations were
not repeated for this tooling-only update; the earlier 0.6.5 proofs below remain
distinct. Full local CI/release gates and native consumer qualification remain
outstanding. `/tmp/ic-query-shared066-adoption-path` records the pinned source,
isolated export, setup, retained old payload hashes, actual hook logs, macro probe,
scoped lock diff, archive review and final static checks. No real hook activation,
source staging, commit, version bump, tag, push or CI dispatch ran.

## Pending 0.54.12 Shared 0.6.5 qualification

Based on released Query `50cb2c951169c3a6cf837457bc0e5a1067179388`, the
canonical exporter selects 86 files from Shared 0.6.5
`42f2091768b5f71c98a684d3dab563f408ca2dd9`, adding the reviewed formatting
hook. Its original-tool receipt context preserves Query's stronger prerequisite.
The new consumer fixture runs actual manifest/Rust formatting, verifies identical
physical tool reuse in its explicitly prepared clone, preserves unrelated edits
and rejects partial staging. It passes on modern Bash/Make and genuine
Bash 3.2.57/GNU Make 3.81 on Linux, including the current caller manifests/lock.
The real hook configuration and index are unchanged; activation is explicit.

Ordinary CI checkpoints eight prepared inputs. The outer release logger captures
the root manifest/lock before inner CI extends that checkpoint. Consumer CI-script
and simulated release checks pass on both shell/Make profiles, including changed
lock/member/pin/toolchain refusal and inherited-checkpoint preservation. Shared
receipt and validation-runner fixtures also pass on both profiles. Complete
offline tool admission, actual formatting, ShellCheck, notes, local links and
snapshot integrity are checked without running full local CI or a release gate.

During this pass the incoming lock changed from Host 0.14.2 to 0.14.3. The initial
preservation checks flagged the change; their failed logs are retained rather
than relabelled. The incoming selection is preserved. Its three published
archives identify `543b137458feba0f6dfd3e458b680fb2be3d9e6d`, match the
selected checksums and have library source identical to 0.14.2. Locked fetching
and complete offline metadata pass; library tests were not repeated for this
tooling-only producer release. Other manifests and the real index remain unchanged.
`/tmp/ic-query-shared065-adoption-path` identifies the immutable source, canonical
export, initial observations, current lock identity and focused validation logs.

[Exact Shared CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38218219902)
passes Linux regression and lint/security, with both native jobs queued at
inspection. [Exact Host CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38219424683)
is unfinished. Linux/Bash 3.2 results do not qualify native macOS or this pending
consumer's remote CI. Consumer acceptance remains under
[#44](https://github.com/dragginzgame/ic-query/issues/44).

## Pending 0.54.11 Shared 0.6.4 qualification

Based on released Query `4a18ccfe04198482352c6effa8ff17abc16e690b`, the
canonical exporter selects 85 files from Shared 0.6.4
`6fc8eb9a94f83fb0b9d0a26377d4232508cdbe2d`. The root Makefile uses the
combined consumer entrypoint; all isolated Make exports include it. Cargo
installer/store helpers retain their previous bytes. Offline tool admission,
actual formatting, consumer CI-script and simulated release checks pass on
Linux. The fixtures also pass with genuine Bash 3.2.57/GNU Make 3.81. These
portable checks do not qualify native macOS. The incoming lockfile and real
index are preserved; all 58 cache-file tests pass with its current serde_json
1.0.154 and syn 3.0.8 selection under locked/offline validation. The initial
pass selected Host 0.14.1; the follow-up selects published 0.14.2 below.

The follow-up admits the published artifacts, filesystem and process 0.14.2
archives from `fed8724e79862d8273d01823619fe648c981c019`. Registry checksums
and archive VCS identities match; their library source trees match 0.14.1.
Only the three Host package records change in the selected lock, preserving
other versions and dependency edges. Locked/offline metadata, all 58 cache-file
tests and all 27 Governance smoke fixtures pass. Manifests and the real index
remain unchanged. `/tmp/ic-query-05411-host0142-path` identifies this separate
registry/archive review, scoped lock diff and validation evidence.
[Exact Host CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38159733484)
passes Linux native checks and MSRV with macOS jobs queued; it does not qualify
this pending consumer source or replace native consumer acceptance.

Released-source [Intel macOS CI](https://github.com/dragginzgame/ic-query/actions/runs/38152148553/job/114507631316)
stops in the completion fixture before its injected nounset fault: the dependency
owner returns setup status 2 instead of the expected failure status 1. That
fixture assumes executable `/bin/true` inputs. The pending repair uses its own
executable, preserving completion and failure-evidence assertions. Focused
completion and CI-script checks pass on both Linux shell/Make profiles; matching
native execution remains required under [#45](https://github.com/dragginzgame/ic-query/issues/45).

The shared formatting hook remains unselected. A disposable, explicitly prepared
Query clone passes actual formatting but the canonical hook cannot admit tool
receipts in its exported index tree. Refusal preserves the staged payload,
selected source and unrelated evidence. The owning
[Shared #130 feedback](https://github.com/dragginzgame/shared-tooling/issues/130#issuecomment-6111442463)
requests shared read-only receipt admission without weakening Query's prerequisite.
`/tmp/ic-query-shared064-adoption-path` identifies the pinned source, canonical
export, native failure log, reproduction and focused validation logs. Full local
CI and release gates were not run; native adoption acceptance remains outstanding
under [#44](https://github.com/dragginzgame/ic-query/issues/44).

## Pending 0.54.9 Shared 0.6.3 and Host 0.14 qualification

Based on released Query `1f6ab7eff5d1d527749412d42f0e7b9ad4b9da98`,
canonical refresh selects the same 82 files from Shared 0.6.3
`4fd979d13c32adb3cb2f51f213ebfc4f98339b3d`. The engineering baseline stays
separate. Incoming Host 0.14.0 manifests/lock and the real index are preserved;
published archives match their lock checksums and identify
`4723befe8d65c760c00e05c42652759d691fbd5e`.

`/tmp/ic-query-upstream0549-path` identifies the pinned source, canonical
preview/refresh and retained logs. Explicit locked fetching and offline metadata
pass. All 27 Governance smoke fixtures and 58 cache-file tests pass under
locked/offline validation (`host-smoke-tests.log`, `host-cache-tests.log`). The
initial required-status compilation failure is retained in
`host-command-initial.log`; the fixture assertion now uses the current Host API.
There was no live canister execution. Shared runner and local adapter simulations
pass on modern Bash/Make and genuine Bash 3.2.57/Make 3.81
(`consumer-{modern,portable}.log`). Same-candidate restart preserves the original
pointer/attempt, rejects conflicting selection or dirty source, retains failed
revalidation and binds successful fresh validation to the new source.
No real release plan or receipt was modified. Complete offline tool admission,
Rustfmt, ShellCheck, both Bash syntax checks, snapshot integrity, notes, local
links, whitespace and input preservation pass (`tools-offline.log`,
`static-final.log`).

[Exact Shared CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38145666246)
passes Linux regression and lint/security, with native jobs unfinished.
[Exact Host CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38146341421)
passes Linux native and MSRV, with both macOS jobs queued.
[Released Query CI](https://github.com/dragginzgame/ic-query/actions/runs/38145852854)
passes Linux canister smoke/MSRV, with Linux checks running and native jobs queued
at inspection. They do not qualify this uncommitted batch. Native consumer
acceptance remains required under [#44](https://github.com/dragginzgame/ic-query/issues/44),
[#45](https://github.com/dragginzgame/ic-query/issues/45) and
[#46](https://github.com/dragginzgame/ic-query/issues/46).
No full local CI, release gate, version bump, staging, commit, tag, push or
workflow dispatch ran.

## Pending 0.54.8 Shared 0.6.2 qualification

The same pending slice now canonically selects committed Shared 0.6.2
`7f1ca16b0dfa98021d2671017ecb7de767409b0e`, retaining 82 files. The follow-up
changes only the release runner, its simulation fixture, release recovery guide
and local setup guide. The four Cargo payloads qualified below remain
byte-identical; their setup/reuse evidence still applies.
`/tmp/ic-query-shared062-adoption-path` identifies the clean pinned source,
preview, refresh and follow-up evidence. Canonical runner simulations pass on
modern Bash/Make and genuine Bash 3.2.57/Make 3.81, covering preparation-source
conflicts, retained evidence and fresh validation after simulated retirement
(`runner-{modern,portable}.log`). No real release plan is retired or rewritten.

The maintainer's incoming Host 0.13.5 lock replaces the previously qualified
0.13.4 selection. Explicit locked fetching, offline metadata and all 58
cache-file tests pass (`host-fetch.log`, `metadata.json`, `host-cache-tests.log`).
Published archive checksums match the lock and identify release
`b32021c67515f0a0dcf35f1dd0b2029b9599ac75` (`host-published-source.log`).
The released Host library sources are unchanged from 0.13.4. The current
manifest, incoming lock and real index remain preserved.
Complete offline tool admission also passes (`tools-offline.log`). ShellCheck
with sourced companions, both Bash syntax checks, snapshot integrity,
current-version notes, documentation links and whitespace checks pass
(`static-final.log`); the initial lint observation without companion following
is retained separately (`static-initial.log`).

The [exact Shared 0.6.2 run](https://github.com/dragginzgame/shared-tooling/actions/runs/38143010876)
passes lint/security and Linux portable regression; both macOS jobs remain
unfinished at inspection.
The [exact Host 0.13.5 run](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38142886238)
and [released Query 0.54.7 run](https://github.com/dragginzgame/ic-query/actions/runs/38142141323)
are recorded in `host-ci.json` and `query-ci.json`. Host's Linux native and
MSRV jobs pass, with both macOS jobs running. Query's Linux checks, canister smoke
and MSRV, plus Intel macOS canister smoke, pass; its other macOS jobs remain queued. Those results
do not qualify this uncommitted source or establish native acceptance. Issues
[#44](https://github.com/dragginzgame/ic-query/issues/44) and
[#45](https://github.com/dragginzgame/ic-query/issues/45) remain open for native
consumer qualification. No full local CI, release gate or release effect ran.

### Earlier 0.6.1 qualification in this slice

Based on released Query `4f4f8e97ea60bce180efb4fb5bec6ed2618de8e0`,
canonical refresh selects the same 82 files from committed Shared 0.6.1
`731909525038c58d6f80b498e80f8630b11c685d`. Only the Cargo build companion,
installer, store and Cargo fixture payloads change. The separate engineering
baseline, release runner, tool pins and Query product contracts remain unchanged.
`/tmp/ic-query-shared061-adoption-path` identifies the clean pinned source,
reviewed preview, refresh and retained evidence.

Canonical Cargo fixtures pass on modern Bash/Make and genuine Bash 3.2.57/Make
3.81, including padded counts and conditional lock, receipt, reference and
activation failures (`rust-{modern,portable}.log`). Consumer dispatch, execution
and retention fixtures pass on both profiles (`consumer-{modern,portable}.log`).
Real complete offline setup passes (`real-install-offline.log`); a second
temporary checkout selects identical common/Audit/Machete payloads, and repeated
Query setup preserves generation metadata, receipts and build logs
(`real-reuse.log`, `reuse-generations-{before,after}`). Complete offline admission
passes on both profiles (`real-tools-{modern,portable}.log`), as does actual
Audit/Machete dependency execution with offline metadata (`real-dependency.log`).
Previous physical and global tools retain their hashes.

The incoming Host 0.13.4 lock selection is preserved. Explicit locked fetching,
offline workspace metadata and all 58 cache-file tests pass (`host-fetch.log`,
`metadata.json`, `host-cache-tests.log`). Published archive checksums match the
lock, and packaged VCS metadata identifies release
`46483b96c21419aa41ee2cbcdfa2e8efbbfdab21` (`host-published-source.log`).
The reviewed new path-based private sidecar API does not supply Query's
descriptor-relative confinement and is not substituted into managed cache IO.
No package manifest, selected lock or real index changed during qualification.
ShellCheck, both Bash syntax checks, snapshot integrity, current-version notes,
local links, whitespace and preservation checks pass (`static-final.log`).

The [exact Shared source CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38141614488)
passes Linux portable regression and lint/security; both macOS jobs were cancelled
by a successor. [Released Query 0.54.7 CI](https://github.com/dragginzgame/ic-query/actions/runs/38142141323)
remains queued at inspection. Neither establishes native acceptance for this
uncommitted consumer batch. Delivery and native setup/reuse/retention qualification
remain under [#44](https://github.com/dragginzgame/ic-query/issues/44) and
[Shared #124](https://github.com/dragginzgame/shared-tooling/issues/124).
No full workspace CI, native macOS execution, live canister smoke, release gate,
version bump, staging, commit, tag, push or workflow dispatch ran for this batch.

## Pending 0.54.7 Shared 0.6.0 qualification

Based on released Query 0.54.6 `40bb75ae5bbc8c08f2d29cfc03ae9874e70eef3c`,
the canonical refresh selects 82 files from committed Shared Tooling 0.6.0
`75180980a0c4e4252d8a915aaecfaefd92bf9a4e`, explicitly adding the Cargo
build companion. The engineering baseline remains separately pinned. This
contributor-tooling adoption preserves published APIs, runtime/CLI, schema-1
contracts and the consumer build floor, and uses the next patch's notes.

`/tmp/ic-query-shared060-adoption-path` identifies the clean pinned source,
reviewed preview, canonical refresh and retained logs. Real Linux complete
setup passes (`real-install-initial.log`), followed by complete offline admission
on modern Bash/Make and genuine Bash 3.2.57/Make 3.81
(`real-tools-check.log`, `real-tools-portable.log`). Audit 0.22.2 and Machete
0.9.2 use the same selected Cargo owner as the common tools. A second temporary
checkout selects the identical physical common/Audit/Machete payloads; repeated
complete Query setup passes offline without force-overwrite advice. The selected
generations' metadata, receipts and build logs remain unchanged through reuse
(`real-reuse.log`, `reuse-generations-{before,after}`). Original local and global
executables retain their hashes. Builds use the shared standard environment,
the prepared Rust 1.99.0 compiler and managed source cache; these results do not
establish reuse across arbitrary application build configurations.

Canonical installer fixtures pass on both shell/Make profiles
(`rust-{modern,portable}.log`), as do consumer Make dispatch, explicit executable
execution, formatting admission and actual workflow-retention fixtures
(`consumer-second.log`, `consumer-portable.log`, `workflow-initial.log`).
The collector consumes the canonical selector's safe partial output, archives
original local build/receipt bytes before reporting incomplete collection, and
retains only referenced shared Cargo generations without following unknown links.
Both native failure upload paths retain this evidence for 30 days. The initial
completion-fixture failure remains in `consumer-initial.log`; the current fixture
supplies the dependency owner's required selected executable arguments.

Snapshot integrity, ShellCheck, both Bash syntax checks, actionlint,
current-version changelog admission, local documentation links and whitespace
checks pass (`static.log`, `static-final.log`). Manifest and real Git index hashes
remain unchanged. A concurrent lockfile update selected Host 0.13.2; its observed
hash remains preserved in `concurrent-host-lock.sha256`. This tooling task did
not select or run product tests for that dependency update.

At inspection, [the exact Shared source CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38136593268)
and [released Query base CI](https://github.com/dragginzgame/ic-query/actions/runs/38135439111)
are queued. Neither establishes complete or native acceptance for this pending
consumer source. Native macOS setup/reuse, consumer CI and hosted retention
qualification remain required under [#44](https://github.com/dragginzgame/ic-query/issues/44)
and the owning [Shared #123](https://github.com/dragginzgame/shared-tooling/issues/123).
No full workspace CI, native macOS execution, live canister smoke, release gate,
package version change, staging, commit, tag, push or CI dispatch ran for this
adoption.

The subsequent release dependency failure is repaired on source commit
`4baa9801` within the same pending 0.54.7 slice. Direct Machete execution now
passes only `--with-metadata` and clears its inherited Cargo launcher marker.
The separate Audit/Machete fixtures verify tool-specific arguments, original
failure status and retained advisory-database identity. A plain external call
preserves Machete's status on Bash 3.2; initial failing subshell attempts remain
retained. Real `make dependency-check` passes on Linux modern tools and genuine
Bash 3.2.57/Make 3.81, including an inherited real Cargo path. Consumer fixtures
pass on both profiles. Evidence: `/tmp/ic-query-machete-0547-fix/`, especially
`real-dependency-check.log`, `real-dependency-cargo-path.log`,
`consumer-{modern,portable}-direct.log` and `static-final.log`. The deliberately
invalid Cargo-marker checks retain their Audit registry warning; the final real
Cargo-path check has no such warning. The incoming Host 0.13.3 lock, manifest,
shared snapshot and real index retain their captured hashes. No remote workflow
run matches that local source at inspection; native macOS and full-gate acceptance
remain outstanding. Original release-failure logs are preserved.

## Pending 0.54.6 cleanup qualification

Released 0.54.5's explicitly authorized tag `workflow_dispatch`
[run](https://github.com/dragginzgame/ic-query/actions/runs/38123499895)
now passes all nine checks/MSRV/canister jobs on Linux and both macOS
architectures at `9cda0af83d873433a49cf813ae1336cf3aeb5575`.
This completes [#34](https://github.com/dragginzgame/ic-query/issues/34)'s manual
tag-route acceptance. It does not qualify the pending source below.

The pending consumer cleanup is based on released 0.54.5 commit
`9cda0af83d873433a49cf813ae1336cf3aeb5575`. Host 0.13.1 is selected through
a narrow lockfile update; the immutable 81-file Shared Tooling 0.5.0 snapshot
is selected at `a306180ce534227fcb98b882fba9a526d5d747e0`. Cache refresh-lock
completion is consolidated without changing public APIs or schema-1 evidence;
the development smoke supervisor preserves the original typed Host execution
error without double boxing. Its TERM grace and bounded reaping remain intact.

After explicit `cargo fetch --locked`, all 17 cache-lock tests and 27 Governance
smoke-runner tests pass locked/offline on Linux with Host 0.13.1. Clippy with
warnings denied passes for the host-feature library/tests and the smoke
example/tests. Evidence: `/tmp/ic-query-0546-host0131-cache-tests.log`,
`/tmp/ic-query-0546-host0131-smoke-tests.log`,
`/tmp/ic-query-0546-host0131-cache-clippy.log`, and
`/tmp/ic-query-0546-host0131-smoke-clippy.log`. The preceding Host 0.13.0
qualification logs remain intact. The original typed-error regression failure
is retained in `/tmp/ic-query-0546-smoke-tests-before-unboxing.log`.

All three published packages are unyanked in Cargo's registry observations;
archive checksums match the lock, and packaged VCS metadata identifies release
`95fa4828996a6116e84b5ed01909b6e8590a355d`. Library sources match the reviewed
0.13.0 owner sources. Evidence:
`/tmp/ic-query-0546-host0131-registry-observations.log`.

Consumer temporary paths are now physically anchored across directory changes.
The actual setup/handler reproduction fails before repair and succeeds after
repair; original failed evidence survives. Its directory is recorded in
`/tmp/ic-query-0546-relative-path-review`. The nine-owner completion checker and
focused CI-script fixtures pass on modern Bash/Make and genuine Bash 3.2.57/Make
3.81. Evidence: `/tmp/ic-query-0546-relative-{modern,portable}.log` and
`/tmp/ic-query-0546-ci-scripts-{modern,portable}.log`.
[#45](https://github.com/dragginzgame/ic-query/issues/45) retains delivery and
native consumer qualification.

Targeted formatting, local documentation links, current-version changelog
admission, snapshot integrity, ShellCheck, both Bash syntax checks and whitespace
checks pass. Manifest, real Git index and snapshot hashes remain unchanged.
The final lock changes only the three Host versions/checksums and remains
unchanged through validation. Evidence: `/tmp/ic-query-0546-continuation-static.log`,
`/tmp/ic-query-0546-continuation-preserved`, and
`/tmp/ic-query-0546-host0131-lock-selected`. Initial cleanup evidence in
`/tmp/ic-query-0546-cleanup-static.log` and
`/tmp/ic-query-0546-cleanup-inputs-before` remains intact.
No full workspace CI, native macOS execution, live canister smoke, release gate,
version bump, commit, tag or push ran for this cleanup. Released 0.54.5's remote
CI does not qualify these uncommitted changes.

Contributor-tooling adoption remains in 0.54.6 under the reviewed package-impact
guidance and explicit maintainer selection. Public APIs, product CLI/runtime,
schema-1 contracts and consumer build requirements remain unchanged. The canonical
export adds the evidence archiver explicitly; the engineering baseline stays pinned.
Query removes its duplicate cargo-sort setup and formatter PATH override, admits
the common slot selection before formatting, and retains physical Rust build/receipt
evidence in CI without following symlinks. Old tools and failed evidence remain intact.

`/tmp/ic-query-shared041-adoption-path` records the exact-source checkout and logs.
Real Linux common setup and offline admission pass (`real-install.log`,
`real-check.log`); reuse passes with Cargo replaced by a refusing recorder and no
Cargo dispatch (`real-reuse.log`). Prior tool hashes remain unchanged.
Canonical common/selected installer fixtures, consumer CI-script fixtures and
simulated preparation/cache dispatch pass on modern Bash/Make and genuine Bash
3.2.57/Make 3.81 (`rust-{modern,portable}.log`,
`consumer-{modern,portable}.log`, `cache-{modern,portable}.log`). The consumer
workflow collector preserves build/receipt bytes and archives symbolic links
without dereferencing them; redirected roots do not select outside evidence.
Real formatting admission, snapshot integrity, local links, ShellCheck, both
Bash syntax checks, actionlint and whitespace checks pass (`static.log`).
Manifest, selected Host lock and real index hashes match `preserved-before.sha256`.
Native macOS installation/reuse and consumer CI remain required under
[#44](https://github.com/dragginzgame/ic-query/issues/44); upstream CI and Linux
fixtures do not qualify this uncommitted consumer source.

The preceding canonical 0.4.2 refresh changed only storage-ownership guidance and source
annotation from the tested 0.4.1 selection; all 77 other selected payloads,
including installer, Make helpers, fixtures and archiver, have identical hashes.
Evidence: `preview-042.log`, `refresh-042.log`, and `helpers-{041,042}` in that
same retained adoption directory. At that revision storage remained at physical
checkout-local `.tools` paths; 0.5.0 now replaces host/IC storage admission.
The preceding exact-source [0.4.1 Linux CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38130259856)
passes real setup, portable regressions and formatting-hook qualification, but
fails a later native-retention qualification because its Cargo substitute still
expects the retired invocation and evidence path. Its qualification marker is
empty; this is not evidence that a qualified payload was lost. The source owner
has reports in [Shared #118](https://github.com/dragginzgame/shared-tooling/issues/118).
That fixture is outside Query's snapshot. Both macOS jobs were cancelled by the
successor; neither establishes native acceptance.

The canonical 0.5.0 refresh selects 81 files, explicitly adding the shared store
companion and evidence selector. `/tmp/ic-query-shared050-adoption-path` identifies
the clean pinned source, reviewed preview and refresh logs. Real Linux host/IC
setup and offline checks pass (`real-setup-check.log`). A second temporary
checkout selects the same physical bundles with a refusing curl recorder and
zero download dispatch (`real-reuse.log`); no physical executable payload is
created there. Prior tool bytes, Cargo metadata/lock and index retain their hashes.
Store and installer fixtures pass under modern Bash and genuine Bash 3.2.57/Make
3.81 (`host-{modern,portable}.log`, `ic-{modern,portable}.log`), as do consumer
CI-script and actual collector fixtures (`consumer-{modern,portable}.log`,
`workflow-{modern,portable}.log`). The collector retains admitted shared bundles,
local references and Rust builds/receipts, excludes unrelated store entries and
unknown-link targets, and archives safe local evidence before a rejected store
aborts shared collection ([Shared #121](https://github.com/dragginzgame/shared-tooling/issues/121)).
Both checks and canister jobs upload these archives/diagnostics for 30 days.

Real portable host/IC/formatter offline admission passes in
`offline-portable-recheck.log`. The initial IC refusal without detailed diagnostics
is retained in `offline-portable.log`; a direct traced check and subsequent
complete admission pass without reinstalling or changing selections. Its original
cause was not established. Static checks, snapshot integrity and local links pass
in `static.log`. The [exact-source upstream run](https://github.com/dragginzgame/shared-tooling/actions/runs/38132790945)
passes Linux setup/reuse, native failure/archive round-trip qualification and
lint/security; both macOS jobs remain queued at inspection. The former CI-only
Rust retention fixture is aligned with the current slot invocation in this source.
No full consumer CI or native macOS execution ran locally. Matching native
consumer qualification remains required after delivery; package version, real
index, release refs and retained installations are unchanged by this adoption.

The subsequent local Audit/Machete setup correction is based on the maintainer's
source commit `03f8c7a5`, with the same package version and 0.54.6 draft.
`/tmp/ic-query-idempotent-audit-path` identifies retained evidence. Two complete
real `make install-tools` runs pass offline with a Cargo recorder refusing any
installation: zero Cargo install dispatch and no force-overwrite advice
(`real-{first,second}.log`, `real-reuse.log`). Existing Audit/Machete binaries,
Cargo metadata/lock, immutable snapshot and real index retain their hashes.
Consumer aggregate/extension fixtures pass on modern Bash/Make and genuine Bash
3.2.57/Make 3.81, including missing tools, exact reuse, mismatched versions,
failed matching probes and both installation failure boundaries
(`consumer-{modern,portable}.log`). ShellCheck, both syntax profiles, changelog
admission, local links and whitespace checks pass (`static.log`). Shared
ownership is requested in [#123](https://github.com/dragginzgame/shared-tooling/issues/123).
No full CI, native macOS execution, package version edit, staging or release
effect ran for this consumer correction.

## Pending 0.54.5 Shared 0.3.11 qualification

Released 0.54.5 `9cda0af83d873433a49cf813ae1336cf3aeb5575` now passes
all nine [exact-source branch CI jobs](https://github.com/dragginzgame/ic-query/actions/runs/38122851819):
complete checks, MSRV and live canister smoke on Linux, Intel macOS and Apple
Silicon macOS. This accepts the delivered Shared 0.3.11 adoption and consumer
assertion repair, closing #42 and #43. The earlier pending evidence below records
the original adoption attempt; it does not qualify pending 0.54.6 source.

Canonical refresh now selects Shared 0.3.11
`db143ae9ae140eb66826c413360723d23c078f66`. It corrects the source-version
annotation from the preceding export while all 78 selected file bytes remain
identical to the prior qualified working-tree snapshot. The producer-only
validation-runner fixture separates each diagnostic family beyond neighboring
context windows so one error cannot conceal a missing classifier rule.
That unmodified exact-source fixture passes on Linux Bash 5/Make 4.3 and genuine
Bash 3.2.57/Make 3.81. Prior selected-payload qualification below remains applicable;
the fixture is still outside Query's selection.

Evidence: `/tmp/ic-query-shared0311-review-path` identifies the clean exact-source
checkout; `/tmp/ic-query-0545-shared0311-{refresh,payloads,validation-modern,validation-portable,static}.log`
records canonical export, unchanged payloads, focused diagnostics and static
checks. Snapshot integrity, selected fixture ShellCheck/Bash 3.2 syntax,
current-version changelog admission, local links and whitespace checks pass.
Package manifest, incoming lock, real index and consumer-owned fixture hashes
match `/tmp/ic-query-0545-shared0311-inputs-before`.

The adopted source's [CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38121526696)
is in progress at inspection. The preceding source's native jobs were cancelled
by the newer run and do not establish native acceptance. Released Query 0.54.4's
CI remains green; pending-source native macOS acceptance and delivery remain
required. No Rust compilation, full local CI, release/version bump, commit, tag
or push ran for this refresh.

## Earlier pending 0.54.5 Shared snapshot qualification

On released Query 0.54.4 `6fa935e1a0e3fe6cc1dbcee8c1b5f04777b196f1`,
canonical refresh selects committed Shared revision
`37af29b2f8fb4532744b600c5f85ce716a8b57cf`, retaining the same 78 files
and separate engineering baseline. The source includes its 0.3.10 changelog
fixes but committed `VERSION` remains 0.3.9; the exporter preserves that value
([Shared #114](https://github.com/dragginzgame/shared-tooling/issues/114)).
Source and refresh evidence: `/tmp/ic-query-shared0310-review.30MjBL/source`
and `/tmp/ic-query-0545-shared0310-refresh.log`.

Focused checks pass on Linux Bash 5/Make 4.3 and genuine Bash 3.2.57/Make 3.81:
selected Rust-tool fixtures (substitute Cargo), exact-source validation-runner,
release-command and host-tool fixtures, complete consumer CI-script fixtures,
and actual parallel offline `make -j4 tools-check`. Evidence:
`/tmp/ic-query-0545-shared-{rust,validation,release,host,consumer,tools}-{modern,portable}.log`.
The actual consumer release-command checker also passes with both caller-relative
and symlink-alias `TMPDIR`, with complete successful cleanup under the selected
physical parent. Dispatch uses a substitute runner and performs no release
effects. Evidence: `/tmp/ic-query-0545-shared-consumer-release-{modern,portable}-{relative-temp,temp-alias}.log`.

Selected ShellCheck, Bash 3.2 syntax, snapshot integrity, current-version
changelog admission, local links and whitespace checks pass. Package manifest,
incoming Host 0.12.9 lock selection and real index hashes remain identical to
`/tmp/ic-query-0545-shared-adoption-inputs-before`. These fixture checks do not
qualify the incoming dependency update. No Rust compilation, full local CI,
release/version bump or Git delivery effect ran.

Released 0.54.4's [CI](https://github.com/dragginzgame/ic-query/actions/runs/38066714998)
now passes all nine Linux, Intel macOS and Apple Silicon macOS jobs. That source
does not qualify this uncommitted batch. The exact adopted Shared source's
[CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38120869060)
passes Linux portable regression and lint/security at inspection; both native
macOS jobs remain in progress. Native consumer delivery and acceptance remain
required. Query retains its existing formatting policy and does not select or
activate the shared hook or derive-formatting hook fixture.

## Pending 0.54.5 completion-assertion qualification

On released 0.54.4 `6fa935e1a0e3fe6cc1dbcee8c1b5f04777b196f1`, the
consumer completion checker now explicitly fails retained-payload and cleanup
comparisons. Its existing child-status and completion/retention contracts remain.
Behavioral regressions use disposable copies of the actual checker with substitute
payload reads and skipped successful cleanup; all original payload bytes stay
unchanged. Only owner setup/EXIT-handler prefixes execute before the refusal.
[Query #42](https://github.com/dragginzgame/ic-query/issues/42).

The new maintained regression catches the released checker accepting wrong
payload observations on genuine Bash 3.2.57. With only the payload assertion
fixed in a disposable copy, it separately catches failed cleanup being accepted.
Evidence: `/tmp/ic-query-0545-before-portable.log`,
`/tmp/ic-query-0545-before-cleanup-portable.log` and
`/tmp/ic-query-0545-cleanup-before/`; failed attempts remain retained.
The repaired checker and complete consumer CI-script fixtures pass on Linux
Bash 5/Make 4.3 and genuine Bash 3.2.57/Make 3.81, including both refusal cases,
54 normal owner/status cases and the eight documentation-parser unit tests.
Evidence: `/tmp/ic-query-0545-ci-{modern,portable}.log`.

The subsequent issue follow-through extends explicit assertion failure across
consumer CI-script, release/recovery, cache-preparation, metadata and publication
fixtures, including their generated stubs. Intentional command predicates retain
their status contracts. A further maintained regression runs the actual release
fixture setup and completion assertions with a contradictory plan-phase read;
the real simulated plan stays complete. The old fixture accepts it on Bash 3.2;
the repaired fixture refuses with status 1 and retains prior artifacts.
Before-repair evidence: `/tmp/ic-query-0545-release-assertions-before.sh`,
`/tmp/ic-query-0545-release-assertions-before-portable.log` and the failed parent
`/tmp/ic-query-ci-scripts.zRrYeh`. Complete affected CI-script, release/recovery
(including cache and real-Git metadata) and publication suites pass on both
Bash/Make profiles. Final evidence:
`/tmp/ic-query-0545-assertions-{ci-scripts,release-guards,publish-guards}-{modern,portable}.log`.

ShellCheck, Bash 3.2 syntax, snapshot integrity, current-version changelog
admission, local links and whitespace checks pass. The initial checker batch
preserved its recorded inputs in `/tmp/ic-query-0545-inputs-before`. This
follow-through preserves the later incoming lockfile update; package manifest,
incoming lockfile, real Git index and Shared snapshot hashes match
`/tmp/ic-query-0545-assertions-inputs-before`. Fixture-only checks do not qualify
that dependency update.
No full local CI, Rust compilation, release/version bump or Git delivery effect
ran; delivery and native macOS acceptance remain outstanding. Released 0.54.4's
[CI](https://github.com/dragginzgame/ic-query/actions/runs/38066714998) passes its
three Linux jobs and Apple Silicon canister job, with five macOS jobs queued at
the latest inspection; that source does not qualify the uncommitted 0.54.5 repair.

## Pending 0.54.4 release-guard qualification

The consumer release adapter now admits selected-increment and both preflight
HEAD observations before comparing successful output. Against released 0.54.3
`fcd9214970ad38332f3d653db4591e4fb629b189`, matching-output fault injection
reproduces all four paths returning success after an observation exits 23. The
repair preserves status 23 and diagnostics, source and prior artifacts, and
refuses subsequent preparation, validation or release effects. Targeted cases
and complete simulated release/recovery fixtures pass on Linux Bash 5/Make 4.3
and genuine Bash 3.2.57/Make 3.81. Evidence:
`/tmp/ic-query-0544-before-*.log`, `/tmp/ic-query-0544-after-*.log` and
`/tmp/ic-query-0544-release-{modern,portable}.log`.
[Query #41](https://github.com/dragginzgame/ic-query/issues/41).

The same pending slice adopts committed Shared Tooling 0.3.7
`34e5ad7aac3599306c9572bb547f2239d09df1a3` canonically with 78 selected files.
The new lockfile-selected Cargo-tool fixtures pass on both Bash/Make profiles
using real yq/jq and substitute Cargo, including matching-output parser failures,
selection changes, retained candidates and offline reuse, plus absent-host and
non-executable-candidate refusal. The shared release simulation passes with
explicit assertion failures on both profiles. Actual parallel offline common
and Query tool checks pass on both profiles. Evidence:
`/tmp/ic-query-0544-shared037-{rust,release,tools}-{modern,portable}.log`.
The incoming workflow uses the reviewed newest-run concurrency policy while
retaining all nine required host jobs; event/host fixtures and actionlint pass.
Cancelled jobs leave their exact source unqualified on unfinished hosts.
The pre-commit hook and its
adoption checker remain unselected; no hook is activated.

The incoming Host artifacts/filesystem/process 0.12.5 selection is preserved.
Published registry checksums match the lock and packaged VCS identities match
released source `85f051c60b2a6f37717c4274b1e31caf5b9d3453`. After explicit locked
fetching, all 26 Governance smoke tests and 55 host-feature cache tests pass
locked/offline on Linux. Evidence: `/tmp/ic-query-0544-host-{fetch,smoke-tests,cache-tests}.log`
and `/tmp/ic-query-host0125-registry-{artifacts,fs,process}/`.

The separate [Query #42](https://github.com/dragginzgame/ic-query/issues/42) /
[Shared #107](https://github.com/dragginzgame/shared-tooling/issues/107)
inspection confirms a consumer assertion gap: a disposable copy of Query's
completion checker accepts deliberately contradictory payload bytes on Bash 3.2,
while current Bash rejects them. This is an injected false assertion, not an
observed lost production payload. Evidence:
`/tmp/ic-query-0544-assertion-review.*/`; normal portable passes do not qualify
that checker's refusal behavior. The completed installer fixtures and original
status-23 release probes remain their own source-bound evidence.

ShellCheck, both Bash syntax checks, snapshot integrity, current-version
changelog admission, local documentation links and whitespace checks pass.
Manifest, incoming lockfile and real index hashes are unchanged during this
adoption. No full local consumer CI, native macOS execution, actual release/version
bump, commit, tag or push ran. Delivery and native acceptance remain outstanding;
released 0.54.3's Linux CI results do not qualify this patch.
[Shared 0.3.7 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38064027998)
passes Linux portable regression and lint/security, and Host 0.12.5 passes Linux native
and MSRV; their native macOS jobs remain queued at inspection.

## Tool-specific dependencies

Released 0.54.0 selects published Host artifacts/filesystem/process
0.12.0, with the incoming manifest and matching lockfile preserved. All three
crates.io entries are unyanked; registry checksums match the lock and packaged VCS
metadata identifies release `1ba4591868a83b367d56bae9e3c213418c66a192`.
Library source/APIs are unchanged from 0.11.0; the minor release changes Host's
complete setup/CI/release tooling contract. Its
[exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38045025680)
passes Linux native, MSRV and both macOS architectures.

After explicit locked fetching, Query's 26 Governance smoke tests and 55
host-feature cache tests pass locked/offline on Linux. Evidence:
`/tmp/ic-query-host012-fetch.log`, `/tmp/ic-query-host012-smoke-tests.log`,
`/tmp/ic-query-host012-cache-tests.log`, and
`/tmp/ic-query-host012-registry-{artifacts,fs,process}/`. No full workspace gate,
native consumer execution or release/deployment ran for this selection. Producer
passes do not qualify pending consumer macOS coverage.

The shared IC executable pins still match current Quill, ICP CLI, didc and
ic-wasm releases. Binaryen 133 is newer than the selected verified wasm-opt 132;
its archive metadata and required owner/native qualification are reported in
[Shared #102](https://github.com/dragginzgame/shared-tooling/issues/102).
Query retains the immutable shared pins and existing installations pending a
reviewed owner update; 133 has not been installed or tested here.

| Workflow | Explicit prerequisites |
| --- | --- |
| Host setup and offline verification | Bash, curl for setup, tar/gzip for ripgrep, Perl, SHA-256 backend, reviewed `ci/tool-versions.env` |
| IC setup and offline verification | Bash, curl for setup, tar with xz/gzip support, Perl, SHA-256 backend, reviewed `ci/ic-tools.tsv` |
| Dependency declaration checks | Git, local jq/yq, Cargo for workspace discovery |
| Workspace version queries, changelog defaults and release/publication version admission | Prepared Cargo toolchain, local jq/yq; offline manifest validation without dependency resolution |
| Formatting and its offline checks | Prepared rustfmt and exact cargo-sort 2.1.4 from `ci/tool-versions.env`; common receipts installed by `install-rust-tools`, included in `install-tools` |
| Common Cargo-tool set | Prepared Cargo toolchain and native compilation prerequisites; `install-rust-tools` / offline `rust-tools-check`, included in the complete aggregates, using the three pins in `ci/tool-versions.env` |
| Local documentation links | Perl core modules; current guide and contract roster selected by Make |
| RustSec preparation and auditing | Bash, Git, explicit HTTPS advisory source, Cargo Audit; failed databases and preparation logs retained |
| Focused CI script fixtures | Bash, Make, Git, Perl and ordinary utilities; Cargo/network effects use stubs |
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

The shared Cargo-tool set is required for Query. `make install-rust-tools`
prepares shared cargo-sort and cargo-sort-derives selections
with references under `.tools/rust`; `make rust-tools-check` verifies the complete
set offline. The standard aggregates include both through the same installer and
activate `.tools/rust/common/bin` after complete admission. Formatter targets
require this common selection before checking cargo-sort's pin and rustfmt;
global copies cannot substitute for missing selections. Existing copies remain
intact. Cargo Audit and Cargo Machete use explicit setup/check extension targets
through the shared installer; dependency execution uses its returned checked
paths. Caller pins remain in `Makefile`. Callers use the complete aggregate.
Installer substitution tests establish
dispatch and failure handling, not native compilation of these executables.

## Testkit canister qualification

The native development graph selects Testkit and its PocketIC client through
`Cargo.lock`, and the server through Testkit's provisioning owner. Native source review
and compilation do not qualify a server on another host. The configured
Linux, macOS ARM64 and macOS x86-64 canister lanes explicitly prepare/check
Testkit, then execute `make canister-test` before local-NNS deployment smoke.
Local evidence and outstanding native CI results are reported at handoff.

Testkit and WAT are native development dependencies only. Normal minimal,
canister and host library consumers retain their dependency boundaries and
Rust 1.91 floor. Published `ic-platform` 0.1.11 supplies bounded Candid reply
decoding and qualified contract types on native and Wasm targets without the
simulator. Testkit 0.54's native fixture calls use Platform's limits and errors
directly, without a separate decoder package. The `canister` feature selects
Platform's CDK facade and generator for calls, canister identity and the typed
probe. Native Agent APIs and the probe's raw clock also use Platform; no direct
Candid/CDK declarations remain. Native macOS and installed-IC callback/trap
qualification remain separate from Linux compilation and simulator evidence.
Query retains transport limits, proportional work quotas and error projection.
PocketIC's exact `thiserror =2.0.18` requirement selects that
compatible patch in the workspace lockfile; the previously selected 2.0.21
cannot coexist under Cargo's compatible-version unification. Query's direct
`thiserror =2` requirement and public error types remain unchanged.
