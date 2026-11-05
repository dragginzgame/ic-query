use super::*;

#[test]
fn neuron_rows_require_canonical_ids() {
    let valid = neuron(&"01".repeat(32));
    assert!(validate_sns_neuron_rows(std::slice::from_ref(&valid)).is_ok());

    let mut uppercase = valid;
    uppercase.neuron_id = "0A".repeat(32);
    assert!(
        validate_sns_neuron_rows(&[uppercase])
            .expect_err("uppercase id rejected")
            .contains("canonical lowercase hexadecimal")
    );
}

#[test]
fn bounded_neuron_results_require_unique_ids_and_requested_limit() {
    let duplicate = vec![neuron(&"01".repeat(32)), neuron(&"01".repeat(32))];
    assert!(matches!(
        validate_mainnet_sns_neurons(&duplicate, 2),
        Err(SnsHostError::InvalidSourceData {
            capability: "SNS neurons",
            ..
        })
    ));

    let over_limit = vec![neuron(&"01".repeat(32)), neuron(&"02".repeat(32))];
    let error = validate_mainnet_sns_neurons(&over_limit, 1).expect_err("limit enforced");
    assert!(matches!(
        error,
        SnsHostError::InvalidSourceData {
            capability: "SNS neurons",
            reason,
        } if reason == "returned 2 rows for requested limit 1"
    ));
    let mut unordered = over_limit.clone();
    unordered.reverse();
    validate_mainnet_sns_neurons(&unordered, 2)
        .expect("bounded results need uniqueness without requiring page ordering");

    let page = MainnetSnsNeuronPage {
        neurons: over_limit,
        last_cursor: None,
    };
    assert!(matches!(
        validate_mainnet_sns_neuron_page(&page, 1, None),
        Err(SnsHostError::InvalidSourceData {
            capability: "SNS neuron page",
            ..
        })
    ));
}

#[test]
fn neuron_pages_bind_cursors_and_preserve_inclusive_boundaries() {
    let cursor = SnsNeuronId { id: vec![2; 32] };
    let mut page = MainnetSnsNeuronPage {
        neurons: vec![neuron(&"02".repeat(32)), neuron(&"03".repeat(32))],
        last_cursor: Some(SnsNeuronId { id: vec![3; 32] }),
    };
    validate_mainnet_sns_neuron_page(&page, 2, Some(&cursor)).unwrap();
    page.last_cursor = Some(SnsNeuronId { id: vec![4; 32] });
    assert!(validate_mainnet_sns_neuron_page(&page, 2, Some(&cursor)).is_err());
    page.last_cursor = None;
    assert!(validate_mainnet_sns_neuron_page(&page, 2, Some(&cursor)).is_err());
    page.last_cursor = Some(SnsNeuronId { id: vec![3; 32] });
    page.neurons[0] = neuron(&"01".repeat(32));
    assert!(validate_mainnet_sns_neuron_page(&page, 2, Some(&cursor)).is_err());
    page.neurons.reverse();
    assert!(validate_mainnet_sns_neuron_page(&page, 2, None).is_err());
    let mut terminal = MainnetSnsNeuronPage {
        neurons: vec![neuron(&"02".repeat(32))],
        last_cursor: Some(cursor.clone()),
    };
    validate_mainnet_sns_neuron_page(&terminal, 2, Some(&cursor)).unwrap();
    assert!(validate_mainnet_sns_neuron_page(&terminal, 1, Some(&cursor)).is_err());
    terminal.last_cursor = None;
    validate_mainnet_sns_neuron_page(&terminal, 2, Some(&cursor)).unwrap();
    assert!(validate_mainnet_sns_neuron_page(&terminal, 1, None).is_err());
    let empty = MainnetSnsNeuronPage {
        neurons: vec![],
        last_cursor: None,
    };
    validate_mainnet_sns_neuron_page(&empty, 2, Some(&cursor)).unwrap();
    for ids in [vec![2, 2], vec![2, 3, 2]] {
        let last_id = *ids.last().unwrap();
        let page = MainnetSnsNeuronPage {
            neurons: ids
                .iter()
                .map(|id| neuron(&crate::hex::hex_bytes(&[*id; 32])))
                .collect(),
            last_cursor: Some(SnsNeuronId {
                id: vec![last_id; 32],
            }),
        };
        assert!(matches!(
            validate_mainnet_sns_neuron_page(&page, u32::try_from(ids.len()).unwrap(), None),
            Err(SnsHostError::InvalidSourceData {
                capability: "SNS neuron page",
                ..
            })
        ));
    }
}

fn neuron(neuron_id: &str) -> SnsNeuronRow {
    SnsNeuronRow {
        permissions: Vec::new(),
        neuron_id: neuron_id.to_string(),
        cached_neuron_stake_e8s: 100,
        maturity_e8s_equivalent: 10,
        staked_maturity_e8s_equivalent: None,
        created_timestamp_seconds: 1_700_000_000,
        source_nns_neuron_id: None,
        auto_stake_maturity: None,
        aging_since_timestamp_seconds: 1_700_000_000,
        dissolve_state: None,
        voting_power_percentage_multiplier: 100,
        vesting_period_seconds: None,
        neuron_fees_e8s: 0,
    }
}
