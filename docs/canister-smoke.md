<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-query/ic-query-readme-header.svg" alt="IC Query — a read-only Internet Computer explorer">
</p>

<!-- helper-navigation:start -->
<p align="center">
  <a href="https://github.com/dragginzgame/canic"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/canic.svg" width="18" height="18" alt=""> <strong>canic</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/icydb"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/icydb.svg" width="18" height="18" alt=""> <strong>icydb</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-timers"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-timers.svg" width="18" height="18" alt=""> <strong>ic-timers</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-memory"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-memory.svg" width="18" height="18" alt=""> <strong>ic-memory</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-query"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-query.svg" width="18" height="18" alt=""> <strong>ic-query</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-backup"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-backup.svg" width="18" height="18" alt=""> <strong>ic-backup</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-blob-storage"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-blob-storage.svg" width="18" height="18" alt=""> <strong>ic-blob-storage</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-testkit"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-testkit.svg" width="18" height="18" alt=""> <strong>ic-testkit</strong></a>
</p>
<!-- helper-navigation:end -->

# Governance canister smoke tests

The harness exercises the four direct NNS Governance reports through
`CanisterNnsSource` in a deployed Wasm canister. It uses the existing
`ic-query` package's `governance_probe` example and the development-only
`governance_artifact` helper in `ic-query-cli`. The helper adopts
`ic-host-artifacts` for bounded streams, digesting and Wasm inspection,
`ic-host-fs` for artifact file reads and atomic receipt publication, and
`ic-host-tools` for response decoding;
it adds no production CLI operation or canister-runtime dependency.
The prepared adoption selects published IC Host Tooling 0.4.3 at
[`644d49c`](https://github.com/dragginzgame/ic-host-tooling/tree/644d49c096ae05c2e17e1b6aacf14770988c5cf6).
Its native macOS compile defect needs a corrected release before qualification;
see the [host matrix](supported-hosts.md#tool-specific-dependencies) and
[Host #18](https://github.com/dragginzgame/ic-host-tooling/issues/18).
The helper uses retained bounded file-read, durable publication and response
APIs; it has no callers of the removed durable/private readers or lock wrapper.
The workspace lockfile selects registry packages; archive support is unnecessary
for this helper and remains disabled. The response-only tools profile disables
default features, excluding Candid extraction and its process execution edge.

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-query/ic-query-canister-smoke-flow.svg" alt="Canister smoke-test lifecycle from building and deploying the probe through report validation, cleanup, and receipt finalization, with failure and interruption handling">
</p>

## Prerequisites

- Linux or macOS; interruption cleanup uses POSIX process groups.
- Rust 1.99.0, selected by `rust-toolchain.toml`, with
  `wasm32-unknown-unknown` installed.
- Python 3.10 or later and the verified repository-local IC toolset, including
  ICP CLI **1.6.0**. Prepare it explicitly with `make install-ic-tools`;
  `make ic-tools-check` verifies the installed set offline.
- The selected workspace dependency cache, including the development-only
  IC Host Tooling dependencies. Prepare it explicitly with `cargo fetch --locked`;
  helper compilation and validation use locked/offline Cargo commands.
- Internet access for the first local-runtime download and enough resources
  to run the local NNS/SNS network.

The managed runtime is pinned to network launcher **16.0.0** in `icp.yaml`.
ICP CLI 1.6.0 bundles launcher **16.0.0-2026-09-18-03-28**; the project pin
keeps local NNS execution on the existing stable runtime. ICP identities,
settings and launcher packages use the persistent ignored `.icp-smoke-home/`
by default. Receipts, Wasm and bundles live under `target/canister-smoke/`;
runtime state lives in `tests/canister/.icp/`. These directories belong outside
commits and release artifacts. The library's supported minimum Rust version remains **1.91.0**;
`make msrv` verifies it separately from the development toolchain.

## Identity storage and cleanup

The pinned [ICP CLI 1.6.0 home implementation](https://github.com/dfinity/icp-cli/blob/18435e1747162447fca231548b74d97e4cf48888/crates/icp-app/src/directories.rs)
places identities, settings and launcher packages beneath one `ICP_HOME` on
Linux and macOS. The harness selects that home only for ICP children; it
preserves the parent environment and caller XDG settings. An explicit
`ICP_HOME` keeps its selection, including a relative path resolved from this
repository's root. Empty values or homes beneath `target/`, Cargo's configured
target directory, or `tests/canister/.icp/` are rejected before ICP dispatch.
Resolved symlink aliases follow the same admission rule.

`cargo clean` removes Cargo output; it does not remove `.icp-smoke-home/`.
Local network reset affects runtime state, not the selected identity home.
Back up any required identities through ICP's
[identity backup procedure](https://github.com/dfinity/icp-cli/blob/18435e1747162447fca231548b74d97e4cf48888/docs/guides/managing-identities.md)
before deleting a persistent home or the entire checkout. A shared
or funded identity home should be selected deliberately for development use.

Previous harness versions selected XDG data/config/cache roots beneath
`target/canister-smoke/`. Existing files are preserved in place. Before cleaning
old output, inspect and back up any required identity metadata and keys;
macOS's platform directory behavior and explicit `ICP_HOME` overrides may have
selected another location. Set `ICP_HOME` to an existing persistent home or
explicitly restore required identities into one. The new default begins with
ICP's anonymous identity until the maintainer explicitly selects another.
There is no automatic copy, migration, reset or deletion of existing state.
This repair is tracked in [#16](https://github.com/dragginzgame/ic-query/issues/16).

## Build and local execution

```bash
rustup target add wasm32-unknown-unknown
make install-ic-tools
make ic-tools-check
make canister-build
make canister-smoke
```

The harness selects `.tools/ic/bin/icp` explicitly, verifies the installed set
offline before ICP execution, and never installs tools implicitly. The check
uses `IC_TOOL_PINS` when explicitly selected in Make or the environment, otherwise
the snapshot matrix, which owns executable version qualification. Receipts record
the observed ICP version. Direct invocation needs no global ICP selection.

`canister-build` invokes `icp build -e local` against the isolated project in
`tests/canister/`. It builds the example with only the `canister` feature and
embeds public `candid:service` and `ic-query:build` metadata. The latter records
the compiler, lockfile hash, and source-input hash. The final Wasm is retained
at `target/canister-smoke/governance_probe.wasm`.

The shared host helper admits core Wasm framing under a 64 MiB artifact limit,
10,000 sections/exports and 1,000 custom sections. The builder checks the admitted
digest again before attaching consumer-owned metadata. Inspection is structural;
it does not replace the compiler or replica's Wasm validation. Response admission
bounds the complete ICP JSON envelope to 8 MiB and decoded Candid bytes to 2 MiB,
requires one Candid text value without trailing arguments/bytes, and returns that
text unchanged for report validation. These limits apply to helper admission;
the existing Python subprocess capture and process-group deadlines remain its
owner's contract. Receipts retain the response before admission fails.

The development-only helper can inspect an existing artifact without modifying it:

```bash
cargo run -p ic-query-cli --example governance_artifact --locked --offline -- \
  inspect-wasm target/canister-smoke/governance_probe.wasm
```

`canister-smoke` starts a loopback-only local network with NNS canisters,
deploys the probe there, and calls it once for each of economics, metrics,
latest reward event, and maturity modulation. Each probe request performs
one bounded replicated call to the fixed Governance principal. The runner
uses the bounded `inspect-wasm` helper to admit and hash the local module, then
checks the deployed module hash and Candid metadata before collecting,
validates each report's schema, network identity, Governance principal,
collector principal, timestamp, and replicated source, and checks the module
hash again afterward. It attempts network cleanup after success, collection
failure, or an interrupted or timed-out startup attempt. It refuses to take
ownership of an already running harness network.

This command creates and installs only a **local test canister**. Production
`icq` remains a read-only reporting CLI. The probe has no stable-memory writes
and does not invoke Governance mutations.

Each attempt writes a unique
`target/canister-smoke/receipt-*/receipt.json` before starting network work,
then atomically replaces it as work proceeds, including on handled failures.
The harness serializes complete strict JSON before sending it to the
development helper's `write-receipt` operation. The helper streams stdin through
`ic-host-fs::durable::write_with` without buffering another complete receipt;
an input failure leaves the prior complete file intact. Shared filesystem publication
owns the same-directory temporary file, file synchronization, replacement and
parent-directory synchronization. The helper runs with an explicit child-only
umask of 077 so replacement receipts remain mode 0600. Receipt schema, lifecycle,
timestamps, paths and retained evidence remain harness-owned.
Receipts retain the environment, timestamps, tool versions, network details,
module evidence, build metadata, raw Candid replies, and decoded reports.
Receipts record the active `phase` and `updated_at`; raw replies are saved
before validation, and validated report payloads are added afterward.
`status: passed` requires all four reports, the final module check, and local
network cleanup to succeed. A missing optional maturity value is a valid
successful response.

SIGINT and SIGTERM stop the active command's process group and trigger local
network cleanup, including when startup has not returned yet. Further
termination signals are ignored during that interruption's cleanup. A failed
receipt retains `failed_phase`, `interrupted_by` when applicable, and any
`cleanup_error`, together with evidence collected so far. Timeouts also stop
the command's child processes before network cleanup is attempted.

SIGKILL cannot execute cleanup handlers. In a surviving workspace, the last
complete receipt remains on disk with `status: running` and the last recorded
phase; it must be treated as incomplete evidence. A failed replacement never
truncates the previously published receipt. A surviving local runtime may
still need to be stopped manually after a hard kill.

The local NNS uses the same principal and the library's `ic` request identity,
but its receipt explicitly says `evidence_scope: local_nns`. Those reports
are test-network evidence. Sequential collection is not a point-in-time
snapshot, and checking the module before and after does not prove that no
intermediate upgrade occurred.

## Environment-specific bundle

```bash
make canister-bundle
```

This uses ICP 1.6.0's `icp project bundle -e mainnet-smoke` to produce
`target/canister-smoke/governance-probe.tar.gz`. The archive contains only the
probe Wasm and a manifest using a prebuilt step with its SHA-256 hash. It
needs no Rust checkout to deploy. The ordinary `ic` environment contains no
canisters; mainnet use requires selecting `mainnet-smoke` explicitly.

Metadata verification runs directly through ICP's read-only metadata command.
It stays outside deployment sync steps so that the bundle remains portable
and verification can be rerun against an existing deployment.

## Mainnet evidence

The harness does not create or install a mainnet canister automatically.
Deploy the reviewed bundle to a dedicated, funded test canister using the
`mainnet-smoke` environment. Preserve the corresponding source checkout and
Rust toolchain so the verifier can reproduce the Wasm hash. Then run:

```bash
python3 scripts/canister/smoke.py verify-mainnet --canister <probe-principal>
```

The verifier selects the built-in mainnet endpoint `https://icp-api.io/` and
mainnet root key, verifies the existing probe, and submits four report calls.
It records `evidence_scope: mainnet`. It neither funds, installs, upgrades,
nor deletes that canister. Its isolated identity needs no controller access:
the probe endpoints and metadata are public, and status uses public state-tree
evidence. An incorrect principal or mismatched Wasm fails before report calls.

A successful current mainnet receipt demonstrates this checkout's execution;
it cannot establish whether the historical 0.38 smoke was run. The historical
receipt gap remains documented in the [0.38 design](design/0.38/0.38-design.md).

## CI and retention

The separate `canister` CI matrix runs on Ubuntu 24.04, macOS 15 Apple Silicon,
and macOS 15 Intel. It explicitly installs and checks the common local IC set
from the snapshot's single pin matrix, retaining ICP CLI 1.6.0 and its existing
archive digests. It selects `.tools/ic/bin`, runs the local smoke, and builds
the bundle. It uploads receipts, the probe Wasm,
and the bundle under `canister-smoke-<host>` for 30 days, including receipts from
failed runs. Identities and runtime state are excluded from uploads.
The newly configured macOS runtime jobs require matching native runs before
claiming qualification.
Retain a reviewed receipt separately if it must outlive that retention window.

The ordinary feature-boundary gate compile-checks the Wasm example. The native
`checks` matrix runs receipt, interruption, malformed-reply and provenance tests
through `make ci-scripts-check`. Artifact-helper Rust unit tests run through
`make test`, whose all-target selection includes examples. Each unit suite has
one owner in the complete CI gate. `make ci` does not start a local network; use
`make canister-smoke` for the separate runtime gate.

Upstream contracts: [ICP 1.6.0 release notes](https://github.com/dfinity/icp-cli/releases/tag/v1.6.0)
and [configuration reference](https://github.com/dfinity/icp-cli/blob/v1.6.0/docs/reference/configuration.md).
