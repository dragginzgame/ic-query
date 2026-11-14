# Development setup

The root virtual workspace contains `ic-topology`, `ic-sns-migration` and
`ic-query-cli` (`icq`). The topology library supplies bounded mainnet
[placement](../design/canister-placement.md) and
[subnet observation](../design/subnet-observation.md) reports, plus
[registered subnet inventory](../design/subnet-list.md) and
[certified resource metrics](../design/subnet-metrics.md) and
[two-sample activity](../design/subnet-activity.md); SNS remains an API-free shell.
CLI process tests exercise local help, version and rejected arguments without
network calls. Running report commands does access mainnet.

The [configuration integration](../design/subnet-configuration.md) and
[membership comparison](../design/subnet-membership.md) use published Platform
0.33.3 and Types 0.20.4, also supplying the
[node report](../design/node-inventory.md). The temporary sibling-path selections have been removed;
dependency preparation requires no sibling checkout.

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
automation remains deferred and publication disabled. `deps-update`
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

[CI run 43611996586](https://github.com/dragginzgame/ic-query/actions/runs/43611996586)
passed on 2026-11-14 for commit `e8da5a9966732009a7eef3afa5f1b5f4a24d854b`:
Clippy, Rustdoc and tests succeeded on all three hosts; formatting, snapshot and
documentation admission, and the secret scan also passed. This qualifies the
implemented offline contracts at that commit, through selected-node inventory;
it does not qualify the subsequent subnet node collection addition. Broader protocol, retained-data
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

Toko Tooling **0.43.1** was verified as the latest stable, non-yanked registry
release on 2026-11-14. Its authenticated archive SHA-256 is:

```text
467162f616caafe69c965fe4e96173a0df37256f4767bfefd28114781aad76b3
```

The [receipt](../../.toko-tooling.snapshot) records the eight selected roots and
all required executable companions: baseline, canonical root files, CI tool
installer, documentation link checker, download-cache action, common Make
interface and Cargo dependency Make interface. The common interface brings the
LOC reporter. Reference documents and producer regression suites stay upstream.

See the release-pinned
[consumer adoption guide](https://github.com/dragginzgame/toko-tooling/blob/b50aeb42a9e23fca22f08bb0c31cd19b728e8591/docs/adoption/consuming-snapshots.md)
and [Make interface](https://github.com/dragginzgame/toko-tooling/blob/b50aeb42a9e23fca22f08bb0c31cd19b728e8591/docs/adoption/make-interface.md).
Refresh through the authenticated exporter with the complete retained entrypoint
set and review its preview. Never edit managed files or the receipt by hand.
