use super::super::*;
use super::{FixtureSnsListSource, GOVERNANCE_A};

pub(in crate::sns::report::tests) struct FixtureSnsNeuronsSource;

impl SnsListSource for FixtureSnsNeuronsSource {
    fn fetch_deployed_snses(
        &self,
        request: &SnsFetchRequest,
    ) -> Result<MainnetSnsList, SnsHostError> {
        FixtureSnsListSource.fetch_deployed_snses(request)
    }
}

impl SnsNeuronsSource for FixtureSnsNeuronsSource {
    fn fetch_sns_neurons(
        &self,
        _request: &SnsFetchRequest,
        sns: &MainnetSns,
        limit: u32,
        owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeurons, SnsHostError> {
        assert_eq!(sns.governance_canister_id, GOVERNANCE_A);
        assert_eq!(limit, 10);
        assert_eq!(owner_principal_id, Some(GOVERNANCE_A));
        Ok(MainnetSnsNeurons {
            neurons: vec![SnsNeuronRow {
                neuron_id: "0001020304".to_string(),
                cached_neuron_stake_e8s: 123,
                maturity_e8s_equivalent: 456,
                staked_maturity_e8s_equivalent: Some(789),
                created_timestamp_seconds: 1_780_272_000,
                created_at: "2026-06-01T00:00:00Z".to_string(),
            }],
        })
    }

    fn fetch_sns_neuron_page(
        &self,
        _request: &SnsFetchRequest,
        sns: &MainnetSns,
        limit: u32,
        start_page_at: Option<&SnsNeuronId>,
        owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeuronPage, SnsHostError> {
        assert_eq!(sns.governance_canister_id, GOVERNANCE_A);
        assert_eq!(limit, 10);
        assert!(start_page_at.is_none());
        assert_eq!(owner_principal_id, None);
        Ok(MainnetSnsNeuronPage {
            neurons: vec![SnsNeuronRow {
                neuron_id: "0001020304".to_string(),
                cached_neuron_stake_e8s: 123,
                maturity_e8s_equivalent: 456,
                staked_maturity_e8s_equivalent: Some(789),
                created_timestamp_seconds: 1_780_272_000,
                created_at: "2026-06-01T00:00:00Z".to_string(),
            }],
            last_cursor: Some(SnsNeuronId {
                id: vec![0, 1, 2, 3],
            }),
        })
    }
}

pub(in crate::sns::report::tests) struct PagedFixtureSnsNeuronsSource;

impl SnsListSource for PagedFixtureSnsNeuronsSource {
    fn fetch_deployed_snses(
        &self,
        request: &SnsFetchRequest,
    ) -> Result<MainnetSnsList, SnsHostError> {
        FixtureSnsListSource.fetch_deployed_snses(request)
    }
}

impl SnsNeuronsSource for PagedFixtureSnsNeuronsSource {
    fn fetch_sns_neurons(
        &self,
        _request: &SnsFetchRequest,
        _sns: &MainnetSns,
        _limit: u32,
        _owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeurons, SnsHostError> {
        unreachable!("paged fixture is only used by complete cache refresh tests")
    }

    fn fetch_sns_neuron_page(
        &self,
        _request: &SnsFetchRequest,
        sns: &MainnetSns,
        limit: u32,
        start_page_at: Option<&SnsNeuronId>,
        owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeuronPage, SnsHostError> {
        assert_eq!(sns.governance_canister_id, GOVERNANCE_A);
        assert_eq!(limit, 2);
        assert_eq!(owner_principal_id, None);
        let cursor = start_page_at.map(|cursor| cursor.id.as_slice());
        let (neurons, last_cursor) = match cursor {
            None => (
                vec![neuron_row("01", 10), neuron_row("02", 30)],
                Some(vec![2]),
            ),
            Some([2]) => (
                vec![neuron_row("02", 30), neuron_row("03", 50)],
                Some(vec![3]),
            ),
            Some([3]) => (vec![neuron_row("03", 50)], Some(vec![3])),
            Some(other) => panic!("unexpected cursor {other:?}"),
        };
        Ok(MainnetSnsNeuronPage {
            neurons,
            last_cursor: last_cursor.map(|id| SnsNeuronId { id }),
        })
    }
}

pub(in crate::sns::report::tests) struct NoLiveSnsNeuronsSource;

impl SnsListSource for NoLiveSnsNeuronsSource {
    fn fetch_deployed_snses(
        &self,
        _request: &SnsFetchRequest,
    ) -> Result<MainnetSnsList, SnsHostError> {
        unreachable!("cache-backed neuron report should not fetch deployed SNS list")
    }
}

impl SnsNeuronsSource for NoLiveSnsNeuronsSource {
    fn fetch_sns_neurons(
        &self,
        _request: &SnsFetchRequest,
        _sns: &MainnetSns,
        _limit: u32,
        _owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeurons, SnsHostError> {
        unreachable!("cache-backed neuron report should not fetch live neurons")
    }

    fn fetch_sns_neuron_page(
        &self,
        _request: &SnsFetchRequest,
        _sns: &MainnetSns,
        _limit: u32,
        _start_page_at: Option<&SnsNeuronId>,
        _owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeuronPage, SnsHostError> {
        unreachable!("cache-backed neuron report should not fetch neuron pages")
    }
}

fn neuron_row(neuron_id: &str, stake: u64) -> SnsNeuronRow {
    SnsNeuronRow {
        neuron_id: neuron_id.to_string(),
        cached_neuron_stake_e8s: stake,
        maturity_e8s_equivalent: stake / 2,
        staked_maturity_e8s_equivalent: None,
        created_timestamp_seconds: 1_780_272_000 + stake,
        created_at: format_utc_timestamp_secs(1_780_272_000 + stake),
    }
}
