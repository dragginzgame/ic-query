# ic-query

[![CI](https://github.com/dragginzgame/ic-query/actions/workflows/ci.yml/badge.svg)](https://github.com/dragginzgame/ic-query/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/ic-query.svg)](https://crates.io/crates/ic-query)
[![docs.rs](https://docs.rs/ic-query/badge.svg)](https://docs.rs/ic-query)
[![License](https://img.shields.io/crates/l/ic-query.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.91.0-blue.svg)](Cargo.toml)
[![Internal Rust](https://img.shields.io/badge/internal%20rust-1.96.0-orange.svg)](rust-toolchain.toml)

`ic-query` provides the `icq` executable for read-only Internet Computer
metadata queries.

`icq` currently supports NNS and SNS metadata queries: registry version,
subnet catalog lookup, node/provider/operator/data-center inventory, topology
reports, and deployed SNS listings.

## Install

From this checkout:

```sh
make install
```

From crates.io after publication:

```sh
cargo install ic-query
```

## Commands

```sh
icq nns help
icq nns registry version
icq nns subnet [list|info|refresh]
icq nns node [list|info|refresh]
icq nns node-provider [list|info|refresh]
icq nns node-operator [list|info|refresh]
icq nns data-center [list|info|refresh]
icq nns topology [summary|coverage|versions|health|gaps|capacity|regions|providers|refresh]
icq sns [list|info]
```

Use `icq nns <family> help`, `icq nns topology <report> help`, or
`icq sns <command> help` for command options.

Most commands support text output by default and JSON output with
`--format json`:

```sh
icq --network ic nns subnet info ryjl3-tyaaa-aaaaa-aaaba-cai --format json
```

## Cache

The NNS subnet, node, provider, operator, data-center, and topology commands
use project-local cache files under `.icq/`. Refresh commands fetch current
mainnet registry data and replace the matching cache atomically:

```sh
icq nns subnet refresh
icq nns topology refresh
```

List/info commands populate their component cache on first use and print the
API endpoint they are calling before creating it. Refresh commands force a
fresh fetch and replace the matching cache.

## Development

This repository pins the local toolchain to Rust `1.96.0` while declaring
Rust `1.91.0` as the crate MSRV.

```sh
make fmt-check
make clippy
make test
make package
```

The combined local gate is:

```sh
make ci
```

Release version bumps are available after the release contents are committed and
the worktree is clean:

```sh
make patch
make minor
make major
```

Each target runs `make test`, bumps `Cargo.toml` and `Cargo.lock`, validates the
package, commits the release version, and creates an annotated `vX.Y.Z` tag.

## Integration

`icq` is a standalone metadata lookup tool. Orchestration, deployment, and
application repositories can call the CLI when they need IC metadata instead of
linking registry adapters directly. For one integration example, see
[Canic](https://github.com/dragginzgame/canic).

## Status

The command namespace is intentionally small:

- `nns` is implemented.
- `sns list` and `sns info` are implemented for deployed mainnet SNS
  instances.
- Additional IC query families can be added without coupling query code to
  deployment tooling.
