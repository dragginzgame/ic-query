# SNS migration changelog

Release group: `sns-migration`. Package: `ic-sns-migration`.

## [0.104.1] - 2026-11-17

### 🔧 Tooling

- Add the independent `ic-sns-migration` package, with publication enabled and
  independently selected native-read, ledger and SNS-neuron capabilities.
- Enable this release group's standard publication adapter. The first published
  API advances the unpublished 0.104.0 package baseline to 0.104.1.
- Select published Platform 0.34.10, Protocol 0.3.0 and Types 0.21.2 for the
  initial API. Protocol's changed management APIs are not used by these reports;
  balance and neuron evidence contracts remain unchanged.

### ➕ Added

- Implement exact ICRC account balance reports with independent metadata outcomes,
  and selected SNS neuron lookup/list reports. Accept explicit compatible ledger
  and Governance targets; Dragginz and Toko are examples, not restrictions.
- Use the bounded supplier DTOs/codecs from
  [Platform #21 — SNS neuron and ICRC metadata types](https://github.com/dragginzgame/ic-platform/issues/21).
  Preserve partial evidence, raw optional fields, unknown permission/error codes,
  source/method identity, local observation windows and query authentication.
- Bound listing to 100 neurons per page, 1–20 pages and 60 seconds overall.
  Explicit exclusive cursors support further bounded unfiltered continuations.
  Principal-filtered calls use one page because Governance ignores their cursor.
  Full pages do not establish exhaustion; failed later pages retain the prefix.

### 🧪 Testing

- Qualify migration policy using six frozen independent supplier frames and
  consumer regression tests for exact balances, partial outcomes, ID/cursor
  admission, limits, deadlines and cancellation. Linux live Dragginz balance,
  exact-neuron, two-page and principal-filtered reads succeeded. This does not
  establish a full export, atomic snapshot, Toko interoperability or macOS execution.

### 📚 Documentation

- Define the [migration evidence contract](../../docs/design/migration-evidence.md)
  and retained predecessor invariants. Keep this package independent of
  `ic-topology`; broad SNS governance support and migration execution remain excluded.
