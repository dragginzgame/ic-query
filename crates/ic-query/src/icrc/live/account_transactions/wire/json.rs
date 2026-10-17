//! Lossless JSON scalar formatting for the two supported account-history wire contracts.

use crate::hex::hex_bytes;
use icrc_ledger_types::icrc1::account::Account as LedgerAccount;
use serde::{Serialize, Serializer};
use std::fmt::Display;

// JSON field names must not become Serde rename attributes on Candid wire records.
macro_rules! json_record {
    ($record:ty, $value:ident, { $($key:literal => $field:expr),* $(,)? }) => {
        impl serde::Serialize for $record {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let $value = self;
                super::json::json_record!(serializer, { $($key => $field),* })
            }
        }
    };
    ($serializer:expr, { $($key:literal => $value:expr),* $(,)? }) => {{
        let mut record = serde::Serializer::serialize_map(
            $serializer,
            Some([$($key),*].len()),
        )?;
        $(serde::ser::SerializeMap::serialize_entry(&mut record, $key, &$value)?;)*
        serde::ser::SerializeMap::end(record)
    }};
}

pub(super) use json_record;

///
/// Decimal
///
/// Exact decimal scalar encoded as JSON text rather than a potentially lossy number.
///

pub(super) struct Decimal<T>(pub(super) T);

impl<T: Display> Serialize for Decimal<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&self.0)
    }
}

///
/// Hex
///
/// Raw memo or subaccount bytes encoded as lowercase hexadecimal JSON text.
///

pub(super) struct Hex<'a>(pub(super) &'a [u8]);

impl Serialize for Hex<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&hex_bytes(self.0))
    }
}

///
/// Account
///
/// Borrowed ICRC account projected to canonical principal text and an optional raw subaccount.
///

pub(super) struct Account<'a>(pub(super) &'a LedgerAccount);

json_record!(Account<'_>, account, {
    "owner" => account.0.owner.to_text(),
    "subaccount_hex" => account.0.subaccount.as_ref().map(|bytes| Hex(bytes)),
});

#[cfg(test)]
mod tests {
    use super::*;
    use candid::{CandidType, Principal};
    use serde_json::json;

    #[test]
    fn canonical_accounts_validate_wire_lengths_and_preserve_explicit_subaccounts() {
        #[derive(CandidType)]
        struct UncheckedAccount {
            owner: Principal,
            subaccount: Option<Vec<u8>>,
        }
        for length in [0, 31, 32, 33] {
            let bytes = candid::encode_one(UncheckedAccount {
                owner: Principal::anonymous(),
                subaccount: Some(vec![0; length]),
            })
            .unwrap();
            assert_eq!(
                candid::decode_one::<LedgerAccount>(&bytes).is_ok(),
                length == 32
            );
        }
        for (subaccount, expected) in [
            (None, json!(null)),
            (Some([0; 32]), json!("00".repeat(32))),
            (Some([0xff; 32]), json!("ff".repeat(32))),
        ] {
            let account = LedgerAccount {
                owner: Principal::anonymous(),
                subaccount,
            };
            assert_eq!(
                serde_json::to_value(Account(&account)).unwrap(),
                json!({
                    "owner": "2vxsx-fae", "subaccount_hex": expected,
                })
            );
        }
    }
}
