use super::*;

#[test]
fn sns_neurons_rejects_invalid_clap_values() {
    assert!(matches!(
        SnsLookupOptions::parse([OsString::from("not-a-principal")], sns_info_command,),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        SnsLookupOptions::parse([OsString::from("0")], sns_token_command),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        SnsNeuronsOptions::parse([
            OsString::from("1"),
            OsString::from("--limit"),
            OsString::from("0"),
        ]),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        SnsNeuronsOptions::parse([
            OsString::from("1"),
            OsString::from("--limit"),
            OsString::from("101"),
        ]),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        SnsNeuronsOptions::parse([
            OsString::from("1"),
            OsString::from("--owner"),
            OsString::from("not-a-principal"),
        ]),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        SnsNeuronsOptions::parse([
            OsString::from("1"),
            OsString::from("--owner"),
            OsString::from("zqfso-syaaa-aaaaq-aaafq-cai"),
            OsString::from("--sort"),
            OsString::from("stake"),
        ]),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        SnsNeuronsRefreshOptions::parse([
            OsString::from("1"),
            OsString::from("--page-size"),
            OsString::from("0"),
        ]),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        SnsNeuronsCacheStatusOptions::parse([OsString::from("not-a-principal")]),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        SnsProposalsOptions::parse([
            OsString::from("1"),
            OsString::from("--limit"),
            OsString::from("101"),
        ]),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        SnsProposalsOptions::parse([
            OsString::from("1"),
            OsString::from("--status"),
            OsString::from("not-a-status"),
        ]),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        SnsProposalsOptions::parse([
            OsString::from("1"),
            OsString::from("--topic"),
            OsString::from("not-a-topic"),
        ]),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        SnsProposalsOptions::parse([OsString::from("1"), OsString::from("--asc"),]),
        Err(SnsCommandError::Usage(_))
    ));
    assert!(matches!(
        SnsProposalOptions::parse([OsString::from("1"), OsString::from("0")]),
        Err(SnsCommandError::Usage(_))
    ));
}

#[test]
fn sns_metrics_rejects_invalid_or_excessive_windows() {
    for value in ["0", "1.5h", "366d"] {
        assert!(matches!(
            SnsMetricsOptions::parse([
                OsString::from("1"),
                OsString::from("--window"),
                OsString::from(value),
            ]),
            Err(SnsCommandError::Usage(_))
        ));
    }
}
