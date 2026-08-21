//! Module: ic_registry::routing_shards
//!
//! Responsibility: parse and validate DFINITY `canister_ranges_*` shard boundaries.

use super::proto::{CanisterId, RoutingTable};
use crate::hex::{decode_lowercase_hex, is_canonical_lowercase_hex};
use candid::Principal;

pub fn canister_range_start_from_key(prefix: &str, key: &str) -> Result<Principal, String> {
    let suffix = key
        .strip_prefix(prefix)
        .filter(|suffix| is_canonical_lowercase_hex(suffix))
        .ok_or_else(|| format!("invalid canister-ranges key {key:?}"))?;
    let bytes = decode_lowercase_hex(suffix)
        .ok_or_else(|| format!("invalid canister-ranges key suffix in {key:?}"))?;
    validate_routable_canister_id(&bytes, key)?;
    Principal::try_from_slice(&bytes)
        .map_err(|error| format!("invalid range-start principal in key {key:?}: {error}"))
}

pub fn validate_routing_table_shard_bounds(
    key: &str,
    lower_bound: &Principal,
    shard: &RoutingTable,
    next_start: Option<&Principal>,
) -> Result<(), String> {
    if shard.entries.is_empty() {
        return Err(format!("routing shard {key:?} is empty"));
    }
    for entry in &shard.entries {
        let range = entry
            .range
            .as_ref()
            .ok_or_else(|| format!("routing shard {key:?} has an entry without a range"))?;
        let start = routing_canister_id_bytes(range.start_canister_id.as_ref(), key, "start")?;
        let end = routing_canister_id_bytes(range.end_canister_id.as_ref(), key, "end")?;
        if start > end {
            return Err(format!("routing shard {key:?} has an inverted range"));
        }
        if start < lower_bound.as_slice() {
            return Err(format!(
                "routing shard {key:?} starts below the lower bound encoded by its key"
            ));
        }
        if next_start.is_some_and(|next| end >= next.as_slice()) {
            return Err(format!(
                "routing shard {key:?} extends into the next shard's key interval"
            ));
        }
    }
    Ok(())
}

fn routing_canister_id_bytes<'a>(
    canister: Option<&'a CanisterId>,
    shard_key: &str,
    boundary: &str,
) -> Result<&'a [u8], String> {
    let bytes = canister
        .and_then(|canister| canister.principal_id.as_ref())
        .map(|principal| principal.raw.as_slice())
        .ok_or_else(|| {
            format!("routing shard {shard_key:?} has a range without a {boundary} canister id")
        })?;
    validate_routable_canister_id(bytes, shard_key)?;
    Ok(bytes)
}

fn validate_routable_canister_id(bytes: &[u8], key: &str) -> Result<(), String> {
    if bytes.len() != 10 || bytes[8..] != [1, 1] {
        return Err(format!("{key:?} contains a non-routable canister id"));
    }
    Ok(())
}
