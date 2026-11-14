# Using topology evidence for Canic decisions

IC Query supplies observations for Canic's planning. Canic owns application
requirements, topology changes and deployment. Use each report for the question
its source can answer; retain its evidence when comparing results.

Run the setup commands below from the `ic-query` repository root after the
Rust/tool prerequisites in [development setup](../development/setup.md):

```bash
make fetch-dependencies
make icq
```

Fetching needs network access. Installation replaces the Cargo-installed `icq`
and checks PATH. The report commands below make anonymous, read-only mainnet
requests; they require network access but no credentials or token transfers.
Reinstall after changing the checkout to use its new CLI behavior.

## Locate an application and identify candidates

Look up the ICP ledger canister's subnet, or list registered subnet IDs:

```bash
icq topology canister show ryjl3-tyaaa-aaaaa-aaaba-cai --json
icq topology subnet list --json
```

A mapping is routing evidence, not proof of canister existence or uptime.
Registration identifies a candidate to investigate; it does not establish spare
capacity, placement eligibility or permission to deploy there. Query does not
rank candidates automatically.

## Inspect one subnet before planning maintenance

These examples use the NNS subnet so they run without replacing placeholders.
Select the actual candidate subnet from the preceding reports for application
planning; this example is not a placement recommendation.

```bash
icq topology subnet configuration tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
icq topology subnet membership tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
icq topology subnet metrics tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
icq topology subnet activity tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --interval-seconds 10 --json
```

| Planning question | Available evidence | Further input needed |
| --- | --- | --- |
| What configuration is registered? | Membership, configured replica version, raw subnet type and halt flags | Running version and maintenance completion; an empty version may use a separate cloud-engine record |
| Do registered members match the returned node keys? | Membership comparison retains both reads and their set differences | Transition history, source age and live-node observations; neither equality nor a difference is a health verdict |
| How much state and activity is observed? | Certified canister count, state bytes, nominal cycles and message total | Applicable limits, workload demand and measured headroom before sizing or consolidation |
| Did the message counter change across two reads? | Activity retains both certified samples and the direction of change | Sustained observations, failure classification and coverage before estimating uptime |
| Can workloads move to a TEE-based authentication strategy? | Selected-node Registry operator, optional chip ID and configured HostOS | Verified attestation, usable hardware and an Auth trust policy; a Registry chip ID alone will not establish these |

The membership report includes its own configuration read. When only that
comparison is needed, use it directly instead of also issuing the standalone
configuration command. Each command acquires new evidence; their results are
not an atomic snapshot.

## Inspect one node's registered configuration

Select a node ID from subnet configuration or an independently known candidate.
The node report does not require subnet assignment, perform provider/location
joins or resolve running HostOS software. This concrete example reads one public
node record:

```bash
icq topology node show nuass-yihti-wszdr-r7qze-32dgz-osovf-pracd-hg5tt-mwlhf-rvuyq-4ae --json
```

JSON retains absent, empty and nonempty chip/HostOS values distinctly. A missing
chip ID is unknown registration data, not proof of missing hardware capability.
Any later authentication change requires evidence and policy beyond this report.

## Preserve uncertainty when consuming results

Use the typed library reports or JSON for Canic integration. Text is a
human-facing view. JSON preserves raw values, including large integers; select
a consumer that retains their precision rather than coercing all numbers into
floating-point values. Configured strings are not measured software identities.

Retain endpoint, source, record mutation metadata and verification flags with
the values. A Registry query signature is distinct from consensus certification;
a local observation timestamp is distinct from certified time. Separate reports
do not acquire a shared version merely because they were requested close together.

A failed command exits unsuccessfully and emits no success JSON. Treat the
failure as unavailable evidence, not zero usage, an empty membership or downtime.
The library preserves typed causes; membership also retains its first report
when the second read fails. Query adds no persistent history or background retry.

Before deciding whether to add cloud engines or consolidate workloads, combine
these observations with application demand, resource limits, availability needs
and placement constraints. Query neither selects the final topology nor performs
the change. The [report contracts](../design/0.104-restart.md#report-guarantees)
define the evidence boundary for future additions.
