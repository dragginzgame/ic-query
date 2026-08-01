# Documentation

This index separates user guidance, current architecture, planning, and
historical release material.

## Use ic-query

| Document | Purpose |
| --- | --- |
| [Project README](../README.md) | Installation, quick start, supported reporting, and trust model |
| [CLI Usage](cli-usage.md) | Current command hierarchy, target identity, and collection modes |
| [Library Usage](library-usage.md) | Rust feature boundary, adapters, builders, caches, and examples |
| [Roadmap to 1.0](roadmap/1.0.md) | Coverage estimates, prioritized workstreams, and the 1.0 completion bar |
| [Changelog](../CHANGELOG.md) | Concise release ledger |

Command help is the definitive option reference:

```bash
icq help
icq help nns topology
icq icrc account transaction --help
```

## Current design contracts

| Document | Contract owned |
| --- | --- |
| [IC Reporting Adapters](design/ic-reporting-adapters.md) | Authority families, provenance, validation, and typed follow-up queries |
| [Cache Policy](design/cache-policy.md) | Cache identity, refresh behavior, locking, and local inspection |
| [0.22 Structural Consolidation](design/0.22/0.22-design.md) | Ordered CLI, inventory, snapshot, feature, and module-ownership cleanup |
| [0.23 Bounded SNS Completeness](design/0.23/0.23-design.md) | Targeted discovery hard cut plus bounded native swap and upgrade evidence |
| [0.24 Bounded SNS Governance Metrics](design/0.24/0.24-design.md) | Bounded proposal-window, cached treasury, voting-power, and ledger-timestamp evidence |
| [IC Dashboard Canister Reporting](design/ic-dashboard-canister-reporting.md) | Official REST authority and bounded canister detail, count, and page reports |
| [IC Dashboard Network Metrics](design/ic-dashboard-network-metrics.md) | Official Metrics API authority, bounded time-series queries, and raw series contract |
| [IC Dashboard Daily Statistics](design/ic-dashboard-daily-stats.md) | Bounded daily network activity, selected raw rate fields, and one-request contract |
| [IC Dashboard Boundary-Node Reporting](design/ic-dashboard-boundary-node-reporting.md) | Official boundary-node data-center aggregates, raw location fields, and one-request contract |
| [Exact-Version NNS Subnet Topology](design/nns-subnet-topology.md) | One-version Registry join and atomic topology cache |
| [NNS Governance Reporting](design/nns-governance-reporting.md) | Economics, metrics, reward event, and maturity modulation |
| [NNS Neuron Reporting](design/nns-neuron-reporting.md) | Public neuron views and complete collection behavior |
| [SNS Root Canister Reporting](design/sns-root-canister-reporting.md) | Root inventory, health, provenance, and relation gaps |
| [SNS Proposal Cache](design/sns-proposal-cache.md) | Complete SNS proposal snapshot behavior |
| [Certified CMC System Reporting](design/cmc-system-reporting.md) | Certified ICP/XDR evidence and exact cycles-per-ICP derivation |

These documents describe the current contract. Detailed release notes under
[`docs/changelog/`](changelog/) explain when each contract changed.

## Historical material

Completed numbered directories under [`docs/design/`](design/) preserve
earlier design work and implementation handoffs. They are historical context,
not a compatibility promise; a numbered document marked active remains a
current implementation plan. The current code, current design contracts above,
`AGENTS.md`, and the root changelog take precedence.

Engineering audit material under [`docs/governance/`](governance/) records
past code-hygiene reviews and is not a user-facing API specification.
