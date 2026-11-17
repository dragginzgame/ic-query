use super::*;
#[test]
fn filtered_default_and_continuation_are_admitted_without_io() {
    let gov = "zqfso-syaaa-aaaaq-aaafq-cai";
    let filtered = sns_command()
        .try_get_matches_from(["sns", "neuron", "list", gov, "--principal", "2vxsx-fae"])
        .unwrap();
    assert_eq!(
        *filtered
            .subcommand()
            .unwrap()
            .1
            .subcommand()
            .unwrap()
            .1
            .get_one::<u8>("max-pages")
            .unwrap(),
        1
    );
    let cursor = "AB".repeat(32);
    let continuation = sns_command()
        .try_get_matches_from(["sns", "neuron", "list", gov, "--start-after", &cursor])
        .unwrap();
    assert_eq!(
        *continuation
            .subcommand()
            .unwrap()
            .1
            .subcommand()
            .unwrap()
            .1
            .get_one::<[u8; 32]>("start-after")
            .unwrap(),
        [0xab; 32]
    );
    assert!(
        sns_command()
            .try_get_matches_from([
                "sns",
                "neuron",
                "list",
                gov,
                "--start-after",
                &cursor,
                "--principal",
                "2vxsx-fae"
            ])
            .is_err()
    );
}
