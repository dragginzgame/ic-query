//! Module: cloud_engine::wire
//!
//! Responsibility: mirror the pinned CloudEngine control-plane Candid interface.
//! Does not own: report provenance, source validation, transport, or rendering.
//! Boundary: changes here follow the ICP CLI 1.3.0 engine-canister contracts exactly.

use super::{CloudEngineNodeType, CloudEnginePriceRow};
use candid::{CandidType, Deserialize, Nat, Principal};

///
/// GetEngineOperatorBySubnetArgs
///
/// Wire argument for `getEngineOperatorBySubnet`.
///

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize)]
pub(super) struct GetEngineOperatorBySubnetArgs {
    pub(super) subnet_id: Option<Principal>,
}

///
/// GetEngineOperatorBySubnetResult
///
/// Wire response from `getEngineOperatorBySubnet`.
///

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize)]
pub(super) struct GetEngineOperatorBySubnetResult {
    pub(super) engine_operator_id: Option<Principal>,
}

///
/// GetEngineOwnerResult
///
/// Wire response from an engine operator's `getEngineOwner` query.
///

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize)]
pub(super) struct GetEngineOwnerResult {
    pub(super) engine_owner: Option<Principal>,
}

///
/// GetPlatformAdminResult
///
/// Wire response from an engine operator's `getPlatformAdmin` query.
///

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize)]
pub(super) struct GetPlatformAdminResult {
    pub(super) platform_admin: Option<Principal>,
}

///
/// CaffeineSettings
///
/// Wire Caffeine configuration nested in an operator response.
///

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize)]
pub(super) struct CaffeineSettings {
    pub(super) enabled: Option<bool>,
}

///
/// GetCaffeineSettingsResult
///
/// Wire response from an engine operator's `getCaffeineSettings` query.
///

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize)]
pub(super) struct GetCaffeineSettingsResult {
    pub(super) settings: Option<CaffeineSettings>,
}

///
/// ListDomainsResult
///
/// Wire response from an engine operator's `listDomains` query.
///

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize)]
pub(super) struct ListDomainsResult {
    pub(super) domains: Option<Vec<String>>,
}

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize)]
struct CloudEnginePriceWire {
    gross_cycles: Nat,
    net_cycles: Nat,
}

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize)]
struct CloudEnginePriceEntryWire {
    node_type: CloudEngineNodeType,
    dc: Option<String>,
    provider: Option<Principal>,
    price: CloudEnginePriceWire,
    updated_at: i64,
}

///
/// CloudEngineMarketplaceEntryWire
///
/// Wire row returned by `listMarketplacePrices`.
///

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize)]
pub(super) struct CloudEngineMarketplaceEntryWire {
    key: String,
    entry: CloudEnginePriceEntryWire,
}

impl CloudEngineMarketplaceEntryWire {
    pub(super) fn into_report_row(self) -> CloudEnginePriceRow {
        CloudEnginePriceRow {
            key: self.key,
            node_type: self.entry.node_type,
            data_center_id: self.entry.dc,
            provider_id: self.entry.provider.map(|principal| principal.to_text()),
            net_cycles_per_month: self.entry.price.net_cycles.0.to_str_radix(10),
            gross_cycles_per_month: self.entry.price.gross_cycles.0.to_str_radix(10),
            updated_at_unix_nanos: self.entry.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operator_resolution_shape_round_trips() {
        let args = GetEngineOperatorBySubnetArgs {
            subnet_id: Some(Principal::from_text("aaaaa-aa").unwrap()),
        };
        let encoded = candid::encode_one(&args).expect("encode operator args");
        let decoded: GetEngineOperatorBySubnetArgs =
            candid::decode_one(&encoded).expect("decode operator args");
        assert_eq!(decoded, args);

        let result = GetEngineOperatorBySubnetResult {
            engine_operator_id: Some(Principal::from_text("wlnge-zyaaa-aaabw-aaaaa-cai").unwrap()),
        };
        let encoded = candid::encode_one(&result).expect("encode operator result");
        let decoded: GetEngineOperatorBySubnetResult =
            candid::decode_one(&encoded).expect("decode operator result");

        assert_eq!(decoded, result);
    }

    #[test]
    fn marketplace_node_classes_decode_exact_candid_labels() {
        // Independent Candid fixtures: one null variant with its wire label hash.
        for (expected, label, bytes) in [
            (
                CloudEngineNodeType::Type4_1,
                "type4_1",
                &[
                    0x44, 0x49, 0x44, 0x4c, 0x01, 0x6b, 0x01, 0xac, 0x8b, 0xcf, 0xc5, 0x0f, 0x7f,
                    0x01, 0x00, 0x00,
                ][..],
            ),
            (
                CloudEngineNodeType::Type4_2,
                "type4_2",
                &[
                    0x44, 0x49, 0x44, 0x4c, 0x01, 0x6b, 0x01, 0xad, 0x8b, 0xcf, 0xc5, 0x0f, 0x7f,
                    0x01, 0x00, 0x00,
                ][..],
            ),
            (
                CloudEngineNodeType::Type4_3,
                "type4_3",
                &[
                    0x44, 0x49, 0x44, 0x4c, 0x01, 0x6b, 0x01, 0xae, 0x8b, 0xcf, 0xc5, 0x0f, 0x7f,
                    0x01, 0x00, 0x00,
                ][..],
            ),
            (
                CloudEngineNodeType::Type4_4,
                "type4_4",
                &[
                    0x44, 0x49, 0x44, 0x4c, 0x01, 0x6b, 0x01, 0xaf, 0x8b, 0xcf, 0xc5, 0x0f, 0x7f,
                    0x01, 0x00, 0x00,
                ][..],
            ),
            (
                CloudEngineNodeType::Type4_5,
                "type4_5",
                &[
                    0x44, 0x49, 0x44, 0x4c, 0x01, 0x6b, 0x01, 0xb0, 0x8b, 0xcf, 0xc5, 0x0f, 0x7f,
                    0x01, 0x00, 0x00,
                ][..],
            ),
        ] {
            let decoded: CloudEngineNodeType = candid::decode_one(bytes).expect("wire node class");
            assert_eq!(decoded, expected);
            assert_eq!(serde_json::to_value(decoded).unwrap(), label);
            let encoded = candid::encode_one(decoded).expect("encode node class");
            assert_eq!(
                candid::decode_one::<CloudEngineNodeType>(&encoded).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn marketplace_shape_preserves_arbitrary_precision_cycle_amounts() {
        let row = CloudEngineMarketplaceEntryWire {
            key: "type4.1".to_string(),
            entry: CloudEnginePriceEntryWire {
                node_type: CloudEngineNodeType::Type4_1,
                dc: None,
                provider: None,
                price: CloudEnginePriceWire {
                    net_cycles: Nat::from(471_065_106_452_u64),
                    gross_cycles: Nat::from(588_831_383_065_u64),
                },
                updated_at: 1_785_946_128_242_156_275,
            },
        };
        let encoded = candid::encode_one(vec![row]).expect("encode marketplace rows");
        let decoded: Vec<CloudEngineMarketplaceEntryWire> =
            candid::decode_one(&encoded).expect("decode marketplace rows");
        let projected = decoded.into_iter().next().unwrap().into_report_row();

        assert_eq!(projected.node_type, CloudEngineNodeType::Type4_1);
        assert_eq!(projected.net_cycles_per_month, "471065106452");
    }
}
