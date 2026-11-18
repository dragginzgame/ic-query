# Changelogs

Each independently versioned release group owns its release notes. This file is
the navigation index with quick starts for changed CLI workflows; there is no
aggregate workspace release or root release ledger.

| Group | Package | Release notes |
| --- | --- | --- |
| Query umbrella | `ic-query` | [Query changelog and legacy migration](releases/query/CHANGELOG.md) |
| Topology | `ic-query-topology` | [Topology changelog](releases/topology/CHANGELOG.md) |
| SNS migration | `ic-query-sns` | [SNS changelog](releases/sns-migration/CHANGELOG.md) |
| CLI | `ic-query-cli` (`icq`) | [CLI changelog and copyable command examples](releases/cli/CHANGELOG.md) |

Group ownership is declared in [ci/cargo-release-groups.json](ci/cargo-release-groups.json).
See the [restart design](docs/design/0.104-restart.md) for version continuity and
consumer-retirement obligations. Published legacy history remains in the
[legacy repository](https://github.com/dragginzgame/ic-query-legacy).

## CLI 0.105.21 — 2026-11-18

### ▶️ Try it

With Cargo and network access:

```bash
cargo install ic-query-cli --version 0.105.21 --locked
icq --version
```

Commands and reports are unchanged. See the [CLI ledger](releases/cli/CHANGELOG.md)
for reduced release-admission overhead.

## CLI 0.105.18 — 2026-11-17

### ▶️ Try it

With Cargo and network access:

```bash
cargo install ic-query-cli --version 0.105.18 --locked
icq --version
```

Commands and reports are unchanged. See the [CLI ledger](releases/cli/CHANGELOG.md)
for release-validation changes.

## CLI 0.105.17 — 2026-11-17

### ▶️ Try it

With Cargo and network access, install the CLI maintenance release:

```bash
cargo install ic-query-cli --version 0.105.17 --locked
icq --version
```

See the [CLI ledger](releases/cli/CHANGELOG.md) for test-layout and lockfile notes.

## CLI 0.105.16 — 2026-11-17

### ▶️ Try it

With Cargo and network access, install the qualified supplier update:

```bash
cargo install ic-query-cli --version 0.105.16 --locked
icq --version
```

Commands and reports are unchanged. See the [CLI ledger](releases/cli/CHANGELOG.md).

## CLI 0.105.15 — 2026-11-17

### ▶️ Try it

With Cargo and network access, install the CLI composing the
renamed specialist crates; commands are unchanged:

```bash
cargo install ic-query-cli --version 0.105.15 --locked
icq --version
```

Library users can select the [new `ic-query` umbrella](releases/query/CHANGELOG.md)
or either specialist directly.

## CLI 0.105.14 — 2026-11-17

### ▶️ Try it

With Cargo and network access, install the dependency-range fix:

```bash
cargo install ic-query-cli --version 0.105.14 --locked
icq --version
```

Commands and reports are unchanged. See the [CLI ledger](releases/cli/CHANGELOG.md).

## CLI 0.105.13 — 2026-11-17

### ▶️ Try it

With Cargo and network access, install the supplier patch update:

```bash
cargo install ic-query-cli --version 0.105.13 --locked
icq --version
icq icrc balance zfcdd-tqaaa-aaaaq-aaaga-cai 2vxsx-fae --json
```

Commands and report contracts are unchanged. See the [CLI ledger](releases/cli/CHANGELOG.md).

## CLI 0.105.12 — 2026-11-17

### ▶️ Read migration evidence

From a prepared checkout with network access, read one Dragginz ICRC account or
one bounded SNS neuron page. A full page exits 1 with useful partial output:

```bash
cargo run --locked --offline -p ic-query-cli -- icrc balance \
  zfcdd-tqaaa-aaaaq-aaaga-cai 2vxsx-fae --json
cargo run --locked --offline -p ic-query-cli -- sns neuron list \
  zqfso-syaaa-aaaaq-aaafq-cai --json
```

See the [CLI ledger](releases/cli/CHANGELOG.md) for exact-neuron and
principal-filter examples, limits and exit statuses.

## CLI 0.105.11 — 2026-11-16

### ▶️ Try it

From a prepared checkout, help now identifies IC Query and the build's version
at every command depth. These commands make no network reads:

```bash
cargo run --locked --offline -p ic-query-cli -- --help
cargo run --locked --offline -p ic-query-cli -- topology subnet --help
```

See the [CLI ledger](releases/cli/CHANGELOG.md) for the change.

## CLI 0.105.10 — 2026-11-16

### ▶️ Try it

With Rust 1.100.0 or newer and network access, install from crates.io:

```bash
cargo install ic-query-cli --version 0.105.10 --locked
icq topology subnet progress \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe \
  --interval-seconds 10 --json
```

No sibling checkout is required. The command reads mainnet and returns two
authenticated state-time observations, preserving partial failures. It makes
no uptime or maintenance-completion claim. Rust library consumers must migrate
to Types 0.21.2 for [topology 0.108.0](releases/topology/CHANGELOG.md).

## CLI 0.105.9 — GitHub delivery 2026-11-16

### ▶️ Observe certified state-time progress

From this repository, run `make fetch-dependencies` and `make icq`. Dependency
preparation fetches the pinned Platform revisions; mainnet reads need network access.
The command retains two state times; it does not pronounce uptime or maintenance completion.

```bash
icq topology subnet progress \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe \
  --interval-seconds 10 --json
```

### ▶️ Compare registered settings

After `make icq`, include independent settings evidence for both candidates.
Mainnet access is required; omit `--json` for bordered tables.

```bash
icq topology subnet compare \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe \
  brlsh-zidhj-3yy3e-6vqbz-7xnih-xeq2l-as5oc-g32c4-i5pdn-2wwof-oae \
  --settings --json
```

### ▶️ Inspect registered settings

After setup, run `make icq`; these reads require mainnet network access.

```bash
icq topology subnet settings \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe
icq topology subnet settings \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
```

## CLI 0.105.8 — GitHub delivery 2026-11-15

### ▶️ Try it

After the [setup prerequisites](docs/development/setup.md), install this checkout
and inspect explicitly selected mainnet canisters:

```bash
make icq
icq topology canister placements \
  ryjl3-tyaaa-aaaaa-aaaba-cai \
  rwlgt-iiaaa-aaaaa-aaaaa-cai
icq topology canister placements \
  ryjl3-tyaaa-aaaaa-aaaba-cai \
  rwlgt-iiaaa-aaaaa-aaaaa-cai --json
```

Up to 32 unique IDs share a 60-second budget. Partial output exits 1 and preserves
failed, cancelled and unstarted lookups. The examples are public IDs, not inferred
fleet membership. See the [CLI notes](releases/cli/CHANGELOG.md).

## CLI 0.105.7 — GitHub delivery 2026-11-15

### ▶️ Try it

After the [setup prerequisites](docs/development/setup.md), compare shared
registered infrastructure (network access required):

```bash
make icq
icq topology subnet compare \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe \
  brlsh-zidhj-3yy3e-6vqbz-7xnih-xeq2l-as5oc-g32c4-i5pdn-2wwof-oae \
  --overlap-details
icq topology subnet compare \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe \
  brlsh-zidhj-3yy3e-6vqbz-7xnih-xeq2l-as5oc-g32c4-i5pdn-2wwof-oae
icq topology subnet compare \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe \
  brlsh-zidhj-3yy3e-6vqbz-7xnih-xeq2l-as5oc-g32c4-i5pdn-2wwof-oae --json
```

The bordered overlap summary shows node fractions over full selected membership,
separate provider/location coverage and unknown nodes. JSON retains shared identities and per-side node counts. No additional
reads are made; zero observed overlap does not prove independent failure domains.
Use `--overlap-details` for shared-identity text tables, or `--json` for raw evidence.

## CLI 0.105.6 — GitHub delivery 2026-11-15

### ▶️ Try it

After the [setup prerequisites](docs/development/setup.md), install this checkout
and compare two subnets or observe one over two rounds (network access required):

```bash
make icq
icq topology subnet compare \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe \
  brlsh-zidhj-3yy3e-6vqbz-7xnih-xeq2l-as5oc-g32c4-i5pdn-2wwof-oae
icq topology subnet compare \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe \
  brlsh-zidhj-3yy3e-6vqbz-7xnih-xeq2l-as5oc-g32c4-i5pdn-2wwof-oae --json
icq topology subnet maintenance \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe \
  --interval-seconds 10
icq topology subnet maintenance \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe \
  --interval-seconds 10 --json
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
icq topology subnet distribution \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe
icq topology subnet distribution \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
icq topology node changes \
  nuass-yihti-wszdr-r7qze-32dgz-osovf-pracd-hg5tt-mwlhf-rvuyq-4ae \
  --interval-seconds 10
icq topology node changes \
  nuass-yihti-wszdr-r7qze-32dgz-osovf-pracd-hg5tt-mwlhf-rvuyq-4ae \
  --interval-seconds 10 --json
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
icq topology subnet nodes \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe
icq topology subnet nodes \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
icq topology subnet changes \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe \
  --interval-seconds 10
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
icq topology subnet configuration \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
icq topology subnet membership \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
icq topology node show \
  nuass-yihti-wszdr-r7qze-32dgz-osovf-pracd-hg5tt-mwlhf-rvuyq-4ae --json
```

The first prints Registry membership, configured software and halt flags with
source evidence. The second retains both source reports and node-set differences;
matching sets do not establish live nodes or an atomic snapshot. The third prints
the node's operator, optional raw chip bytes and configured HostOS, with
attestation explicitly unverified. See the
[CLI group notes](releases/cli/CHANGELOG.md) for details and text-output examples.
