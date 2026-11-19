# IC Query

A Cargo workspace for read-only IC evidence supporting Canic application topology,
uptime and subnet maintenance, with a separately scoped SNS migration library.

The libraries use the family names `ic-query-topology` and
`ic-query-sns`. Rust imports are `ic_query_topology` and `ic_query_sns`;
`ic-query-cli` and the `icq` binary keep their names. This is a hard cut with no
old-name aliases. The SNS name permits future expansion; its current reports
remain selected ICRC balances and SNS neuron evidence for migration.

The four-crate delivery is published as of 2026-11-19.
[Qualification and setup](docs/development/setup.md)
records the hosted checks and installed CLI smoke observations. Resume product
work for a concrete Canic decision or migration evidence requirement. Dragginz
is the live migration example; Toko account checks remain deferred. Neuron reads
retain selected fields and bounded pages, not a complete or atomic export.
Compatible supplier patches remain independent of Query's lockfile
(Platform 0.34.17 / Protocol 0.3.6). CLI commands and specialist report contracts
are unchanged; the umbrella deliberately replaces the legacy library API.

| Package | Version | Responsibility |
| --- | --- | --- |
| `ic-query` | 0.104.10 | Thin umbrella with optional `topology` and `sns` re-exports; empty defaults |
| `ic-query-topology` | 0.110.10 | Infrastructure topology, capabilities, resources, costs and changes for Canic decisions |
| `ic-query-sns` | 0.105.10 | Selected ICRC account balances and SNS neuron evidence; future SNS capabilities require explicit contracts |
| `ic-query-cli` (`icq`) | 0.105.26 | Parsing, process IO and presentation through compile-time adapters to specialist libraries |

The four-crate workspace keeps the specialist libraries independent. The umbrella
is an optional convenience; direct specialist dependencies remain supported.
`ic-query` 0.104.0 replaces the legacy 0.103.1 API with
an explicit breaking cut. See the [migration notes](releases/query/CHANGELOG.md).

```toml
ic-query = { version = "0.104.10", features = ["topology", "sns"] }
```

Use `ic_query::topology` and `ic_query::sns`. No feature means no specialist
dependency. The CLI continues to compose the specialist libraries directly.

Native reports list registered subnets, look up one canister's subnet through the
mainnet NNS Registry, or observe subnet type and node identities in certified state.
The [selected placement collection](docs/design/placement-collection.md) reads
1–32 explicit canister IDs with a shared deadline and per-canister partial outcomes.
These reports return explicit
source and observation evidence, with typed failures and bounded reads. See the
[placement](docs/design/canister-placement.md) and
[subnet observation](docs/design/subnet-observation.md) contracts, plus the
[Registry inventory](docs/design/subnet-list.md) contract.
The [subnet metrics report](docs/design/subnet-metrics.md) adds certified raw
resource values for one subnet, without inferring spare capacity or uptime.
The [activity report](docs/design/subnet-activity.md) compares two certified
message totals with a bounded wait, preserving both observations without a health verdict.
The [progress report](docs/design/subnet-progress.md) compares authenticated
subnet state times using the published replacement Platform projection.
The [maintenance report](docs/design/subnet-maintenance.md) coordinates configuration
and metrics observations, retaining partial evidence and optional exact configured-version
expectations without claiming running software, uptime or completed maintenance.
The [configuration report](docs/design/subnet-configuration.md) adds registered
membership, configured replica version and halt flags using published Platform.
The [settings report](docs/design/subnet-settings.md) reads registered feature flags
and resource limits, preserving absent values and unresolved replica defaults.
The [membership comparison](docs/design/subnet-membership.md) retains that
configuration and a separate certified node-key observation, exposing differences
without treating matching sets as proof of live nodes or an atomic snapshot.
The [node report](docs/design/node-inventory.md) reads one node's registered
operator, optional SEV-SNP chip ID and configured HostOS version. It supplies
no verified attestation or running-software claim.
The [node comparison](docs/design/node-changes.md) detects changes in those
registration fields across supplied reports or two bounded reads.
The [subnet node inventory](docs/design/subnet-nodes.md) reads those records for
one subnet's registered membership, with bounded concurrency and explicit
per-node failures or unfinished reads.
The [configuration comparison](docs/design/subnet-changes.md) identifies selected
membership, replica-version and halt-flag differences across two observations.
The [distribution report](docs/design/subnet-distribution.md) joins providers and
registered data centers at one requested Registry version, retaining independent
coverage and unknown counts for placement concentration decisions.
The [two-subnet comparison](docs/design/subnet-compare.md) presents two selected
placement candidates' configuration, concentration and resources side by side,
preserving independent versions and partial results without ranking capacity.
Opt-in `compare --settings` adds both candidates' registered feature/resource
declarations and retains independent failures; default comparison reads stay unchanged.
It also derives shared provider/data-center identities from retained distributions,
keeping unknown coverage explicit without claiming independent failure domains.
The [cloud-engine price report](docs/design/cloud-engine-marketplace.md) retains
exact daily marketplace amounts and a separate network markup, preserving prices
if the fee read fails. The available-node report traverses public eligibility
pages with bounded collection, preserving earlier pages on failures or version
changes. Exhausted pagination does not establish usable capacity or a snapshot.
The candidate view joins those observations to every matching public declaration,
preserving missing/overlapping scopes without selecting a tariff or quote.
The [selected-engine report](docs/design/cloud-engine-topology.md) connects an
opaque engine ID to its subnet/operator and compares its record nodes with the
embedded Registry copy. Missing or hidden engines remain explicit outcomes.
The [migration library](docs/design/migration-evidence.md) now reads selected ICRC
account balances and SNS neuron records with bounded, partial evidence. The workspace selects published Platform
0.34.17 native reads, Protocol 0.3.6 and Types 0.21.3,
with no legacy Platform or Agent in the product/test graph. Full SNS
support, NNS voting, governance analytics and Node Rewards are outside scope.
The NNS Registry remains a source for infrastructure observations.

The retained restart baseline is **0.104**. The specialist libraries start at
`0.104.0`; the incompatible CLI replacement's GitHub baseline is `0.105.0`, after
the published legacy CLI `0.104.1`. Current group versions and delivery history
are in the group ledgers.
GitHub delivery does not imply crates.io publication or waive consumer migration.

After preparing the tools described in [setup](docs/development/setup.md):

```bash
make fetch-dependencies
make deps-list
toko-cloc -d 0
make qualify-cli
cargo run --locked -p ic-query-cli -- --help
cargo run --locked -p ic-query-cli -- topology subnet list --json
cargo run --locked -p ic-query-cli -- topology canister show \
  ryjl3-tyaaa-aaaaa-aaaba-cai --json
```

`make deps-update` explicitly updates dependencies. `make check` is the aggregate
validation gate; use the focused commands in setup during ordinary work.

Placement queries access mainnet. A subnet mapping does not prove canister
existence or uptime, and verified query signatures do not make the returned
Registry value consensus-certified. No custom endpoint or trust-root override is
accepted. The subnet report authenticates returned node-key identities but does
not establish complete Registry membership, node liveness or capacity.

- [Architecture and open report decisions](docs/design/0.104-restart.md)
- [Using topology evidence for Canic decisions](docs/usage/topology.md)
- [Canic integration boundary](docs/design/canic-consumption.md)
- [Development and qualification](docs/development/setup.md)
- [Release-group notes](CHANGELOG.md)
- [CLI release notes and copyable examples](releases/cli/CHANGELOG.md)
- [Local agent instructions](AGENTS.md)

The [legacy repository](https://github.com/dragginzgame/ic-query-legacy) retains
its source, published history, consumer contracts and recovery evidence. Its
implementation and backlog are reference material, not the new project's scope.
