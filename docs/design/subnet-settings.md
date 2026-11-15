# Registered subnet features and resource limits

Status: source contract prepared; implementation awaits a published Platform decoder.
Supplier prerequisite:
[#169 — Add bounded Registry subnet feature and resource-limit projection for Query](https://github.com/dragginzgame/ic-platform/issues/169).

## Canic decision and source

Canic needs registered feature and resource settings when screening a subnet for
an application's requirements. The existing resource report observes usage;
these settings add configuration evidence, not measured headroom or readiness.
Use one mainnet Registry `get_value` query for
`subnet_record_<canonical subnet principal>`, with no historical version, through
the existing fixed endpoint, anonymous verified-query and get-value envelope path.

Reviewed upstream IC revision `00e4d8e3ee6081c1862e6119d63cffa76bd4af3d`:
[SubnetRecord schema](https://github.com/dfinity/ic/blob/00e4d8e3ee6081c1862e6119d63cffa76bd4af3d/rs/protobuf/def/registry/subnet/v1/subnet.proto)
and [feature conversion](https://github.com/dfinity/ic/blob/00e4d8e3ee6081c1862e6119d63cffa76bd4af3d/rs/registry/subnet_features/src/lib.rs).
The first slice selects only:

| Field path | Wire field number | Retained representation |
| --- | --- | --- |
| `max_number_of_canisters` | 24 | Raw u64 |
| `features` | 23 | Optional selected nested message |
| `features.http_requests` | 3 | Proto3 bool |
| `features.sev_enabled` | 9 | Optional bool |
| `resource_limits` | 34 | Optional selected nested message |
| `resource_limits.maximum_state_size` | 1 | Optional u64, bytes |
| `resource_limits.maximum_state_delta` | 2 | Optional u64, bytes |
| `resource_limits.maximum_query_instructions` | 3 | Optional u64 |
| `resource_limits.maximum_query_walltime_seconds` | 4 | Optional u64, seconds |

Preserve outer-message absence versus present-empty, and optional scalar absence
versus explicit zero/false. Inside a present feature message, absent
`http_requests` retains its protobuf false default. Do not substitute the Rust
feature struct's different construction defaults or resolve effective settings.
The schema defines zero canister count and zero query instruction/wall-time limits
as selecting replica defaults. Report raw zero as default-selected, not zero
capacity or unlimited. Other absent/zero limits remain raw declarations without
inventing fallback rules.

Configured `sev_enabled` is not a node chip identifier, an attestation report,
verified code measurement or proof that an authentication check can be removed.
Do not conflate it with the cloud-engine availability method's similarly named
chip-registration-derived flag. Configured HTTP support does not establish
successful outbound requests. This report neither probes nor changes features.

## Supplier and report boundary

The latest non-yanked Platform release observed in the Cargo index on 2026-11-15
was 0.33.7, checksum
`3e07e0565c70df9222016e93d6f488312176ef5e7036ded60db8b4bc3f61c05f`.
Its selected `SubnetRecord` decoder exposes membership, version, type and halt
flags only. Add the selected projection at Platform, not a Query protobuf decoder,
direct dependency or sibling-path workaround. Prefer a separate projection so
the existing borrowed record and its public construction contract need not change.

Planned library surface: `subnet::settings::fetch(subnet_id)`. Planned CLI:
`icq topology subnet settings SUBNET_ID [--json]`. Neither exists yet. Keep this
report independent from configuration/maintenance/compare until an explicit
composition contract accounts for extra reads and independently observed versions.

Schema 1 retains the subnet ID, raw selected settings and source evidence:
endpoint, Registry canister/method/key, record mutation version/timestamp and local
completion time. Use the existing 20-second read deadline, 1 MiB HTTP ceiling and
64 KiB protobuf input/4096-field/128 KiB allocation budgets. Nested selected
messages share the field/allocation budget; they must not reset it recursively.
One logical query, no fanout, pagination, retries or persistence. Query signatures
remain distinct from consensus certification; explicitly retain attestation as
unverified. The report does not measure running software or uptime.

Missing inline content, malformed selected fields, excessive payloads or failed
reads return typed errors with no success report; valid zero and absent settings
are successful evidence. JSON preserves raw values and presence. The CLI owns a
compact bordered table with units and distinct absent/default-selected labels,
exit 0 for admitted evidence, exit 1 for read/admission failure and exit 2 for
syntax errors. Add runnable changelog examples only once implemented.

## Regression requirements and legacy guidance

Platform owns independent wire fixtures for omitted/present-empty nested messages,
optional false/true/zero/full-width values, malformed booleans/varints/lengths,
duplicate selected fields, wrong wire types and shared exact/exceeded budgets.
Skip unselected fields within bounds; retain existing Registry decoder behavior.
Query will test raw-presence projection, zero/default labels, source evidence,
typed failure behavior and unchanged configuration reports, with no derived
capacity subtraction or authentication verdict.

The frozen predecessor review at `5b4951b23c2adfcd3c615961f131e766c4b50283`
already identified [authority regressions](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/subnet_catalog/tests/authority.rs)
and [separate component provenance](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/nns/topology/report/registry_versions.rs).
Retain their unknown-value/source-authority lessons, informed by the inspected
[API consolidation history](https://github.com/dragginzgame/ic-query-legacy/commit/b1fca76345efb465d327984d70e9862e1127bbe9).
Targeted legacy code searches found no `sev_enabled` or `maximum_state_size`
matches; that search is not proof of whole-repository absence. No old catalog,
classification policy, cache or source code is imported.
