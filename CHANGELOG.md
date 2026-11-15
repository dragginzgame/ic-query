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

## CLI 0.105.6 — GitHub delivery 2026-11-15

### ▶️ Try it

After the [setup prerequisites](docs/development/setup.md), install this checkout
and compare two subnets or observe one over two rounds (network access required):

```bash
make icq
icq topology subnet compare tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe brlsh-zidhj-3yy3e-6vqbz-7xnih-xeq2l-as5oc-g32c4-i5pdn-2wwof-oae
icq topology subnet compare tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe brlsh-zidhj-3yy3e-6vqbz-7xnih-xeq2l-as5oc-g32c4-i5pdn-2wwof-oae --json
icq topology subnet maintenance tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --interval-seconds 10
icq topology subnet maintenance tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --interval-seconds 10 --json
```

Optional `--expected-version` checks exact configured text, including explicit
empty text. Partial evidence or refused comparisons exit 1; mismatches and observed
changes exit 0 when collection/comparison succeeds. No uptime or completion verdict.
`compare` shows two selected subnets' configuration, provider/location coverage and
resource observations. It retains independent Registry versions and emits partial
evidence with exit 1; it does not rank capacity or establish merge feasibility.

## CLI 0.105.5 — GitHub delivery 2026-11-15

### ▶️ Try it

Read cloud-engine candidate nodes and daily prices, provider/location concentration and selected-node
registration changes. From the
repository root after the [setup prerequisites](docs/development/setup.md):

```bash
make fetch-dependencies
make icq
icq topology cloud-engine show engine-0b56b29a-7ea0-40b8-a963-6c4539ea4414
icq topology cloud-engine show engine-0b56b29a-7ea0-40b8-a963-6c4539ea4414 --json
icq topology cloud-engine candidates --max-pages 20
icq topology cloud-engine candidates --max-pages 20 --json
icq topology cloud-engine available
icq topology cloud-engine available --json
icq topology cloud-engine available --max-pages 20 --json
icq topology cloud-engine prices
icq topology cloud-engine prices --json
icq topology subnet distribution tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe
icq topology subnet distribution tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
icq topology node changes nuass-yihti-wszdr-r7qze-32dgz-osovf-pracd-hg5tt-mwlhf-rvuyq-4ae --interval-seconds 10
icq topology node changes nuass-yihti-wszdr-r7qze-32dgz-osovf-pracd-hg5tt-mwlhf-rvuyq-4ae --interval-seconds 10 --json
```

Requires network access for fetching and mainnet reads. Text shows compact
provider/location counts and unknown totals; JSON retains version-pinned joins,
raw evidence and failures. Distribution partial results exit 1. Node comparison
shows only changed fields and exits 0 even if unchanged; read/comparison failure
exits 1. Chip registration and configured HostOS do not establish usable TEE
protection or running software. Registry declarations do not
prove independent failure domains, uptime or spare capacity. See the
[CLI notes](releases/cli/CHANGELOG.md) for limits and coverage fields.
Cloud-engine prices are exact whole cycles per node per day. Gross already
includes its fee; a separate fee-read failure retains the prices with exit 1.
Available-node text groups candidates by class/data center. JSON retains nodes
and page evidence; partial traversal exits 1. Exhaustion does not prove an atomic
inventory, reserved capacity or verified TEE protection.
Use `--max-pages` (1–100, default 10) for a larger bounded collection; 20 pages
allows up to 2,000 nodes under the same 60-second deadline.
These reports use `ic-host-text` bordered tables with visible column delimiters.
`candidates` matches observed nodes to all public declaration scopes. Missing and
overlapping matches stay explicit; no tariff or quote is selected. It retains
partial results with exit 1 and uses sequential availability/price budgets of
60/30 seconds.
`cloud-engine show` reads one selected engine in one bounded query, showing its
subnet/operator and separate engine-record/Registry-copy node lists. Successful
omission means unknown or hidden, not confirmed nonexistence. No liveness or
Registry-copy freshness is inferred. This batch adopts Platform 0.33.7 and Toko
Tooling 0.44.1.

The updated LOC reporter defaults to crate rows. These first two commands now
produce the same report; the third expands modules:

```bash
toko-cloc
toko-cloc -d 0
toko-cloc -d 1
```

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
