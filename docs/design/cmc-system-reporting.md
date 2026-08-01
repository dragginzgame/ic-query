# Certified CMC System Reporting

## Status

- Status: implemented
- Authority: mainnet Cycle Minting Canister (CMC)
- Canister: `rkp4c-7iaaa-aaaaa-aaaca-cai`
- Native method: `get_icp_xdr_conversion_rate`
- Collection mode: bounded live query, no cache

## Purpose

`ic-query` exposes the CMC rate needed for ICP/XDR and cycles operating-cost
evidence without scraping metrics, enumerating canisters, or adding a generic
Candid caller. The one native CMC value supports two views:

- `CmcXdrReport` preserves the certified ICP/XDR conversion rate; and
- `CmcCyclesReport` derives cycles per ICP from that rate and the IC protocol
  constant of one trillion cycles per XDR.

The CLI mirrors those views as `icq system xdr` and
`icq system cycles`.

## Native interface

The public CMC Candid interface returns:

```text
get_icp_xdr_conversion_rate : () -> (record {
  data : record {
    timestamp_seconds : nat64;
    xdr_permyriad_per_icp : nat64;
  };
  hash_tree : blob;
  certificate : blob;
}) query
```

`xdr_permyriad_per_icp` is the number of ten-thousandths of XDR corresponding
to one ICP. JSON preserves that raw integer. Human-facing text additionally
renders the exact four-decimal XDR value without replacing the native field.

Each report records the fixed CMC principal, requested network, replica
endpoint, collection timestamp, collector identity, rate timestamp, raw rate,
and certificate/hash-tree byte counts and lowercase hex.

## Certification contract

The built-in `LiveCmcSource` accepts a response only after all of these checks
succeed:

1. decode the CBOR system certificate;
2. authenticate its signature, delegation, time, and CMC canister authority
   through `ic-agent` and the mainnet root of trust;
3. decode the CBOR witness hash tree;
4. prove that the witness digest equals the CMC `certified_data` value in the
   certificate;
5. require the native `ICP_XDR_CONVERSION_RATE` leaf; and
6. require that leaf to equal the Candid encoding of the returned `data`
   value.

Malformed CBOR, certificate authentication failures, missing or partial
leaves, digest mismatches, and rate mismatches are typed failures. A successful
built-in result records `certificate_verified: true`. Certificate validation
shares the same internal canister-certified-data path used by ICRC tip
evidence; authority-specific leaf validation remains in each report family.

This is application-level certified data. It is stronger than merely receiving
a transport query response and is distinct from a Registry-version snapshot.

## Cycles derivation

The IC protocol defines:

```text
cycles_per_xdr = 1_000_000_000_000
cycles_per_icp = xdr_permyriad_per_icp * cycles_per_xdr / 10_000
```

The multiplication uses `u128`, and division is exact because
`cycles_per_xdr` is divisible by 10,000. `CmcCyclesReport` retains the raw
certified rate, both derived cycle values, the formula string, the explicit
`ic_protocol_constant` source label, and the same certificate evidence. It
does not present `cycles_per_xdr` as a separately queried CMC field.

## Adapter and network contract

`CmcSourceRequest` carries network, endpoint, collection timestamp, and
collector identity. `CmcSource` owns the one coherent certified-rate
capability, and both report builders use it; there is no adapter per view.
Custom sources can reuse the report construction contract through the
`*_with_source` builders.

The built-in source and both custom-source builders reject every network other
than `ic` before constructing an agent or invoking a source. Endpoint syntax
is validated before agent construction. The top-level `--network` option is
honored for `icq system` and is never silently ignored.

## Collection and non-goals

Each command makes exactly one native CMC query and never reads or writes a
cache. Exchange rates are changing point values, so implicit caching would
obscure freshness rather than reduce meaningful fan-out.

The current public CMC Candid interface does not expose total cycles minted.
The CMC Prometheus endpoint includes cycles metrics, but the official IC
documentation explicitly describes that HTTP response as uncertified.
`ic-query` therefore does not scrape it into these certified reports. This
surface also does not call hidden CMC methods, convert ICP, mint cycles, create
canisters, enumerate canisters, or expose arbitrary Candid invocation.

Official references:

- [IC system canisters](https://docs.internetcomputer.org/references/system-canisters/)
- [CMC Candid interface](https://github.com/dfinity/ic/blob/master/rs/nns/cmc/cmc.did)
