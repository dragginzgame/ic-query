# CLI changelog

Release group: `cli`. Package: `ic-query-cli` (binary `icq`).

## [0.105.30] - 2026-11-26

### 🔧 Tooling

- Build against Protocol 0.3.11 and Types 0.21.6 for report reads and
  `ic-host-text` 0.77.7 for terminal tables in the workspace lockfile.
- Update the packaged development dependency and installed tooling to
  Toko Tooling 0.46.8.

## [0.105.29] - 2026-11-25

### 🔧 Tooling

- Build the topology and SNS command adapters with Platform 0.35.1, Protocol
  0.3.10 and Types 0.21.5 in the workspace lockfile.
- Update the packaged development dependency and shared release tooling to
  Toko Tooling 0.46.4. Relative publication evidence paths resolve from the
  checkout even when `CDPATH` is set.

## [0.105.28] - 2026-11-21

### 🔧 Tooling

- Use Toko Tooling's shared setup diagnostics for CI command capture and
  redaction. Keep Query's attempt-log selection and report successful setup with
  a short phase summary. [Query #6 — Use shared setup diagnostics in CI](https://github.com/dragginzgame/ic-query/issues/6).
- Stop scanning additional setup log roots once the evidence limit is met.
- Adopt Toko Tooling 0.46.0. Release dispatch preserves the caller's Cargo
  network mode, and reuse rejects stage-selected overrides.
  [Toko Tooling #402 — Make caller ownership of Cargo network mode explicit across shared tooling](https://github.com/dragginzgame/toko-tooling/issues/402).

Install the CLI with Cargo and network access:

```bash
cargo install ic-query-cli --version 0.105.28 --locked
icq --version
```

## [0.105.27] - 2026-11-19

### 🔧 Tooling

- Update the packaged development dependency to Toko Tooling 0.45.21 and apply
  its return spacing rule.

Install the CLI with Cargo and network access:

```bash
cargo install ic-query-cli --version 0.105.27 --locked
icq --version
```

## [0.105.26] - 2026-11-19

### 🔧 Tooling

- Update the packaged development dependency to Toko Tooling 0.45.18.

Install the CLI with Cargo and network access:

```bash
cargo install ic-query-cli --version 0.105.26 --locked
icq --version
```

## [0.105.25] - 2026-11-19

### 🔧 Tooling

- Update the packaged development dependency to Toko Tooling 0.45.16.

Install the CLI with Cargo and network access:

```bash
cargo install ic-query-cli --version 0.105.25 --locked
icq --version
```

## [0.105.24] - 2026-11-19

### 🔧 Tooling

- Select Toko Tooling 0.45.13 and align CLI module layout with its baseline.
  Command syntax and report output are unchanged.

Install the CLI with Cargo and network access:

```bash
cargo install ic-query-cli --version 0.105.24 --locked
icq --version
```

## [0.105.23] - 2026-11-18

### 🔧 Tooling

- Adopt Toko Tooling 0.45.11 and its Rust layout and dependency-catalog rules.
  Cargo may fetch missing locked dependencies; explicit offline selections remain
  effective, and candidate dependencies are prepared before proof sealing.
- Qualify the locked Platform 0.34.17, Protocol 0.3.6 and Types 0.21.3 graph
  while retaining compatible manifest floors.
- Group imports and move test declarations to module tops; retain command and presentation contracts.

### 🎯 Qualification

- Linux workspace checks pass with the selected graph; release wiring passes
  32 assertions. Native four-group validation executes products once and admits
  fresh reuse for the other three groups. See the [setup evidence](../../docs/development/setup.md).

Install the released CLI with Cargo and network access:

```bash
cargo install ic-query-cli --version 0.105.23 --locked
icq --version
```

## [0.105.22] - 2026-11-18

### 🔧 Tooling

- Adopt Toko Tooling 0.45.3 for reduced canonical JSON processing during stage
  reuse, detailed observation timings and shared batch summaries. Preserve fresh
  input hashing and proof custody. [Tooling #394](https://github.com/dragginzgame/toko-tooling/issues/394).

Install with Cargo and network access:

```bash
cargo install ic-query-cli --version 0.105.22 --locked
icq --version
```

## [0.105.21] - 2026-11-18

### 🔧 Tooling

- Retire the unused standalone provenance helper through the exporter; preserve
  existing publication admission. [Query #5](https://github.com/dragginzgame/ic-query/issues/5).

- Prepare CI Rust through the shared helper and `rust-toolchain.toml`, preserving
  setup diagnostics and using the prepared host parsers. Remove duplicated
  channel parsing, profile and component selections from both jobs.

Install with Cargo and network access:

```bash
cargo install ic-query-cli --version 0.105.21 --locked
icq --version
```

## [0.105.20] - 2026-11-17

### 🔧 Tooling

- Adopt Toko Tooling 0.45.0 for grouped candidate preparation and streaming proof
  admission with finer timing spans. Preserve fresh graph checks and original
  proof custody; CLI commands and report contracts are unchanged.
  [Tooling #390](https://github.com/dragginzgame/toko-tooling/issues/390),
  [Tooling #391](https://github.com/dragginzgame/toko-tooling/issues/391).

Install with Cargo and network access:

```bash
cargo install ic-query-cli --version 0.105.20 --locked
icq --version
```

## [0.105.19] - 2026-11-17

### 🔧 Tooling

- Admit the combined product proof once per group and run actual-version CLI
  tests as a separate stage. Preserve original evidence and final source/archive
  admission. Check prepared tools before installation and record detailed timings.
  [Query #4](https://github.com/dragginzgame/ic-query/issues/4).

Install with Cargo and network access:

```bash
cargo install ic-query-cli --version 0.105.19 --locked
icq --version
```

## [0.105.18] - 2026-11-17

### 🔧 Tooling

- Select CLI-only validation without unrelated specialist or umbrella tests; keep the full fallback for shared or unknown changes.
  [Query #3 — Select affected packages and consumers](https://github.com/dragginzgame/ic-query/issues/3).

- Reuse exact-input Linux product checks across group releases using Tooling's
  canonical final-version source and retained execution proofs. Keep current
  CLI version/help, actual source/archive and registry checks fresh, and recover
  evidence-output failures without repeating qualified product checks. See the
  [validation scope and limitations](../../docs/development/setup.md#release-validation-selection).

- Preserve the locked Platform 0.34.14 patch selection for this delivery.

Install with Cargo and network access:

```bash
cargo install ic-query-cli --version 0.105.18 --locked
icq --version
```

## [0.105.17] - 2026-11-17

### 🔧 Tooling

- Move migration command and presentation unit tests into adjacent `tests.rs`
  files, with their owning modules in `mod.rs`. Preserve assertions, private
  visibility and ordinary module discovery; production files retain only
  conditional test-module declarations. CLI behavior is unchanged.
- Qualify the retained lockfile update to zeroize 1.9.2 alongside the CLI checks.

Install with Cargo and network access:

```bash
cargo install ic-query-cli --version 0.105.17 --locked
icq --version
```

## [0.105.16] - 2026-11-17

### 🔧 Changed

- Compose topology 0.110.1 and SNS 0.105.1, qualifying locked Platform 0.34.13
  and Protocol 0.3.5. Existing commands, output and exit statuses are unchanged.

Install with Cargo and network access:

```bash
cargo install ic-query-cli --version 0.105.16 --locked
icq --version
```

## [0.105.15] - 2026-11-17

### 🔧 Changed

- Compose the renamed `ic-query-topology` and `ic-query-sns` libraries using
  their new Rust import names. The CLI package, `icq` binary, commands, JSON and
  exit statuses are unchanged. Existing command examples still apply.
- Qualify the retained Platform 0.34.12 / Protocol 0.3.4 lockfile selection.

Install with Cargo and network access:

```bash
cargo install ic-query-cli --version 0.105.15 --locked
icq --version
```

## [0.105.14] - 2026-11-17

### 🐛 Fixed

- Use compatible Types and report-library supplier requirements so downstream
  dependency resolution can select compatible patches. Qualify Protocol 0.3.3
  in the lockfile. Commands and output are unchanged.

Install with Cargo and network access:

```bash
cargo install ic-query-cli --version 0.105.14 --locked
icq --version
```

## [0.105.13] - 2026-11-17

### 🔧 Changed

- Adopt published Platform 0.34.11 and Protocol 0.3.2 through the independent
  report libraries. Commands, presentation, report JSON and exit statuses are
  unchanged; the existing copyable examples below still apply.

Install with Cargo and network access, then check the version:

```bash
cargo install ic-query-cli --version 0.105.13 --locked
icq --version
```

## [0.105.12] - 2026-11-17

### 🔧 Changed

- Qualify the existing CLI against published Platform 0.34.10 and Protocol 0.3.0.
  Existing commands, report JSON and Types 0.21.2 identities remain unchanged.
- Qualify the retained lockfile's Clap 4.6.8 / clap_lex 1.1.2 patches with the
  existing help, parsing and process-exit contracts before delivery.

### ➕ Added

- Add `icrc balance` for any explicitly selected ICRC ledger/account, and
  `sns neuron show` / `sns neuron list` for any compatible SNS Governance source.
  Retain exact base units, independent metadata, selected raw neuron fields,
  observation times and partial failures. These are evidence reads, not migration
  execution or entitlement decisions. See the
  [migration contract](../../docs/design/migration-evidence.md).
- Add bounded listing with an explicit page budget, optional exclusive
  `--start-after` continuation and one-page principal
  filtering. Full limited pages exit 1 and preserve data; only exhausted
  traversal exits 0. A principal filter is not an ownership claim.

From a prepared checkout with network access, read Dragginz's ledger using the
public anonymous account as an example (not a user identity):

```bash
cargo run --locked --offline -p ic-query-cli -- icrc balance \
  zfcdd-tqaaa-aaaaq-aaaga-cai 2vxsx-fae --json
```

Inspect selected neuron fields from Dragginz Governance. The exact ID below was
observed in the live listing; later removal is a reported Governance outcome.
The two-page example intentionally exits 1 if more records may exist:

```bash
cargo run --locked --offline -p ic-query-cli -- sns neuron list \
  zqfso-syaaa-aaaaq-aaafq-cai --max-pages 2 --json
cargo run --locked --offline -p ic-query-cli -- sns neuron show \
  zqfso-syaaa-aaaaq-aaafq-cai \
  0000aee89cb1db1d60e8480c7e324cfa8b4e16f46c55d197092f1472bb997a11 --json
cargo run --locked --offline -p ic-query-cli -- sns neuron list \
  zqfso-syaaa-aaaaq-aaafq-cai --principal 2vxsx-fae --json
```

Continue after the last ID observed in the initial 100-record example page;
this is another live observation, not a snapshot continuation:

```bash
cargo run --locked --offline -p ic-query-cli -- sns neuron list \
  zqfso-syaaa-aaaaq-aaafq-cai \
  --start-after 020e6a2de649f082807613f83d681037dc92fa741a51a2bfdfee7a0cc5c6a279 \
  --json
```

The principal-filtered example can return no matching neurons. Remove `--json`
for escaped bordered tables. `icrc balance --subaccount` accepts exactly 64 hex
digits; omitting it preserves an absent subaccount selecting the ICRC default.

## [0.105.11] - 2026-11-16

### 🔧 Changed

- Show `IC Query` and the build's version in every help header, including bare
  namespaces and nested commands, matching Canic and Blob. Report output and
  `--version` remain unchanged.

From a prepared checkout, inspect the versioned help without network reads:

```bash
cargo run --locked --offline -p ic-query-cli -- --help
cargo run --locked --offline -p ic-query-cli -- topology subnet --help
```

## [0.105.10] - 2026-11-16

### 🔧 Changed

- Align CLI principal parsing with published Types 0.21.2 and the topology library;
  command syntax and serialized principal values remain unchanged.

- Adopt Toko Tooling 0.44.11 through the authenticated updater, retaining the
  selected tooling entrypoints and refreshing the baseline and receipt.

- Add standard group release and publication commands with frozen source capture,
  complete local validation and exact source/tag admission.

- Adopt Host Text 0.77.2 and use its `bordered_table` and `escape` APIs directly,
  retaining CLI-owned rows, values and output sinks.

- Adopt the topology native-read hard cut to Platform 0.34.7 and Protocol 0.2.6;
  remove legacy Platform and Agent without changing command syntax or JSON schema.

Install with Rust 1.100.0 or newer and network access:

```bash
cargo install ic-query-cli --version 0.105.10 --locked
icq topology subnet progress \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe \
  --interval-seconds 10 --json
```

The read requires mainnet access and retains both authenticated state times;
it does not establish uptime or maintenance completion.

## [0.105.9] — GitHub delivery 2026-11-16

GitHub-only version; not published to crates.io.

### 🔧 Changed

- Adopt Toko Tooling 0.44.4 for development, including package-directory naming
  and copyable shell-example conventions. Existing package locations already comply.

### ➕ Added

- Add `topology subnet progress`: two authenticated state times with explicit
  ordering, freshness limits and retained partial evidence. Partial reads exit 1;
  unchanged/regressed times remain successful observations. Uses the immutable
  replacement Platform selection; no sibling checkout is required.

From this repository, run `make fetch-dependencies` and `make icq`, then:

```bash
icq topology subnet progress \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe \
  --interval-seconds 10 --json
```


- Add opt-in `topology subnet compare --settings`, retaining separate source
  versions and per-side failures. Combined partial output exits 1. Default reads
  and JSON remain unchanged; opt-in JSON wraps the original `comparison` with
  `left_settings`, `right_settings` and overall completion/duration.
  The maximum asynchronous budget with settings is 200 seconds.

After `make icq`, with network access (omit `--json` for text):

```bash
icq topology subnet compare \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe \
  brlsh-zidhj-3yy3e-6vqbz-7xnih-xeq2l-as5oc-g32c4-i5pdn-2wwof-oae \
  --settings --json
```

- Add `topology subnet settings` with bordered raw-value/unit columns and explicit
  absent/default-selected labels. JSON preserves raw values and source evidence.
  Success exits 0, read/admission failure 1, invalid arguments 2.

Install this checkout with `make icq`; mainnet reads require network access.

```bash
icq topology subnet settings \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe
icq topology subnet settings \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
```

## [0.105.8] — GitHub delivery 2026-11-15

GitHub-only version; not published to crates.io.

### ➕ Added

- Add `topology canister placements` for 1–32 explicit IDs, with bordered
  per-canister outcomes or raw JSON. Partial results remain on stdout with exit 1;
  complete collection exits 0. Empty, excessive and duplicate selections are
  rejected before network calls. Reads share 60 seconds, without fleet discovery.

After the [setup prerequisites](../../docs/development/setup.md), install this
checkout with `make icq`. Mainnet reads require network access:

```bash
icq topology canister placements \
  ryjl3-tyaaa-aaaaa-aaaba-cai \
  rwlgt-iiaaa-aaaaa-aaaaa-cai
icq topology canister placements \
  ryjl3-tyaaa-aaaaa-aaaba-cai \
  rwlgt-iiaaa-aaaaa-aaaaa-cai --json
```

These public IDs demonstrate syntax, not Canic fleet bindings. Existing
single-canister `show` is unchanged.

## [0.105.7] — GitHub delivery 2026-11-15

GitHub-only version; not published to crates.io.

### 🔧 Changed

- Adopt the selected-engine codec patch documented in the
  [topology ledger](../topology/CHANGELOG.md), preserving command syntax and report
  JSON fields. Malformed or over-budget responses remain failures, including the
  replacement decoder's explicit vector limit. Reinstall `icq` for the update.

### ➕ Added

- Show observed provider/data-center overlap in `topology subnet compare`, with
  separate coverage, unknown-node counts and per-side node fractions over full
  selected membership. Shared identities and per-side node
  counts are retained in JSON. Missing input is unavailable, not zero overlap.
- Add `--overlap-details` to display the shared identities and per-side node counts
  in bordered text tables, using the same collected evidence. It is a text-only
  option and cannot be combined with `--json`.

Install from this checkout after the [setup prerequisites](../../docs/development/setup.md);
mainnet reads require network access:

```bash
make icq
icq topology subnet compare \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe \
  brlsh-zidhj-3yy3e-6vqbz-7xnih-xeq2l-as5oc-g32c4-i5pdn-2wwof-oae \
  --overlap-details
icq topology subnet compare \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe \
  brlsh-zidhj-3yy3e-6vqbz-7xnih-xeq2l-as5oc-g32c4-i5pdn-2wwof-oae
icq topology subnet compare \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe \
  brlsh-zidhj-3yy3e-6vqbz-7xnih-xeq2l-as5oc-g32c4-i5pdn-2wwof-oae --json
```

Different Registry versions and incomplete coverage remain visible. Zero observed
overlap does not establish independent failure domains; exit behavior is unchanged.

## [0.105.6] — GitHub delivery 2026-11-15

GitHub-only version; not published to crates.io.

### ➕ Added

- Add `topology subnet compare LEFT RIGHT` with a bordered side-by-side view of
  configuration, provider/location coverage and resources. Preserve each source's
  version and evidence; partial results exit 1. Duplicate selections fail before IO.

- Add `topology subnet maintenance`: two coordinated configuration/metrics rounds,
  one 1–60 second wait (default 10), optional exact `--expected-version` and a
  bordered before/after table including raw registered subnet type and plain-text
  expectation outcomes. Retain later failures in JSON with exit 1. Successful
  version disagreements, configuration changes and counter changes exit 0 without
  a health or maintenance-completion verdict. Invalid parameter values fail before IO.

From the repository root after the [setup prerequisites](../../docs/development/setup.md):

```bash
make icq
icq topology subnet compare \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe \
  brlsh-zidhj-3yy3e-6vqbz-7xnih-xeq2l-as5oc-g32c4-i5pdn-2wwof-oae
icq topology subnet compare \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe \
  brlsh-zidhj-3yy3e-6vqbz-7xnih-xeq2l-as5oc-g32c4-i5pdn-2wwof-oae --json
icq topology subnet maintenance \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe \
  --interval-seconds 10
icq topology subnet maintenance \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe \
  --interval-seconds 10 --json
```

Mainnet reads need network access. Maintenance has four independent 20-second
read deadlines plus the selected wait. Comparison has two 60-second distribution
budgets and two 20-second metrics reads, at most 160 asynchronous seconds plus
bounded synchronous work. Neither command adds retries, history or background polling.

## [0.105.5] — GitHub delivery 2026-11-15

GitHub-only version; not published to crates.io.

### 🎨 Presentation

- Use published `ic-host-text` 0.76.0 for bordered tables in subnet node and
  provider/location reports, cloud-engine prices and available-node summaries.
  Shared rendering owns visible `|` column delimiters, `+---+` borders, terminal
  escaping and Unicode-width alignment. Raw JSON and report limits stay unchanged.

### ➕ Added

- Add `topology cloud-engine show ENGINE_ID`: one public selected-engine read,
  showing subnet/operator bindings and a bordered comparison of engine-record
  nodes with embedded Registry-copy membership. Preserve missing versus empty
  membership and deletion evidence. Unknown/hidden IDs yield an explicit
  not-returned report with exit 0; failed reads/admission exit 1 without a report.
  IDs are opaque nonempty text, at most 256 UTF-8 bytes, checked before IO.

- Add `topology cloud-engine candidates --max-pages 20`: join observed candidates
  to every matching public price declaration. Show missing/single/multiple match
  counts and a bordered daily-price table; JSON retains both source reports and
  node-to-declaration indices. No precedence, customer discount or quote inferred.
  Partial collection exits 1; successful missing/overlapping matches exit 0.

- Add `topology cloud-engine available`: compact class/data-center counts with
  provider and chip-registration counts. JSON retains full node identities,
  page cursors and versions, copy status and stop details. Partial collections
  remain on stdout with exit 1; exhausted pagination exits 0 without claiming
  an atomic inventory or reserved capacity.
  Add `--max-pages` (1–100, default 10) to collect more candidates while retaining
  the 60-second deadline. JSON records the selected page and node ceilings.

- Add `topology cloud-engine prices`: a compact daily-price table and independently
  observed network markup. JSON preserves exact decimal amounts, raw scopes and
  timestamps. Fee failures retain price output with exit 1; initial price failure
  produces no report. These declarations are not binding quotes or usable capacity.

- Add `topology node changes` for registered operator, chip-ID and configured
  HostOS differences. Wait 1–60 seconds after the first read (default 10); show
  changed fields with compact chip presence/byte counts, while JSON retains raw
  values and both evidence records. Successful unchanged comparisons exit 0.

- Add `topology subnet distribution`: compact provider and registered data-center
  counts with independent coverage and unknown totals. JSON retains pinned-version
  evidence, per-node relationships and all read outcomes. Partial results remain
  on stdout with exit 1; initial version/configuration failures produce no report.

### ▶️ Try it

From the repository root after the [setup prerequisites](../../docs/development/setup.md):

```bash
make fetch-dependencies
make icq
icq topology cloud-engine show engine-0b56b29a-7ea0-40b8-a963-6c4539ea4414
icq topology cloud-engine show engine-0b56b29a-7ea0-40b8-a963-6c4539ea4414 --json
icq topology cloud-engine candidates --max-pages 20
icq topology cloud-engine candidates --max-pages 20 --json
icq topology cloud-engine available
icq topology cloud-engine available --json
icq topology cloud-engine available --max-pages 20 --json
icq topology cloud-engine prices
icq topology cloud-engine prices --json
icq topology subnet distribution \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe
icq topology subnet distribution \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
icq topology node changes \
  nuass-yihti-wszdr-r7qze-32dgz-osovf-pracd-hg5tt-mwlhf-rvuyq-4ae \
  --interval-seconds 10
icq topology node changes \
  nuass-yihti-wszdr-r7qze-32dgz-osovf-pracd-hg5tt-mwlhf-rvuyq-4ae \
  --interval-seconds 10 --json
```

Fetching and anonymous mainnet reads require network access. Installation refreshes
`icq` on PATH. Counts describe the selected membership at one requested Registry
version, not verified failure domains or usable capacity. Collection allows 128
nodes, batches of four and 60 seconds. Inspect `complete`, `summary` and the
per-node `relationships` before using partial evidence for placement decisions.
Node comparison failures emit no success report and exit 1. Changes in chip
registration do not establish usable TEE protection; configured HostOS changes
do not establish running software or completed maintenance.
Prices allow up to 1,000 unique rows and 30 seconds across both queries. Amounts
are whole cycles per node per day; supplied gross values already include their
fee. `network_fee` records the separate observation or failure; `complete` means
both reads passed admission, without claiming an atomic or fresh marketplace.
Availability requests public new-engine pages only: up to 100 nodes per page,
ten pages/1,000 nodes by default and 60 seconds including status. An explicit
`--max-pages 20` permits up to 2,000 nodes without increasing that deadline. Inspect
`pagination_exhausted`, `registry_copy`, `pages` and `stop`; a source version
change stops traversal rather than mixing versions. No IP addresses are retained.

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
icq topology subnet nodes \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe
icq topology subnet nodes \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
```

Installation refreshes Cargo's `icq` on PATH. Fetching and mainnet reads need
network access; queries require no credentials. Each node inventory acquires
configuration and up to 128 node records, in batches of four within 60 seconds.
Expect registered operators, optional chip IDs, configured HostOS and source
evidence. Inspect `all_node_reads_succeeded` and each row; partial output exits 1.

Compare selected subnet configurations with the same setup:

```bash
icq topology subnet changes \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe \
  --interval-seconds 10
icq topology subnet changes \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe \
  --interval-seconds 10 --json
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
icq topology subnet configuration \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe
icq topology subnet configuration \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
```

Membership comparison makes two sequential report reads.
Neither text nor JSON claims an atomic snapshot:

```bash
icq topology subnet membership \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe
icq topology subnet membership \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
```

For node inventory, run these commands from the repository root after the same
setup and installation. They make anonymous, read-only mainnet queries and print
one record's operator, chip and HostOS data with source evidence:

```bash
icq topology node show nuass-yihti-wszdr-r7qze-32dgz-osovf-pracd-hg5tt-mwlhf-rvuyq-4ae
icq topology node show \
  nuass-yihti-wszdr-r7qze-32dgz-osovf-pracd-hg5tt-mwlhf-rvuyq-4ae --json
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
icq topology subnet show \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
```

Activity takes two network reads separated by the requested wait. These examples
wait ten seconds; equal totals do not establish downtime:

```bash
icq topology subnet activity \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe
icq topology subnet activity \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe \
  --interval-seconds 10 --json
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
icq topology subnet metrics \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe
icq topology subnet metrics \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
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
icq topology subnet show \
  tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
```

### 🧪 Tests

- Qualify matching bare/explicit help at specialist and entity levels, version
  output, typed show arguments, list argument rejection and report presentation
  offline. Verify the installed list and both show commands against mainnet
  separately; the list smoke observation returned 78 registered subnets.
