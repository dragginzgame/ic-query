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

## CLI 0.105.4 — GitHub delivery 2026-11-14

### ▶️ Try it

Read the registered node inventory for one subnet. From the repository root with
the Rust/tool prerequisites in [setup](docs/development/setup.md):

```bash
make fetch-dependencies
make icq
icq topology subnet nodes tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe
icq topology subnet nodes tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
icq topology subnet changes tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --interval-seconds 10
```

Fetching and anonymous mainnet reads need network access. Installation refreshes
`icq` on PATH. Text shows a compact node table with operator/HostOS keys and
chip-ID presence; JSON retains full configuration, chip IDs and per-node evidence.
Both identify failed or unfinished reads.
Collection is limited to 128 nodes, batches of four and 60 seconds. Partial output
is retained on stdout with exit 1; inspect `all_node_reads_succeeded` and each row.
Successful reads do not prove live nodes or an atomic snapshot. See the
[CLI group notes](releases/cli/CHANGELOG.md) for the detailed contract and text example.

The `changes` command reads configuration twice, waiting ten seconds after the
first read. It shows changed fields and membership differences, or reports no
selected changes. Add `--json` for both original reports and their evidence.
Two observations do not establish complete change history or maintenance completion.

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
