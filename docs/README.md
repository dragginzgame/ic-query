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
icq nns topology help
icq icrc account transaction help
```

## Current design contracts

| Document | Contract owned |
| --- | --- |
| [IC Reporting Adapters](design/ic-reporting-adapters.md) | Authority families, provenance, validation, and typed follow-up queries |
| [Cache Policy](design/cache-policy.md) | Cache identity, refresh behavior, locking, and local inspection |
| [IC Dashboard Canister Reporting](design/ic-dashboard-canister-reporting.md) | Official REST authority and bounded canister detail, count, and page reports |
| [IC Dashboard Network Metrics](design/ic-dashboard-network-metrics.md) | Official Metrics API authority, bounded time-series queries, and raw series contract |
| [Exact-Version NNS Subnet Topology](design/nns-subnet-topology.md) | One-version Registry join and atomic topology cache |
| [NNS Governance Reporting](design/nns-governance-reporting.md) | Economics, metrics, reward event, and maturity modulation |
| [NNS Neuron Reporting](design/nns-neuron-reporting.md) | Public neuron views and complete collection behavior |
| [SNS Root Canister Reporting](design/sns-root-canister-reporting.md) | Root inventory, health, provenance, and relation gaps |
| [SNS Proposal Cache](design/sns-proposal-cache.md) | Complete SNS proposal snapshot behavior |

These documents describe the current contract. Detailed release notes under
[`docs/changelog/`](changelog/) explain when each contract changed.

## Historical material

Numbered directories under [`docs/design/`](design/) preserve earlier design
work and implementation handoffs. They are historical context, not a
compatibility promise. The current code, current design contracts above,
`AGENTS.md`, and the root changelog take precedence.

Engineering audit material under [`docs/governance/`](governance/) records
past code-hygiene reviews and is not a user-facing API specification.
