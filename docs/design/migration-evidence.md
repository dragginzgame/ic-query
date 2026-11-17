# ICRC balance and SNS neuron migration evidence

Status: implemented in `ic-sns-migration`, composed by the CLI's `icrc` and `sns`
adapters. The package name is retained. It supports migration consumers of
Motoko, Toko, Dragginz and similar applications; it remains independent of
`ic-topology`.

## Selection and purpose

Balances accept any explicitly selected ICRC ledger and account owner, with an
optional exact 32-byte subaccount. SNS membership is not required. Neuron reads
accept any explicitly selected compatible SNS Governance canister, with either
one exact 32-byte neuron ID or a bounded listing. Dragginz is the initial live
example, not an allowlist. Toko-specific live qualification is deferred at the
maintainer's request; there is no Toko-specific implementation branch.

These reads supply evidence for migration planning. They do not select destination
entitlements, transfer tokens, manage neurons, execute migrations or discover all
holders. There are no proposals, voting, swap, treasury analytics, ledger-history
or SNS-W discovery adapters. A principal in a neuron permission entry is not
necessarily its owner or sole beneficiary.

## Source and trust

The native adapter queries `https://ic0.app` with the mainnet root key and root
subnet pinned in this package. The key bytes match topology's existing mainnet
key, but the libraries do not depend on one another. Platform 0.34.11 owns
transport, certificates and query-signature verification. Query owns endpoint
admission, 300-second ingress expiry, certificate/delegation freshness (300-second
maximum age and 60-second future skew), and collection deadlines. Transport
redirects, proxy routing and retries remain disabled by Platform.

Each call has a 20-second whole-read timeout and a 1 MiB response budget. Parser
limits are explicit: 64 KiB header, 256 types, 4 million work units, 4 MiB logical
allocation, 4096 elements per vector and 512 KiB skipped-value work. Encoded
requests are limited to 1024 bytes. The certificate verifier allows 16384 tree
nodes, depth 64, 128 KiB delegation bytes and 4096 canister ranges. These are
logical bounds, not a process RSS guarantee.

Every observation retains method, local start/finish Unix milliseconds, a
`query_signature_verified` flag, `result_certified: false`, and either data or a
classified failure. A decoded/admission failure can still retain successful
transport authentication. Local clock failures remain explicit; absent timestamps
never become fabricated zero timestamps. Responses are observations of application
state, not consensus-certified balances or neuron records. Local times are not
state timestamps. Original response/signature bytes are not retained for replay.

## Balance contract

`balance::fetch(ledger, account)` reads `icrc1_balance_of`, then independently
`icrc1_decimals` and `icrc1_symbol`, within 60 seconds overall. All three outcomes
remain visible even if the first fails. A successful balance survives either
metadata failure; failed balances do not become zero. Each call uses an independent
current-state query and `atomic_snapshot` is always false.

Schema 1 records endpoint, ledger, exact owner/subaccount presence, three
observations and `complete`. Balances are exact base-unit decimal strings, limited
to 4096 bits before decimal expansion. Excess magnitude is an explicit admission
failure, never truncation. Decimals remain nat8, with no assumed default. Symbols
retain raw UTF-8 up to 256 bytes. Human output escapes source text and displays
base units without floating point conversions. Absent and explicit-zero
subaccounts are equivalent ICRC accounts but their requested representations remain
distinct in the evidence.

## Neuron contract

`neuron::fetch(governance, id)` uses `get_neuron`, requires a returned exact 32-byte
matching ID, and distinguishes missing results, Governance error variants (including
raw signed codes/messages), decode refusals and admission failures.

`neuron::list(ListRequest)` requests 100 records per page, with 1–20 pages admitted
(default CLI budget: one page) and 60 seconds overall. An optional `start_after`
cursor permits another bounded continuation without a cache or snapshot claim.
Each retained page has its
exclusive starting cursor and an independent observation. Unfiltered pages must
have strictly increasing, unique 32-byte IDs beyond the preceding cursor. A bad
page is rejected as a whole; earlier pages survive. Failed reads do not establish
an empty or exhausted source.

A short page marks `traversal_exhausted`; a full final page marks `page_limit`.
Separate pages do not share a certified version or snapshot: concurrent changes
can still affect collection. Exhaustion is scoped to the observed traversal, not
an immutable population census. At most 2000 neurons are retained.

For `of_principal`, upstream ignores `start_page_at`. Such requests therefore admit
exactly one page, validate uniqueness and matching permission principals, and mark
a full page `filter_limit`, without cursor retries or a completeness claim. Source
order is retained; no ordering assumption is imposed on filtered results.

The exact pagination behavior is pinned to
[SNS Governance implementation fe5a15be](https://github.com/dfinity/ic/blob/fe5a15bebf3490ef8f0cec5b5704422ee683c660/rs/sns/governance/src/governance.rs):
unfiltered pagination excludes the supplied cursor; filtered selection ignores it.

Protocol 0.3.2's selected neuron projection preserves ID bytes, ordered permission
entries (including absent principals and unknown signed codes), cached stake,
fees, maturity, optional staked maturity, creation/aging times, optional dissolve
state, auto-stake flag, vesting period, source NNS reference and raw voting-power
multiplier. Dissolve delay and absolute dissolve timestamp remain distinct.
Optional zero/false differs from absence. Raw e8s quantities stay separate; they
are not summed into spendable balance or destination entitlement.

This is not a full neuron export: `followees`, `topic_followees`,
`disburse_maturity_in_progress` and `latest_reward_event_participation` are omitted
by the supplier projection. Query does not infer their absence. The report exposes
selected Protocol DTOs directly; JSON preserves their raw integer and byte fields.
Consumers must use a JSON reader that preserves u64 values exactly.

## CLI and compatibility

- `icq icrc balance LEDGER OWNER [--subaccount HEX] [--json]`
- `icq sns neuron show GOVERNANCE NEURON_ID [--json]`
- `icq sns neuron list GOVERNANCE [--max-pages 1..20] [--start-after NEURON_ID] [--json]`
- `icq sns neuron list GOVERNANCE --principal PRINCIPAL [--json]` (no cursor or page-budget override)

The text adapter uses Host Text bordered rendering. JSON keeps all admitted fields,
method/time/authentication evidence and partial failures. Exit status is 0 for all
three successful balance observations, one successful neuron observation, or an
exhausted listing; 1 for failed/partial observations or limited traversal; 2 for
CLI argument errors. Limited output is still useful evidence and remains on stdout.
See the [CLI ledger](../../releases/cli/CHANGELOG.md) for copyable concrete examples.

These are additive commands and the shell's first query APIs. There is no old SNS
report/cache reader to retain or reset. Topology contracts and retained schema-1
evidence are unchanged. No legacy cache, source tree or release state is imported.

## Predecessor and qualification

Targeted predecessor inspection used
[IC Query Legacy 5b4951b](https://github.com/dragginzgame/ic-query-legacy/tree/5b4951b23c2adfcd3c615961f131e766c4b50283):
`src/sns/report/live/fetch/neurons.rs`, `live/types/neurons.rs`,
`live/convert/tests.rs`, `source/model/neurons/tests.rs`, and the ICRC
`src/icrc/live/fetch/account.rs` under `crates/ic-query/`.
The historical Root reporting design records its retirement in favor of explicit
Governance targets; generic discovery, health/status, caching and analytics remain
excluded here.

No legacy implementation code was reused. Retained invariants are exact balances,
separate stake/maturity/fees, raw permission codes, matching IDs, bounded replies,
unique page IDs and partial outcomes. Changed assumptions are strict published
Protocol ownership, metadata independent of balance success, retained absent
permission principals, and the pinned exclusive-cursor contract instead of the
legacy inclusive-boundary tolerance. Filtered pagination is explicitly bounded
because the upstream method ignores cursors.

Six small independent wire frames were copied byte-for-byte from committed
[Platform 4c30e4ba](https://github.com/dragginzgame/ic-platform/tree/4c30e4ba3db5d7bc6e624fc0ad740054766443c8/qualification/engine-view-wasm/fixtures):
`sns-neuron.bin`, `sns-list.bin`, `sns-error.bin`, `sns-missing.bin`,
`icrc-balance.bin`, `icrc-symbol.bin`. Their producer is the independent schema in
`qualification/engine-view-consumer/tests/sns.rs` at that revision. They are
consumer-test fixtures, not a new codec or a dependency on a sibling workspace.

Focused tests replay those frames through the consumer and exercise raw amounts
above u128, metadata failure, optional fields and unknown codes, wrong IDs,
Governance/missing/decode failures, invalid pages, page/filter limits, retained
prefixes, deadlines and cancellation. CLI tests check argument admission before
network IO and safe text versus raw JSON. Historical or supplier tests alone do
not qualify this consumer. Live checks are bounded observations, not continuous
availability, full-network interoperability or a complete migration export.
