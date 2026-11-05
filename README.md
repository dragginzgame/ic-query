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

[![CI](https://github.com/dragginzgame/ic-query/actions/workflows/ci.yml/badge.svg)](https://github.com/dragginzgame/ic-query/actions/workflows/ci.yml)
[![docs.rs](https://docs.rs/ic-query/badge.svg)](https://docs.rs/ic-query)
[![License](https://img.shields.io/crates/l/ic-query.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.99.1-blue.svg)](Cargo.toml)

IC Query makes it easy to find answers about any part of the Internet Computer
blockchain.

You give it a question—such as which computers run part of the network, what a
token ledger reports, or how a project is governed—and its `icq` command
gathers the answer from the relevant public source.

Each report says where its information came from, when it was retrieved, and
what checks were possible. You can read the result in a terminal or save the
complete data as JSON for further analysis.

IC Query never changes anything on the network. It cannot submit proposals,
move tokens, manage neurons, deploy software, or modify applications.

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-query/ic-query-where-it-fits.svg" alt="IC Query observes and reports, while tools such as dfx can deploy or change things">
</p>

Rust consumers use `ic-query` for typed reports, JSON, collection and cache policy,
and `ic-query-text` for human-facing report strings. The presentation package and
`ic-query-cli` release together; library releases advance independently. See the
[release groups and changelogs](CHANGELOG.md).

## A few IC terms

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-query/ic-query-conceptual-map.svg" alt="Simplified conceptual map of the relationships between the NNS, Registry, Subnets, nodes, canisters, SNS governance, and ICRC ledgers">
</p>

- A **node** is a computer that participates in the Internet Computer network.
- A **Subnet** is a group of nodes that work together to run applications.
- A **canister** is an application or service, together with its stored data,
  running on a Subnet.
- The **Registry** is the network's official configuration record.
- The **NNS** is the Network Nervous System, which governs and configures the
  overall network.
- An **SNS** is a governance system for an individual decentralized project.
- A **neuron** represents staked tokens used to participate in NNS or SNS
  governance.
- A **ledger** records a token's balances and transactions. **ICRC** names
  standards used by many IC token ledgers.
- **Cycles** are the units applications spend for computation and storage on
  the IC.

You do not need to memorize these terms before using the tool. Start with a
question that interests you and follow the matching command family.

## When is it useful?

`ic-query` can help you:

- check what is publicly known about an application;
- see which nodes and providers support part of the network;
- follow governance proposals, voting activity, economics, and rewards;
- inspect a token's supply, balances, or transaction history;
- compare the network's official records with operational observations;
- save a dated report for research, auditing, or monitoring; or
- build another read-only tool from the same validated reports.

Different questions are answered by different sources. `ic-query` identifies
the source, retrieval time, and important limitations instead of presenting
every answer as equally authoritative or equally current.

## How it works

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-query/ic-query-how-it-works.svg" alt="How IC Query turns your question into a readable answer with its source, retrieval time, and important limitations">
</p>

1. Start with something you want to learn about an application, project,
   token, or the network itself.
2. Run the matching `icq` command. It contacts the specific public source that
   owns or observes that information.
3. `ic-query` checks the response, applies safety limits, and records where and
   when the information was obtained.
4. It presents a readable answer in the terminal. Add `--json` when you need
   the complete data for another tool.

Some larger reports keep a local copy so they do not have to download the same
information every time. Commands always state when they use or refresh one of
these cached copies.

## What can you investigate?

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-query/ic-query-investigation-areas.svg" alt="Six IC Query investigation areas: applications, network and nodes, governance, projects, tokens, and rates and system data">
</p>

| You want to… | What `ic-query` can show | Try |
| --- | --- | --- |
| Learn about an application | Its name, Subnet, managing accounts, software details, and recorded upgrades | `icq ic canister info <canister-id>` |
| See how the network is changing | Node counts and other network measurements over time | `icq ic metrics ic-node-count` |
| Understand who supports the network | Nodes, providers, locations, capacity, software versions, and missing information | `icq nns topology summary` |
| Follow network governance | Recent proposals and their status, plus public neuron, economics, and reward reports | `icq nns proposal list --limit 25` |
| Inspect existing neuron stakes | Principal permissions, raw stake, fees and dissolve state for a configured Governance canister | `icq sns neuron list <governance-canister-id> --json` |
| Inspect a token | Its name, symbol, fees, supply, balances, transactions, and available history | `icq icrc ledger token <ledger-canister-id>` |
| Check the ICP/XDR conversion rate | The current certified rate used by the IC to price cycles | `icq system xdr` |
| Explore public CloudEngine infrastructure | Subnets, operators, providers, nodes, and published prices | `icq cloud-engine list` |

In the example commands, `<canister-id>` and `<ledger-canister-id>` are
placeholders. Replace them with the identifier you want to inspect.

## Install

You do not need to write a program to use `icq`, but you do need to be
comfortable running commands in a terminal. The published installation
currently uses Cargo, the package manager included with the Rust toolchain.
Once Rust and Cargo are installed, run:

```bash
cargo install ic-query-cli
```

This installs the `icq` command. Confirm it is available with:

```bash
icq --version
```

Contributors working from a repository checkout can instead use:

```bash
make install
```

The local install replaces an existing `icq` binary. Canister-adapter
contributors should also read the
[Governance canister smoke harness](https://github.com/dragginzgame/ic-query/blob/main/docs/canister-smoke.md).

## Your first session

After installation, try a report that does not require you to know an id:

```bash
icq ic metrics ic-node-count
```

The report contains both the answer and context about where it came from. A
shortened response has this shape; counts and timestamps change over time:

```text
network: ic
authority: official_ic_dashboard_api
metric: ic-node-count
returned_series_count: 2
certified: no
fetched_at: 2026-10-05T12:00:00Z

total_nodes:
  1791201600  1234

up_nodes:
  1791201600  1230
```

Here, `authority` identifies the source, `fetched_at` says when the report was
collected, and `certified` states whether this particular source supplied
cryptographic proof. The two sections contain the reported total and available
node counts.

To inspect existing neuron ownership and stake, supply the Governance principal:

```bash
icq sns neuron list "$GOVERNANCE_CANISTER_ID" --owner "$USER_PRINCIPAL" --json
```

JSON preserves permission codes and raw stake/fee values for application policy.
Use ICRC balance and transaction reads for known accounts, and paginated holder
analytics to enumerate the selected index. Query remains read-only; Toko owns
acceptance, burns, credits and purchase gating.

The [CLI Usage guide](https://github.com/dragginzgame/ic-query/blob/main/docs/cli-usage.md)
contains the complete command map. The living
[Roadmap to 1.0](https://github.com/dragginzgame/ic-query/blob/main/docs/roadmap/1.0.md)
records current coverage and remaining work.

If you only want to explore the IC from the terminal, the sections above and
the CLI Usage guide are the best places to start. The linked references below
cover advanced commands, evidence, caching, and use as a Rust library.

## Technical reference

| Guide | Contents |
| --- | --- |
| [Command and evidence reference](docs/command-reference.md) | Command gallery, evidence guarantees, command families, collection and cache behavior |
| [Rust API reference](docs/rust-api-reference.md) | Features, report builders, source adapters and reusable collection contracts |
| [Library usage](docs/library-usage.md) | Complete Rust integration examples |

## Documentation

- [Documentation index](https://github.com/dragginzgame/ic-query/blob/main/docs/README.md)
- [CLI usage and collection modes](https://github.com/dragginzgame/ic-query/blob/main/docs/cli-usage.md)
- [Library usage](https://github.com/dragginzgame/ic-query/blob/main/docs/library-usage.md)
- [Local developer tool setup](https://github.com/dragginzgame/ic-query/blob/main/docs/local-setup.md)
- [Supported native hosts](https://github.com/dragginzgame/ic-query/blob/main/docs/supported-hosts.md)
- [Governance canister smoke tests](https://github.com/dragginzgame/ic-query/blob/main/docs/canister-smoke.md)
- [Roadmap to 1.0](https://github.com/dragginzgame/ic-query/blob/main/docs/roadmap/1.0.md)
- [Cache policy](https://github.com/dragginzgame/ic-query/blob/main/docs/design/cache-policy.md)
- [Subnet Catalog acquisition and performance](https://github.com/dragginzgame/ic-query/blob/main/docs/design/subnet-catalog-acquisition-performance.md)
- [Subnet Catalog downstream adoption](https://github.com/dragginzgame/ic-query/blob/main/docs/design/subnet-catalog-downstream-adoption.md)
- [0.22 structural consolidation](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.22/0.22-design.md)
- [0.25 fuller fixed-size SNS neuron evidence](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.25/0.25-design.md)
- [0.27 bounded official ICRC analytics](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.27/0.27-design.md)
- [0.28 observed IC node and Subnet status](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.28/0.28-design.md)
- [0.29 Subnet Catalog authority and embedder hardening](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.29/0.29-design.md)
- [0.30 certified Registry evidence](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.30/0.30-design.md)
- [0.31 public CloudEngine reporting](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.31/0.31-design.md)
- [0.32 bounded replica-version reporting](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.32/0.32-design.md)
- [0.33 certified API boundary-node reporting](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.33/0.33-design.md)
- [0.34 CloudEngine provider reporting](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.34/0.34-design.md)
- [0.35 CloudEngine Type4 node reporting](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.35/0.35-design.md)
- [0.36 node-provider reward reporting](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.36/0.36-design.md)
- [0.37 ICRC account and holder index reporting](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.37/0.37-design.md)
- [0.38 canister-native NNS Governance reporting](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.38/0.38-design.md)
- [0.39 portable NNS proposal activity analytics](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.39/0.39-design.md)
- [0.40 portable NNS public-neuron distribution analytics](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.40/0.40-design.md)
- [IC Dashboard canister reporting](https://github.com/dragginzgame/ic-query/blob/main/docs/design/ic-dashboard-canister-reporting.md)
- [IC Dashboard network metrics](https://github.com/dragginzgame/ic-query/blob/main/docs/design/ic-dashboard-network-metrics.md)
- [IC Dashboard daily statistics](https://github.com/dragginzgame/ic-query/blob/main/docs/design/ic-dashboard-daily-stats.md)
- [IC Dashboard boundary-node reporting](https://github.com/dragginzgame/ic-query/blob/main/docs/design/ic-dashboard-boundary-node-reporting.md)
- [Exact-version NNS Subnet topology](https://github.com/dragginzgame/ic-query/blob/main/docs/design/nns-subnet-topology.md)
- [Certified CMC system reporting](https://github.com/dragginzgame/ic-query/blob/main/docs/design/cmc-system-reporting.md)
- [Release ledger](https://github.com/dragginzgame/ic-query/blob/main/CHANGELOG.md)
- [Maintainer release workflow](docs/release-workflow.md)
- [Supported native hosts and prerequisites](docs/supported-hosts.md)

## Scope

`ic-query` is read-only metadata and evidence tooling. It does not replace
`dfx`, expose arbitrary Candid invocation, or perform canister, governance,
ledger, or network mutations.

Deployment and orchestration projects can call the CLI or use the Rust library
when they need these reporting contracts. One downstream integration is
[Canic](https://github.com/dragginzgame/canic).
