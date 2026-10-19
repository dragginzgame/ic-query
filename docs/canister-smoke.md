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
`ic-host-artifacts` for bounded streams, digesting, Wasm inspection and metadata
section encoding,
`ic-host-fs` for file reads, raw reply evidence and atomic receipt publication,
`ic-host-process` for command IO, process-group ownership and cleanup, and
`ic-agent` for direct protocol IO. It adds no production CLI operation or
canister-runtime dependency. The workspace's Host registry selections are
recorded in [Cargo.lock](../Cargo.lock). The canister Make entry points prepare
that locked graph before offline runner builds. Archive support remains disabled
and `ic-host-tools` is not selected.
Host's unified durable writer uses explicit publication options for receipts.
Receipt, Wasm and raw-reply failures retain the complete typed publication error,
including whether output is already visible and any staging-cleanup failure.
Reconcile visible output after a completion failure before retrying. Existing
permissions, schema-1 evidence and the create-only raw-reply policy remain intact.
This consumer needs its own native smoke qualification; see the
[host matrix](supported-hosts.md#tool-specific-dependencies).

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-query/ic-query-canister-smoke-flow.svg" alt="Canister smoke-test lifecycle from building and deploying the probe through report validation, cleanup, and receipt finalization, with failure and interruption handling">
</p>

The probe appends `icp:public ic-query:protocol` and `icp:public ic-query:build`
sections through Host's encoder, preserving exact Wire-contract/build payloads
and their order. Each append checks the remaining 64 MiB whole-artifact allowance before
changing the module. Encoding refusal leaves the input and publication destinations
unchanged. IC Build admits the named example and output path through caller-bounded
input resolution; Query retains its build executor and metadata publication.

The owned `report` endpoint uses IC Wire 0.4.9 with method ID 1, a 128-byte
request allowance and a 2 MiB reply allowance. `tests/canister/probe-wire.json`
records the contract; metadata is checked against the certified deployed module.
The report body remains JSON text. Calls from the probe to the external NNS
Governance interface retain Candid and their existing decoding quotas.

Rebuild/redeploy the probe and use its matching runner after this hard cut. New
receipts use `protocol_sha256` and preserve original responses as `.wire` files.
Existing receipts and reply evidence are retained; no fallback reader or conversion
is introduced. Native Testkit coverage also sends Wire frames directly.

## Deterministic Testkit coverage

Native development dependencies select `ic-testkit` through the workspace
manifest and lockfile. Host's bounded HTTP request reader owns the Agent,
Dashboard and ICRC socket-fixture framing loops.
Query retains response contents, injected failures and report assertions.
These ordinary tests need no running simulator.

The separate simulator check installs the actual `governance_probe` Wasm and
an in-memory Governance fixture at the fixed NNS principal. Testkit owns server
admission/lifecycle, instance startup and probe installation. Query sends
raw Wire frames and decodes replies through the published Wire codec. Query checks all four reports, collector provenance, raw metric buckets,
Governance application errors, malformed replies and inter-canister rejects.
Entrypoint checks reject wrong-direction, malformed, wrong-type, trailing and
oversized frames, then verify subsequent calls still succeed after those traps.
The fixture uses the existing private Governance wire contracts rather than
copying them into a mock application.

Prepare the selected CLI and authenticated server explicitly, then run offline:

```bash
make install-testkit
make testkit-check canister-test
```

The CLI version comes from `Cargo.lock` through the shared Cargo-tool installer.
Testkit owns authenticated server setup, offline admission and managed launch
under `.tools/testkit-server/`; these are separate from the common tool bundle.

`canister-test` builds the probe under `target/testkit/` and compiles the native
test executable before starting PocketIC. It then runs the selected ignored test
through `ic-testkit-server run`, with separate 300-second hard and idle lifetimes.
Cold compilation therefore does not consume the managed server lifetime. It never
downloads a server during validation. An explicitly selected `POCKET_IC_BIN` or
`IC_TESTKIT_POCKET_IC_URL` conflicts with this managed directory selection and
is refused by Testkit. Direct test invocations can instead select those inputs
through Testkit's environment contract and provide the compiled Wasm path as
`IC_QUERY_GOVERNANCE_PROBE_WASM`.

The regular native suite compiles this check but leaves execution to the
explicit target; the three-host canister CI job selects it. ICP local-NNS smoke
and mainnet deployment verification retain their separate deployment scope.
The library and smoke harness use the published simulator-independent
`ic-platform` codec. Query retains its caller-selected decoding budgets;
the harness additionally requires exactly one text result and preserves the
original reply before admission. Harness unit tests use Host's
`FixtureDirectory`, which removes successful fixtures and retains their evidence
when a test panics.

## Prerequisites

- Linux or macOS; interruption cleanup uses POSIX process groups.
- Rust 1.99.0, selected by `rust-toolchain.toml`, with
  `wasm32-unknown-unknown` installed.
- The verified repository-local IC toolset, including
  ICP CLI **1.7.0**. Prepare it explicitly with `make install-ic-tools`;
  `make ic-tools-check` verifies the installed set offline.
- The selected workspace dependency cache, including the development-only
  IC Host Tooling dependencies. The canister Make entry points prepare it with
  `cargo fetch --locked`; direct Cargo callers prepare it explicitly with the
  same command. Caller-selected Cargo offline settings remain authoritative;
  runner compilation and validation use locked/offline Cargo commands.
- Internet access for the first local-runtime download and enough resources
  to run the local NNS/SNS network.

The common tool bundle supplies Quill, ICP CLI, didc, ic-wasm and wasm-opt.
The harness uses ICP's managed runtime, so that deployment check does not require PocketIC or
Testkit; the separate deterministic check above does. After adopting Shared 0.2.0, explicitly reinstall the common bundle
before offline checks; prior six-tool bundles remain retained evidence.

The managed runtime is pinned to network launcher **16.0.0** in `icp.yaml`.
The ICP binary selection is independent of this project pin, which keeps
local NNS execution on the existing stable runtime. ICP identities,
settings and launcher packages use the persistent ignored `.icp-smoke-home/`
by default. Receipts, Wasm and bundles live under `target/canister-smoke/`;
runtime state lives in `tests/canister/.icp/`. These directories belong outside
commits and release artifacts. The library's supported minimum Rust version remains **1.91.0**;
`make msrv` verifies it separately from the development toolchain.

### ICP binary selection

Query uses ICP CLI 1.7.0 from the adopted Shared Tooling catalog
`ci/ic-tools.tsv`. Make setup/checks, validation inputs, the direct harness,
release pin admission and retained tool evidence use that same catalog.
`IC_TOOL_PINS` still permits an explicit reviewed catalog selection.

The three ICP archive hashes come from the official 1.7.0 release metadata.
Linux x86-64 installation and offline admission are checked locally; native
macOS Intel/Apple Silicon execution and full local-NNS smoke remain pending
the configured CI matrix. The shared adoption is tracked in
[Shared #268 — Adopt ICP CLI 1.7.0](https://github.com/dragginzgame/shared-tooling/issues/268).

## Identity storage and cleanup

The pinned [ICP CLI 1.7.0 home implementation](https://github.com/dfinity/icp-cli/blob/eda771516212045748dc14933c10148c118dbc2c/crates/icp-app/src/directories.rs)
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
uses the explicit `IC_TOOL_PINS` selection, defaulting to Query’s reviewed
[IC matrix](../ci/ic-tools.tsv), including for direct Cargo invocation. Receipts record
the observed ICP version. Direct invocation needs no global ICP selection.

`canister-build` invokes `icp build -e local` against the isolated project in
`tests/canister/`. It builds the example with only the `canister` feature and
embeds public `ic-query:protocol` and `ic-query:build` metadata. The latter records
the compiler, lockfile hash, and source-input hash. The final Wasm is retained
at `target/canister-smoke/governance_probe.wasm`.

The shared host helper admits core Wasm framing under a 64 MiB artifact limit,
10,000 sections/exports and 1,000 custom sections. The builder checks the admitted
digest again before attaching consumer-owned metadata. Inspection is structural;
it does not replace the compiler or replica's Wasm validation. Direct agent calls
bound response bodies to 8 MiB and admit at most 2 MiB of reply bytes. The helper
saves the original binary reply before admitting one canonical Wire response
for method 1, with no trailing bytes. It returns the text unchanged for report
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
checks the deployed module hash and Wire protocol metadata before collecting,
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
module evidence, build metadata, raw Wire replies, and decoded reports.
Receipts record the active `phase` and `updated_at`; raw replies are saved
before validation, and validated report payloads are added afterward.
Wire admission limits original replies to 2 MiB, with a 24 MiB allocation
allowance, four visited nodes and depth four. Only a canonical method-1 response
text is accepted, with no trailing bytes. Original private reply files survive
every admission failure. JSON report values retain arbitrary-precision numeric
evidence.

Starting with the 0.50 hard cut, schema-1 report entries use `reply_file` for an
adjacent binary Wire file and `report` for validated JSON. Module hashes are
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
on the failed receipt, with one Host-rendered string per typed cleanup attempt;
the original command failure remains separate. A refusal is not successful cleanup. Receipt
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

This uses ICP 1.7.0's `icp project bundle -e mainnet-smoke` to produce
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
from Query’s explicitly selected IC matrix, including ICP CLI 1.7.0 and its
reviewed archive digests. It selects `.tools/ic/bin`, runs the local smoke, and builds
the bundle. It uploads receipts and their adjacent binary Wire replies, the probe Wasm,
and the bundle under `canister-smoke-<host>` for 30 days, including receipts from
failed runs. Identities and runtime state are excluded from uploads.
The newly configured macOS runtime jobs require matching native runs before
claiming qualification.
Retain a reviewed receipt separately if it must outlive that retention window.

The ordinary feature-boundary gate compile-checks the Wasm example. The native
`checks` matrix compiles the runner tests with the CLI's default feature selection
through `make ci-scripts-check`, without executing them. Receipt, interruption,
malformed-reply, provenance and artifact-helper Rust unit tests execute through
`make test`, whose all-target selection includes examples. Each unit suite has
one execution owner in the complete CI gate. `make ci` does not start a local
network; use `make canister-smoke` for the separate runtime gate.

Upstream contracts: [ICP 1.7.0 release notes](https://github.com/dfinity/icp-cli/releases/tag/v1.7.0)
and [configuration reference](https://github.com/dfinity/icp-cli/blob/v1.7.0/docs/reference/configuration.md).

## Shared probe build mechanics

The development harness uses IC Build's named-example Cargo command factory,
standalone input inventory and root-relative captured-evidence digest. The source
fingerprint covers every selected file, directory and missing-path observation,
including non-Rust files. Query selects the required manifests, lockfile, toolchain,
probe DID and source roots. IC Build hashes their captured values using sorted
relative paths; Query validates the inventory again before returning the digest.
Symlink and special-file inputs are refused. No second raw-file hashing pass runs.

The 0.59.0 hard cut replaces the descriptive `source_sha256` contract in schema-1
probe metadata. Generate a new probe build and matching receipt together, and
update any external fingerprint comparison to use the complete captured inventory.
Existing artifacts and receipts remain evidence of their original builds; they
are not rewritten or migrated. The digest is descriptive source evidence, not
compiler or executable attestation.

Cargo target discovery uses Cargo's canonical metadata schema through IC Build
while retaining Query's bounded command executor. IC Build's caller-controlled
resolver admits the exact named `cdylib` example and configured output path before
compilation, then revalidates semantic and raw input identities before Query reads
or publishes the artifact. Each resolution has a 600-second probe deadline, the
harness's cancellation predicate, five-second TERM grace and five-second bounded
reaping; original Host process evidence remains in typed Build errors. The build
command retains its existing 600-second policy. Query owns the selected inventory,
probe Wire contract, metadata and receipts. No artifact cache, deployment or runtime is created.

## Dependency roles and deployment admission

The workspace catalog labels `Production dependencies` and `Tests and tooling
only`. The latter contains `ic-build`, `ic-host-process`, `ic-testkit` and `ic-wire`.
Platform owns required external Candid decoding; the private probe and native
callers use Wire. Testkit retains its own older Platform dependency; no newer
Platform limit/error values are passed across that version boundary.
`serde_cbor`, `libc` and `time` have fixture uses and also occur in the native
CLI production graph, so they are not tooling-only exclusions. Query has no
direct Ciborium declaration or usage. Governance fixtures use Testkit's native
`reply-fixtures` feature with exact replies and explicit update exports. Host
owns binary Wasm encoding and inspection; no WAT parser is selected. Futures'
executor is enabled only for native library tests, while production retains its
joins and stream combinators without selecting a second executor.

The feature-boundary CI gate checks the locked normal/build graph for the native
CLI and reusable canister library's `ic-query --target wasm32-unknown-unknown
--no-default-features --features canister` selection. Both reject explicitly forbidden
tooling packages transitively. The development probe intentionally selects Wire
through a dev dependency; the reusable library has no Wire runtime dependency.
The exclusion list is explicit; role comments are not
machine-validated, so adding another tooling-only catalog entry also requires
updating the gate. These checks establish graph exclusions for those
selections, not every supported target/feature combination or registry availability.

Deployment admission is incomplete: direct ICP build/deploy does not require a
successful exclusion check bound to the same source, lockfile, target/features
and final Wasm digest. The harness compares deployed module/metadata evidence,
but that does not bind a dependency-graph receipt. A future deployment gate should
carry that evidence through the existing build/receipt flow before claiming this
property; catalog comments and source-level dependency checks are not enforcement
at the deployment boundary. The Governance probe is development tooling, not a
production canister supplied by Query.

## Platform adoption qualification

The selected published `ic-platform` release is 0.2.1. Its source and feature
contracts were inspected from the downloaded registry package; sibling source
is not substituted. The runtime and macro bodies are unchanged from 0.2.0;
0.2.1 clarifies owned Wire endpoint policy and runtime maintenance guidance.

Platform supplies the upstream Candid, certification, Principal and native Agent
identities. Host retains strict bounded CBOR decoding. Query retains external
Governance Candid arguments, byte-dependent work quotas, skipping/type/header
ceilings, error projection, trust roots, deadlines and original reply evidence.
There are no local generated Candid bindings; Registry Protobuf is a separate
external wire boundary. Portable/canister graphs exclude native parser, Agent
and ledger DTOs; every supported production graph rejects stable-memory storage.

Platform now owns the non-cryptographic runtime adapters, call/response and
error identities. Its `cdk` feature is Candid-free, while Query selects Candid
separately for required external NNS methods. The full `ic-cdk` and its macros
are excluded from production graphs. There is one registry executor 2.0.0 and
ic0 1.2.0 identity; Query does not copy executor or callback implementations.
The adapter retains explicit Candid unit arguments, bounded waits, raw replies,
byte limits and replicated provenance. The owned probe's explicit Wire hooks
admit request length before copying and trap malformed requests before effects.

The smoke harness and deterministic Testkit fixture use raw Wire calls to the
owned probe. External Governance fixture replies remain Candid. The new
`ic-query:protocol` metadata and `.wire` evidence are a hard cut; existing evidence
is preserved without fallback readers. Testkit still owns simulator provisioning
and lifecycle. A compiled probe or native unit tests do not qualify its runtime:
run `make canister-test` with the selected prepared Testkit server. Installed-IC
callback/trap and native macOS qualification remain separate obligations under
[Platform #1](https://github.com/dragginzgame/ic-platform/issues/1).

`make dependency-boundary-check` uses Shared Tooling's canonical Cargo metadata
checker, including aliases and normal/build/dev/target-specific declarations.
It rejects direct upstream Candid, CDK, certification, Agent, ledger DTO and macro
dependencies without package exceptions. Normal/build graph checks separately
exclude retired CDK packages and unsupported Host features. They do not validate
role comments or bind a receipt to the final deployed Wasm. The private probe
intentionally uses the development-only Wire codec; the reusable library has no
Wire runtime dependency. Neither graph inspection nor package availability is
installed-canister qualification.
