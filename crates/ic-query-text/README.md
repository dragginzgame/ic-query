# IC Query Text

Human-facing presentation for typed reports from `ic-query`. Renderers return
strings; they do not collect reports, read caches, select terminals or write to
process output. JSON contracts remain in `ic-query`; reusable escaping and table
layout remain in `ic-host-text`.

Import report types and builders from `ic_query`, and rendering functions from
`ic_query_text` under the matching command-family namespace. Native report
families retain their existing feature selections; enable `host` for the CLI's
complete rendering surface.

This package and `ic-query-cli` share the CLI release group. See the
[group changelog](../../releases/cli/CHANGELOG.md).
