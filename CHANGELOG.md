# Changelogs

Each independently versioned release group owns its release notes. This file is
the navigation index with quick starts for changed CLI workflows; there is no
aggregate workspace release or root release ledger.

| Group | Package | Release notes |
| --- | --- | --- |
| Topology | `ic-topology` | [Topology changelog](releases/topology/CHANGELOG.md) |
| SNS migration | `ic-sns-migration` | [SNS migration changelog](releases/sns-migration/CHANGELOG.md) |
| CLI | `ic-query-cli` (`icq`) | [CLI changelog and copyable command examples](releases/cli/CHANGELOG.md) |

Group ownership is declared in [ci/cargo-release-groups.json](ci/cargo-release-groups.json).
See the [restart design](docs/design/0.104-restart.md) for version continuity and
consumer-retirement obligations. Published legacy history remains in the
[legacy repository](https://github.com/dragginzgame/ic-query-legacy).

## CLI 0.105.3 — GitHub delivery 2026-11-14

### ▶️ Try it

Inspect registered subnet configuration, compare it with separately observed
node keys, and read one node's registered configuration. Run from the `ic-query`
repository root with the Rust/tool prerequisites
in [setup](docs/development/setup.md). Dependencies come from the registry;
no sibling checkout is required.

Prepare dependencies once, then install or refresh the checkout's `icq` on PATH:

```bash
make fetch-dependencies
make icq
```

Fetching needs network access; installation replaces the local Cargo-installed
`icq`. Normal use needs mainnet network access, with no credentials or on-chain
mutations:

```bash
icq topology subnet configuration tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
icq topology subnet membership tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
icq topology node show nuass-yihti-wszdr-r7qze-32dgz-osovf-pracd-hg5tt-mwlhf-rvuyq-4ae --json
```

The first prints Registry membership, configured software and halt flags with
source evidence. The second retains both source reports and node-set differences;
matching sets do not establish live nodes or an atomic snapshot. The third prints
the node's operator, optional raw chip bytes and configured HostOS, with
attestation explicitly unverified. See the
[CLI group notes](releases/cli/CHANGELOG.md) for details and text-output examples.
