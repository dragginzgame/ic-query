use super::commands::{
    DATA_CENTER_FILTER_ARG, NODE_OPERATOR_FILTER_ARG, NODE_PROVIDER_FILTER_ARG, SUBNET_FILTER_ARG,
    SUBNET_KIND_FILTER_ARG,
};
#[cfg(test)]
use super::commands::{NODE_SPEC, node_list_command};
#[cfg(test)]
use crate::nns::leaf::{NnsLeafInfoOptions, NnsLeafRefreshOptions};
use crate::{
    cli::clap::typed_option,
    nns::{OutputFormat, leaf::NnsCommonOptions},
};
use clap::ArgMatches;
#[cfg(test)]
use ic_query::nns::node::DEFAULT_NNS_NODE_SOURCE_ENDPOINT;
use ic_query::nns::node::NnsNodeListFilters;
#[cfg(test)]
use std::ffi::OsString;

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
            subnet_kind: typed_option(matches, SUBNET_KIND_FILTER_ARG),
            data_center: typed_option(matches, DATA_CENTER_FILTER_ARG),
            node_provider: typed_option(matches, NODE_PROVIDER_FILTER_ARG),
            node_operator: typed_option(matches, NODE_OPERATOR_FILTER_ARG),
        },
    }
}

#[cfg(test)]
pub(in crate::nns) fn node_list_options<I>(
    args: I,
) -> Result<NnsNodeListOptions, crate::nns::NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let matches = crate::nns::parse_nns_matches(node_list_command(), args)?;
    Ok(node_list_options_from_matches(
        &matches,
        ic_query::subnet_catalog::MAINNET_NETWORK,
    ))
}

#[cfg(test)]
pub(in crate::nns) fn node_info_options<I>(
    args: I,
) -> Result<NnsLeafInfoOptions, crate::nns::NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    NnsLeafInfoOptions::parse(args, &NODE_SPEC, DEFAULT_NNS_NODE_SOURCE_ENDPOINT)
}

#[cfg(test)]
pub(in crate::nns) fn node_refresh_options<I>(
    args: I,
) -> Result<NnsLeafRefreshOptions, crate::nns::NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    NnsLeafRefreshOptions::parse(args, &NODE_SPEC, DEFAULT_NNS_NODE_SOURCE_ENDPOINT)
}
