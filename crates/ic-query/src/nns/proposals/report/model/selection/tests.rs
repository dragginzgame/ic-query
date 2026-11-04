use super::*;

#[test]
fn proposal_status_codes_and_labels_round_trip() {
    for (status, code, label) in [
        (NnsProposalStatus::Unspecified, 0, "unspecified"),
        (NnsProposalStatus::Open, 1, "open"),
        (NnsProposalStatus::Rejected, 2, "rejected"),
        (NnsProposalStatus::Adopted, 3, "adopted"),
        (NnsProposalStatus::Executed, 4, "executed"),
        (NnsProposalStatus::Failed, 5, "failed"),
    ] {
        assert_eq!(status.code(), code);
        assert_eq!(NnsProposalStatus::from_code(code), status);
        assert_json_label(status, label);
    }
    assert_eq!(
        NnsProposalStatus::from_code(99),
        NnsProposalStatus::Unspecified
    );
}

#[test]
fn proposal_reward_status_codes_and_labels_round_trip() {
    for (status, code, label) in [
        (NnsProposalRewardStatus::Unspecified, 0, "unspecified"),
        (NnsProposalRewardStatus::AcceptVotes, 1, "accept-votes"),
        (NnsProposalRewardStatus::ReadyToSettle, 2, "ready-to-settle"),
        (NnsProposalRewardStatus::Settled, 3, "settled"),
        (NnsProposalRewardStatus::Ineligible, 4, "ineligible"),
    ] {
        assert_eq!(status.code(), code);
        assert_eq!(NnsProposalRewardStatus::from_code(code), status);
        assert_json_label(status, label);
    }
    assert_eq!(
        NnsProposalRewardStatus::from_code(99),
        NnsProposalRewardStatus::Unspecified
    );
}

#[test]
fn proposal_topic_codes_and_labels_round_trip() {
    for (topic, code, label) in [
        (NnsProposalTopic::Unspecified, 0, "unspecified"),
        (NnsProposalTopic::NeuronManagement, 1, "neuron-management"),
        (NnsProposalTopic::ExchangeRate, 2, "exchange-rate"),
        (NnsProposalTopic::NetworkEconomics, 3, "network-economics"),
        (NnsProposalTopic::Governance, 4, "governance"),
        (NnsProposalTopic::NodeAdmin, 5, "node-admin"),
        (
            NnsProposalTopic::ParticipantManagement,
            6,
            "participant-management",
        ),
        (NnsProposalTopic::SubnetManagement, 7, "subnet-management"),
        (
            NnsProposalTopic::NetworkCanisterManagement,
            8,
            "network-canister-management",
        ),
        (NnsProposalTopic::Kyc, 9, "kyc"),
        (
            NnsProposalTopic::NodeProviderRewards,
            10,
            "node-provider-rewards",
        ),
        (
            NnsProposalTopic::IcOsVersionDeployment,
            12,
            "ic-os-version-deployment",
        ),
        (
            NnsProposalTopic::IcOsVersionElection,
            13,
            "ic-os-version-election",
        ),
        (
            NnsProposalTopic::SnsAndCommunityFund,
            14,
            "sns-and-community-fund",
        ),
        (
            NnsProposalTopic::ApiBoundaryNodeManagement,
            15,
            "api-boundary-node-management",
        ),
        (NnsProposalTopic::SubnetRental, 16, "subnet-rental"),
        (
            NnsProposalTopic::ApplicationCanisterManagement,
            17,
            "application-canister-management",
        ),
        (
            NnsProposalTopic::ProtocolCanisterManagement,
            18,
            "protocol-canister-management",
        ),
    ] {
        assert_eq!(topic.code(), code);
        assert_eq!(NnsProposalTopic::from_code(code), topic);
        assert_json_label(topic, label);
    }
    assert_eq!(
        NnsProposalTopic::from_code(11),
        NnsProposalTopic::Unspecified
    );
    assert_eq!(
        NnsProposalTopic::from_code(99),
        NnsProposalTopic::Unspecified
    );
}

fn assert_json_label<T>(value: T, label: &str)
where
    T: Copy + std::fmt::Debug + Eq + Serialize + serde::de::DeserializeOwned,
{
    assert_eq!(
        serde_json::to_string(&value).unwrap(),
        format!("\"{label}\"")
    );
    assert_eq!(
        serde_json::from_str::<T>(&format!("\"{label}\"")).unwrap(),
        value
    );
}
