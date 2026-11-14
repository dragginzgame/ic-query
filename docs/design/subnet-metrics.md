# Certified subnet resource observations

`icq topology subnet metrics SUBNET_ID [--json]` supplies one selected subnet's
resource observations for Canic placement comparisons. The report is a useful
input to workload planning, not an estimate of available capacity or a deployment
recommendation. It does not measure uptime, rates or a maintenance interval.

## Source and meaning

Read `/subnet/<id>/metrics` from the subnet-scoped mainnet read-state endpoint.
The published Platform 0.33.0 Agent already exposes `read_state_subnet_metrics`;
locked Agent 0.49.3 verifies subnet authority and decodes the external CBOR.
Query adds no external protocol types, codec or new supplier dependency.
The [IC state-tree specification](https://docs.internetcomputer.org/references/ic-interface-spec/#subnet-information),
checked 2026-11-14, defines the four selected fields:

| Field | Meaning and limit |
| --- | --- |
| `num_canisters` | Current count; deletion can reduce it |
| `canister_state_bytes` | Current canister state size; refreshed every ten blocks, initially zero before the first refresh |
| `consumed_cycles_total` | Nominal-cycle accounting total; refunds and subnet splits can decrease it, and inherited histories can predate the subnet |
| `update_transactions_total` | Cumulative messages executed in replicated mode since subnet creation or metric introduction |

Do not turn nominal cycles into a price, assume monotonic cycle consumption,
label the state-size gauge as a lifetime total, or derive throughput from one
sample. Zero observations are valid; missing or malformed fields are failures.

## Report, bounds and ownership

Schema `1` retains the selected subnet, all four raw values and endpoint, HTTP
path, tree path, local completion time and certificate-verification evidence.
The library keeps cycles as `u128`; JSON encodes that value as an exact decimal
string. Counts and bytes remain raw `u64` JSON numbers. Text rendering is owned
by the CLI and labels units and limitations without floating-point conversion.

Reuse the mainnet owner's anonymous Agent, built-in root, 1 MiB HTTP ceiling,
20-second asynchronous deadline, 300-second certificate-age policy and zero TCP
retries. Agent owns synchronous certificate/CBOR work; the deadline does not
preempt that work. Local completion time is not a certificate timestamp. The
Agent's timestamp policy does not independently reject future certificates.
An unavailable observation is an error, not zero resource use or subnet downtime.
No Registry version, current free capacity, multi-source atomic snapshot or
historical coverage is implied. No polling or persistence is added.

## Predecessor review and changed assumptions

Reviewed `ic-query-legacy` at `5b4951b23c2adfcd3c615961f131e766c4b50283`:

- [Metrics design](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/docs/design/ic-dashboard-network-metrics.md),
  [live adapter/tests](https://github.com/dragginzgame/ic-query-legacy/tree/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/ic/live/metric)
  and [source validation](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/ic/source/metric.rs)
  preserve exact values, explicit source authority and bounded collection. Retain
  those invariants and precision/argument regressions. The old source was an
  uncertified network-wide REST time series; its metric enum, range options,
  HTTP adapter and source traits do not fit this single certified subnet read.
- [0.27.5 history](https://github.com/dragginzgame/ic-query-legacy/commit/4537a7c7175154c5ea22e487ff4d7ec457a9398d)
  tightened credential-free endpoint/no-redirect documentation. The rewrite's
  fixed mainnet endpoint avoids restoring arbitrary endpoint configuration.
- [Registry topology projection/tests](https://github.com/dragginzgame/ic-query-legacy/tree/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/ic_registry/projection/subnet_topology)
  reject duplicate assignments and missing provider relations. Those belong to
  a future joined Registry inventory, not to these resource values. Do not
  manufacture membership or provider diversity from metrics.

This implementation retains first-party lessons, not copied source. The current
mainnet helper and CLI adapter are reused. Offline checks cover full-width/zero
serialization, truthful presentation and local argument/help behavior; existing
shared transport tests cover errors and deadlines. They do not prove certificate
verification. A separately recorded live read exercises the published supplier
against mainnet; it does not establish continuous availability.

The 2026-11-14 smoke read for
`tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe` through
`https://ic0.app` succeeded at local Unix time `1794656158`. It returned 18
canisters, 32,403,397,318 state bytes, `193409221753315771` nominal cycles and
6,904,005,183 replicated messages, with certificate verification successful.
This is one observation under the documented freshness policy, not a history.
