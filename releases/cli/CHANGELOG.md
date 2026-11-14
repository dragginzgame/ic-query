# CLI changelog

Release group: `cli`. Package: `ic-query-cli` (binary `icq`).

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
