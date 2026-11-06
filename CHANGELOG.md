# Changelog

| Release group | Packages | Current notes |
| --- | --- | --- |
| Query | `ic-query`, private `ic-query-testkit` | [Query changelog](releases/query/CHANGELOG.md) |
| CLI and text | `ic-query-cli` (`icq`), `ic-query-text` | [CLI changelog](releases/cli/CHANGELOG.md) |

[Coordinated history through 0.85.0](releases/history.md) retains earlier releases
once, under their original scope. Both groups keep the existing workspace,
dependency catalog and lockfile; release/publication commands require an explicit
`RELEASE_GROUP=query`, `RELEASE_GROUP=cli`, or the supplier-first
`RELEASE_GROUPS=query,cli` batch. See the
[frozen release workflow](docs/release-workflow.md).
