# Cloud-engine marketplace and available-node evidence

Status: daily prices, separate fee evidence, bounded available-node traversal and
candidate-to-declaration matching
are implemented using published Platform 0.33.7 (the marketplace/availability
capabilities were introduced in 0.33.6).

## Canic decisions and scope

Canic needs public price inputs and candidate nodes when assessing whether to
add cloud engines or consolidate workloads. Select two bounded read-only reports:
marketplace price declarations first, then new-engine available-node pages.
Do not create/reserve engines, request binding quotes, manage subscriptions,
transfer tokens, acquire signing keys or inspect private owner/admin data.

The implemented surfaces are `cloud_engine::prices::fetch` /
`icq topology cloud-engine prices` and `cloud_engine::available::fetch` /
`icq topology cloud-engine available`. Both belong within `ic-topology`, with pure CLI presentation.
Do not add another crate or general canister-call interface.

`cloud_engine::candidates::fetch(page_limit)` / `icq topology cloud-engine candidates`
compose those collectors for Canic's candidate cost investigation. The join
identifies matching public declarations; it does not select an effective tariff.

Marketplace classes are raw source categories needed to interpret prices. Do not
restore Node Rewards reports or infer CPU, memory, storage, performance or
attestation from a reward-type label. Hardware specifications need a separately
identified source. Available-node listings are control-plane eligibility
observations, not reservations, liveness tests or measured spare capacity.

## Source verification and differences from legacy

On 2026-11-14, read the public `candid:service` metadata of mainnet engine canister
`q6cfj-fyaaa-aaaar-qb77q-cai` through `https://ic0.app`, selecting the mainnet root
key and anonymous identity with installed ICP CLI 1.6.0. The exact metadata
output SHA-256 was
`d9f0f383b3dff911d827060f92993ff92de557361df0b828fbec5b1159f68724`.
Reproduction for source review (not an `icq` command):

```bash
icp canister metadata q6cfj-fyaaa-aaaar-qb77q-cai candid:service \
  --network https://ic0.app --root-key mainnet --identity anonymous
```

Also read the public `getApiDoc` query, which specifies whole cycles per node per
day and timestamps as signed Unix nanoseconds. Anonymous, explicitly query-only
smoke calls returned 20 marketplace rows, all six price classes, network-fee
fraction `0.25`, and a caught-up Registry copy at version `65719`. A one-node
available-page request returned one row and a continuation cursor at that version.
These are point observations, not publication or product qualification.

Two legacy assumptions must not survive migration:

- Legacy price fields are named `*_cycles_per_month`; the live interface and
  API documentation explicitly define **daily** rates. Do not divide, multiply
  or relabel the raw response based on the old field names.
- Live `NodeType` includes plain `type4` in addition to `type4_1` through
  `type4_5`. The live marketplace includes plain `type4` rows. Do not silently
  discard them or decode the whole response using only the five legacy variants.

Current [ICP CLI interface declarations](https://github.com/dfinity/icp-cli/blob/8f98455d021f1f7dacf91f3244844f324a1c503b/crates/icp-canister-interfaces/src/engine_canister.rs)
confirm the default canister and subnet/operator binding, but do not declare
marketplace methods. Neither does the inspected
[1.3.0 interface](https://github.com/dfinity/icp-cli/blob/bf55d05b0b98ca7566acc3acc67764b7d931848d/crates/icp-canister-interfaces/src/engine_canister.rs).
The legacy wire module's broad ICP CLI attribution is therefore insufficient
provenance for pricing; use the observed live declaration and its digest.

## Candidate declaration matching

The live IDL above defines optional provider/DC scope: both absent is a global
default and absent DC is provider-wide. Its `getApiDoc` also describes separate
customer-specific prices. Neither inspected declaration nor guide establishes
precedence among overlapping public scopes. Do not infer cheapest-price,
most-specific-wins, or provider-before-DC billing rules. Matching all scopes is a
Query-owned evidence projection, not an implementation of the billing algorithm.

Collect availability first using its selected 1–100 page budget and 60-second
deadline, then prices using their existing 30-second deadline. Reuse both
collectors and their admission rules; do not add wire DTOs, retries, a second
pagination loop or normalization. These sequential network budgets total at most
90 seconds, plus bounded local projection. An initial availability failure emits
no report. Partial availability still proceeds to pricing. Any pricing failure
retains availability and marks matching coverage unknown. A fee-only failure
retains declarations and therefore still permits matching.

For each admitted node, retain every declaration with the same raw class and
either absent or equal provider and DC scope. DC comparison is exact: no case
folding, trimming or merging absent with present-empty. No declaration means
missing public price evidence, not free capacity. One declaration is still not a
quote. Multiple declarations remain unresolved, even if their amounts coincide.
No owner discounts, monthly totals, currency conversion or fee recalculation.

Schema 1 retains both complete input reports (or the typed price failure), total
elapsed time and node matches in acquisition order. Each match contains the node
principal and zero-based indices into retained `pricing.report.prices.entries`.
The admitted limits bound work to 10,000 nodes against 1,000 declarations and at
most four matching scopes per node. References avoid duplicating raw prices.
`collection_complete` means exhausted availability and both price reads succeeded;
it says nothing about matched-price coverage or current capacity. It controls CLI
exit 0/1; unmatched and multiply matched nodes are successful evidence outcomes.
`atomic_snapshot` stays false. All original source times, page versions, stop
reasons and verification flags survive composition.

The CLI shows zero/single/multiple match counts and one bordered row per matched
declaration, with exact daily net/gross amounts and matching-node count. Counts
overlap where scopes overlap. JSON retains node-level references and unreferenced
declarations. A failed pricing read is explicitly unknown rather than zero matches.

Before this join, rechecked frozen legacy
[`cloud_engine/build.rs`](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/cloud_engine/build.rs)
and [`tests.rs`](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/cloud_engine/tests.rs),
with the pricing history cited below. They validate raw declaration identity,
precision, sorting and provenance, but do not supply a candidate tariff-selection
contract. Keep those invariants through the existing collectors; do not carry
forward their monthly-unit labels or stricter DC normalization assumptions.

Focused join regressions cover all six classes, all four overlapping scopes,
wrong provider/class, case/whitespace and absent/empty DC distinctions, unmatched
nodes, partial collection and fee failure, initial failure/cancellation, pricing
failure retention, exact numeric evidence and maximum bounded input sizes.

Local Linux qualification on 2026-11-15 passed five join tests, 16 CLI unit tests
and 19 CLI process tests, affected-package all-target Clippy, warning-denied
topology Rustdoc, formatting and local links. Live text and JSON calls through
the fixed mainnet endpoint exited 0: 1,444 candidates in 15 pages at Registry-copy
version 65724, with 20 retained declarations. Each candidate matched one
declaration; the text summarized three matched rows. JSON collection started at
Unix second 1794731036 and took 3.91 seconds. An independent projection of the
retained JSON confirmed every match index. These observations do not verify the
billing algorithm, future availability or the pending batch on other native hosts.

## Price report contract

Read `listMarketplacePrices` with empty Candid arguments, then `getNetworkFee`
with empty arguments. Use the existing fixed mainnet root/endpoint and verified
anonymous query policy. Limit each HTTP response to 1 MiB, each read to 20 seconds
and the whole two-query collection to 30 seconds. No added retry, cache or scan.

Platform owns the external Candid projection and bounded codec. Selected decode
budgets are 512 KiB response bytes, 64 KiB header, 256 types, 2,000,000 work units
and 4 MiB logical allocation. These aggregate bounds apply even when each field
independently fits its limit. Query admits at most 1,000 rows, 512 UTF-8 bytes
per key and 256 per optional data-center reference. Preserve optional presence
and raw text; do not apply Registry data-center lowercase/trim policy to this
different source. Require unique keys matching the declared flattened identity
`[provider,]type[,dc]`, using the exact optional fields and source class spelling.
The key's class component is `type4` or `type4.1` through `type4.5`; the numbered
Candid tags instead use underscores. Keep this explicit mapping in the price
identity projection, separate from terminal formatting.

Retain raw net and gross cycle amounts as canonical decimal strings under
`net_cycles_per_node_per_day` and `gross_cycles_per_node_per_day`. Bound each to
256 decimal digits, using a bounded magnitude check before decimal conversion.
Reject gross below net. Preserve the signed source update timestamp, without
making positivity or recency an unsupported wire requirement. Refuse unknown
future variants explicitly rather than mapping them to a known class.

The network fee is a finite nonnegative float representing a markup on net.
Retain it separately; it is not a percentage of gross. Gross prices are already
supplied by the source: do not add the fee again or recompute historical rows
using a separately observed current fee. No currency conversion, monthly
projection, provider payout/reward report or total deployment quote is selected.

Schema 1 retains exact canister/method, local completion time and verification
flags per observation. Prices and fee are separate reads, not one atomic or
consensus-certified view. A price-read/admission failure produces no report.
If prices succeed but the fee fails or the shared deadline expires, retain the
prices with an explicit fee failure, incomplete status and CLI exit 1. Empty
successful price lists mean no returned declarations, not free engines.

## Available-node report contract

Use `registryCopyStatus`, then `listAvailableNodesPage` with `rejoining = null`.
This selects public eligibility for a new engine; no authenticated owner-specific
scope is selected. Request at most 100 rows per page; the default traversal allows
ten pages/1,000 unique nodes. `fetch_with_page_limit` and CLI `--max-pages` accept
1–100 pages, with a corresponding ceiling of page limit times 100 nodes (at most
10,000). Reject invalid limits before IO and retain selected `page_limit` and
`node_limit` in the report. The 60-second overall and 20-second leaf deadlines
do not increase with the requested page budget.
Retain source-copy status independently from each page's `registry_version`.
The status is an earlier observation, so its caught-up flag does not gate a later
page or pin its version. Initial status failure produces no report. After status
succeeds, even a failed first page returns that status with explicit partial
coverage and exit 1.

`ok = null` means the copy is not ready, not an empty marketplace. Preserve typed
application rejection separately from transport/codec failures. Require unique
node IDs and nonrepeating continuation cursors; do not assume principal ordering
from a declaration that only promises a fixed order. A page version change stops
the traversal with explicit partial coverage; do not silently restart or merge
versions. Retain admitted earlier pages and the failure/observed version.
Admit each page as a unit, so an invalid row never leaves an apparently valid
prefix from that page. Reject empty node/operator/provider principals and
data-center references over 256 UTF-8 bytes; retain present-empty and unnormalized
text within the bound. Decode under the price report's codec budgets plus a
128 KiB skipped-work budget for unselected fields, including IP text. Encode
page arguments within 512 bytes. Preserve raw application-error payloads in
Rust/JSON without rendering them as terminal diagnostics.

Even equal Registry versions do not create a snapshot: the canister declaration
says eligibility can change between pages as nodes become held. Report whether
pagination exhausted separately from completeness of a point-in-time inventory,
which is not established. A limit, failed page, unready copy or changed version
is explicit partial output with exit 1, never a successful complete census.
The report uses `pagination_exhausted` for traversal success and always sets
`atomic_snapshot` and `point_in_time_inventory_complete` false. Each admitted
page retains requested `start`, returned `next`, version and observation time.
`stop` identifies exhaustion, page/node limit, page failure or overall deadline;
a failed page retains its requested cursor and observed time/version when known.
The selected page/node ceiling with a continuation stops before any
extra request. An exhausted final page succeeds even at the exact limit. Empty
pages with new cursors continue within the page bound; no principal sort order
is assumed. Cancellation drops pending work and starts no further reads.
Larger budgets start at the first page and retain the same version/cursor checks.
This is not cross-command resume or a provider/region filter; the selected source
method does not expose those filters. No generic pagination layer is introduced.

Retain node, operator and provider principals, optional raw data-center text,
raw marketplace class and the source's `sev_enabled` boolean. Document that this
boolean means the copied node record carries a chip ID, not verified attestation
or usable TEE protection. Node IP addresses, broader Registry mirrors, reward
analytics, billing history and private reservations are outside this report.

## Published Platform request

Supplier delivery is tracked in
[Platform #166](https://github.com/dragginzgame/ic-platform/issues/166).

Platform 0.33.6 was published on 2026-11-14 and its archive was verified against
crates.io SHA-256
`24586654cc055365afd1c4bfb8dcd8f26f389bedba5cbee3dc722d99fce4a00a`.
The workspace selects its `cloud-engine-types` feature and `cloud_engine_types`
module, which supply the following contracts. The page decoder uses a strict
optional success payload: an unknown nested class fails instead of becoming
`ok null`. It also supports a separate skipped-work limit for unselected IP text.
Pricing uses the first two methods; availability uses the latter two. Publication
does not replace Query's collection, admission and presentation qualification.

| Method | Arguments | Selected response |
| --- | --- | --- |
| `listMarketplacePrices` | Empty | Vector of key plus node class, optional provider/DC, arbitrary natural net/gross, signed update timestamp |
| `getNetworkFee` | Empty | Float64 |
| `registryCopyStatus` | Empty | Nat64 version/latest and boolean caught-up status |
| `listAvailableNodesPage` | Optional text rejoining, optional principal start, optional natural limit | Lowercase `ok` optional page / `err` EngineError; page has nodes, Nat64 Registry version and optional principal next |

Use the existing owned Candid codec, caller-supplied decode budgets and exact
single-result arity. Preserve optional `None` versus empty vector/text, full
natural and int64 ranges, principal identity and all six known node-class tags.
The page projection may skip unneeded `ip_address` through the codec's bounded
record-width handling. Support the declared EngineError variants and raw text
payloads without turning source messages into terminal diagnostics.

Supplier regression cases must cover independently authored literal fixtures,
plain `type4` and every class tag, large naturals, signed timestamps, optional
presence, page cursors, `ok null`, application errors, malformed/extra-argument
frames, unknown variants and each resource limit. Qualify a small public consumer
without direct Agent/Candid/CDK or a second codec dependency. Platform owns wire
semantics only; Query owns endpoint policy, deadlines, limits, field admission,
pagination, price units, provenance and interpretation.

Do not add Query-local external DTO macros, copied decoders or path substitutions. This
document is the consumer contract, not a second supplier issue tracker.

## Query integration acceptance

Prices form the first library/CLI slice. Its focused checks exercise the published
decoder with independently authored literal response frames and test Query-owned
behavior through the source boundary:

- All six classes and exact key mapping; default, provider-only, DC-only and
  provider/DC identities; duplicate or mismatched keys; exact/excess row, UTF-8
  and decimal-digit limits; gross below net. Keep absent and present-empty DCs
  distinct and retain full signed timestamps.
- Reject nonfinite and negative fee values without losing admitted prices;
  preserve zero and valid markup values. Do not recompute gross prices or infer
  monthly units. JSON uses exact decimal strings for cycle amounts.
- First-read failure starts no fee read. Fee failure and the shared deadline
  retain prices with distinct typed outcomes. Cancellation starts no later work.
  Test the whole deadline spanning both reads, not only independent leaf limits.
- Compact human output escapes source text and labels daily units. JSON retains
  both observations and verification flags. Exercise complete/partial exit
  behavior, namespace help and the absence of an arbitrary-method CLI path.

Availability is a separate slice. Its consumer checks distinguish
unready from an empty exhausted page, reject repeated cursors and duplicate nodes,
enforce page/total bounds and preserve prior pages on version change, application
error or deadline. Exhausted pagination still makes no atomic-inventory claim.
Supplier codec tests do not replace these collection and presentation checks.

Local Linux validation on 2026-11-14 passed eight pricing tests, 14 CLI unit
tests and 16 CLI process tests, affected-package all-target Clippy and
warning-denied topology Rustdoc. The pricing literal is independently authored
from the selected IDL; DTO-encoded fixtures exercise report admission, including
1,000 rows with 256-digit amounts. Source fixtures verify cancellation and a
30-second deadline spanning two individually shorter reads. This is focused
consumer evidence, not a new native-host matrix qualification.

After refreshing `icq` on PATH, live text and JSON queries both exited 0 with
20 price declarations, all six raw classes and markup `0.25`. The text collection
took 0.35 seconds and recorded Unix second `1794682728`. JSON validation confirmed
canonical digit-only cycle strings and `complete: true`. These are bounded point
observations, not an availability census, price promise or freshness guarantee.

Availability qualification on the same Linux host passed 11 focused tests:
independently authored page/request literals, unknown nested-class refusal,
bounded skipped IP text, cursor identity/cycles, duplicate nodes, changed/full-width
versions, empty versus unready pages, all application errors, exact/excess limits,
shared deadline and cancellation. The eight price tests also passed after sharing
the private cloud-engine transport. CLI validation passed 15 unit and 17 process
tests, plus affected all-target Clippy and warning-denied topology Rustdoc.

Live availability text and JSON runs on 2026-11-14 each admitted 1,000 unique nodes
across ten pages at Registry-copy version `65719`, then returned exit 1 with
`stop.reason = "node_limit"` and the final continuation retained. The JSON run
started at Unix second `1794683530` and took 3.23 seconds; its cursor chain,
unique IDs, version consistency, absent IP fields and partial-coverage flags were
checked. The text run took 2.67 seconds and grouped the retained nodes into
39 class/data-center rows. The source had more candidates than the selected
bound; neither run qualifies an exhausted live traversal or full inventory.
Exhausted and failure paths are covered by the source fixtures above.

The page-budget follow-up on 2026-11-15 passed 14 availability tests, including
1-, 20- and 100-page exact limits, invalid budgets before IO and the unchanged
60-second deadline. CLI validation passed 15 unit and 18 process tests; affected
Clippy, Rustdoc, formatting and documentation checks also passed. `icq` was
refreshed on PATH. A live `--max-pages 20 --json` run exited 0 after 15 pages and
1,444 unique nodes at Registry version `65724`, starting at Unix second
`1794730512` and taking 4.00 seconds. Cursor continuity, version consistency,
null final cursor, selected budgets and false snapshot/completeness claims were
verified. This qualifies an exhausted live traversal as a point observation,
not an atomic inventory or a promise that future traversals will fit 20 pages.

## Legacy lessons

Review used `ic-query-legacy` revision `5b4951b23c2adfcd3c615961f131e766c4b50283`:
[source](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/cloud_engine/source.rs),
[wire projection](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/cloud_engine/wire/mod.rs),
[admission](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/cloud_engine/build.rs),
[report tests](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/cloud_engine/tests.rs)
and [wire tests](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/cloud_engine/wire/tests.rs).
History `0f242c5e467a5e2ec1d4ab28de68678f4cc0bcd8` moved protocol work toward
owned Platform codecs; `8435331c8e66621d3fd5350dfd298506ed181e6a` consolidated
adapters and caches. Retain bounded collection, unique identity, raw exact cycle
amounts, source validation and non-atomicity. Replace the stale units/class set,
move external DTO ownership to Platform, and omit unrelated operator settings,
cache infrastructure and Dashboard reward-derived node classification.

For available-node work, also reviewed the frozen
[node source](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/cloud_engine/node/host.rs),
[model](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/cloud_engine/node/model.rs)
and [regressions](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/cloud_engine/node/tests.rs),
plus path history `8435331c8e66621d3fd5350dfd298506ed181e6a` and
`b1fca76345efb465d327984d70e9862e1127bbe9`. That report selected complete
Dashboard Type4/status-filtered results; it did not implement this page contract.
Carry forward source-scope/identity admission and duplicate-node refusal.
Replace reward/status filtering and complete-list assumptions with direct public
eligibility pages and explicit traversal coverage. No legacy source was copied.
