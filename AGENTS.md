# AGENTS.md

Apply the installed [Toko Tooling baseline](TOKO-TOOLING.md) and its
revision-bound references. [.toko-tooling.snapshot](.toko-tooling.snapshot)
records the adopted payload; never hand-edit it or managed files under
`vendor/toko-tooling/`. Common engineering rules belong to that baseline.

## Current handoff

Read [README.md](README.md), [CHANGELOG.md](CHANGELOG.md) and the
[restart design](docs/design/0.104-restart.md) at session start. This is an
initial topology phase: the [placement](docs/design/canister-placement.md),
[subnet observation](docs/design/subnet-observation.md) and
[Registry inventory](docs/design/subnet-list.md) contracts own the selected
mainnet reports and their limits. Further report implementation
requires a concrete Canic decision and source contract.
The legacy repository is read-only reference material. Do not import its source,
backlog, caches, build outputs or release state wholesale.

## Learn from IC Query's predecessor

- Before substantive report or subsystem work, inspect the relevant implementation,
  regression tests and explanatory history in
  [ic-query-legacy](https://github.com/dragginzgame/ic-query-legacy). Use the local
  legacy checkout when available, otherwise inspect a pinned GitHub revision.
  Keep this targeted to the selected report and its Canic or SNS migration need.
- Identify the problem the old solution addressed and which assumptions have
  changed. Reuse or adapt sound first-party solutions where they fit the current
  ownership and published Platform boundaries; explain replacements or exclusions.
  Legacy breadth does not authorize restoring excluded reports or infrastructure.
- Carry applicable failure and regression cases into the owning implementation's
  tests and verify them here. Cite the inspected revision and distinguish reused
  code from retained invariants; historical passing tests do not qualify new code.
- Keep legacy research read-only. Work in this task is scoped to IC Query;
  guidance from another project does not authorize changes to that project.

## Product purpose

IC Query supports Canic application topology planning, uptime assessment and
subnet maintenance decisions. It is not a general-purpose IC querying library.
Select reports for the placement, capacity, performance, cost, availability and
trust decisions that Canic applications need to make.

Representative questions include:

- Does an application need ten cloud engines, or would another topology serve
  its workload better?
- Should application workloads be consolidated across subnets, or should a
  subnet merge or other topology change be considered where supported?
- Have new TEE-enabled machines become available that could support a different
  placement or authentication strategy and reduce authentication overhead?
- Are selected subnets making progress, and what observed configuration or
  software changes matter to application uptime and maintenance?

Reports should expose relevant topology, machine capabilities, availability and
resource evidence for comparing those choices. Distinguish registered hardware
capabilities from verified attestation and usable capacity. State feasibility,
workload and trust assumptions; TEE presence alone does not establish that
authentication checks can be removed or that performance costs will fall.

Canic is the primary intended consumer. Add a report only for an identified Canic
infrastructure decision or the narrow SNS migration purpose below; broad IC API coverage
and the legacy feature set are not product goals. Query supplies read-only evidence
and comparisons; topology changes and deployment remain outside its execution
authority.

Merge node, operator, provider, data-center, subnet and cloud-engine inventory
views under topology. Keep capability, resource, cost and change reports focused
on Canic decisions. Uptime evidence must identify its observation window, source,
coverage and failures; a failed endpoint read alone does not prove subnet downtime.
Query observes maintenance state and changes; Canic owns planning and execution.

Exclude NNS voting, proposals, neurons, governance analytics and Node Rewards
reports/adapters, including using Node Rewards as an indirect performance source.
The NNS Registry remains an infrastructure source. Drop generic token analytics,
holder indexing and network-wide canister exploration; retain only reads needed
by the accepted Canic or separate SNS migration purpose.

## SNS migration support

SNS migration belongs in the separate `ic-sns-migration` crate in this
repository. The `ic-topology` crate must contain no SNS module, feature or
dependency on that crate. Each library selects its own Platform capabilities;
neither depends on the other. A CLI may explicitly compose both.

Support is limited to reading balances and neuron information for an explicitly
selected SNS. Its sole purpose is to support migration of users of Motoko, Toko,
Dragginz and similar applications to our new platform. This is an exception to
the topology focus, not a commitment to full SNS support.

The scope reflects the maintainers' concern about SNS longevity; do not present
possible Foundation replacement or removal as an established external fact.
Do not expand this surface into SNS proposals, voting, swaps, treasury analytics,
framework upgrades or a general SNS explorer. Supporting source resolution and
bounded pagination must serve the selected balance or neuron report.

Preserve exact balances, neuron fields, source identities, observation times and
coverage needed by migration consumers. Reading these records does not execute
a migration, transfer tokens, manage neurons or decide destination entitlements.
Exact fields and migration evidence requirements remain to be specified.

## Product boundaries

- `ic-query` names the repository and virtual Cargo workspace, not an umbrella
  library. `ic-topology` and `ic-sns-migration` own independent report contracts.
  `ic-query-cli` (`icq`) composes them through compile-time command adapters when
  reports exist. Libraries must not depend on each other or on the CLI.
- A future NNS capability would need a separately approved specialist crate and
  CLI adapter; it does not expand topology's scope. Do not add dynamic plugin
  loading, speculative feature flags or a generic query-core library. Extract
  shared helpers only for demonstrated common contracts not supplied by Platform.
- Query is read-only. No token transfers, neuron management, proposals,
  deployment, signing-key custody or arbitrary method invocation.
- The main library and separate SNS migration crate have distinct product scope;
  exact reports and the CLI surface remain to be specified. Retaining a package
  name does not authorize restoring its old feature set.
- The libraries own reports and query policy; the CLI owns
  parsing, process IO, progress and pure human-facing presentation. The library
  must not select process output sinks or depend on CLI presentation.
- Use Platform's published Agent, external DTO and bounded codec capabilities.
  Agent owns authentication; Query owns endpoint/root-key admission, limits,
  deadlines and error projection. Do not add direct Agent, Candid or CDK packages.
- Keep endpoint, provenance, completeness and certification claims explicit.
  JSON preserves raw values; presentation owns display conversions.
- Query-owned product schemas remain `1` before 1.0. Retained evidence and
  external consumer obligations survive a hard cut; identify explicit reset
  and retirement actions before replacing a contract.
- Retain the 0.104 baseline. An incompatible CLI replacement after the existing
  0.104 releases requires a later minor; do not describe it as a patch.

## Delivery and publication

Publication is deferred until the project is mature and working and the maintainer
explicitly lifts this hold. Keep all packages `publish = false`. Requests to
"push" authorize GitHub delivery only, never crates.io publication. This local
restriction takes precedence over Toko Tooling's general delivery defaults;
release or ship wording alone does not lift the publication hold.

## Local development

After every coherent patch, review the changed code, callers, tests and docs for
architectural drift or accumulated scaffolding. Remove obsolete paths, duplicated
policy, unused abstractions and misleading claims in the same patch; keep report,
transport and CLI responsibilities with their owners. Do not let architectural
sediment accumulate. Report any boundary that cannot be resolved within authorized
scope instead of concealing it behind a workaround. This is a focused review of
the patch, not an instruction to rerun unrelated suites or audit other repositories.

Keep root `CHANGELOG.md` as the navigation index for the group ledgers declared
in `ci/cargo-release-groups.json`; release notes belong to their owning group.
Include copyable command examples in fenced `bash` blocks in the owning ledger,
especially every added or changed `icq` command in `releases/cli/CHANGELOG.md`. Use actual
public subnet and canister IDs whenever required, not placeholders such as
`<SUBNET_ID>` or `CANISTER_ID`, so the examples run without editing. Keep them
beside the relevant release note, identify prerequisites, and show only
implemented syntax. Apply the same concrete-example convention in user-facing
handoffs.

[Setup](docs/development/setup.md) owns preparation, focused checks and the
aggregate `make check` gate. CLI process qualification lives beside the CLI;
add a separate `qualification/` workspace only when real test-only packages need
it. Broader protocol and publication qualification remain deferred.

Keep the local overlay small. Add implementation-specific commands and ownership
rules when their corresponding product surface exists. The initial native support
requirement remains Linux x86-64 and macOS ARM64/x86-64. The
[setup guide](docs/development/setup.md#focused-validation-and-qualification)
records the successful hosted checks and their source commit; keep qualification
claims bounded to the contracts and inputs actually checked.
