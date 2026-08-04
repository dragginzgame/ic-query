use super::{FixtureSnsDiscoverySource, GOVERNANCE_A, LEDGER_A, ROOT_A};
use crate::sns::report::{source::SNS_METRICS_CALL_TYPE, tests::*};

///
/// FixtureSnsMetricsSource
///
/// Successful bounded SNS Governance metrics source used by report tests.
///

pub(in crate::sns::report::tests) struct FixtureSnsMetricsSource;

delegate_sns_discovery!(FixtureSnsMetricsSource);

impl SnsMetricsSource for FixtureSnsMetricsSource {
    fn fetch_sns_metrics(
        &self,
        _request: &SnsSourceRequest,
        sns: &MainnetSns,
        time_window_seconds: u64,
    ) -> Result<MainnetSnsMetrics, SnsHostError> {
        assert_eq!(sns.governance_canister_id, GOVERNANCE_A);
        Ok(fixture_mainnet_sns_metrics(time_window_seconds))
    }
}

///
/// MutatingFixtureSnsMetricsSource
///
/// SNS metrics source that mutates valid evidence for invariant tests.
///

pub(in crate::sns::report::tests) struct MutatingFixtureSnsMetricsSource(
    pub(in crate::sns::report::tests) fn(&mut MainnetSnsMetrics),
);

delegate_sns_discovery!(MutatingFixtureSnsMetricsSource);

impl SnsMetricsSource for MutatingFixtureSnsMetricsSource {
    fn fetch_sns_metrics(
        &self,
        _request: &SnsSourceRequest,
        _sns: &MainnetSns,
        time_window_seconds: u64,
    ) -> Result<MainnetSnsMetrics, SnsHostError> {
        let mut metrics = fixture_mainnet_sns_metrics(time_window_seconds);
        self.0(&mut metrics);
        Ok(metrics)
    }
}

///
/// NoCallSnsMetricsSource
///
/// Source that panics on every capability call to prove pre-source validation.
///

pub(in crate::sns::report::tests) struct NoCallSnsMetricsSource;

impl SnsDiscoverySource for NoCallSnsMetricsSource {
    fn fetch_sns_inventory(
        &self,
        _request: &SnsSourceRequest,
    ) -> Result<MainnetSnsInventory, SnsHostError> {
        panic!("invalid metrics requests must fail before discovery")
    }

    fn fetch_sns_metadata(
        &self,
        _request: &SnsSourceRequest,
        _targets: &[MainnetSnsCanisters],
    ) -> Result<Vec<MainnetSnsMetadata>, SnsHostError> {
        panic!("invalid metrics requests must fail before metadata")
    }
}

impl SnsMetricsSource for NoCallSnsMetricsSource {
    fn fetch_sns_metrics(
        &self,
        _request: &SnsSourceRequest,
        _sns: &MainnetSns,
        _time_window_seconds: u64,
    ) -> Result<MainnetSnsMetrics, SnsHostError> {
        panic!("invalid metrics requests must fail before Governance")
    }
}

pub(in crate::sns::report::tests) fn fixture_mainnet_sns_metrics(
    time_window_seconds: u64,
) -> MainnetSnsMetrics {
    MainnetSnsMetrics {
        governance_canister_id: GOVERNANCE_A.to_string(),
        method: SnsCanisterMethod::GetMetrics,
        call_type: SNS_METRICS_CALL_TYPE,
        time_window_seconds,
        point_in_time_guaranteed: false,
        treasury_metrics_cached: true,
        num_recently_submitted_proposals: Some(12),
        num_recently_executed_proposals: Some(9),
        last_ledger_block_timestamp: Some(1_780_531_100),
        genesis_timestamp_seconds: Some(1_700_000_000),
        treasury_metrics: vec![
            SnsTreasuryMetricRow {
                treasury: 2,
                treasury_kind: SnsTreasuryKind::SnsToken,
                name: Some("SNS token treasury".to_string()),
                ledger_canister_id: Some(LEDGER_A.to_string()),
                account_owner: Some(ROOT_A.to_string()),
                account_subaccount_hex: Some("00".repeat(32)),
                amount_e8s: Some(2_500_000_000),
                original_amount_e8s: Some(5_000_000_000),
                timestamp_seconds: Some(1_780_531_010),
            },
            SnsTreasuryMetricRow {
                treasury: 1,
                treasury_kind: SnsTreasuryKind::Icp,
                name: Some("ICP treasury".to_string()),
                ledger_canister_id: None,
                account_owner: None,
                account_subaccount_hex: None,
                amount_e8s: Some(750_000_000),
                original_amount_e8s: Some(1_000_000_000),
                timestamp_seconds: Some(1_780_531_000),
            },
        ],
        voting_power_metrics: Some(SnsVotingPowerMetrics {
            governance_total_potential_voting_power: Some(42_000_000_000),
            timestamp_seconds: Some(1_780_531_020),
        }),
    }
}
