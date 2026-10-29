use super::commands::{
    DATA_CENTER_FILTER_ARG, NODE_OPERATOR_FILTER_ARG, NODE_PROVIDER_FILTER_ARG, SUBNET_FILTER_ARG,
    SUBNET_KIND_FILTER_ARG,
};
use crate::{cli::clap::typed_option, nns::leaf::NnsCommonOptions};
use clap::ArgMatches;
use ic_query::nns::node::NnsNodeListFilters;

///
/// NnsNodeListOptions
///
/// Parsed options accepted by `icq nns node list`.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsNodeListOptions {
    pub(in crate::nns) common: NnsCommonOptions,
    pub(in crate::nns) verbose: bool,
    pub(in crate::nns) filters: NnsNodeListFilters,
}

pub(in crate::nns) fn node_list_options_from_matches(
    matches: &ArgMatches,
    network: &str,
) -> NnsNodeListOptions {
    let common = NnsCommonOptions::from_matches(matches, network);
    NnsNodeListOptions {
        common,
        verbose: matches.get_flag("verbose"),
        filters: NnsNodeListFilters {
            subnet: typed_option(matches, SUBNET_FILTER_ARG),
            subnet_kind: typed_option(matches, SUBNET_KIND_FILTER_ARG),
            data_center: typed_option(matches, DATA_CENTER_FILTER_ARG),
            node_provider: typed_option(matches, NODE_PROVIDER_FILTER_ARG),
            node_operator: typed_option(matches, NODE_OPERATOR_FILTER_ARG),
        },
    }
}
