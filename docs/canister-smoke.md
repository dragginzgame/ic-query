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
`ic-query` package's `governance_probe` example and one development-only
`governance_smoke` Rust runner in `ic-query-cli`. The runner adopts
`ic-host-artifacts` for bounded streams, digesting and Wasm inspection,
`ic-host-fs` for file reads, raw reply evidence and atomic receipt publication,
`ic-host-process` for command IO, process-group ownership and cleanup, and
`ic-agent` for direct protocol IO. It adds no production CLI operation or
canister-runtime dependency. The workspace's Host registry selections are
recorded in [Cargo.lock](../Cargo.lock). The canister Make entry points prepare
that locked graph before offline runner builds. Archive support remains disabled
and `ic-host-tools` is not selected.
This consumer needs its own native smoke qualification; see the
[host matrix](supported-hosts.md#tool-specific-dependencies).

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-query/ic-query-canister-smoke-flow.svg" alt="Canister smoke-test lifecycle from building and deploying the probe through report validation, cleanup, and receipt finalization, with failure and interruption handling">
</p>

## Prerequisites

- Linux or macOS; interruption cleanup uses POSIX process groups.
- Rust 1.99.0, selected by `rust-toolchain.toml`, with
  `wasm32-unknown-unknown` installed.
- The verified repository-local IC toolset, including
  ICP CLI **1.6.0**. Prepare it explicitly with `make install-ic-tools`;
  `make ic-tools-check` verifies the installed set offline.
- The selected workspace dependency cache, including the development-only
  IC Host Tooling dependencies. The canister Make entry points prepare it with
  `cargo fetch --locked`; direct Cargo callers prepare it explicitly with the
  same command. Caller-selected Cargo offline settings remain authoritative;
  runner compilation and validation use locked/offline Cargo commands.
- Internet access for the first local-runtime download and enough resources
  to run the local NNS/SNS network.

The common tool bundle supplies Quill, ICP CLI, didc, ic-wasm and wasm-opt.
The harness uses ICP's managed runtime, so it does not require PocketIC or
Testkit. After adopting Shared 0.2.0, explicitly reinstall the common bundle
before offline checks; prior six-tool bundles remain retained evidence.

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
it does not replace the compiler or replica's Wasm validation. Direct agent calls
bound response bodies to 8 MiB and admit at most 2 MiB of reply bytes. The helper
saves the original binary reply before requiring exactly one Candid text value
without trailing arguments/bytes. It returns that text unchanged for report
validation, without ICP response wrapping or hex decoding. Each agent operation,
including local root-key fetching and update polling, has a 590-second deadline;
the Host command deadline remains 600 seconds. Process-group cleanup
and receipt publication still own interruption and failure handling.

Agent state reads verify certificates. Local calls fetch a root key only from
an explicitly selected loopback HTTP endpoint. Mainnet calls use HTTPS and the
built-in IC root key. The probe's `report` calls are updates: they perform the
replicated inter-canister reads tested by this harness.
The agent's HTTP client refuses redirects so requests retain the selected
endpoint and root-key trust boundary.

ICP JSON is retained only to discover the managed network's random API endpoint
and the deployed probe principal. `ic-agent` uses these explicit values; it does
not resolve ICP project aliases. Canister calls and certified state reads use
the agent directly. Deployment must identify exactly one `governance-probe`
before verification begins.

The development-only helper can inspect an existing artifact without modifying it:

```bash
cargo run -p ic-query-cli --example governance_smoke --locked --offline -- \
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
Candid admission limits original replies to 2 MiB and requires exactly one
text value with no extra values or trailing bytes. Decoder work is charged at
32 times wire size plus 1,000,000 units, capped at 256,000,000; skipped work is
limited to 100,000, type tables to 4,096 entries and headers to 64 KiB. Original
private reply files survive every admission failure. JSON report values retain
arbitrary-precision numeric evidence.

Starting with the 0.50 hard cut, schema-1 report entries use `reply_file` for an
adjacent binary Candid file and `report` for validated JSON. Module hashes are
unprefixed SHA-256 text in `module_hash_before` and `module_hash_after`, with
one `canister_id` identifying the explicitly selected principal. The agent's
certified reads bind both hashes to that principal; comparing two copied IDs
adds no identity evidence. External receipt readers must update and each attempt
uses a fresh directory. Keep previous receipts and their evidence; no reader,
migration or automatic cleanup of the old contract is supplied.
`status: passed` requires all four reports, the final module check, and local
network cleanup to succeed. A missing optional maturity value is a valid
successful response.

The harness runs directly as one Rust example; `make ci-scripts-check` runs
its offline fixtures. Commands and agent operations return typed results inside
that process. Host owns the actual command pipes, cancellation, process group
and reaping. Inherited stdout/stderr stay attached to the caller's terminals.
There are no serialized command requests or cancellation sockets.

```bash
cargo test -p ic-query-cli --example governance_smoke --locked --offline
```

The 0.51.0 hard cut retires the Python entry point and the two separate Rust
helper executables. Callers must select `governance_smoke`; receipt directories
and the schema-1 receipt contract remain current and previous evidence must
be retained. No fallback entry point or evidence migration is supplied.

SIGINT and SIGTERM attempt to stop the active command's process group and trigger local
network cleanup, including when startup has not returned yet. Further
termination signals are ignored during that interruption's cleanup. A failed
receipt retains `failed_phase`, `interrupted_by` when applicable, and any
`cleanup_error`, together with evidence collected so far. Timeouts also attempt
command cleanup before network cleanup. Group escalation reserves the unreaped
leader PID through the five-second TERM grace period and KILL signal; an already
reaped leader is not used to address a process group. Refused group KILL falls
back to killing the leader directly, and reaping has a five-second deadline.
Successful background network startup explicitly hands off after command
completion and output admission; subsequent runtime ownership remains with
`icp network stop`. Other successful commands clean up their remaining command
group. Escaped descendants are outside the command group and may need manual cleanup.

An operation failure remains the returned error if network cleanup or subsequent
receipt publication also fails. Failed receipts retain available command stdout
and stderr in `command_stdout` and `command_stderr`; incomplete byte diagnostics
are decoded as UTF-8 with replacement for invalid bytes. Command
signal, reaping and pipe cleanup failures appear as `command_cleanup_errors`
on the failed receipt; a refusal is not successful cleanup. Receipt
publication failures are reported separately as `receipt_errors` and retained
in a later complete receipt when storage recovers. If storage remains unavailable,
the previous complete receipt remains the on-disk evidence. Cleanup or publication
failure after otherwise successful verification still fails the attempt.

Successful background startup transfers runtime ownership to the harness's
network cleanup phase only after zero exit, UTF-8 admission, deadline and
cancellation checks. Host's owned-child API preserves the maintainer-selected
five-second TERM grace and five-second bounded reaping. Receipt publication
cannot skip that network cleanup or replace an earlier operation failure.
The shared process boundary is recorded in
[Host #5](https://github.com/dragginzgame/ic-host-tooling/issues/5).

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
cargo run -p ic-query-cli --example governance_smoke --locked --offline -- \
  verify-mainnet --canister <probe-principal>
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
the bundle. It uploads receipts and their adjacent binary Candid replies, the probe Wasm,
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
