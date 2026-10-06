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
- `make install-dev` explicitly installs the exact Cargo Audit, Cargo Machete,
  and ripgrep versions declared in `Makefile`, plus the common jq 1.8.2 and
  Mike Farah yq 4.47.2 pair under `.tools/host/bin`. Parser versions and native
  binary digests have one owner in the immutable `ci/tool-versions.env` snapshot.
  `make host-tools-check` verifies both hashes before executing version checks;
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
  excluded. Linux ARM64 has a parser mapping but no complete IC set and remains
  outside the qualified consumer matrix.
- Explicitly prepare the selected dependency cache before a gate. Hosted CI
  uses `cargo fetch --locked`, then runs the gate with `CARGO_NET_OFFLINE=true`.
  Local release preflight uses `cargo fetch --locked --offline`, reporting missing
  inputs without fetching or changing their versions.
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

The reviewed Shared Tooling 0.1.6 snapshot passed its native provisioning CI on
[Linux and both macOS hosts](https://github.com/dragginzgame/shared-tooling/actions/runs/37450707625).
IC Query's adoption changes require their own matching native CI; upstream
qualification does not establish consumer execution or deployment compatibility.

## Tool-specific dependencies

| Workflow | Explicit prerequisites |
| --- | --- |
| Host setup and offline verification | Bash, curl for setup, Perl, SHA-256 backend, reviewed `ci/tool-versions.env` |
| IC setup and offline verification | Bash, curl for setup, tar with xz/gzip support, Perl, SHA-256 backend, reviewed `ci/ic-tools.tsv` |
| Dependency declaration checks | Git, local jq/yq, Cargo for workspace discovery |
| Focused CI script fixtures | Bash, Make, Git, Python 3, Perl and ordinary utilities; Cargo/network effects use stubs except the separately selected artifact-helper tests |
| Artifact-helper and receipt tests | Selected Rust toolchain and locked/offline dependency cache, Python 3, POSIX process groups |
| Complete gate | Declared Rust toolchain, local host pair, Cargo Audit/Machete, ripgrep and ordinary utilities |
| Governance integration | Verified local IC set, Wasm Rust target, Python 3 and explicit local-runtime network access |

Installer implementation regression suites stay in Shared Tooling. This
consumer's gate exercises Make ordering, explicit pin selection, local PATH and
failure propagation, verifies the immutable snapshot and installed host tools,
and requires actual setup qualification in the configured native CI jobs.

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

Shared Tooling's own regression suite also uses cargo-sort. It remains outside
IC Query's gate prerequisites.
