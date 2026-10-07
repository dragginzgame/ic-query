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
  Python 3 for the maintained documentation/canister tools, and standard Unix
  utilities. macOS needs Xcode Command Line Tools for its compiler, Git and Make.
  Release/checksum helpers support both GNU `sha256sum` and macOS `shasum`.
- `make install-dev` explicitly installs the exact Cargo Audit and Cargo Machete
  versions declared in `Makefile`, cargo-sort 2.1.4 from `ci/tool-versions.env`,
  plus common jq 1.8.2, Mike Farah yq 4.47.2 and
  ripgrep 15.2.0 with PCRE2 and cloc 2.10 under `.tools/host/bin`. Host versions
  and payload digests have one owner in the immutable `ci/tool-versions.env` snapshot.
  `make host-tools-check` authenticates all payloads before version and PCRE2 checks;
  the complete CI/release gate includes that offline check and
  `make dependency-pins-check`. Installation is separate from ordinary validation.
- `make install-tools` prepares host tools followed by the complete IC set:
  Quill 0.5.4, ICP CLI 1.6.0, didc 0.6.2, ic-wasm 0.11.1, PocketIC 16.0.0 and
  wasm-opt 132 under `.tools/ic/bin`. IC versions and archive digests have one
  owner in `ci/ic-tools.tsv`; `make tools-check` verifies both sets offline.
  Make selects the local paths; interactive shells use
  `export PATH="$PWD/.tools/host/bin:$PWD/.tools/ic/bin:$PATH"`.
  See [local setup](local-setup.md) and [IC tools](ic-tools.md) for bootstrap
  packages, complete-set activation, locks and retained failed/previous sets.
  curl, tar with xz/gzip support and a SHA-256 backend are required. dfx is
  excluded. Linux ARM64 has a host-tool mapping but no complete IC set and remains
  outside the qualified consumer matrix.
- Explicitly prepare the selected dependency cache before a gate. Hosted CI
  uses `cargo fetch --locked`, then runs the gate with `CARGO_NET_OFFLINE=true`.
  Local release preflight uses `cargo fetch --locked --offline`, reporting missing
  inputs without fetching or changing their versions.
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
has passed all three MSRV and canister jobs, plus Linux checks; the macOS checks
remain in progress. Its [tag run](https://github.com/dragginzgame/ic-query/actions/runs/37619656675)
has completed successfully with all three native check jobs passing; MSRV and
canister jobs are skipped by the configured tag workflow. The branch run has
not completed as a green gate yet.
The selected 0.1.19 revision `a06e4719e3839b8eefcfb88ec8923aa88eb63ccc`
adds Rust-tool path admission, release-note EOF preservation, combined failure
logs and portable temporary-root fixture handling. No matching upstream CI run
was available at adoption review. Focused Linux fixtures do not qualify macOS;
the prepared 0.47.11 consumer changes require matching native CI after commit.
IC Query's adoption changes require their own matching native CI; upstream
qualification does not establish consumer execution or deployment compatibility.
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
The selected 0.4.2 registry crates record published VCS revision
`6501d0e9fa7ba0439ec7a4010ca7bf0205e1d712`; their packaged Rust sources and
original manifests match the committed owner. Its
[matching CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37624014360)
has passed Linux and MSRV; both macOS jobs remain queued. The shared fixture
repair is committed, but native qualification and the consumer's adoption
qualification remain incomplete.

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
| Artifact-helper and receipt tests | Selected Rust toolchain and locked/offline dependency cache, Python 3, POSIX process groups |
| Complete gate | Declared Rust toolchain, local jq/yq/ripgrep/cloc set, Cargo Audit/Machete and ordinary utilities |
| Governance integration | Verified local IC set, Wasm Rust target, Python 3 and explicit local-runtime network access |

The Governance harness selects a persistent `ICP_HOME` only for ICP children,
using the same override on Linux and macOS. Identities, settings and launcher
packages share that home; build receipts and local runtime output retain their
separate owners. It refuses identity homes beneath standard/configured Cargo
output or harness runtime state, without moving existing keys. See the
[home selection and recovery procedure](canister-smoke.md#identity-storage-and-cleanup).
The prepared 0.47.11 repair requires native consumer smoke qualification after
commit; Linux sentinel/process fixtures do not establish macOS execution.

Installer implementation regression suites are owned by Shared Tooling. This
consumer runs the reviewed canonical Rust-tool fixture with substitute Cargo;
the wider owner suites remain upstream. The consumer gate also exercises Make
ordering, explicit pin selection, local PATH and failure propagation, verifies
the immutable snapshot and installed host tools, and requires actual setup
qualification in the configured native CI jobs.

The native `checks` matrix runs receipt tests through `ci-scripts-check` and
artifact-helper Rust unit tests through `test`, which selects all targets,
including examples. Receipt tests build the helper with the prepared
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

`make cloc-tooling` inventories sibling CI and tooling without invoking consumer
commands or Cargo. `CLOC_PARENT` selects another parent directory. The selected
0.1.16 snapshot fixes custom-manifest roots and SSH source identities under
[Shared Tooling #39](https://github.com/dragginzgame/shared-tooling/issues/39).
`--snapshot-root MANIFEST ROOT` explicitly selects another ownership root;
without that override, custom manifests use their owning Git root. Classification
requires exact hashes and modes; modified shared files count as local drift.

IC Query's formatting gate now requires the same reviewed cargo-sort pin used
by explicit development setup. `fmt-check` checks dependency order before Rust
formatting and preserves source, the Git index, lockfiles and unrelated edits.

The shared Cargo-tool set is optional for Query. `make install-rust-tools`
prepares cargo-sort, cargo-sort-derives and candid-extractor under `.tools/rust`;
`make rust-tools-check` verifies that complete local set offline. Query does not
need the latter two executables for its current workflows, so `install-tools`,
`tools-check` and `install-dev` retain their existing selections. The shared
Make include adds `.tools/rust/bin` to PATH; if this optional set is prepared,
its cargo-sort must satisfy the same formatter pin. Installer substitution
tests establish dispatch and failure handling, not native compilation of these
optional executables in this consumer.
