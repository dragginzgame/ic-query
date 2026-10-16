# 0.2 Design: Automatic SNS Proposal Cache

## Status

- Status: implemented; root-addressed cache lookup selected for 0.60
- Decision style: command behavior and cache boundary
- Primary command: `icq sns proposal list <root-principal>`

## Summary

SNS proposal listing now treats the complete local proposal snapshot as an
implementation detail for normal list views. If a valid current complete
snapshot exists, `icq sns proposal list <root-principal>` reads it and
applies view options locally.
If the snapshot is missing, the command visibly refreshes the complete
collection, publishes it only after governance pagination is exhausted, and
then renders the requested view.

Cache-backed requests address the root principal directly. Numeric list ids
remain supported by live lookup and explicit refresh, but no local operation
scans snapshots or attempt sidecars to reconstruct that id. Stored snapshots
and attempt evidence remain schema 1 and need no reset.

Manual commands remain available:

```bash
icq sns proposal refresh 1
icq sns proposal cache list
icq sns proposal cache status be2us-64aaa-aaaaa-qaabq-cai
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

Snapshots outside the current schema and required field set are rejected. The
reader does not detect, upgrade, or reinterpret predecessor shapes; refresh the
collection explicitly to create a current snapshot.

Proposal rows retain raw action and ballot-vote codes beside typed canonical
classifications. Public sources, refresh pages, and cache loads all enforce
that those pairs agree, that `ballot_count` equals the number of retained
ballot rows, and that proposal ids are unique within the collection. A
contradictory row is invalid source or cache evidence rather than a value the
view layer repairs.

Live and cached proposal rows share eligibility, proposer, case-insensitive
text matching and sorting in the view owner. Query normalization happens once
there while reports retain the original text. Live requests remain bounded by
the selected Governance page; cached reads apply before/status/topic scope first
and truncate only after the shared view, over the complete snapshot.

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
`--json` do not change snapshot identity.

Published cache files remain complete-only:

```text
<cache-root>/sns/ic/<root-principal>/proposals/full.json
```

Refresh attempts remain separate:

```text
<cache-root>/sns/ic/<root-principal>/proposals/full.refresh-attempt.json
```

If a refresh fails, the previous complete snapshot remains authoritative.
