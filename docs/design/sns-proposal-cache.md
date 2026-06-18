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
- `--sort created`
- `--sort decided`
- `--sort executed`
- `--sort failed`
- `--asc`
- `--desc`
- `--status any`
- `--status open`
- `--status decided`
- `--status executed`
- `--status failed`

The existing bounded live path remains in use when the requested view cannot
yet be reproduced from cached rows without changing semantics:

- `--topic <topic>`
- `--status adopted`
- `--status rejected`

Those live fallbacks should move to cache-backed views once cached proposal
rows carry enough source detail to classify proposal topic and final adopted
versus rejected state locally.

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
`--status`, `--topic`, `--verbose`, and `--format` do not change snapshot
identity.

Published cache files remain complete-only:

```text
.icq/sns/ic/<root-principal>/proposals/full.json
```

Refresh attempts remain separate:

```text
.icq/sns/ic/<root-principal>/proposals/full.refresh-attempt.json
```

If a refresh fails, the previous complete snapshot remains authoritative.
