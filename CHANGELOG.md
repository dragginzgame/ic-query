# Changelog

All notable changes to `ic-query` will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/), and this
crate follows [Semantic Versioning](https://semver.org/).

## [0.0.2] - 2026-06-13

### Changed

- Groups NNS command implementation modules under `src/nns/` by command family.
- Renames project-local `icq` cache/state directory from `.ic-query/` to
  `.icq/`.
- Updates README positioning so `ic-query` is documented as a standalone IC
  metadata lookup tool, with Canic mentioned only as an optional integration
  example.
- Clarifies cache behavior for subnet versus node/provider/operator/data-center
  commands.

## [0.0.1] - 2026-06-13

### Added

- Adds the `icq` executable.
- Adds the `icq nns` command family for read-only Internet Computer mainnet
  metadata queries.
- Adds cache-backed subnet list, subnet info, and subnet refresh commands.
- Adds cache-backed NNS registry version, data-center, node, node-provider,
  node-operator, and joined-topology commands.
- Adds text and JSON output for automation-friendly metadata reports.

### Changed

- Moves the former `canic nns ...` command surface into `icq nns ...`.
- Folds the former subnet catalog model and resolver into `ic-query`, so the
  metadata query tool owns subnet classification and lookup.
