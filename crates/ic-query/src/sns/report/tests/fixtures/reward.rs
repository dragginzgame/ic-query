use super::{FixtureSnsDiscoverySource, GOVERNANCE_A, ROOT_A, fixture_sns_governance_parameters};
use crate::sns::report::SNS_REWARD_CHECKPOINT_PAGE_SIZE;
use crate::sns::report::tests::*;
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
};

///
/// FixtureSnsRewardSource
///
/// Stable bracketed reward source with configurable strict neuron pages.
///

pub(in crate::sns::report::tests) struct FixtureSnsRewardSource {
    calls: RefCell<Vec<&'static str>>,
    pages: RefCell<VecDeque<MainnetSnsRewardNeuronPage>>,
    unstable_component: Option<&'static str>,
    version_calls: Cell<u32>,
    parameter_calls: Cell<u32>,
    event_calls: Cell<u32>,
    max_number_of_neurons: Option<u64>,
}

impl FixtureSnsRewardSource {
    pub(in crate::sns::report::tests) fn new(pages: Vec<MainnetSnsRewardNeuronPage>) -> Self {
        Self {
            calls: RefCell::new(Vec::new()),
            pages: RefCell::new(pages.into()),
            unstable_component: None,
            version_calls: Cell::new(0),
            parameter_calls: Cell::new(0),
            event_calls: Cell::new(0),
            max_number_of_neurons: Some(200_000),
        }
    }

    pub(in crate::sns::report::tests) fn unstable(
        pages: Vec<MainnetSnsRewardNeuronPage>,
        component: &'static str,
    ) -> Self {
        Self {
            unstable_component: Some(component),
            ..Self::new(pages)
        }
    }

    pub(in crate::sns::report::tests) fn calls(&self) -> Vec<&'static str> {
        self.calls.borrow().clone()
    }

    pub(in crate::sns::report::tests) const fn with_max_number_of_neurons(
        mut self,
        max_number_of_neurons: Option<u64>,
    ) -> Self {
        self.max_number_of_neurons = max_number_of_neurons;
        self
    }
}

delegate_sns_discovery!(FixtureSnsRewardSource);

impl SnsRewardSource for FixtureSnsRewardSource {
    fn fetch_sns_reward_running_version(
        &self,
        _request: &SnsSourceRequest,
        sns: &MainnetSns,
    ) -> Result<SnsRunningVersionResponse, SnsHostError> {
        assert_eq!(sns.governance_canister_id, GOVERNANCE_A);
        self.calls.borrow_mut().push("version");
        let call = self.version_calls.get();
        self.version_calls.set(call + 1);
        let mut response = fixture_reward_running_version();
        if self.unstable_component == Some("version") && call > 0 {
            response.pending_version = None;
        }
        Ok(response)
    }

    fn fetch_sns_reward_parameters(
        &self,
        _request: &SnsSourceRequest,
        _sns: &MainnetSns,
    ) -> Result<SnsGovernanceParameters, SnsHostError> {
        self.calls.borrow_mut().push("parameters");
        let call = self.parameter_calls.get();
        self.parameter_calls.set(call + 1);
        let mut parameters = fixture_sns_governance_parameters();
        parameters.max_number_of_neurons = self.max_number_of_neurons;
        if self.unstable_component == Some("parameters") && call > 0 {
            parameters.transaction_fee_e8s = Some(20_000);
        }
        Ok(parameters)
    }

    fn fetch_sns_reward_event(
        &self,
        _request: &SnsSourceRequest,
        _sns: &MainnetSns,
    ) -> Result<SnsRewardEvent, SnsHostError> {
        self.calls.borrow_mut().push("event");
        let call = self.event_calls.get();
        self.event_calls.set(call + 1);
        let mut event = fixture_reward_event();
        if self.unstable_component == Some("event") && call > 0 {
            event.distributed_e8s_equivalent += 1;
        }
        Ok(event)
    }

    fn fetch_sns_reward_neuron_page(
        &self,
        _request: &SnsSourceRequest,
        _sns: &MainnetSns,
        limit: u32,
        _start_page_at: Option<&SnsNeuronId>,
    ) -> Result<MainnetSnsRewardNeuronPage, SnsHostError> {
        assert_eq!(limit, SNS_REWARD_CHECKPOINT_PAGE_SIZE);
        self.calls.borrow_mut().push("page");
        self.pages
            .borrow_mut()
            .pop_front()
            .ok_or_else(|| SnsHostError::InvalidSourceData {
                capability: "fixture SNS reward checkpoint",
                reason: "unexpected page request".to_string(),
            })
    }
}

pub(in crate::sns::report::tests) const fn fixture_reward_page(
    rows: Vec<SnsRewardCheckpointRow>,
    next_cursor: Option<SnsNeuronId>,
) -> MainnetSnsRewardNeuronPage {
    MainnetSnsRewardNeuronPage {
        neurons: rows,
        next_cursor,
    }
}

pub(in crate::sns::report::tests) fn fixture_reward_row(seed: u8) -> SnsRewardCheckpointRow {
    let mut row = SnsRewardCheckpointRow {
        neuron_id: format!("{seed:02x}").repeat(32),
        created_timestamp_seconds: 1_700_000_000 + u64::from(seed),
        maturity_e8s_equivalent: u64::from(seed) * 100,
        staked_maturity_e8s_equivalent: Some(u64::from(seed) * 10),
        combined_maturity_e8s_equivalent: u64::from(seed) * 110,
        auto_stake_maturity: Some(seed.is_multiple_of(2)),
        permissions: vec![SnsNeuronPermissionRow {
            principal: Some(ROOT_A.to_string()),
            permission_types: vec![
                SnsNeuronPermissionValue::from_code(2),
                SnsNeuronPermissionValue::from_code(4),
            ],
        }],
        disburse_maturity_in_progress: Vec::new(),
        maturity_mint_conversion_observed_disabled: SnsPolicyObservationStatus::ObservedSatisfied,
        manual_maturity_staking_observed_disabled: SnsPolicyObservationStatus::ObservedSatisfied,
    };
    let (mint, staking) = row.derived_policy_observations();
    row.maturity_mint_conversion_observed_disabled = mint;
    row.manual_maturity_staking_observed_disabled = staking;
    row
}

fn fixture_reward_event() -> SnsRewardEvent {
    SnsRewardEvent {
        rounds_since_last_distribution: Some(1),
        actual_timestamp_seconds: 1_780_531_100,
        end_timestamp_seconds: Some(1_780_531_200),
        total_available_e8s_equivalent: Some(1_000),
        distributed_e8s_equivalent: 500,
        round: 42,
        settled_proposals: vec![SnsRewardProposalId { id: 7 }],
    }
}

fn fixture_reward_running_version() -> SnsRunningVersionResponse {
    SnsRunningVersionResponse {
        deployed_version: Some(fixture_reward_version(1)),
        pending_version: Some(SnsPendingUpgrade {
            mark_failed_at_seconds: 1_780_617_600,
            checking_upgrade_lock: 9,
            proposal_id: 42,
            target_version: Some(fixture_reward_version(11)),
        }),
    }
}

fn fixture_reward_version(seed: u8) -> SnsVersion {
    SnsVersion {
        archive_wasm_hash_hex: format!("{seed:02x}").repeat(32),
        root_wasm_hash_hex: format!("{:02x}", seed + 1).repeat(32),
        swap_wasm_hash_hex: format!("{:02x}", seed + 2).repeat(32),
        ledger_wasm_hash_hex: format!("{:02x}", seed + 3).repeat(32),
        governance_wasm_hash_hex: format!("{:02x}", seed + 4).repeat(32),
        index_wasm_hash_hex: format!("{:02x}", seed + 5).repeat(32),
    }
}
