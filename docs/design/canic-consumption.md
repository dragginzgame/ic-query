# Consuming topology evidence in Canic

Status: integration boundary reviewed; no Canic adapter or dependency added.

## What the current consumer owns

Read-only inspection of the local sibling `/home/adam/projects/canic` on
2026-11-15 found Canic's App → Fleets → Canisters model
and explicit network/identity bindings in `canic-host/src/config/mod.rs`.
Its CLI topology module renders declared parent/child application relationships;
those relationships are distinct from subnet placement, controllers and shared
provider/data-center infrastructure. Query must not interpret them as equivalent.

The latest inspected checkout HEAD was `df9321fc17c99c93e07d631173200a99c2a890bc`.
The checkout also contained pending work, including modifications to
`canic-host/src/store_bootstrap/mod.rs`. Its Store placement
path uses Agent `fetch_subnet_by_canister`, retains an approved Root subnet and
checks the created Store's placement. This is an observation of local work, not
a claim about delivered Canic behavior or a permanent API. No sibling file was
changed by the inspection.
Use this current sibling for consumer guidance, not `canic-legacy`. Query's own
predecessor research remains separately scoped to `ic-query-legacy`.

Query's `placement::fetch` uses Registry `get_subnet_for_canister` with verified
query signatures and `result_certified = false`. It is diagnostic routing
evidence, not a substitute for Canic's operation-bound placement checks. Before
replacing any existing consumer read, compare the exact source, proof, network,
identity binding, deadline and retained-authority contracts. Do not infer
equivalence from both operations returning a subnet ID.

## First integration: explicit mainnet diagnostics

The existing reports can support an explicitly requested diagnostic view:

1. Canic selects the fleet and admits its network. Query currently accepts mainnet
   only. Local or custom-network fleets must remain unsupported by this adapter;
   never send their canister IDs to mainnet as a fallback.
2. Canic supplies one explicit bound canister ID or two selected candidate subnet
   IDs. Unbound slots remain unbound. Query does not load `canic.json`, inspect
   journals, discover application children or infer fleet ownership.
3. Use `placement::fetch` for the Registry mapping, or `subnet::compare::fetch`
   for selected candidate configuration/distribution/metrics evidence. These
   methods already own read limits, partial outcomes and source claims.
4. Preserve the complete typed reports and their independent times/versions.
   Canic may display them beside the application tree, but must not turn them
   into controller relationships, deployment approval or a claim of current
   convergence. A successful CLI exit or complete collection is not suitability.

An initial CLI consumer can use `icq` directly; structured integration should
consume the report types when package delivery is agreed. Packages remain
`publish = false`: this design does not authorize a registry publication, sibling
path override, Canic manifest edit or parsing human tables as an API.

### Using the existing CLI today

After [installing this checkout's CLI](../development/setup.md), an operator can
inspect an explicitly selected mainnet canister and then compare selected subnets:

```bash
icq topology canister show ryjl3-tyaaa-aaaaa-aaaba-cai --json
icq topology subnet compare \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe \
  brlsh-zidhj-3yy3e-6vqbz-7xnih-xeq2l-as5oc-g32c4-i5pdn-2wwof-oae \
  --overlap-details
```

These public IDs demonstrate syntax; they are not inferred Canic fleet bindings
or recommended deployment targets. Select the real fleet's bound mainnet ID and
candidate subnets explicitly. The comparison does not consume the first command's
output, and no automatic placement or candidate selection occurs.

For structured comparison evidence, replace `--overlap-details` with `--json`.
Preserve stdout even when comparison exits 1: it can contain partial evidence.
Exit 0 means collection succeeded, not that either candidate meets application
requirements. Exit 2 reports invalid CLI arguments. A consumer must also handle
process/IO failures and must not assume every failure contains a report.

Review provider and data-center coverage before interpreting shared counts, then
consider raw resource observations alongside application demand. Query does not
yet supply configured resource limits, allocatable headroom or a suitability score.

## Partial results and retained evidence

For pair comparison, inspect distribution and metrics outcomes independently.
An unavailable distribution makes overlap unavailable. A partial distribution
can still show observed shared identities, with unknown coverage for each kind.
An unavailable metric must not erase successfully collected infrastructure data.
Registry-version agreement does not create an atomic snapshot, and zero shared
identities does not establish independent failure domains.

Canic owns any persistence and freshness policy. Current Query report types
serialize evidence but do not provide a validated retained-report loader.
Do not deserialize an old JSON file into authority or discard source metadata
because its fields resemble a live result. A retained-report adapter needs its
own bounded input, schema/identity checks, age policy and retirement contract.
Query adds no storage or background watcher for this integration.

## What remains to be selected

Before implementing a consumer adapter, identify the exact Canic diagnostic
command/view, how operators select candidate subnets, and whether its refresh
budget accommodates the existing pair comparison (up to 160 asynchronous seconds).
If the requirement is whole-fleet placement instead, specify an explicit bounded
ID collection, per-canister outcomes and shared deadline before adding that report.
Do not hide unbounded fanout inside a fleet refresh or use a universal query plugin.

Platform's rewrite does not require a speculative bridge in Query. Keep the
existing published reader boundary until the replacement's API and trust contract
are available. Registered settings still await the supplier prerequisite in the
[settings contract](subnet-settings.md); this does not prevent using the existing
[comparison](subnet-compare.md) and [maintenance](subnet-maintenance.md) reports.
