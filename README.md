# ic-query

[![CI](https://github.com/dragginzgame/ic-query/actions/workflows/ci.yml/badge.svg)](https://github.com/dragginzgame/ic-query/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/badge/MSRV-1.91.0-blue.svg)](Cargo.toml)
[![Internal Rust](https://img.shields.io/badge/internal%20rust-1.96.0-orange.svg)](rust-toolchain.toml)

`ic-query` provides the `icq` executable for read-only Internet Computer
metadata queries.

The `0.0.1` release starts with the NNS query surface that previously lived
under `canic nns`. It is intended to fill the metadata-query gaps around the
ICP CLI while keeping Canic focused on deployment orchestration.

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
icq nns registry version
icq nns subnet refresh
icq nns subnet list
icq nns subnet info <subnet|canister|subnet-prefix>
icq nns data-center list
icq nns node list
icq nns node-provider list
icq nns node-operator list
icq nns topology summary
icq nns topology coverage
icq nns topology versions
icq nns topology health
icq nns topology gaps
icq nns topology capacity
icq nns topology regions
icq nns topology providers
```

Most commands support text output by default and JSON output with
`--format json`:

```sh
icq --network ic nns subnet info ryjl3-tyaaa-aaaaa-aaaba-cai --format json
```

## Cache

The NNS subnet, node, provider, operator, data-center, and topology commands
use project-local cache files under `.ic-query/`. Refresh commands fetch current
mainnet registry data and replace the matching cache atomically:

```sh
icq nns subnet refresh
icq nns topology refresh
```

Read-only commands fail with an explicit refresh hint when the required cache
is missing.

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

## Boundary

`icq` owns IC metadata lookup and classification. Canic may store operator
intent such as a subnet principal and may execute `icq` when it needs subnet
facts, but Canic should not link the NNS registry adapter or duplicate subnet
catalog logic.

## Status

`0.0.1` is an initial extraction release. The command namespace is intentionally
small:

- `nns` is implemented.
- `sns` is reserved for follow-up work.
- Additional IC query families can be added without expanding Canic itself.
