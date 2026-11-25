# AGENTS.md

Apply the installed [Toko Tooling baseline](TOKO-TOOLING.md) and its
revision-bound references. [.toko-tooling.snapshot](.toko-tooling.snapshot)
records the adopted payload; never hand-edit it or managed files under
`vendor/toko-tooling/`. Common engineering rules belong to that baseline.

## Task entrypoints

Read [README.md](README.md) and the opening status of the
[restart design](docs/design/0.104-restart.md). That design indexes the implemented
report contracts; read the affected contract before changing a report. Use
[the root changelog](CHANGELOG.md) for the latest delivered CLI examples. Do not
load every report or historical release at session start.

Further reports need a concrete Canic decision and source contract. Preserve
explicit source, coverage, units and partial outcomes; registered hardware is not
attestation, failed reads are not downtime, and inventory is not usable capacity.
The legacy repository is read-only reference material. Do not import its source,
backlog, caches, build outputs or release state wholesale.

## Predecessor research

Before substantive report/subsystem work, inspect the relevant predecessor
implementation, regressions and explanatory history read-only, at a pinned
[ic-query-legacy](https://github.com/dragginzgame/ic-query-legacy) revision. Identify
the solved invariant and changed assumptions; reuse sound solutions within current
Canic/SNS scope and published Platform boundaries. Carry applicable regressions
into their new owner, cite provenance and explain exclusions. Historical breadth
and passing tests neither authorize restored features nor qualify new code.

## Product purpose

Read the affected [comparison](docs/design/subnet-compare.md),
[maintenance](docs/design/subnet-maintenance.md) or
[progress](docs/design/subnet-progress.md) contract before changes. Preserve
independent sources, coverage, observation windows, signing/delegation times and
root-subnet binding. Comparisons do not establish capacity/merge feasibility;
configuration agreement or repeated state is not uptime or maintenance completion.

IC Query supports Canic application topology planning, uptime assessment and
subnet maintenance decisions. It is not a general-purpose IC querying library.
Select reports for the placement, capacity, performance, cost, availability and
trust decisions that Canic applications need to make.

Keep node, operator, provider, data-center, subnet and cloud-engine views under
topology. Reports must identify workload/trust assumptions, observation window,
source, coverage and failures. TEE presence does not justify removing authentication
or claiming lower costs. Query provides read-only comparisons; Canic owns topology
changes, deployment and maintenance execution.

Exclude NNS voting, proposals, neurons, governance analytics and Node Rewards
reports/adapters, including using Node Rewards as an indirect performance source.
The NNS Registry remains an infrastructure source. Drop generic token analytics,
holder indexing and network-wide canister exploration; retain only reads needed
by the accepted Canic or separate SNS migration purpose.

## SNS migration support

SNS migration belongs in the separate `ic-query-sns` crate in this
repository. The `ic-query-topology` crate must contain no SNS module, feature or
dependency on that crate. Each library selects its own Platform capabilities;
neither depends on the other. A CLI may explicitly compose both.

Read only balances from explicitly selected ICRC ledgers and neuron information
from compatible SNS Governance canisters. This supports application migration
research, not token transfers, neuron management or destination entitlements.
Dragginz and Toko are examples, not deployment restrictions. Broader SNS proposals,
voting, swaps, treasury analytics, framework upgrades and explorer functionality
remain excluded. Further capabilities need an explicit report contract.
Preserve exact values, identities, observation times and coverage under the
[migration evidence contract](docs/design/migration-evidence.md). Speculation about
SNS longevity or Foundation changes is not an established external fact.

## Product boundaries

- `ic-query` names the repository, virtual Cargo workspace and thin umbrella
  crate. Its empty defaults and optional `topology`/`sns` features select direct
  re-exports only; it owns no query policy, transport or shared implementation.
  `ic-query-topology` and `ic-query-sns` own independent report contracts.
  `ic-query-cli` (`icq`) composes them through compile-time command adapters when
  reports exist. Specialist libraries must not depend on each other, the umbrella
  or the CLI. The CLI may continue to depend on specialists directly.
- A future NNS capability would need a separately approved specialist crate and
  CLI adapter; it does not expand topology's scope. Do not add dynamic plugin
  loading, speculative feature flags or a generic query-core library. Extract
  shared helpers only for demonstrated common contracts not supplied by Platform.
- Query is read-only. No token transfers, neuron management, proposals,
  deployment, signing-key custody or arbitrary method invocation.
- Implemented topology reports and CLI commands are defined by their linked
  contracts. Further reports require a concrete Canic decision; migration
  evidence follows its scoped contract. Retaining a package name does not
  authorize restoring its old feature set.
- The libraries own reports and query policy; the CLI owns
  parsing, process IO, progress and pure human-facing presentation. The library
  must not select process output sinks or depend on CLI presentation.
- Use Platform's published native read, external DTO and bounded codec capabilities.
  The replacement supplier is now `dragginzgame/ic-platform` (formerly
  `ic-platform-new`); direct capability requests to that owner.
  Manifests own compatible supplier floors and locks own qualified selections.
  Keep exact pins only for documented constraints, including the tooling bundle.
  Public principals share canonical Types with Protocol and authentication; do
  not restore Agent or legacy identity/class bridges. Published suppliers must
  be qualified before delivery; local sibling paths are development-only.
  Portable test frames and provenance stay beside their owners.
  Platform owns authentication; Query owns endpoint/root-key admission, limits,
  deadlines and error projection. Do not add direct Agent, Candid or CDK packages.
- Keep endpoint, provenance, completeness and certification claims explicit.
  JSON preserves raw values; presentation owns display conversions.
- Query-owned product schemas remain `1` before 1.0. Retained evidence and
  external consumer obligations survive a hard cut; identify explicit reset
  and retirement actions before replacing a contract.
- Retain the 0.104 baseline. An incompatible CLI replacement after the existing
  0.104 releases requires a later minor; do not describe it as a patch.

## Delivery and publication

The maintainer lifted the publication hold on 2026-11-16 and requested the next
topology and CLI versions on crates.io and GitHub. Workspace packages inherit
`publish = true`; publish only affected, qualified release groups. Migration
reports own the independent `sns-migration` release group. Canic is the identified downstream
consumer; coordinate its Rust type migration without changing that sibling here.
In IC Query, a maintainer request to "push" the product changes authorizes
version preparation, release notes, GitHub delivery and crates.io publication
for the affected groups. This is Query’s local convention. Honor any narrower
request, including GitHub-only or branch-only delivery.
Compatible follow-ups normally increment the patch; incompatible pre-1.0 contracts
require a minor. Never reuse delivered versions. Local supplier paths are for
development only: publication requires available, qualified registry dependencies.
Supplier publication remains separately authorized work at its owning repository.

## Local development

After each coherent patch, review the changed responsibility and its callers for conflicting
contracts, duplicated owners, unused mechanisms and misleading claims. Repair
within the authorized scope or report the boundary; do not broaden tests or
sibling work merely for this review.

CLI tables use published `ic-host-text` bordered rendering for visible column
delimiters, terminal escaping and Unicode-width alignment. Keep row selection
and value semantics in the CLI; JSON retains raw report values.

Keep root `CHANGELOG.md` as the navigation index for the group ledgers declared
in `ci/cargo-release-groups.json`, including the baseline's required CLI quick
starts. Detailed release notes belong to their owning group.
Include copyable command examples in fenced `bash` blocks in the owning ledger,
especially every added or changed `icq` command in `releases/cli/CHANGELOG.md`. Use actual
public subnet and canister IDs whenever required, not placeholders such as
`<SUBNET_ID>` or `CANISTER_ID`, so the examples run without editing. Keep them
beside the relevant release note, identify prerequisites, and show only
implemented syntax. Apply the same concrete-example convention in user-facing
handoffs.
Wrap long shell command examples with `\` line continuations and indent continued
arguments. Keep each identifier intact and the backslash last on its line so
examples remain directly copyable.

[Setup](docs/development/setup.md) owns preparation, focused checks and the
aggregate `make check` gate. CLI process qualification lives beside the CLI;
add a separate `qualification/` workspace only when real test-only packages need
it. Current release and archive qualification are owned by the setup/release
guides; do not revive the initial blanket publication deferral.

Keep the local overlay small. Add implementation-specific commands and ownership
rules when their corresponding product surface exists. The initial native support
requirement remains Linux x86-64 and macOS ARM64/x86-64. The
[setup guide](docs/development/setup.md#focused-validation-and-qualification)
records the successful hosted checks and their source commit; keep qualification
claims bounded to the contracts and inputs actually checked.
