# Item-level rustdoc debt

`scripts/ci/check-public-docs.pl` compares Cargo/rustdoc JSONL missing-doc warnings
with an explicitly selected JSON baseline of item identities. It uses Perl and
its core JSON::PP module, and runs no Cargo command, installer, network request
or source edit.
Query owns this temporary debt checker, its fixtures and this guide alongside
`scripts/ci/check-public-docs.sh` and `scripts/ci/public-docs-baseline.json`.
They are not Shared Tooling snapshot payloads. Snapshot refreshes must preserve
these local files; do not select them as shared entrypoints.

Run from the source checkout so relative diagnostic paths name the intended
files. The caller owns source/diagnostic custody, package/features, compiler
selection, warning policy and admission of Cargo's exit status. A warning set is
not proof that compilation completed successfully. Preserve failed compiler logs
before invoking the comparison. For an already admitted diagnostic file:

```bash
perl scripts/ci/check-public-docs.pl target/rustdoc.jsonl scripts/ci/public-docs-baseline.json
```

Identities include source path, owner scopes, item kind and declared name.
Raw identifiers use their logical name without `r#`, including owner scopes;
Unicode owner names and lowercase enum variants retain their own field namespaces.
Line movement does not change an identity. Multiplicity is retained: documenting
an unrelated field cannot offset new debt, and resolved baseline entries must
be removed explicitly. Added or resolved items return status 1; malformed JSON,
missing source/baseline and unsupported macro identities refuse with nonzero
status. Success does not rewrite the caller's baseline or sources.

The small scanner skips literals, raw strings and nested comments while finding
struct/enum/variant/impl owners. It is not a Rust parser, resolver or API inventory.
Expanded diagnostics need explicit identities and remain refused. Compiler
warning shape and source custody remain consumer qualification obligations.

Baseline creation/replacement is a separately selected operation:

```bash
perl scripts/ci/check-public-docs.pl target/rustdoc.jsonl scripts/ci/public-docs-baseline.json \
  --write-baseline
```

That option writes sorted identities only after complete input processing. It
must not be used automatically to make a failing gate pass. Declaration identities
use the declared name, fixing consumer keys that previously used keywords such
as `struct` or `mod`. Adoption requires explicit review of any
changed baseline identities; no fallback identities or automatic rewrite exists.

Query retains its Cargo wrapper, current debt file and section-style type-doc
checker; those have separate contracts. Query's Perl identity/CLI fixtures
use synthetic admitted compiler records and temporary sources. The separate `test-public-docs.sh` fixture runs actual locked,
offline Cargo/rustdoc on a dependency-free crate to qualify declaration names and
unchanged-count debt replacement. Run both fixtures when changing the checker;
the wrapper's compiler-failure retention remains covered by the consumer CI
script fixtures. Once the debt is resolved, replace the ratchet with rustdoc's
`-D missing-docs` and remove the baseline and item-identity machinery.
