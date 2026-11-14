# CLI changelog

Release group: `cli`. Package: `ic-query-cli` (binary `icq`).

## [0.105.4] — GitHub delivery 2026-11-14

GitHub-only version; not published to crates.io.

### ➕ Added

- Add `topology subnet changes` for two configuration observations, with a
  1–60-second wait (default 10). Text shows changed fields and membership only;
  JSON retains both reports. No differences is success, not proof of stability
  between samples. This does not implement the pending provider/location joins.

- Add `topology subnet nodes` for a selected subnet's registered node inventory.
  Show a compact table with full node IDs, operator/HostOS keys, chip-ID presence
  and read status. Shared details and caveats appear once; JSON retains full
  per-node evidence and raw chip IDs. Exit 1
  while printing the partial report if any selected node could not be read;
  configuration/admission failures produce no report. Successful reads do not
  prove live nodes or an atomic snapshot.

### ▶️ Try it

From the repository root, with the [setup prerequisites](../../docs/development/setup.md):

```bash
make fetch-dependencies
make icq
icq topology subnet nodes tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe
icq topology subnet nodes tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
```

Installation refreshes Cargo's `icq` on PATH. Fetching and mainnet reads need
network access; queries require no credentials. Each node inventory acquires
configuration and up to 128 node records, in batches of four within 60 seconds.
Expect registered operators, optional chip IDs, configured HostOS and source
evidence. Inspect `all_node_reads_succeeded` and each row; partial output exits 1.

Compare selected subnet configurations with the same setup:

```bash
icq topology subnet changes tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --interval-seconds 10
icq topology subnet changes tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --interval-seconds 10 --json
```

Each command makes two configuration reads. Expect membership, configured
replica version, raw subnet type or halt-flag differences, or an explicit
no-selected-changes result. Both observations retain source evidence; this is
not a complete event history, running-software check or uptime assessment.

## [0.105.3] — GitHub delivery 2026-11-14

GitHub-only version; not published to crates.io.

### 📚 Documentation

- Surface copyable setup and new-command examples directly in the root changelog,
  following Toko Tooling 0.43.1; retain detailed CLI notes here.
- Explain how to use the reports together for Canic planning in the
  [topology usage guide](../../docs/usage/topology.md), including evidence limits
  and the additional inputs needed for sizing, maintenance and TEE decisions.

### ➕ Added

- Add `topology node show` for a selected node's registered operator, optional
  SEV-SNP chip ID and configured HostOS. Preserve null versus empty values in JSON;
  escape text and distinguish Registry data from verified attestation and running software.
- Add `topology subnet membership` for registered-versus-observed node-set
  differences. Preserve both source reports and their separate evidence; matching
  sets do not establish live nodes, and differences are not a health verdict.
- Add `topology subnet configuration` for Registry membership, configured replica
  version, raw subnet type and halt flags. JSON preserves raw values; text escapes
  the version string and distinguishes configured state from observed downtime.

This development checkout uses published Platform 0.33.3; no sibling is required.
After dependency preparation, install it with `make icq`. Reads require mainnet
network access; the effective/running version is not resolved, including when a
cloud engine's configured version is empty.

```bash
make icq
icq topology subnet configuration tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe
icq topology subnet configuration tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
```

Membership comparison makes two sequential report reads.
Neither text nor JSON claims an atomic snapshot:

```bash
icq topology subnet membership tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe
icq topology subnet membership tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
```

For node inventory, run these commands from the repository root after the same
setup and installation. They make anonymous, read-only mainnet queries and print
one record's operator, chip and HostOS data with source evidence:

```bash
icq topology node show nuass-yihti-wszdr-r7qze-32dgz-osovf-pracd-hg5tt-mwlhf-rvuyq-4ae
icq topology node show nuass-yihti-wszdr-r7qze-32dgz-osovf-pracd-hg5tt-mwlhf-rvuyq-4ae --json
```

Chip registration does not establish usable TEE protection, and an absent chip
ID does not prove absent hardware capability. No provider/location join or
running HostOS resolution is performed.

## [0.105.2] — GitHub delivery 2026-11-14

GitHub-only version; not published to crates.io.

### ➕ Added

- Add `topology subnet activity` to compare exactly two certified message totals.
  `--interval-seconds` selects a 1–60-second wait after the first read (default 10).
  JSON retains both observations; text labels source evidence and interpretation
  limits. A failed read exits unsuccessfully without success JSON.
- Include observed subnet type in `topology subnet show`. Text quotes and escapes
  source strings and distinguishes absent values; JSON preserves raw strings or
  null. Keep subnet certification distinct from independent Registry confirmation.

After dependency preparation, install this checkout; report reads require network
access. These commands include the new field without another network request:

```bash
make icq
icq topology subnet show tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe
icq topology subnet show tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
```

Activity takes two network reads separated by the requested wait. These examples
wait ten seconds; equal totals do not establish downtime:

```bash
icq topology subnet activity tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe
icq topology subnet activity tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --interval-seconds 10 --json
```

## [0.105.1] — GitHub delivery 2026-11-14

GitHub-only version; not published to crates.io.

### ➕ Added

- Read certified subnet resource values with `topology subnet metrics`: canister
  count, state bytes, nominal cycles and replicated-message totals. JSON preserves
  the 128-bit cycle total as a decimal string; text explains the observation limits.

After dependency preparation, `make icq` installs this checkout. These reads
require network access and establish neither spare capacity nor uptime:

```bash
make icq
icq topology subnet metrics --help
icq topology subnet metrics tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe
icq topology subnet metrics tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
```

## [0.105.0] — GitHub delivery 2026-11-14

Delivered through `7695e00`; not published to crates.io.

### 🛠 Changed

- Add `icq topology subnet list [--json]` for registered subnet IDs, with record
  mutation version/time, query verification and complete selected-record coverage.
- Organize commands as `icq <specialist> <entity> <action>`: use
  `topology canister show CANISTER_ID` and `topology subnet show SUBNET_ID`.
  This replaces the unpublished direct-ID and `placement` commands; report JSON
  remains unchanged. Bare entity namespaces print local help successfully.
- Add `icq topology subnet show SUBNET_ID [--json]` for observed node identities from
  subnet-certified state, explicitly distinguishing them from full Registry membership.
- Move topology dispatch and pure report rendering out of the process entrypoint;
  render verification claims from report evidence rather than fixed success text.
- Add `make icq` to build and install the checkout's development CLI into Cargo's
  bin directory with locked, offline dependencies, then verify PATH availability.
- Start the `icq` replacement with help, version and `topology canister show CANISTER_ID`
  output, including `--json`. Unsupported
  commands fail explicitly; legacy commands are not restored. Publication is
  disabled pending real reports and explicit consumer migration.
- Compose `ic-topology` through the first compile-time CLI adapter; keep SNS
  independent and add its dependency only when an implemented command uses it.

### 📚 Command examples

After [dependency preparation](../../docs/development/setup.md), install this
checkout's development binary into Cargo's bin directory, normally `~/.cargo/bin`,
which must be on PATH. This replaces an existing Cargo-installed `icq`:

```bash
make icq
icq --help
```

Inspect local help and the CLI version:

```bash
icq
icq help
icq --version
icq topology --help
icq topology canister --help
icq topology canister show --help
icq topology subnet --help
icq topology subnet list --help
icq topology subnet show --help
```

List registered subnets from the mainnet NNS Registry. Network access is required;
registration does not establish liveness, available capacity or deployment permission.
The record version is its last mutation version, not the Registry's global latest version:

```bash
icq topology subnet list
icq topology subnet list --json
```

Look up the ICP ledger canister's current subnet through the mainnet NNS Registry.
These commands require network access; they report routing placement, not
canister liveness or available capacity:

```bash
icq topology canister show ryjl3-tyaaa-aaaaa-aaaba-cai
icq topology canister show ryjl3-tyaaa-aaaaa-aaaba-cai --json
```

Inspect observed node identities in a real subnet's certified state. The returned
node-key inventory does not establish complete Registry membership, node liveness
or capacity:

```bash
icq topology subnet show tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe
icq topology subnet show tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
```

### 🧪 Tests

- Qualify matching bare/explicit help at specialist and entity levels, version
  output, typed show arguments, list argument rejection and report presentation
  offline. Verify the installed list and both show commands against mainnet
  separately; the list smoke observation returned 78 registered subnets.
