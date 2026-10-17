# Selected type documentation layout

`check-type-docs.pl` checks section-style documentation for source-visible public
`struct`, `enum` and `trait` declarations, including restricted visibility and
indented declarations. Select source roots explicitly; the caller owns which
packages/files adopt this convention. Raw identifiers use the name without
`r#` in their documentation heading; Unicode identifiers are inspected too.
The checker runs no Cargo, installer,
network operation or source rewrite. Perl core modules are sufficient.

```bash
perl scripts/ci/check-type-docs.pl crates
```

For each selected declaration the required block contains an empty `///` line,
the declared name, another empty line, one or more description lines, a closing
empty `///` line, then a blank source line before attributes or the item.
Multiline/nested bracket attributes are admitted. Strings, raw strings and nested
comments are excluded from declaration discovery. Violations return status 1;
missing/empty selections, unreadable sources and unsupported attribute/literal
shapes refuse with a nonzero status. Overlapping selections inspect a path once;
symlink roots refuse and directory traversal does not follow symlinks.

This is a focused source-layout check, not a Rust parser, visibility resolver,
public API inventory or missing-documentation comparator. It cannot expand
macros or prove whether a source-visible declaration is exported. Callers retain
compiler/rustdoc validation and review generated declarations separately. Type
aliases are outside this check's selected declaration kinds.

Select the checker as a canonical snapshot entrypoint; the export includes this
guide and `scripts/ci/RustSource.pm`. Type-layout and public-documentation debt
checks share that core-only tokenizer while retaining their own selection policy.
Run `perl scripts/ci/test-type-docs.pl` for isolated selection/layout fixtures.
Expected rejection diagnostics stay in per-case logs; an unexpected result
replays its output and retains the fixture directory for inspection.
