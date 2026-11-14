# CLI changelog

Release group: `cli`. Package: `ic-query-cli` (binary `icq`).

## [0.105.0] — pending

### 🛠 Changed

- Name the canister lookup `icq topology canister CANISTER_ID`, paired with
  `icq topology subnet SUBNET_ID`. This replaces the unpublished `placement`
  command; its routing report and JSON remain unchanged.
- Add `icq topology subnet SUBNET_ID [--json]` for observed node identities from
  subnet-certified state, explicitly distinguishing them from full Registry membership.
- Move topology dispatch and pure report rendering out of the process entrypoint;
  render verification claims from report evidence rather than fixed success text.
- Add `make icq` to build and install the checkout's development CLI into Cargo's
  bin directory with locked, offline dependencies, then verify PATH availability.
- Start the `icq` replacement with help, version and `topology canister CANISTER_ID`
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
icq topology subnet --help
```

Look up the ICP ledger canister's current subnet through the mainnet NNS Registry.
These commands require network access; they report routing placement, not
canister liveness or available capacity:

```bash
icq topology canister ryjl3-tyaaa-aaaaa-aaaba-cai
icq topology canister ryjl3-tyaaa-aaaaa-aaaba-cai --json
```

Inspect observed node identities in a real subnet's certified state. The returned
node-key inventory does not establish complete Registry membership, node liveness
or capacity:

```bash
icq topology subnet tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe
icq topology subnet tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
```

### 🧪 Tests

- Qualify matching bare/explicit namespace help, version output, typed canister
  argument admission and unsupported-command exit/output behavior offline.
