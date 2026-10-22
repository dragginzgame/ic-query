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

Ordinary PR delivery and separately authorized merges follow the shared
[asynchronous CI cadence](../DRAGGINZGAME.md#validation-and-evidence).
Focused review/checks and enforced GitHub requirements govern integration;
pending native runs remain unqualified. Release, publication and deployment
retain their separate acceptance gates.

## Prerequisites

- Prepare the development compiler selected by [rust-toolchain.toml](../rust-toolchain.toml),
  including rustfmt, Clippy and `wasm32-unknown-unknown`. The separate MSRV check
  reads the inherited `rust-version` from [Cargo.toml](../Cargo.toml) through
  Cargo metadata. `make msrv-version` prints that selection; `make msrv MSRV=VERSION`
  permits an explicit compiler experiment without changing the declared floor.
- Bash 3.2 or newer, GNU Make 3.81 or newer, Git, Perl with its core modules
  (including JSON::PP), and standard Unix utilities are required. macOS needs
  Xcode Command Line Tools. Explicit executable setup also needs curl, CA
  certificates, tar with gzip/xz support and a SHA-256 backend. See the common
  [bootstrap prerequisites](local-setup.md#bootstrap-prerequisites).
- `make install-tools` prepares the host and common Cargo tools, followed by
  Query's selected IC executables, Cargo Audit/Machete and release metadata editor.
  IC and Rust preflight run before installations; setup does not install a Rust
  toolchain. `make tools-check` admits the prepared selections offline.
- Query's `IC_TOOLS` in [Makefile](../Makefile) selects didc, ic-wasm, ICP CLI,
  Quill and wasm-opt. Host and common Cargo-tool pins belong to
  [ci/tool-versions.env](../ci/tool-versions.env); IC versions, host mappings and
  archive digests belong to [ci/ic-tools.tsv](../ci/ic-tools.tsv). Audit/Machete
  selections belong to Makefile; Build and Testkit binaries follow Cargo.lock.
  There is no second version list in this guide. dfx is not selected.
- Make uses the admitted checkout-local paths. Interactive shells can select them
  with `export PATH="$PWD/.tools/host/bin:$PWD/.tools/ic/bin:$PWD/.tools/rust/common/bin:$PATH"`.
  Existing global executables cannot substitute for missing managed selections.
  A changed IC selection requires explicit setup before offline admission.
- Use the same `SHARED_TOOLING_STORE` for setup, checks and failure collection.
  The [shared executable store](local-setup.md#shared-executable-storage) retains
  previous and failed bundles; setup does not erase caller evidence or artifacts.
- Explicitly prepare the selected dependency cache before validation with
  `cargo fetch --locked`. Ordinary build, check, Clippy, test, MSRV, rustdoc and
  package checks remain locked/offline. Standard maintainer release entry points
  prepare the cache while preserving explicit offline settings; standalone release
  adapters require the existing cache. RustSec refresh is a separate live Git fetch.
- Publication requires separately authorized crates.io credentials. Release
  pushes require the maintainer's selected remote and branch. Setup and fixture
  tests provide no release, publication or deployment authority.

The [Governance smoke harness](canister-smoke.md) has additional managed-network
and Testkit prerequisites. Linux ARM64 remains outside Query's qualified host
matrix, even where an individual shared executable has an ARM64 mapping.

## Tool-specific dependencies

| Workflow | Explicit prerequisites |
| --- | --- |
| Host setup and offline verification | Bash, curl for setup, tar/gzip, Perl, SHA-256 backend, reviewed host catalog |
| IC setup and offline verification | Bash, curl for setup, tar with xz/gzip support, Perl, SHA-256 backend, reviewed IC catalog |
| Dependency declaration checks | Git, prepared local jq/yq, Cargo for workspace discovery |
| Workspace version and release/publication version admission | Prepared Cargo toolchain and local jq/yq; offline manifest validation |
| MSRV selection | Installed Cargo, Bash and Perl JSON::PP; metadata discovery needs no dependency source cache |
| Formatting and offline admission | Prepared rustfmt and common Cargo tools, verified through the shared installer |
| Cargo-tool installation | Prepared compiler, native compilation prerequisites and managed source cache |
| Local documentation links | Perl core modules; guide and contract roster selected by Make |
| RustSec preparation and auditing | Bash, Git, explicit HTTPS advisory source, prepared Cargo Audit |
| CI script fixtures | Bash, Make, Git, Perl and ordinary utilities; external effects use stubs |
| Governance runner fixtures | Selected compiler, locked/offline dependency cache and POSIX process groups |
| Complete gate | Declared compiler, prepared host/Cargo tools, Audit/Machete and ordinary utilities |
| Governance integration | Verified IC set, Wasm target, prepared Testkit server and explicit local-runtime network access |

The Governance harness selects a persistent `ICP_HOME` only for ICP children.
Identity keys and settings are separate from retained build receipts and runtime
output. Homes beneath Cargo output or harness runtime state are refused without
moving existing keys. See [identity storage and recovery](canister-smoke.md#identity-storage-and-cleanup).

Installer implementation regression suites belong to Shared Tooling. Query
qualifies its snapshot, Make ordering, selected arguments, local PATH, offline
admission and failure propagation, plus native executable behavior in CI. The
consumer's selected release-runner fixtures simulate Git effects; they do not
execute releases or create real release commits, tags or pushes.

## Native CI and qualification

The workflow runs complete checks, MSRV and canister integration on each declared
host. The general checks compile Governance runner fixtures through
`ci-scripts-check`; the all-target test gate executes them. The canister lane
separately prepares/checks Testkit, executes `make canister-test`, then runs local
NNS deployment smoke and bundle construction. Cross-compilation does not qualify
native macOS behavior or installed-IC callbacks, traps and lifecycle semantics.

CI provisioning explicitly uses `/bin/bash` and `/bin` first on PATH. The macOS
complete gate uses Apple's system Bash. Fixtures canonicalize temporary-directory
paths so `/var` aliases do not weaken production symlink rejection.

Failed jobs retain temporary validation directories and complete failure logs as
host-specific artifacts for 30 days, including hidden metadata. Canister jobs also
retain receipts, Wire replies, Wasm and bundles. Local failed helpers print their
retained paths. Successful fixtures remove only their own temporary files; product
builds, earlier evidence and unrelated outputs remain intact.

### Delivered evidence

Released 0.67.4 at `1459e0e5769a0dfeeb4d8e45876e1d53859eecb9` passes all
nine jobs in its [exact-source CI run](https://github.com/dragginzgame/ic-query/actions/runs/39613907899):
complete checks, MSRV and canister integration on Linux and both macOS architectures.
This qualifies that delivered source, not later working-tree changes, dependency
selections or the pending Shared Tooling adoption. Those need their own matching
native results. Keep qualification tied to exact sources; pending, cancelled or
missing jobs are not passes.

Superseded adoption instructions and obsolete pending blockers have been removed.
Historical investigations remain in the guide's
[committed history](https://github.com/dragginzgame/ic-query/blob/1459e0e5769a0dfeeb4d8e45876e1d53859eecb9/docs/supported-hosts.md),
linked issues and retained CI records. Maintain this section for the current
qualification decision rather than appending another dependency-adoption diary.

## Development LOC reports

The canonical [workspace reporter](../scripts/dev/cloc.sh) uses the prepared host
cloc and Cargo workspace discovery. Run it from the checkout root:

```bash
make cloc
make cloc CLOC_MANIFEST=Cargo.toml
```

Nested members count once; the final `TOTAL` sums member rows. LOC and test-attribute
counts use the same selected Rust files. This discovers metadata and counts files;
it does not compile tests or run a broad gate. Compare historical measurements
only after checking their file-selection and counting rules. Fleet reports remain
centralized in Shared Tooling.

## Testkit canister qualification

Testkit and raw-reply fixtures are native development dependencies. Cargo.lock
selects the client; the published Testkit provisioning interface owns its server.
Run `make install-testkit testkit-check` before deterministic canister tests.

Portable and deployed library graphs exclude the simulator and host-only parser.
Platform owns Candid decoding, native Agent APIs, certification types and the CDK
facade/generator; owned test RPCs use Wire. Query retains endpoint/root-key
admission, quotas, transport policy and error projection. The
[dependency graph checks](../scripts/ci/check-library-dependency-graphs.sh) and
[declaration boundary](../ci/cargo-dependency-boundaries.json) enforce these
profiles. Native and installed-IC qualification remains distinct from source
review, Linux tests and Wasm cross-compilation.
