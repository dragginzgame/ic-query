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
- `make install-dev` installs the exact Cargo Audit, Cargo Machete, and ripgrep
  versions declared in `Makefile`, plus Mike Farah yq 4.47.2 under
  `target/ci-tools/yq`. Its host digests live in `ci/tool-versions.env`; the shared
  installer verifies the download before execution and checks its version before
  installation. This setup step uses the network, separately from offline checks.
  Git, jq, and the parser are required for `make dependency-pins-check` in the
  complete gate. `YQ=/path/to/yq` can select an already prepared v4.47.2+ parser.
  Setup for both macOS architectures requires matching native qualification.
- Explicitly prepare the selected dependency cache before a gate. Hosted CI
  uses `cargo fetch --locked`, then runs the gate with `CARGO_NET_OFFLINE=true`.
  Local release preflight uses `cargo fetch --locked --offline`, reporting missing
  inputs without fetching or changing their versions.
- Publication additionally requires separately authorized crates.io credentials.
  Release pushes require an explicitly selected Git remote and branch; neither
  dependency setup nor fixture testing supplies publication authority.

The [Governance canister smoke harness](canister-smoke.md) has additional ICP CLI
and local-network prerequisites. Its separate live integration job is configured
on all three hosts, selecting each architecture's ICP CLI 1.6.0 archive with its
reviewed SHA-256. The downloaded archive is verified before extraction and the
executable version is checked before PATH admission. Native macOS canister-network
qualification requires passing smoke runs on this configuration; Rust
cross-compilation and the general macOS gate do not qualify it.

The macOS workflow puts `/bin/bash` first on PATH before the complete gate,
so its shell helpers and their child fixtures use Apple's system Bash in one
gate execution. CI resolves the temporary directory to a physical path
before fixtures run, so macOS `/var` aliases do not violate managed-cache
confinement. Production symlink rejection is unchanged.
The general gate and MSRV checks run natively on each host.
Build and validation evidence remain consumer-owned; release flows do not clean
them automatically.
