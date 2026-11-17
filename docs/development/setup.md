# Development setup

The root virtual workspace contains `ic-topology`, `ic-sns-migration` and
`ic-query-cli` (`icq`). The topology library supplies bounded mainnet
[placement](../design/canister-placement.md) and
[subnet observation](../design/subnet-observation.md) reports, plus
[registered subnet inventory](../design/subnet-list.md) and
[certified resource metrics](../design/subnet-metrics.md) and
[two-sample activity](../design/subnet-activity.md). The independent
[migration library](../design/migration-evidence.md) reads ICRC balances and
selected SNS neurons.
CLI process tests exercise local help, version and rejected arguments without
network calls. Running report commands does access mainnet.

All topology reads select published Platform 0.34.10 `native-read`, Protocol 0.3.0
for Registry/placement/cloud-engine codecs, and Types 0.21.2. Product and test
graphs no longer select legacy Platform or Agent. The topology tests replay a
[portable corpus](../../crates/ic-topology/src/tests/README.md), packaged with
the library; they do not need a sibling checkout or direct Candid dependency.
The CLI selects published `ic-host-text` 0.77.2 for bordered tables and terminal-safe
Unicode alignment; report libraries remain independent of presentation dependencies.

The requested Platform 0.34.10 / Protocol 0.3.0 adoption passed all 204 workspace
tests, strict Clippy for each affected package, Rustdoc, formatting, local links
and tooling admission on Linux x86-64 on 2026-11-17. Each library's independent
feature graph retains one Platform/Protocol/Types cohort. Protocol's management
contract changes are unused here; public topology Protocol payload identities
nevertheless require the pending topology 0.109.0 release and caller dependency
alignment. CLI syntax and report JSON are unchanged. This adoption did not rerun
live reads or establish new macOS qualification; the earlier live evidence below
used Platform 0.34.9 and Protocol 0.2.8.

The migration implementation passed `make check` on Linux x86-64 on 2026-11-16:
204 tests (139 topology, 24 CLI unit, 28 CLI process, 13 migration), strict Clippy,
Rustdoc, formatting, local links and tooling admission. This evidence describes
the working-tree implementation based on `ca8f66c202c053d686f1cab0ec1d159b778485d7`,
not a delivered release or new hosted macOS qualification. Separate feature-graph
checks confirm topology selects no SNS/ledger capability and migration selects
no topology/Registry capability. Cargo's package file list includes the migration
root key and six frozen consumer fixtures; this is not archive build verification.

Bounded live Dragginz checks read the selected anonymous ICRC account (1000 base
units, 8 decimals, symbol DKP), one exact neuron from the returned list, 200 neurons
across two pages, a separate explicit-cursor continuation and an empty exhausted
anonymous-principal filter. Successful responses carried verified query signatures;
full page budgets returned partial status as specified. These November 16 observations
do not establish current balances, atomic snapshots or complete migration coverage.
Toko-specific live checks and migration-contract macOS execution remain unqualified.
See the [migration contract](../design/migration-evidence.md) for field omissions,
deadlines and the distinction between query authentication and application certification.

## Preparation

Use Bash, GNU Make, Git, Perl with core modules and a SHA-256 backend. Install the
exact Rust 1.100.0 toolchain with rustfmt and Clippy from
[rust-toolchain.toml](../../rust-toolchain.toml), then prepare tools and dependencies:

```bash
rustup toolchain install 1.100.0 --profile minimal --component rustfmt --component clippy
make install-tools
make fetch-dependencies
```

Preparation may use the network and shared user tool store. Validation is locked
and offline; it does not install tools or activate hooks. `make install-host-tools`
prepares the smaller host-tool set, including `cloc`, when only LOC reporting is
needed. Existing global `toko-cloc` launchers select this checkout's reporter;
without a launcher, use `bash vendor/toko-tooling/scripts/dev/toko-cloc -d 0`.

## Certificate-time supplier

The selected delivery is topology 0.108.0 and CLI 0.105.10 to crates.io
and GitHub. Registry inspection on 2026-11-16 confirmed non-yanked Platform
0.34.7, Protocol 0.2.6 and Types 0.21.2. That delivery selected these registry versions;
no sibling checkout is required. The unchanged SNS shell is outside this batch.

Query and the CLI share published Types 0.21.2 with Protocol and certificate
authentication. Legacy Types 0.20.4, Platform 0.33.7, Agent and their identity/class
bridges are retired. The optional canister-signature and native-signed features
are not selected. The thiserror requirement remains compatible at 2.0.18.

The current hard cut adapts the supplier's delivered patch and 313-frame corpus
at revision `53efb18ccd31f4ed7ec6f611488fe27820fbd0ef`, resolving the fixture gap
in [Platform #10](https://github.com/dragginzgame/ic-platform/issues/10).
Query owns the fixed endpoint, pinned mainnet root/subnet, 20-second whole-read
deadline, 1 MiB message limit and 300-second ingress expiry. Platform authenticates
under explicit 300-second past/60-second future windows for certificate and
delegation timestamps. Redirects, proxies and transport retries are disabled.
Progress retains its stricter 4096-node final verification budget and distinct
signing/delegation times. Other native reads allow 16384 tree nodes, depth 64,
128 KiB delegation bytes and 4096 canister ranges. Retained reports stay schema 1;
public Rust supplier/error identities require consumer migration at topology 0.108.0.

On 2026-11-16 the hard cut passed 187 topology/CLI tests (139 topology, 22 CLI
unit and 26 process), strict all-target Clippy and Rustdoc with denied warnings
on Linux x86-64. Both package archives were created with `cargo package --locked
--offline --allow-dirty --no-verify`; inspection confirmed the topology archive
includes the corpus, replay helper and pinned root. Archive creation is not
archive build verification or publication. The product graph selects exactly
one Platform/Protocol/Types cohort and no Agent; no private fixture or direct
Query Candid dependency remains. Upstream transport dependencies remain owned
by Platform. The pinned root was byte-compared with the previous transport's
mainnet trust key. A live node-signature-verified query resolved the ICP ledger to
the NNS subnet at Unix second 1794861868, without claiming result certification.
Two live root-subnet progress observations advanced by 873878127 nanoseconds;
both passed certificate-time admission. These are bounded smoke observations,
not uptime evidence. Formatting, local links and tooling admission also passed.
macOS and publication qualification remain separate.

The following validation and probe records describe earlier selections, not the
current hard-cut qualification.

The registry adoption passed the complete local Linux validation stages on
2026-11-16: strict all-target Clippy, Rustdoc, 184 tests, formatting, local
documentation links and snapshot/Cargo admission. This qualifies the registry
graph and offline report contracts; hosted macOS checks and live network reads
remain separate evidence. Registry metadata contained no external path suppliers.
That validation used Host Text 0.76.0 and Toko Tooling 0.44.5. The subsequent
requested tooling adoption selects Host Text 0.77.2 and Toko Tooling 0.44.8;
its focused CLI and tooling checks are separate from that earlier release gate.
The adoption passed all 48 CLI unit/process tests, strict all-target CLI Clippy,
formatting, documentation links and snapshot/Cargo admission on Linux x86-64.
The supported updater also admitted the selected host and Rust developer tools.

Prepared topology 0.108.0 and CLI 0.105.10 archives passed a combined
`cargo publish --dry-run --locked --allow-dirty -p ic-topology -p ic-query-cli`
on 2026-11-16, including verification of the CLI against the packaged topology
crate. Final-version CLI process tests and documentation admission also passed.
No upload or Git delivery occurred in that earlier preparation. The current
[release adapter](releases.md) now exposes the required shared release and
publication entrypoints; final source and archive admission occur during delivery.

The subsequent registry update to Platform 0.34.6, Protocol 0.2.5 and Types
0.21.2 passed all 184 topology/CLI tests, strict all-target Clippy, formatting,
documentation links and snapshot/Cargo admission on Linux x86-64 on 2026-11-16.
Only these three supplier packages changed in the lockfile. Adoption remains
partial: legacy Platform 0.33.7 still supplies transport, remaining readers and
fixture encoding, and the portable-fixture issue below remains open.

The earlier Platform 0.34.5 / Protocol 0.2.4 hard-cut probe compiled topology
and the CLI against registry packages on Linux x86-64, with one Types 0.21.1 and
no Agent or legacy Platform edge. It adapted the supplier's
[consumer patch and qualification](https://github.com/dragginzgame/ic-platform/blob/1e92703bc803b81dbe1e3d815fd1077d440d62b4/docs/qualification/query-hard-cut.md)
in disposable validation storage only; the working implementation is not migrated.
The supplier patch requires its unpublished `engine-view-consumer` fixture crate
for availability, prices, engine and placement tests. Removing that sibling
dependency makes test compilation fail with unresolved fixture imports. Completing
the cut requires portable fixture inputs or a published supplier fixture API,
tracked in [Platform #10 — Provide portable fixtures for Query's registry-only Platform hard cut](https://github.com/dragginzgame/ic-platform/issues/10),
then qualification here; the supplier's passing local-path run does not establish
registry-only consumer qualification. Do not retain the legacy dev dependency,
import a direct Candid dependency or remove regression coverage to bypass this gap.

The earlier local identity migration passed 184 workspace tests and strict all-target Clippy
on Linux x86-64. The supplier's subsequent 0.34.1 version alignment is requalified
through focused progress tests and Rustdoc. This does not qualify the moving
sibling checkout on macOS or for delivery. Existing report schemas and raw
principal bytes are preserved; Rust identity consumers must align Types and rebuild.
Live read-only checks with local Platform 0.34.1 resolved the ICP ledger to the
NNS subnet and authenticated two advancing NNS state times on 2026-11-16. These
check the legacy Registry/new certificate boundaries, not all live report sources.

## Daily commands

```bash
make help
make icq
make deps-list
make deps-update
toko-cloc -d 0
make qualify-cli
make test PACKAGE=ic-topology
```

`make icq` builds and installs this checkout's CLI in Cargo's installation bin
directory (normally `~/.cargo/bin`), replacing an existing Cargo-installed `icq`.
It uses the development profile and prepared, locked dependencies, then checks
that `icq` is available on PATH. Cargo installation-root overrides are respected;
keep the selected bin directory on PATH. Run `icq --help` or pass report arguments
directly. Installation is offline; report commands still access mainnet.

`deps-list` lists local workspace package versions, not transitive dependencies
or registry publication status. The group column reads the ownership declared in
[`ci/cargo-release-groups.json`](../../ci/cargo-release-groups.json). Release
automation remains deferred; the maintainer has enabled publication. `deps-update`
runs the shared `cargo update -v`
workflow; it intentionally changes lockfiles and may access the registry. To select
one dependency, use `make deps-update DEPENDENCY_PACKAGE=clap`. Review direct
supplier freshness separately: a lockfile update does not widen manifest ranges.

The tooling source is pinned exactly in the root catalog and inherited as an
inactive-target dev-dependency by every member, with the prescribed machete
exemption. When adopting a new tooling release, update that pin and lockfile,
then run `make toko-tooling-update`; it refreshes the authenticated snapshot.

## Focused validation and qualification

```bash
make qualify-cli
make clippy PACKAGE=ic-topology
make rustdoc PACKAGE=ic-topology
make test PACKAGE=ic-query-cli
make fmt-check
make doc-links-check
make toko-tooling-check
```

Use the affected package or CLI test target during ordinary development.
`make check` / `make ci` are aggregate gates: snapshot/Cargo admission, local links,
formatting, Clippy, Rustdoc and tests. `PACKAGE=name` narrows Cargo compilation,
documentation and test selection; formatting and documentation-link checks remain
repository-wide. All three initial packages are covered by the default aggregate.
Warnings are errors in the Rust gates. `make fmt` applies the shared formatter.

CLI integration tests belong in `crates/ic-query-cli/tests/`. Add source fixtures
beside the owning library as reports are implemented. No separate qualification
package is justified yet: when a real external consumer, simulator, benchmark or
other test-only package is needed, put it under `qualification/`, create its own
workspace and lockfile, and exclude it from the product workspace. The shared
`deps-list`, `deps-update` and LOC reporter already discover that standard location.
At that point extend fetch, formatting and validation to the second workspace.

CI checks the implemented packages on Linux x86-64 and macOS ARM64/x86-64. Formatting and a
single pinned Gitleaks history scan run separately from the host matrix. Every job
admits snapshot bytes before preparing host tools and Rust, then admits the Cargo
pin, members and lockfile before validation.

[CI run 43704031732](https://github.com/dragginzgame/ic-query/actions/runs/43704031732)
passed on 2026-11-15 for commit `263a8e92da873543825c25e7dd6530c4db5ab673`:
Clippy, Rustdoc and tests succeeded on all three hosts; formatting, snapshot and
documentation admission, and the secret scan also passed. This qualifies the
implemented offline contracts at that commit, including distribution, cloud-engine
reports, maintenance and two-subnet comparison, with Toko Tooling 0.44.3.
It does not qualify the subsequently implemented subnet-settings report or later changes.
The local Linux working tree subsequently passed the full `make check` gate on
2026-11-15 for registered settings and optional comparison settings: 175 tests,
Clippy, Rustdoc, formatting, snapshot admission and local links. This used the
Protocol pin recorded above and existing package versions, with pending topology
0.107.0 / CLI 0.105.9 notes. It is neither hosted macOS qualification nor GitHub
delivery of those candidate versions.
For the 2026-11-16 delivery, the full local Linux gate passed with the immutable
Platform selections: 181 tests, Clippy, Rustdoc, formatting, snapshot admission
and documentation links. Final topology 0.107.0 / CLI 0.105.9 metadata then passed
CLI process qualification and dependency/format/link admission. Cargo metadata
contains no external path dependency. Native macOS CI remains a separate result.
Broader protocol, retained-data
migration and publication qualification remain future work tied to implemented contracts.
Placement fixtures qualify decoding, projection, bounds and failures; they do not
prove cryptographic verification or live network availability. Run any live smoke
check explicitly and record its endpoint and result separately from offline tests.

## Tooling provenance

Independent groups own their release notes in `releases/<group>/CHANGELOG.md`;
root `CHANGELOG.md` is the navigation index and includes a short `▶️ Try it`
section identifying the affected CLI group/version. Keep setup, runnable commands
and expected results directly in that root quick start, with detailed examples
in the [CLI ledger](../../releases/cli/CHANGELOG.md), using actual public IDs.
This follows the installed baseline and its pinned
[group changelog policy](https://github.com/dragginzgame/toko-tooling/blob/626da7cd9626cc2e152594862a2d82ea82ef51da/rules/changelogs.md#release-group-changelogs-and-shared-history).

Toko Tooling **0.44.11** was selected from crates.io on 2026-11-16 through
the supported updater. Its authenticated archive SHA-256 is:

```text
7d16b815ae322ae3f88580ee6af1611bdafde08b77592cd886bfa0eece0f1be8
```

The [receipt](../../.toko-tooling.snapshot) records the thirteen selected roots and
all required executable companions: baseline, canonical root files, CI tool
installer, documentation link checker, download-cache action, common Make
interface and Cargo dependency Make interface. The common interface brings the
LOC reporter. Reference documents and producer regression suites stay upstream.
The 0.44.11 release-tooling selection accounts for all 77 installed payload files.
Release/publication runners, source/receipt admission and package provenance
checks were added through the supported exporter for the authorized delivery.
The earlier reviewed
preview updates the baseline and keeps the executable payload unchanged; the
supported updater owns the receipt. Versions 0.44.10–0.44.11 add setup-failure
diagnostic requirements and repository-qualified issue references. Local issue
labels now identify their owning repositories. The existing CI workflow still
needs bounded, sanitized setup-failure artifact collection; installing this
baseline does not establish that hosted evidence-retention behavior.
The baseline also clarifies asynchronous hosted
CI for delivery, one-tier qualification package layout and independent compatible
thiserror selection. Query has no separate qualification packages, and its
thiserror requirement is compatible rather than an exact fleet-wide hold.
Version 0.44.9 makes named-supplier adoption cover product, dev/build and
qualification callers by default. Query's maintained manifests and lockfile
were inventoried; no independent qualification workspace exists. Platform
adoption was partial at that revision; the current hard cut above adopts the
fixture delivery from Platform #10.
The working-tree update to Platform 0.34.5, Protocol 0.2.4 and Toko Tooling
0.44.9 passed 184 topology/CLI tests, strict all-target topology Clippy,
formatting, documentation links and snapshot/Cargo admission on Linux x86-64.
It still selects Platform 0.33.7 for transport, remaining readers and fixture
encoding; the earlier registry-only hard-cut probe is separate evidence.
The LOC reporter now defaults to repository totals: `toko-cloc` and `toko-cloc -d 0`
produce the same report; use `toko-cloc -d 1` to expand crates and `-d 2` for
top-level modules. Named crate/module selections expand immediate children at `-d 1`.
Local verification compared both default-depth outputs byte-for-byte successfully.
This checkout has no imports or
snapshot selection for the retired Python evidence API, so no caller migration
is needed. The exporter refreshed the receipt and admitted installed host/Rust
tools. The Cargo pin, lockfile and receipt identify the authenticated release.
Version 0.44.2 adds the baseline's focused post-push review before the next
increment. Use the delivered commits and directly affected callers, retain
existing validation evidence and record actionable findings in existing owning
records. It adds no separate audit framework or automatic follow-up push.
Version 0.44.3 changes only the selected LOC reporter and classifier; the baseline
is unchanged from 0.44.2. Its depth correction leaves JSON counts unchanged.
Version 0.44.4 adds package-directory/name alignment and copyable long-command
continuations to the baseline; executable payload files are unchanged. All three
maintained packages already match their directories, the root is virtual, and
the CLI examples follow the continuation convention. No package moves or product
API changes were required. The supported updater refreshed the authenticated
snapshot and admitted the installed host and Rust tools.

See the release-pinned
[consumer adoption guide](https://github.com/dragginzgame/toko-tooling/blob/69ea17e551f33254d87de9824c30f2e74536eb10/docs/adoption/consuming-snapshots.md)
and [Make interface](https://github.com/dragginzgame/toko-tooling/blob/69ea17e551f33254d87de9824c30f2e74536eb10/docs/adoption/make-interface.md).
Refresh through the authenticated exporter with the complete retained entrypoint
set and review its preview. Never edit managed files or the receipt by hand.
