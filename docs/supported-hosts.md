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
  ripgrep 15.2.0 with PCRE2 under `.tools/host/bin`. Host versions and native
  binary/archive digests have one owner in the immutable `ci/tool-versions.env` snapshot.
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
The selected 0.1.14 revision `25e7ce83149e081e4dcc52c55c33724e44153f2a`
adds isolated Make execution admission and selected-target LOC exclusion. Its
[native CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37586649650)
passed Linux, both macOS architectures and lint/security.
IC Query's adoption changes require their own matching native CI; upstream
qualification does not establish consumer execution or deployment compatibility.
The selected IC Host Tooling 0.3.1 registry crates were reviewed at
`38a2a5127be064014e6d39d72d0300ffb2cf20be`. That exact published source passed
[Linux, both macOS architectures and MSRV CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37580017649).
The split dependency, shared stream and receipt adoption in IC Query 0.47.7 requires
consumer native qualification; local Linux fixtures do not establish it.

## Tool-specific dependencies

| Workflow | Explicit prerequisites |
| --- | --- |
| Host setup and offline verification | Bash, curl for setup, tar/gzip for ripgrep, Perl, SHA-256 backend, reviewed `ci/tool-versions.env` |
| IC setup and offline verification | Bash, curl for setup, tar with xz/gzip support, Perl, SHA-256 backend, reviewed `ci/ic-tools.tsv` |
| Dependency declaration checks | Git, local jq/yq, Cargo for workspace discovery |
| Workspace version queries, changelog defaults and release/publication version admission | Prepared Cargo toolchain, local jq/yq; offline manifest validation without dependency resolution |
| Formatting and its offline checks | Prepared rustfmt and exact cargo-sort 2.1.4 from `ci/tool-versions.env`; installed explicitly by `install-dev` |
| Local documentation links | Perl core modules; current guide and contract roster selected by Make |
| RustSec preparation and auditing | Bash, Git, explicit HTTPS advisory source, Cargo Audit; failed databases and preparation logs retained |
| Focused CI script fixtures | Bash, Make, Git, Python 3, Perl and ordinary utilities; Cargo/network effects use stubs |
| Artifact-helper and receipt tests | Selected Rust toolchain and locked/offline dependency cache, Python 3, POSIX process groups |
| Complete gate | Declared Rust toolchain, local jq/yq/ripgrep set, Cargo Audit/Machete and ordinary utilities |
| Governance integration | Verified local IC set, Wasm Rust target, Python 3 and explicit local-runtime network access |

Installer implementation regression suites stay in Shared Tooling. This
consumer's gate exercises Make ordering, explicit pin selection, local PATH and
failure propagation, verifies the immutable snapshot and installed host tools,
and requires actual setup qualification in the configured native CI jobs.

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

The maintained shared `scripts/dev/cloc.sh` needs Cargo, cloc, jq, Bash and
ordinary Unix utilities. Install cloc through the explicit host bootstrap in
[local setup](local-setup.md); it is not an additional complete-gate prerequisite.
Select the repository root explicitly, or run from the checkout root:

```bash
CARGO_NET_OFFLINE=true PATH="$PWD/.tools/host/bin:$PATH" \
  bash scripts/dev/cloc.sh "$PWD"
```

The reporter selects Cargo workspace members and uses the same Rust file lists
for LOC and test-attribute counts. Nested members count once, and a final `TOTAL`
row sums the member rows. The default root is the caller's current directory;
pass an explicit root when invoking the script from elsewhere. Historical measurements retain their original
method identity; compare them only after checking file selection and counting
rules. This command discovers workspace metadata and counts files; it does not
compile tests or run a broad gate.

IC Query's formatting gate now requires the same reviewed cargo-sort pin used
by explicit development setup. `fmt-check` checks dependency order before Rust
formatting and preserves source, the Git index, lockfiles and unrelated edits.
