# Subnet configuration changes

Canic needs to notice registered membership, software and halt-configuration
differences while inspecting maintenance. This independent slice uses the
existing published Platform configuration reader while provider/location joins
await [Platform #165](https://github.com/dragginzgame/ic-platform/issues/165).
It does not implement those joins, chip/HostOS comparison or a maintenance log.

## Library and CLI contract

`subnet::changes::compare` accepts two owned `SubnetConfigurationReport` values
and performs no IO. The schema-1 result retains both complete inputs and their
evidence, sorted member IDs added/removed relative to the second observation,
and change flags for raw replica version, subnet type, `is_halted` and
`halt_at_cup_height`. `has_selected_changes()` covers exactly those fields.
The comparator does not reauthenticate caller-supplied evidence.

`subnet::changes::fetch` and `icq topology subnet changes` acquire two
configurations for one subnet. The default wait is ten seconds after the first
read completes; `--interval-seconds` permits 1–60. Each read retains its existing
20-second asynchronous deadline, 1 MiB HTTP ceiling, bounded protobuf admission,
1024-member ceiling and 256-byte configured-version limit. No retry, persistent
history, scheduler or background task is added. The asynchronous budget is at
most two leaf deadlines plus the requested wait, with separately bounded
synchronous work and normal runtime-scheduling limitations.

`requested_interval_seconds` is populated for live sampling and null for pure
comparison. It is not an actual or certified observation interval. Local wall
clocks may move backward; retain their raw observation times. Configuration
readers retain query-signature evidence without consensus-certifying values.

Before computing differences, refuse different subnet identities, schemas other
than 1, incomplete membership, more than 1024 members, duplicate or unordered
membership, and a second record mutation version below the first. Refuse changed
selected fields at an equal mutation version. Each comparison refusal retains
both original inputs and a typed reason. A newer version with unchanged selected
fields is valid because other Registry fields may have changed. Empty membership
is valid; absent source content remains a reader error.

Initial source failure stops before waiting or reading again. Second-source
failure retains the first observation and original typed cause. Cancelling the
future starts no further reads. CLI source/comparison failures emit a diagnostic
on stderr, no report on stdout and exit 1; invalid CLI syntax exits 2. A successful
comparison exits 0 whether or not differences were found.

Text prints only changed scalar fields and membership differences, with shared
metadata and one caveat. Configured strings are escaped for terminals. JSON
retains the original values, both records, source metadata and verification
flags. This is an additive command/API; existing reports and saved data require
no reset, and Query-owned schemas remain 1.

## Interpretation

An added member is present only in the second returned record; that does not
establish when it joined consensus or whether it is live. A changed configured
replica version is not a measured running version. Halt flags do not prove a
maintenance action completed. Equal observations cannot exclude intermediate
changes, changes in unselected fields or activity between reads. Version
regression is unavailable comparison evidence, not a rollback diagnosis.

## Predecessor lessons and validation

Review used legacy `5b4951b23c2adfcd3c615961f131e766c4b50283`:
the [exact-version topology contract](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/docs/design/nns-subnet-topology.md),
[authority regressions](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/subnet_catalog/tests/authority.rs)
and [topology regressions](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/ic_registry/projection/subnet_topology/tests.rs).
Previously inspected history `b89d917d5e94f5bd73771076a59347ea29abd328` narrowed
inventory acquisition, and `0f242c5e467a5e2ec1d4ab28de68678f4cc0bcd8` moved
protocol handling toward owned codecs. Keep identity, canonical membership,
version and separate-authority invariants; do not restore global acquisition,
cache lifecycle or retired Registry replay. No legacy implementation was copied.

Use deterministic fixtures for raw fields and separate source evidence, equal or
newer versions, empty membership, each admission refusal, exact two-read timing,
invalid waits before IO, first/second failures and cancellation. CLI checks cover
the new action's alphabetical help, typed arguments, interval rejection and
compact escaped output. Live smoke observations are separate from unit evidence;
no change during a short smoke window is a valid outcome, not a missing test.

On 2026-11-14, focused Linux checks passed for the seven comparison/sampling
tests, changed presentation, command parsing and namespace ordering, plus affected
Clippy, Rustdoc, formatting, snapshot and local-link checks. Drift review kept
collection and comparison in the library, presentation in the CLI, and existing
protocol decoding in Platform; no cache or generic comparison framework was added.
The installed `icq` was refreshed on PATH. A separate one-second live sample of
the NNS subnet `tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe`
returned mutation version `65525` in both observations at local Unix seconds
`1794674012` and `1794674013`, with no selected differences and exit 0.
This is a successful point check, not continuous availability or new macOS
qualification. Provider/location distribution remains a separate blocked slice.
