use super::{FixtureSnsDiscoverySource, INDEX_A, SWAP_A};
use crate::sns::report::tests::*;

///
/// FixtureSnsSwapSource
///
/// Successful bounded SNS swap source used by report tests.
///

pub(in crate::sns::report::tests) struct FixtureSnsSwapSource;

delegate_sns_discovery!(FixtureSnsSwapSource);

impl SnsSwapSource for FixtureSnsSwapSource {
    fn fetch_sns_swap(
        &self,
        _request: &SnsSourceRequest,
        sns: &MainnetSns,
    ) -> Result<MainnetSnsSwap, SnsHostError> {
        assert_eq!(sns.swap_canister_id, SWAP_A);
        Ok(fixture_mainnet_sns_swap(SWAP_A))
    }
}

///
/// PartialFixtureSnsSwapSource
///
/// SNS swap source with one typed component gap.
///

pub(in crate::sns::report::tests) struct PartialFixtureSnsSwapSource;

delegate_sns_discovery!(PartialFixtureSnsSwapSource);

impl SnsSwapSource for PartialFixtureSnsSwapSource {
    fn fetch_sns_swap(
        &self,
        _request: &SnsSourceRequest,
        _sns: &MainnetSns,
    ) -> Result<MainnetSnsSwap, SnsHostError> {
        let mut swap = fixture_mainnet_sns_swap(SWAP_A);
        swap.derived_state = None;
        swap.gaps.push(SnsSwapQueryGap {
            component: SnsSwapComponent::DerivedState,
            method: SnsCanisterMethod::GetDerivedState,
            reason: "query rejected by fixture".to_string(),
        });
        Ok(swap)
    }
}

///
/// WrongTargetFixtureSnsSwapSource
///
/// SNS swap source that returns evidence for the wrong target canister.
///

pub(in crate::sns::report::tests) struct WrongTargetFixtureSnsSwapSource;

delegate_sns_discovery!(WrongTargetFixtureSnsSwapSource);

impl SnsSwapSource for WrongTargetFixtureSnsSwapSource {
    fn fetch_sns_swap(
        &self,
        _request: &SnsSourceRequest,
        _sns: &MainnetSns,
    ) -> Result<MainnetSnsSwap, SnsHostError> {
        Ok(fixture_mainnet_sns_swap(INDEX_A))
    }
}

///
/// MutatingFixtureSnsSwapSource
///
/// SNS swap source that mutates otherwise valid evidence for invariant tests.
///

pub(in crate::sns::report::tests) struct MutatingFixtureSnsSwapSource(
    pub(in crate::sns::report::tests) fn(&mut MainnetSnsSwap),
);

delegate_sns_discovery!(MutatingFixtureSnsSwapSource);

impl SnsSwapSource for MutatingFixtureSnsSwapSource {
    fn fetch_sns_swap(
        &self,
        _request: &SnsSourceRequest,
        _sns: &MainnetSns,
    ) -> Result<MainnetSnsSwap, SnsHostError> {
        let mut swap = fixture_mainnet_sns_swap(SWAP_A);
        self.0(&mut swap);
        Ok(swap)
    }
}

fn fixture_mainnet_sns_swap(swap_canister_id: &str) -> MainnetSnsSwap {
    MainnetSnsSwap {
        swap_canister_id: swap_canister_id.to_string(),
        lifecycle_method: SnsCanisterMethod::GetLifecycle,
        sale_parameters_method: SnsCanisterMethod::GetSaleParameters,
        derived_state_method: SnsCanisterMethod::GetDerivedState,
        point_in_time_guaranteed: false,
        lifecycle: Some(SnsSwapLifecycle {
            lifecycle: Some(2),
            lifecycle_name: Some("open".to_string()),
            decentralization_sale_open_timestamp_seconds: Some(1_780_531_000),
            decentralization_swap_termination_timestamp_seconds: None,
        }),
        sale_parameters: Some(SnsSwapSaleParameters {
            min_icp_e8s: 100_000_000,
            max_icp_e8s: 100_000_000_000,
            min_direct_participation_icp_e8s: Some(1_000_000_000),
            max_direct_participation_icp_e8s: Some(90_000_000_000),
            sns_token_e8s: 250_000_000_000,
            min_participants: 25,
            min_participant_icp_e8s: 100_000_000,
            max_participant_icp_e8s: 10_000_000_000,
            swap_due_timestamp_seconds: 1_781_136_000,
            sale_delay_seconds: Some(86_400),
            neuron_basket_construction_parameters: Some(
                SnsSwapNeuronBasketConstructionParameters {
                    count: 5,
                    dissolve_delay_interval_seconds: 2_592_000,
                },
            ),
        }),
        derived_state: Some(SnsSwapDerivedState {
            sns_tokens_per_icp: Some(2.5),
            buyer_total_icp_e8s: Some(40_000_000_000),
            direct_participation_icp_e8s: Some(35_000_000_000),
            neurons_fund_participation_icp_e8s: Some(5_000_000_000),
            direct_participant_count: Some(120),
            cf_participant_count: Some(8),
            cf_neuron_count: Some(12),
        }),
        gaps: Vec::new(),
    }
}
