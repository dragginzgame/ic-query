# Changelogs

Each independently versioned release group owns its release notes. This file is
the navigation index; there is no aggregate workspace release or root release ledger.

| Group | Package | Release notes |
| --- | --- | --- |
| Topology | `ic-topology` | [Topology changelog](releases/topology/CHANGELOG.md) |
| SNS migration | `ic-sns-migration` | [SNS migration changelog](releases/sns-migration/CHANGELOG.md) |
| CLI | `ic-query-cli` (`icq`) | [CLI changelog and copyable command examples](releases/cli/CHANGELOG.md) |

Group ownership is declared in [ci/cargo-release-groups.json](ci/cargo-release-groups.json).
See the [restart design](docs/design/0.104-restart.md) for version continuity and
consumer-retirement obligations. Published legacy history remains in the
[legacy repository](https://github.com/dragginzgame/ic-query-legacy).
