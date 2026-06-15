use super::super::*;
use super::{FixtureSnsListSource, GOVERNANCE_A};

pub(in crate::sns::report::tests) struct FixtureSnsParamsSource;

impl SnsListSource for FixtureSnsParamsSource {
    fn fetch_deployed_snses(
        &self,
        request: &SnsFetchRequest,
    ) -> Result<MainnetSnsList, SnsHostError> {
        FixtureSnsListSource.fetch_deployed_snses(request)
    }
}

impl SnsParamsSource for FixtureSnsParamsSource {
    fn fetch_sns_params(
        &self,
        _request: &SnsFetchRequest,
        sns: &MainnetSns,
    ) -> Result<SnsGovernanceParameters, SnsHostError> {
        assert_eq!(sns.governance_canister_id, GOVERNANCE_A);
        Ok(SnsGovernanceParameters {
            max_dissolve_delay_seconds: Some(252_460_800),
            max_dissolve_delay_bonus_percentage: Some(100),
            max_followees_per_function: Some(15),
            neuron_claimer_permissions: Some(SnsNeuronPermissionList {
                permissions: vec![1, 2, 3],
            }),
            neuron_minimum_stake_e8s: Some(100_000_000),
            max_neuron_age_for_age_bonus: Some(126_144_000),
            initial_voting_period_seconds: Some(345_600),
            neuron_minimum_dissolve_delay_to_vote_seconds: Some(2_592_000),
            reject_cost_e8s: Some(100_000_000),
            max_proposals_to_keep_per_action: Some(100),
            wait_for_quiet_deadline_increase_seconds: Some(86_400),
            max_number_of_neurons: Some(200_000),
            transaction_fee_e8s: Some(10_000),
            max_number_of_proposals_with_ballots: Some(700),
            max_age_bonus_percentage: Some(25),
            neuron_grantable_permissions: Some(SnsNeuronPermissionList {
                permissions: vec![4, 5],
            }),
            voting_rewards_parameters: Some(SnsVotingRewardsParameters {
                final_reward_rate_basis_points: Some(500),
                initial_reward_rate_basis_points: Some(1000),
                reward_rate_transition_duration_seconds: Some(189_216_000),
                round_duration_seconds: Some(86_400),
            }),
            maturity_modulation_disabled: Some(false),
            max_number_of_principals_per_neuron: Some(10),
            automatically_advance_target_version: Some(true),
            custom_proposal_criticality: Some(SnsCustomProposalCriticality {
                additional_critical_native_action_ids: vec![1, 2],
            }),
        })
    }
}
