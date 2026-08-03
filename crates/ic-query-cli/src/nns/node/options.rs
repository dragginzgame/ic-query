use super::commands::{
    DATA_CENTER_FILTER_ARG, NODE_OPERATOR_FILTER_ARG, NODE_PROVIDER_FILTER_ARG, SUBNET_FILTER_ARG,
    SUBNET_KIND_FILTER_ARG,
};
use crate::{
    cli::clap::typed_option,
    nns::{OutputFormat, leaf::NnsCommonOptions},
};
use clap::ArgMatches;
use ic_query::nns::node::NnsNodeListFilters;
use ic_query::subnet_catalog::SubnetKind;

///
/// NnsNodeListOptions
///
/// Parsed options accepted by `icq nns node list`.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsNodeListOptions {
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
    pub(in crate::nns) source_endpoint: String,
    pub(in crate::nns) verbose: bool,
    pub(in crate::nns) filters: NnsNodeListFilters,
}

pub(in crate::nns) fn node_list_options_from_matches(
    matches: &ArgMatches,
    network: &str,
) -> NnsNodeListOptions {
    let common = NnsCommonOptions::from_matches(matches, network);
    NnsNodeListOptions {
        network: common.network,
        format: common.format,
        source_endpoint: common.source_endpoint,
        verbose: matches.get_flag("verbose"),
        filters: NnsNodeListFilters {
            subnet: typed_option(matches, SUBNET_FILTER_ARG),
            subnet_kind: subnet_kind_option(matches),
            data_center: typed_option(matches, DATA_CENTER_FILTER_ARG),
            node_provider: typed_option(matches, NODE_PROVIDER_FILTER_ARG),
            node_operator: typed_option(matches, NODE_OPERATOR_FILTER_ARG),
        },
    }
}

fn subnet_kind_option(matches: &ArgMatches) -> Option<SubnetKind> {
    typed_option::<String>(matches, SUBNET_KIND_FILTER_ARG).map(|value| {
        value
            .parse()
            .expect("Clap restricts the Subnet kind to supported values")
    })
}
