# 0.2 Design: Automatic SNS Proposal Cache

## Status

- Status: implemented for the 0.2.5 slice
- Decision style: command behavior and cache boundary
- Primary command: `icq sns proposals <id|root-principal>`

## Summary

SNS proposal listing now treats the complete local proposal snapshot as an
implementation detail for normal list views. If a compatible complete snapshot
exists, `icq sns proposals` reads it and applies view options locally. If the
snapshot is missing, the command visibly refreshes the complete collection,
publishes it only after governance pagination is exhausted, and then renders
the requested view.

Manual commands remain available:

```bash
icq sns proposals refresh 1
icq sns proposals cache list
icq sns proposals cache status 1
```

## Behavior

Automatic cache use applies when the requested view can be reproduced from the
cached proposal rows:

- `--limit`
- `--before`
- `--sort api`
- `--sort id`
- `--sort status`
- `--sort topic`
- `--sort proposer`
- `--sort title`
- `--sort action`
- `--sort action-id`
- `--sort yes`
- `--sort no`
- `--sort total-votes`
- `--sort tally-time`
- `--sort ballots`
- `--sort eligible`
- `--sort reject-cost`
- `--sort reward-round`
- `--sort reward-end`
- `--sort created`
- `--sort decided`
- `--sort executed`
- `--sort failed`
- `--asc`
- `--desc`
- `--status any`
- `--status open`
- `--status decided`
- `--status adopted`
- `--status rejected`
- `--status executed`
- `--status failed`
- `--topic <topic>`
- `--eligible any`
- `--eligible yes`
- `--eligible no`
- `--proposer <neuron-id-prefix>`
- `--query <text>`

Older complete proposal snapshots that predate cached raw proposal status
codes or topic labels are refreshed before status or topic filters that depend
on those fields are applied.

## Cache Contract

The cache key describes the collected data, not the rendered view:

```text
domain: sns
network: ic
entity: <root-principal>
collection: proposals
scope: full
```

View options such as `--limit`, `--before`, `--sort`, `--asc`, `--desc`,
`--status`, `--topic`, `--eligible`, `--proposer`, `--query`, `--verbose`, and
`--format` do not change snapshot identity.

Published cache files remain complete-only:

```text
.icq/sns/ic/<root-principal>/proposals/full.json
```

Refresh attempts remain separate:

```text
.icq/sns/ic/<root-principal>/proposals/full.refresh-attempt.json
```

If a refresh fails, the previous complete snapshot remains authoritative.
