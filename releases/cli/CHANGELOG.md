# CLI changelog

Release group: `cli`. Package: `ic-query-cli` (binary `icq`).

## [0.105.0] — pending

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
