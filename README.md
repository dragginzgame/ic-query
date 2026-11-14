# IC Query

A Cargo workspace for read-only IC evidence supporting Canic application topology,
uptime and subnet maintenance, with a separately scoped SNS migration library.

| Package | Responsibility |
| --- | --- |
| `ic-topology` | Infrastructure topology, capabilities, resources, costs and changes for Canic decisions |
| `ic-sns-migration` | Balances and neuron information for an explicitly selected SNS, supporting migration of Motoko, Toko, Dragginz and similar users |
| `ic-query-cli` (`icq`) | Parsing, process IO and presentation through compile-time adapters to specialist libraries |

`ic-query` is the workspace name; there is no umbrella library. The libraries
remain independent. Add CLI adapters and their dependencies when reports exist.
Shared helpers must have demonstrated consumers and respect Platform ownership.

Native reports list registered subnets, look up one canister's subnet through the
mainnet NNS Registry, or observe node identities in one subnet's certified state.
All three return explicit
source and observation evidence, with typed failures and bounded reads. See the
[placement](docs/design/canister-placement.md) and
[subnet observation](docs/design/subnet-observation.md) contracts, plus the
[Registry inventory](docs/design/subnet-list.md) contract.
The SNS library remains a shell. All packages have publication disabled. Full SNS
support, NNS voting, governance analytics and Node Rewards are outside scope.
The NNS Registry remains a source for infrastructure observations.

The retained restart baseline is **0.104**. The specialist libraries start at
`0.104.0`; the incompatible CLI replacement is drafted at `0.105.0`, after the
published legacy CLI `0.104.1`. These are unpublished development versions, not
version reservations or a waiver of consumer migration work.

After preparing the tools described in [setup](docs/development/setup.md):

```bash
make fetch-dependencies
make deps-list
toko-cloc -d 0
make qualify-cli
cargo run --locked --offline -p ic-query-cli -- --help
cargo run --locked --offline -p ic-query-cli -- topology subnet list --json
cargo run --locked --offline -p ic-query-cli -- topology canister show ryjl3-tyaaa-aaaaa-aaaba-cai --json
```

`make deps-update` explicitly updates dependencies. `make check` is the aggregate
offline gate; use the focused commands in setup during ordinary work.

Placement queries access mainnet. A subnet mapping does not prove canister
existence or uptime, and verified query signatures do not make the returned
Registry value consensus-certified. No custom endpoint or trust-root override is
accepted. The subnet report authenticates returned node-key identities but does
not establish complete Registry membership, node liveness or capacity.

- [Architecture and open report decisions](docs/design/0.104-restart.md)
- [Development and qualification](docs/development/setup.md)
- [Release-group notes](CHANGELOG.md)
- [CLI release notes and copyable examples](releases/cli/CHANGELOG.md)
- [Local agent instructions](AGENTS.md)

The [legacy repository](https://github.com/dragginzgame/ic-query-legacy) retains
its source, published history, consumer contracts and recovery evidence. Its
implementation and backlog are reference material, not the new project's scope.
