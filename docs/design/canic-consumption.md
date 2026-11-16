# Consuming topology evidence in Canic

Status: Query collection delivered in topology 0.106.1 and the placement adapter
delivered in Canic 0.210.0. The explicit two-subnet adapter was delivered in
Canic 0.211.0. Canic consumes Query; Query has no dependency on Canic.

Pending Query migration moves public principals to published Types 0.21.2. A read-only
check of the sibling Canic manifest on 2026-11-16 still selects Query revision
`0a3eca919f05d777798406087049cc51ab7ae2a8` / topology 0.106.1. Its current pin is
unaffected by local changes here. Before adopting the next incompatible topology
release, its adapter must align principal types with the selected Types source
and requalify calls. The hard cut also changes native transport/codec error
payloads and cloud-engine class identities, removes unreachable subnet/metrics
identity errors and adds `ReadError::ClockRange`. Update exhaustive Rust matches
against topology 0.108.0. JSON is unchanged and needs no retained-evidence reset.
No Canic source or dependency was changed in this migration.

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

## First Canic adapter

A follow-up read-only inspection of the local sibling at HEAD
`e3e7be86301c377db8b064d2ef914f5cde936265` found `fleet show` already owns optional
live observations, HTML output and watching, while `fleet status` explicitly owns
local retained operation progress. Start with a separate, one-shot
`canic fleet placements` diagnostic rather than silently extending either budget.
The local Canic implementation now provides this command and selects Query at
Git revision `0a3eca919f05d777798406087049cc51ab7ae2a8`.

Use Canic's existing configuration resolution and app/fleet selection. Require
one configured mainnet fleet; do not offer a journal-only fallback. Select its
explicit `CanisterConfig.id` values in canonical slot-name order, retaining slot,
role and parent as application declarations. Show unbound slots as unbound and
make no lookup for them. Retained creation outputs and discovered children are
outside this first configured-binding view; label that coverage explicitly.

The Host adapter should parse every bound ID and reject duplicate identities or
more than 32 selected IDs before IO. Do not silently split a larger fleet into
multiple Query calls, because that would defeat the shared deadline. An all-unbound
fleet produces a local unbound view without calling Query and without claiming
successful placement collection. A nonempty admitted selection calls
`placement::collection::fetch` once, retaining the original typed report alongside
Canic's slot mapping. Per-row failures remain unknown placement. Canic owns the
text/JSON envelope and partial exit status; it must not parse `icq` text or add
an unbounded subprocess bridge.

The diagnostic must make no journal writes, load no signer, refresh no deployment
approval and leave Store's operation-bound checks intact. Its fixture acceptance
should cover local-network refusal, ambiguous selection, unbound slots, bad IDs,
duplicate/over-limit bindings, partial collection and preservation of source
evidence. Keep these with the Host adapter and CLI callers, not Query's wire tests.

Query packages remain unpublished. The collection was delivered to GitHub before
the authorized Canic adapter selected its immutable revision. No local path
override is used. The placement adapter was included in Canic 0.210.0 by its
owning release session; no crates.io publication was performed.

## Explicit two-subnet diagnostic

The `canic fleet compare-subnets LEFT RIGHT` diagnostic, delivered in Canic
0.211.0, uses the same configured app/fleet selection and mainnet admission. The operator selects both
candidates explicitly; Canic neither derives them from fleet bindings nor claims
that either belongs to the fleet. Its Host wrapper retains the complete Query
comparison report under `observation`, alongside app and fleet labels.

The CLI shows resources, configuration, independent coverage/version/time and
observed provider/data-center overlap in bordered tables. `--json` retains the
original evidence, failures and shared identities. Partial or all-failed collection
emits evidence with exit 1; complete collection exits 0 without suitability or
capacity ranking. Query's existing 160-second asynchronous bound and 774-logical-call
maximum are reused, without extra reads or new supplier dependencies. No journal,
signer, automatic candidate selection, watch mode or deployment action is involved.

## Later integration choices

Before adding automatic refresh or broader candidate selection, define its
consumer inputs and whether its refresh budget accommodates the existing pair
comparison (up to 160 asynchronous seconds).
The [selected placement collection](placement-collection.md) now admits 1–32
explicit IDs with per-canister outcomes and a shared 60-second deadline. Canic
must still select bound mainnet identities; Query does not discover the fleet.
Do not hide unbounded fanout inside a fleet refresh or use a universal query plugin.

Platform's rewrite does not require a speculative bridge in Query. Keep the
existing published reader boundary until the replacement's API and trust contract
are available. The independent [settings report](subnet-settings.md) now uses the
pinned Platform Protocol decoder. Default [comparison](subnet-compare.md) and
[maintenance](subnet-maintenance.md) reads are unchanged. Query's opt-in settings
comparison has its own 200-second budget and wrapper; this does not imply Canic
has adopted it or alter the existing Canic diagnostic.
