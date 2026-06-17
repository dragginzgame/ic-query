# Codebase Hygiene Standard

## Purpose

This directory defines source consistency and readability standards for the
`ic-query` crate and the `icq` executable.

The goal is to keep the codebase easy to navigate while preserving
`ic-query`'s read-only query boundaries:

```text
cli parsing -> command runtime -> report construction -> source/cache IO -> model/rendering
```

If this directory conflicts with repository `AGENTS.md`, `AGENTS.md` wins.
Release, versioning, and changelog boundaries remain owned by the root project
instructions and root `CHANGELOG.md`.

This standard is not a request to restructure modules broadly. Use it when
adding new code, splitting already-targeted files, or reviewing cleanup for
consistency with the current command/report/cache shape.

## Example Crate

The `example-crate/` tree is documentation-only Rust that models preferred
`ic-query` crate and module shape. It intentionally has no `Cargo.toml`, is
outside the Cargo workspace, and must not own package metadata or version
numbers.

```text
example-crate/
└── src/
    ├── lib.rs
    ├── diagnostic.rs
    ├── query/
    │   ├── mod.rs
    │   ├── request.rs
    │   ├── snapshot.rs
    │   └── tests.rs
    └── report/
        ├── mod.rs
        └── text.rs
```

The example demonstrates module-level ownership headers, top-of-file ordering,
grouped imports, narrow visibility, public item docs, invariant-bearing
constructors, typed diagnostics, leaf-local inline tests, and boundary-level
`tests.rs`.

When copying from it:

1. Copy structure and ordering, not the example domain names.
2. Keep live host calls, cache reads, and rendering in their owning modules.
3. Use scoped visibility before widening a symbol to `pub`.
4. Put cross-module tests in the owner boundary instead of burying them in a
   leaf module.
5. Keep examples formatted with `rustfmt`.

## 1. Import Organization

Keep imports grouped, stable, and only at the top of each file.

Preferred grouping:

1. `crate` imports
2. `std` imports
3. external crate imports

Rules:

1. Avoid `super::super::...` paths.
2. Avoid `super::...` outside tests unless narrowly justified.
3. Prefer grouped `crate::{...}` imports over scattered long paths.
4. Group imports by root instead of repeating the same path throughout a file.
5. Keep normal imports, re-exports, and module declarations in their own
   blocks.
6. Keep `#[cfg(...)]` imports in the same conceptual block they would occupy
   without the `cfg`.
7. When deriving or implementing `Display`, prefer
   `use std::fmt::{self, Display};` for consistency.

Required top-of-file sequence for module files:

1. `mod ...;` declarations
2. one blank line
3. `use ...;` imports
4. one blank line
5. re-exports: `pub use ...;`, `pub(crate) use ...;`,
   `pub(in ...) use ...;`
6. one blank line
7. constants, types, functions

`#[cfg(test)] mod tests;` belongs with other `mod` declarations.

## 2. Module Header Comments

New non-trivial modules should begin with a module-level documentation header
that states responsibility and boundary.

Keep the first doc paragraph short. Clippy's
`too_long_first_doc_paragraph` lint treats consecutive `//!` lines as one
paragraph, so put a blank doc line after the one-line module name.

Example:

```rust
//! Module: sns::neurons::refresh
//!
//! Responsibility: orchestrate complete SNS neuron snapshot refreshes.
//! Does not own: command parsing, token rendering, or cache file primitives.
//! Boundary: fetches pages through a source and commits complete snapshots.
```

Use these headers to prevent architectural drift. Keep them current when module
ownership changes.

## 3. Type Documentation

Public structs, enums, and traits should document:

1. what the type represents
2. which layer owns it
3. where it is used

Use the repository's scan-friendly section style above structs, enums, and
traits when the type is part of a report, command contract, cache contract, or
source contract:

```rust
///
/// SnsNeuronRefreshRequest
///
/// Request accepted by the SNS neuron refresh report builder.
///
```

Spacing rule for documented type declarations:

1. Leave one blank line before the doc comment block.
2. Leave one blank line after the doc comment block and before attributes or
   the item declaration.
3. Keep related type, inherent impl, and trait impls together when feasible.

Error enum formatting:

1. Keep one blank line between variant blocks.
2. Prefer alphabetical variant order by variant name when no semantic ordering
   is stronger.
3. Assert typed errors in tests; do not test error strings.

## 4. Function Documentation

Comment intent, invariants, ownership, or non-obvious tradeoffs only.

Use item documentation when a function:

1. is public API
2. enforces invariants
3. crosses a layer boundary
4. can panic through a public contract
5. performs non-obvious orchestration or policy work

Avoid comments that restate the next line or describe stale implementation
history. Private helpers do not need comments when the name and local context
are clear.

Ordering rule for documented items with attributes:

1. docs/comments
2. attributes
3. function declaration

Public APIs with reachable panic paths must include a `# Panics` section naming
the condition. Prefer typed errors when callers can recover.

## 5. Section Banners

Use section banners only when grouping multiple related functions in a large
module.

Example:

```rust
// -----------------------------------------------------------------------------
// Validation
// -----------------------------------------------------------------------------
```

Do not add banners to small files where the type and function order is already
obvious.

## 6. Function Ordering

Prefer stable ordering:

1. public API
2. constructors/builders
3. core logic
4. helpers/utilities
5. tests

When a type and its impls live in the same file:

1. place the inherent impl below the type when feasible
2. follow with trait impls for that type
3. keep each type family together

## 7. Function Size

Functions longer than roughly 80 lines should be reviewed for decomposition.

Split by semantic phase when possible:

1. validation
2. mapping/conversion
3. decision/policy
4. execution
5. commit/persistence
6. cleanup

Avoid deeply nested logic blocks. Large `match` bodies should dispatch to
helpers.

## 8. Visibility and Layer Boundaries

Minimize visibility by default.

Guidance:

| Layer | Visibility |
| --- | --- |
| CLI parsing and dispatch | private or `pub(crate)` |
| command option structs | `pub(crate)` unless exported from the crate API |
| report models and builders | private or `pub(crate)` by default |
| source traits and live fetch helpers | `pub(crate)` unless tests or public API require wider |
| cache path/read/write mechanics | private or `pub(crate)` |
| reusable formatting helpers | `pub(crate)` unless intentionally public |
| JSON output models | stable/raw fields; visibility only as wide as serialization requires |

Never widen visibility to bypass command family or cache boundaries.

## 9. Invariants and Error Semantics

Production paths should return typed errors, not stringly failures.

Rules:

1. CLI modules parse arguments and dispatch; they should not build reports by
   hand.
2. Report modules construct report data, call sources through traits or local
   helpers, and keep live calls visible in command behavior.
3. Cache modules own cache paths, locking, JSON loading/writing, atomic
   replacement, and refresh-attempt state.
4. Text output may be compact and human-facing; JSON output should preserve
   raw script-friendly fields and avoid lossy display conversions.
5. Cache keys describe collected data, not view options such as sort, limit,
   verbosity, or text format.
6. NNS and SNS command families stay separate unless a helper is genuinely
   shared.

Avoid `unwrap()` and `expect()` outside tests and intentional invariant checks.
For cache refreshes, live host calls, schema parsing, and command usage errors,
keep failure modes explicit and typed.

## 10. Naming Consistency

Use existing `ic-query` vocabulary consistently.

Examples:

1. `network`, `source_endpoint`, `project_root`
2. `root_principal`, `governance_canister_id`, `principal`
3. `snapshot`, `refresh_attempt`, `refresh_lock`
4. `schema_version`, `fetched_at_unix_secs`, `completeness`
5. `format`, `verbose`, `limit`, `sort`
6. `subnet`, `node`, `node_provider`, `node_operator`, `data_center`
7. `proposal`, `proposals`, `neurons`, `params`, `token`

Do not invent parallel names for existing concepts.

## 11. Data Shape Rules

DTOs are passive boundary data only. Command and report request DTOs should not
implement `Default` unless the default is truly neutral.

Cache records and snapshot envelopes are persisted storage schema. Keep schema
versions explicit, reject unknown future versions unless a migration exists,
and preserve raw JSON fields that scripts may consume.

Reports are read-only projections over live source data or complete local
snapshots. Do not mix display-only text conversions into JSON report models.

Cross-layer data should use named structs/enums, not boundary type aliases.

## 12. Test Placement and Scope

Placement:

1. unit tests live next to code
2. integration tests live in `tests/`
3. split tests use `mod tests;` at the top with other module declarations
4. inline tests stay at the bottom of the module

Scope:

1. unit tests cover pure logic, conversions, and model/ops invariants
2. fixture sources cover live host behavior without network calls
3. tests should assert typed errors or observable state, not strings
4. exact string assertions are appropriate only when CLI text is the contract
5. do not hide live network calls behind tests that look cache-only

Use boundary-level `tests.rs` for behavior crossing sibling modules or shared
fixtures.

## 13. Redundant Code Removal

During hygiene passes, remove:

1. duplicate helpers
2. dead code
3. stale compatibility branches no longer needed
4. outdated comments that describe old behavior
5. duplicate cache, report, text-rendering, or CLI parsing logic after a
   shared local helper exists

Keep removals scoped to the module or concern under review.

## 14. Formatting and Checks

During active development, run:

```text
cargo fmt --all
```

Before handoff, use the checks appropriate for the touched surface. Common
local checks include:

```text
make fmt-check
make clippy
make test
```

Do not change Cargo package versions, release scripts, install URLs, or
changelog entries during ordinary hygiene work unless the maintainer requests a
release slice.

## Why This Is Valuable

Following this standard improves:

1. architectural clarity
2. review quality
3. refactor safety
4. contributor onboarding speed

## Commit Strategy

For large hygiene sweeps, prepare focused working-tree changes by module or
concern rather than one massive edit. Automated agents must not commit, tag, or
push in this repository.

Examples:

1. `cleanup: normalize imports in sns reports`
2. `cleanup: tighten visibility in snapshot cache`
3. `cleanup: remove stale topology helpers`
