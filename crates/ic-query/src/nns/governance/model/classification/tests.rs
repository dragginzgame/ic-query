use super::NnsGovernanceVote;

#[test]
fn vote_codes_and_labels_round_trip() {
    for (code, vote, label) in [
        (0, NnsGovernanceVote::Unspecified, "unspecified"),
        (1, NnsGovernanceVote::Yes, "yes"),
        (2, NnsGovernanceVote::No, "no"),
        (99, NnsGovernanceVote::Unknown(99), "unknown(99)"),
        (
            i32::MIN,
            NnsGovernanceVote::Unknown(i32::MIN),
            "unknown(-2147483648)",
        ),
        (
            i32::MAX,
            NnsGovernanceVote::Unknown(i32::MAX),
            "unknown(2147483647)",
        ),
    ] {
        assert_eq!(NnsGovernanceVote::from_code(code), vote);
        assert_eq!(vote.code(), code);
        assert_eq!(vote.to_string(), label);
        let json = serde_json::to_string(&vote).unwrap();
        assert_eq!(
            serde_json::from_str::<NnsGovernanceVote>(&json).unwrap(),
            vote
        );
        assert_eq!(serde_json::to_value(vote).unwrap(), label);
    }
}

#[test]
fn vote_labels_reject_malformed_evidence() {
    for label in [
        "maybe",
        "",
        "YES",
        "unknown(+9)",
        "unknown(09)",
        "unknown(-0)",
        "unknown(2147483648)",
        "unknown(9) ",
    ] {
        let json = serde_json::to_string(label).unwrap();
        assert!(
            serde_json::from_str::<NnsGovernanceVote>(&json).is_err(),
            "{label}"
        );
    }
}
