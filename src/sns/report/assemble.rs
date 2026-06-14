use super::{
    MainnetSns, MainnetSnsList, MainnetSnsNeurons, MainnetSnsProposal, MainnetSnsProposals,
    MainnetSnsToken, SNS_INFO_REPORT_SCHEMA_VERSION, SNS_LIST_REPORT_SCHEMA_VERSION,
    SNS_NEURONS_REPORT_SCHEMA_VERSION, SNS_PARAMS_REPORT_SCHEMA_VERSION,
    SNS_PROPOSAL_REPORT_SCHEMA_VERSION, SNS_PROPOSALS_REPORT_SCHEMA_VERSION,
    SNS_TOKEN_REPORT_SCHEMA_VERSION, SnsGovernanceParameters, SnsInfoReport, SnsListReport,
    SnsListRow, SnsListSort, SnsNeuronsReport, SnsNeuronsSort, SnsParamsReport, SnsProposalReport,
    SnsProposalStatusFilter, SnsProposalsReport, SnsTokenReport,
};

pub(super) struct SnsNeuronsLiveReportParts {
    pub(super) list: MainnetSnsList,
    pub(super) id: usize,
    pub(super) sns: MainnetSns,
    pub(super) requested_limit: u32,
    pub(super) owner_principal_id: Option<String>,
    pub(super) sort: SnsNeuronsSort,
    pub(super) verbose: bool,
    pub(super) neurons: MainnetSnsNeurons,
}

pub(super) struct SnsProposalReportParts {
    pub(super) list: MainnetSnsList,
    pub(super) id: usize,
    pub(super) sns: MainnetSns,
    pub(super) proposal_id: u64,
    pub(super) verbose: bool,
    pub(super) proposal: MainnetSnsProposal,
}

pub(super) struct SnsProposalsReportParts {
    pub(super) list: MainnetSnsList,
    pub(super) id: usize,
    pub(super) sns: MainnetSns,
    pub(super) requested_limit: u32,
    pub(super) before_proposal_id: Option<u64>,
    pub(super) status: SnsProposalStatusFilter,
    pub(super) verbose: bool,
    pub(super) proposals: MainnetSnsProposals,
}

pub(super) fn sns_list_report_from_list(
    list: MainnetSnsList,
    verbose: bool,
    sort: SnsListSort,
) -> SnsListReport {
    let MainnetSnsList {
        network,
        sns_wasm_canister_id,
        fetched_at,
        fetched_by,
        source_endpoint,
        sns_instances,
    } = list;
    let metadata_error_count = sns_instances
        .iter()
        .filter(|sns| sns.metadata_error.is_some())
        .count();
    let sns_instances = sns_instances
        .into_iter()
        .map(|sns| SnsListRow {
            id: sns.id,
            name: sns.name,
            root_canister_id: sns.root_canister_id,
            governance_canister_id: sns.governance_canister_id,
            ledger_canister_id: sns.ledger_canister_id,
            swap_canister_id: sns.swap_canister_id,
            index_canister_id: sns.index_canister_id,
            metadata_error: sns.metadata_error,
        })
        .collect::<Vec<_>>();
    SnsListReport {
        schema_version: SNS_LIST_REPORT_SCHEMA_VERSION,
        network,
        sns_wasm_canister_id,
        fetched_at,
        source_endpoint,
        fetched_by,
        verbose,
        sort: sort.as_str().to_string(),
        sns_count: sns_instances.len(),
        metadata_error_count,
        sns_instances,
    }
}

pub(super) fn sns_info_report_from_list(
    list: MainnetSnsList,
    id: usize,
    sns: MainnetSns,
) -> SnsInfoReport {
    SnsInfoReport {
        schema_version: SNS_INFO_REPORT_SCHEMA_VERSION,
        network: list.network,
        sns_wasm_canister_id: list.sns_wasm_canister_id,
        fetched_at: list.fetched_at,
        source_endpoint: list.source_endpoint,
        fetched_by: list.fetched_by,
        id,
        name: sns.name,
        description: sns.description,
        url: sns.url,
        root_canister_id: sns.root_canister_id,
        governance_canister_id: sns.governance_canister_id,
        ledger_canister_id: sns.ledger_canister_id,
        swap_canister_id: sns.swap_canister_id,
        index_canister_id: sns.index_canister_id,
        metadata_error: sns.metadata_error,
    }
}

pub(super) fn sns_token_report_from_parts(
    list: MainnetSnsList,
    id: usize,
    sns: MainnetSns,
    token: MainnetSnsToken,
) -> SnsTokenReport {
    SnsTokenReport {
        schema_version: SNS_TOKEN_REPORT_SCHEMA_VERSION,
        network: list.network,
        sns_wasm_canister_id: list.sns_wasm_canister_id,
        fetched_at: list.fetched_at,
        source_endpoint: list.source_endpoint,
        fetched_by: list.fetched_by,
        id,
        name: sns.name,
        root_canister_id: sns.root_canister_id,
        ledger_canister_id: sns.ledger_canister_id,
        sns_index_canister_id: sns.index_canister_id,
        token_name: token.token_name,
        token_symbol: token.token_symbol,
        decimals: token.decimals,
        transfer_fee: token.transfer_fee,
        total_supply: token.total_supply,
        minting_account_owner: token.minting_account_owner,
        minting_account_subaccount_hex: token.minting_account_subaccount_hex,
        ledger_index_canister_id: token.ledger_index_canister_id,
        ledger_index_error: token.ledger_index_error,
        supported_standards: token.supported_standards,
        metadata: token.metadata,
    }
}

pub(super) fn sns_params_report_from_parts(
    list: MainnetSnsList,
    id: usize,
    sns: MainnetSns,
    parameters: SnsGovernanceParameters,
) -> SnsParamsReport {
    SnsParamsReport {
        schema_version: SNS_PARAMS_REPORT_SCHEMA_VERSION,
        network: list.network,
        sns_wasm_canister_id: list.sns_wasm_canister_id,
        fetched_at: list.fetched_at,
        source_endpoint: list.source_endpoint,
        fetched_by: list.fetched_by,
        id,
        name: sns.name,
        root_canister_id: sns.root_canister_id,
        governance_canister_id: sns.governance_canister_id,
        parameters,
    }
}

pub(super) fn sns_proposal_report_from_parts(parts: SnsProposalReportParts) -> SnsProposalReport {
    SnsProposalReport {
        schema_version: SNS_PROPOSAL_REPORT_SCHEMA_VERSION,
        network: parts.list.network,
        sns_wasm_canister_id: parts.list.sns_wasm_canister_id,
        fetched_at: parts.list.fetched_at,
        source_endpoint: parts.list.source_endpoint,
        fetched_by: parts.list.fetched_by,
        id: parts.id,
        name: parts.sns.name,
        root_canister_id: parts.sns.root_canister_id,
        governance_canister_id: parts.sns.governance_canister_id,
        proposal_id: parts.proposal_id,
        verbose: parts.verbose,
        proposal: parts.proposal.proposal,
    }
}

pub(super) fn sns_proposals_report_from_parts(
    parts: SnsProposalsReportParts,
) -> SnsProposalsReport {
    let proposal_count = parts.proposals.proposals.len();
    SnsProposalsReport {
        schema_version: SNS_PROPOSALS_REPORT_SCHEMA_VERSION,
        network: parts.list.network,
        sns_wasm_canister_id: parts.list.sns_wasm_canister_id,
        fetched_at: parts.list.fetched_at,
        source_endpoint: parts.list.source_endpoint,
        fetched_by: parts.list.fetched_by,
        id: parts.id,
        name: parts.sns.name,
        root_canister_id: parts.sns.root_canister_id,
        governance_canister_id: parts.sns.governance_canister_id,
        requested_limit: parts.requested_limit,
        before_proposal_id: parts.before_proposal_id,
        status_filter: parts.status.as_str().to_string(),
        verbose: parts.verbose,
        proposal_count,
        proposals: parts.proposals.proposals,
    }
}

pub(super) fn sns_neurons_report_from_parts(parts: SnsNeuronsLiveReportParts) -> SnsNeuronsReport {
    let neuron_count = parts.neurons.neurons.len();
    SnsNeuronsReport {
        schema_version: SNS_NEURONS_REPORT_SCHEMA_VERSION,
        network: parts.list.network,
        sns_wasm_canister_id: parts.list.sns_wasm_canister_id,
        fetched_at: parts.list.fetched_at,
        source_endpoint: parts.list.source_endpoint,
        fetched_by: parts.list.fetched_by,
        id: parts.id,
        name: parts.sns.name,
        root_canister_id: parts.sns.root_canister_id,
        governance_canister_id: parts.sns.governance_canister_id,
        requested_limit: parts.requested_limit,
        owner_principal_id: parts.owner_principal_id,
        verbose: parts.verbose,
        data_source: "live".to_string(),
        sort: parts.sort.as_str().to_string(),
        cache_path: None,
        cache_complete: None,
        total_neuron_count: neuron_count,
        neuron_count,
        neurons: parts.neurons.neurons,
    }
}
