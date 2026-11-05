use super::*;
use crate::cli::common::OutputFormat;
const GOV: &str = "rrkah-fqaaa-aaaaa-aaaaq-cai";
#[test]
fn neuron_requests_target_explicit_governance() {
    let matches = command()
        .try_get_matches_from([
            "sns", "neuron", "list", GOV, "--owner", "aaaaa-aa", "--json",
        ])
        .unwrap();
    let matches = matches.subcommand().unwrap().1.subcommand().unwrap().1;
    let (format, request) = options::neurons_request(matches, "ic", 0, || {
        panic!("live reads do not need a cache root")
    })
    .unwrap();
    assert_eq!(format, OutputFormat::Json);
    assert_eq!(request.target.governance_canister_id, GOV);
    assert!(
        matches!(request.collection,ic_query::sns::SnsNeuronCollection::Live{owner_principal_id:Some(owner)} if owner=="aaaaa-aa")
    );
}
#[test]
fn cached_neuron_views_keep_target_and_scope() {
    let matches = command()
        .try_get_matches_from([
            "sns", "neuron", "list", GOV, "--sort", "stake", "--limit", "1000",
        ])
        .unwrap();
    let matches = matches.subcommand().unwrap().1.subcommand().unwrap().1;
    let (_, request) = options::neurons_request(matches, "ic", 0, || Ok("/cache".into())).unwrap();
    assert_eq!(request.target.governance_canister_id, GOV);
    assert!(matches!(
        request.collection,
        ic_query::sns::SnsNeuronCollection::Cached {
            sort: ic_query::sns::SnsNeuronsCacheSort::Stake,
            ..
        }
    ));
    assert!(
        command()
            .try_get_matches_from(["sns", "neuron", "list", "1"])
            .is_err()
    );
}
