use super::{FixtureSnsDiscoverySource, GOVERNANCE_A, ROOT_A};
use crate::sns::report::tests::*;

pub(in crate::sns::report::tests) const NEURON_A: &str =
    "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

///
/// FixtureSnsNeuronSource
///
/// Successful exact SNS neuron detail source used by report tests.
///

pub(in crate::sns::report::tests) struct FixtureSnsNeuronSource;

delegate_sns_discovery!(FixtureSnsNeuronSource);

impl SnsNeuronSource for FixtureSnsNeuronSource {
    fn fetch_sns_neuron(
        &self,
        _request: &SnsSourceRequest,
        sns: &MainnetSns,
        neuron_id: &str,
    ) -> Result<MainnetSnsNeuron, SnsHostError> {
        assert_eq!(sns.governance_canister_id, GOVERNANCE_A);
        assert_eq!(neuron_id, NEURON_A);
        Ok(fixture_sns_neuron())
    }
}

pub(in crate::sns::report::tests) fn fixture_sns_neuron() -> MainnetSnsNeuron {
    MainnetSnsNeuron {
        detail: SnsNeuronDetail {
            neuron: SnsNeuronRow {
                neuron_id: NEURON_A.to_string(),
                cached_neuron_stake_e8s: 100_000_000,
                maturity_e8s_equivalent: 50_000_000,
                staked_maturity_e8s_equivalent: Some(25_000_000),
                created_timestamp_seconds: 1_780_272_000,
                created_at: "2026-06-01T00:00:00Z".to_string(),
                source_nns_neuron_id: Some(42),
                auto_stake_maturity: Some(true),
                aging_since_timestamp_seconds: 1_780_272_100,
                dissolve_state: Some(SnsNeuronDissolveState::DissolveDelaySeconds(31_536_000)),
                voting_power_percentage_multiplier: 100,
                vesting_period_seconds: Some(63_072_000),
                neuron_fees_e8s: 10_000,
            },
            permissions: vec![SnsNeuronPermissionRow {
                principal: Some(GOVERNANCE_A.to_string()),
                permission_types: [1, 2, 3, 4, 9, 10]
                    .into_iter()
                    .map(SnsNeuronPermissionValue::from_code)
                    .collect(),
            }],
            disburse_maturity_in_progress: vec![SnsMaturityDisbursementRow {
                timestamp_of_disbursement_seconds: 1_780_272_200,
                amount_e8s: 12_500_000,
                account_to_disburse_to: Some(SnsNeuronAccount {
                    owner: Some(ROOT_A.to_string()),
                    subaccount_hex: Some("ab".repeat(32)),
                }),
                finalize_disbursement_timestamp_seconds: Some(1_780_358_600),
            }],
            followees: vec![SnsNeuronFolloweesRow {
                function_id: 1,
                followee_neuron_ids: vec!["11".repeat(32)],
            }],
            topic_followees: Some(vec![SnsNeuronTopicFolloweesRow {
                topic_code: 5,
                topic: Some("governance".to_string()),
                followees: vec![SnsNeuronFolloweeRow {
                    neuron_id: Some("22".repeat(32)),
                    alias: Some("governance lead".to_string()),
                }],
            }]),
            maturity_mint_conversion_observed_disabled: SnsPolicyObservationStatus::Violated,
            manual_maturity_staking_observed_disabled: SnsPolicyObservationStatus::Violated,
        },
    }
}

///
/// FixtureSnsNeuronsSource
///
/// Successful bounded and paged SNS neuron source used by report tests.
///

pub(in crate::sns::report::tests) struct FixtureSnsNeuronsSource;

delegate_sns_discovery!(FixtureSnsNeuronsSource);

impl SnsNeuronsSource for FixtureSnsNeuronsSource {
    fn fetch_sns_neurons(
        &self,
        _request: &SnsSourceRequest,
        sns: &MainnetSns,
        limit: u32,
        owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeurons, SnsHostError> {
        assert_eq!(sns.governance_canister_id, GOVERNANCE_A);
        assert_eq!(limit, 10);
        assert_eq!(owner_principal_id, Some(GOVERNANCE_A));
        Ok(MainnetSnsNeurons {
            neurons: vec![SnsNeuronRow {
                neuron_id: NEURON_A.to_string(),
                cached_neuron_stake_e8s: 123,
                maturity_e8s_equivalent: 456,
                staked_maturity_e8s_equivalent: Some(789),
                created_timestamp_seconds: 1_780_272_000,
                created_at: "2026-06-01T00:00:00Z".to_string(),
                source_nns_neuron_id: Some(42),
                auto_stake_maturity: Some(true),
                aging_since_timestamp_seconds: 1_780_272_100,
                dissolve_state: Some(SnsNeuronDissolveState::DissolveDelaySeconds(31_536_000)),
                voting_power_percentage_multiplier: 100,
                vesting_period_seconds: Some(63_072_000),
                neuron_fees_e8s: 10,
            }],
        })
    }

    fn fetch_sns_neuron_page(
        &self,
        _request: &SnsSourceRequest,
        sns: &MainnetSns,
        limit: u32,
        start_page_at: Option<&SnsNeuronId>,
        owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeuronPage, SnsHostError> {
        assert_eq!(sns.governance_canister_id, GOVERNANCE_A);
        assert_eq!(limit, 10);
        assert!(start_page_at.is_none());
        assert_eq!(owner_principal_id, None);
        Ok(MainnetSnsNeuronPage {
            neurons: vec![SnsNeuronRow {
                neuron_id: NEURON_A.to_string(),
                cached_neuron_stake_e8s: 123,
                maturity_e8s_equivalent: 456,
                staked_maturity_e8s_equivalent: Some(789),
                created_timestamp_seconds: 1_780_272_000,
                created_at: "2026-06-01T00:00:00Z".to_string(),
                source_nns_neuron_id: Some(42),
                auto_stake_maturity: Some(true),
                aging_since_timestamp_seconds: 1_780_272_100,
                dissolve_state: Some(SnsNeuronDissolveState::DissolveDelaySeconds(31_536_000)),
                voting_power_percentage_multiplier: 100,
                vesting_period_seconds: Some(63_072_000),
                neuron_fees_e8s: 10,
            }],
            last_cursor: Some(SnsNeuronId {
                id: (0..32).collect(),
            }),
        })
    }
}

///
/// PagedFixtureSnsNeuronsSource
///
/// Multi-page SNS neuron source used to exercise complete snapshot refreshes.
///

pub(in crate::sns::report::tests) struct PagedFixtureSnsNeuronsSource;

delegate_sns_discovery!(PagedFixtureSnsNeuronsSource);

impl SnsNeuronsSource for PagedFixtureSnsNeuronsSource {
    fn fetch_sns_neurons(
        &self,
        _request: &SnsSourceRequest,
        _sns: &MainnetSns,
        _limit: u32,
        _owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeurons, SnsHostError> {
        unreachable!("paged fixture is only used by complete cache refresh tests")
    }

    fn fetch_sns_neuron_page(
        &self,
        _request: &SnsSourceRequest,
        sns: &MainnetSns,
        limit: u32,
        start_page_at: Option<&SnsNeuronId>,
        owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeuronPage, SnsHostError> {
        assert_eq!(sns.governance_canister_id, GOVERNANCE_A);
        assert_eq!(limit, 2);
        assert_eq!(owner_principal_id, None);
        let cursor = start_page_at.map(|cursor| cursor.id.as_slice());
        let (neurons, last_cursor) = match cursor {
            None => (
                vec![neuron_row(1, 10), neuron_row(2, 30)],
                Some(vec![2; 32]),
            ),
            Some(cursor) if cursor == [2; 32] => (
                vec![neuron_row(2, 30), neuron_row(3, 50)],
                Some(vec![3; 32]),
            ),
            Some(cursor) if cursor == [3; 32] => (vec![neuron_row(3, 50)], Some(vec![3; 32])),
            Some(other) => panic!("unexpected cursor {other:?}"),
        };
        Ok(MainnetSnsNeuronPage {
            neurons,
            last_cursor: last_cursor.map(|id| SnsNeuronId { id }),
        })
    }
}

///
/// NoLiveSnsNeuronsSource
///
/// SNS neuron source that rejects live calls in cache-backed report tests.
///

pub(in crate::sns::report::tests) struct NoLiveSnsNeuronsSource;

impl SnsDiscoverySource for NoLiveSnsNeuronsSource {
    fn fetch_sns_inventory(
        &self,
        _request: &SnsSourceRequest,
    ) -> Result<MainnetSnsInventory, SnsHostError> {
        unreachable!("cache-backed neuron report should not fetch SNS inventory")
    }

    fn fetch_sns_metadata(
        &self,
        _request: &SnsSourceRequest,
        _targets: &[MainnetSnsCanisters],
    ) -> Result<Vec<MainnetSnsMetadata>, SnsHostError> {
        unreachable!("cache-backed neuron report should not fetch SNS metadata")
    }
}

impl SnsNeuronsSource for NoLiveSnsNeuronsSource {
    fn fetch_sns_neurons(
        &self,
        _request: &SnsSourceRequest,
        _sns: &MainnetSns,
        _limit: u32,
        _owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeurons, SnsHostError> {
        unreachable!("cache-backed neuron report should not fetch live neurons")
    }

    fn fetch_sns_neuron_page(
        &self,
        _request: &SnsSourceRequest,
        _sns: &MainnetSns,
        _limit: u32,
        _start_page_at: Option<&SnsNeuronId>,
        _owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeuronPage, SnsHostError> {
        unreachable!("cache-backed neuron report should not fetch neuron pages")
    }
}

fn neuron_row(neuron_id: u8, stake: u64) -> SnsNeuronRow {
    SnsNeuronRow {
        neuron_id: format!("{neuron_id:02x}").repeat(32),
        cached_neuron_stake_e8s: stake,
        maturity_e8s_equivalent: stake / 2,
        staked_maturity_e8s_equivalent: None,
        created_timestamp_seconds: 1_780_272_000 + stake,
        created_at: format_utc_timestamp_secs(1_780_272_000 + stake),
        source_nns_neuron_id: None,
        auto_stake_maturity: None,
        aging_since_timestamp_seconds: 1_780_272_000 + stake,
        dissolve_state: None,
        voting_power_percentage_multiplier: 100,
        vesting_period_seconds: None,
        neuron_fees_e8s: 0,
    }
}
