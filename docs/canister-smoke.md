# Governance canister smoke tests

The harness exercises the four direct NNS Governance reports through
`CanisterNnsSource` in a deployed Wasm canister. It uses the existing
`ic-query` package's `governance_probe` example, with no additional crate,
production command, persistence policy, or host dependency.

## Prerequisites

- Linux or macOS; interruption cleanup uses POSIX process groups.
- Rust 1.98.1, selected by `rust-toolchain.toml`, with
  `wasm32-unknown-unknown` installed.
- ICP CLI **1.6.0** and Python 3.10 or later.
- Internet access for the first local-runtime download and enough resources
  to run the local NNS/SNS network.

The managed runtime is pinned to network launcher **16.0.0** in `icp.yaml`.
ICP CLI 1.6.0 bundles launcher **16.0.0-2026-09-18-03-28**; the project pin
keeps local NNS execution on the existing stable runtime. Tool settings,
identities, and download caches live under `target/canister-smoke/`. Runtime
state lives in `tests/canister/.icp/`. Neither directory belongs in commits or
release artifacts. The library's supported minimum Rust version remains **1.91.0**;
`make msrv` verifies it separately from the development toolchain.

## Build and local execution

```bash
rustup target add wasm32-unknown-unknown
make canister-build
make canister-smoke
```

`canister-build` invokes `icp build -e local` against the isolated project in
`tests/canister/`. It builds the example with only the `canister` feature and
embeds public `candid:service` and `ic-query:build` metadata. The latter records
the compiler, lockfile hash, and source-input hash. The final Wasm is retained
at `target/canister-smoke/governance_probe.wasm`.

`canister-smoke` starts a loopback-only local network with NNS canisters,
deploys the probe there, and calls it once for each of economics, metrics,
latest reward event, and maturity modulation. Each probe request performs
one bounded replicated call to the fixed Governance principal. The runner
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

The separate `canister` CI job installs ICP CLI 1.6.0 with a pinned SHA-256,
tests the receipt validator, runs the local smoke, and builds the bundle. It
uploads receipts, the probe Wasm, and the bundle for 30 days, including receipts
from failed runs. Identities and runtime state are excluded from uploads.
Retain a reviewed receipt separately if it must outlive that retention window.

The ordinary feature-boundary gate compile-checks the Wasm example. The normal
CI script checks also exercise malformed-reply and provenance rejection
without a network. `make ci` does not start a local network; use
`make canister-smoke` for the separate runtime gate.

Upstream contracts: [ICP 1.6.0 release notes](https://github.com/dfinity/icp-cli/releases/tag/v1.6.0)
and [configuration reference](https://github.com/dfinity/icp-cli/blob/v1.6.0/docs/reference/configuration.md).
