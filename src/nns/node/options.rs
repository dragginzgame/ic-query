use super::commands::{
    DATA_CENTER_FILTER_ARG, NODE_OPERATOR_FILTER_ARG, NODE_PROVIDER_FILTER_ARG, NODE_SPEC,
    SUBNET_FILTER_ARG, SUBNET_KIND_FILTER_ARG, node_list_command, node_list_usage,
};
use crate::{
    cli::clap::typed_option,
    nns::{
        NnsCommandError, OutputFormat,
        leaf::{NnsCommonOptions, NnsLeafInfoOptions, NnsLeafRefreshOptions},
        node::report::{DEFAULT_NNS_NODE_SOURCE_ENDPOINT, NnsNodeListFilters},
        parse_nns_matches,
    },
};
use std::ffi::OsString;

///
/// NnsNodeListOptions
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsNodeListOptions {
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
    pub(in crate::nns) source_endpoint: String,
    pub(in crate::nns) verbose: bool,
    pub(in crate::nns) filters: NnsNodeListFilters,
}

pub(in crate::nns) fn node_list_options<I>(args: I) -> Result<NnsNodeListOptions, NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let matches = parse_nns_matches(node_list_command(), args, node_list_usage)?;
    let common = NnsCommonOptions::from_matches(&matches);
    Ok(NnsNodeListOptions {
        network: common.network,
        format: common.format,
        source_endpoint: common.source_endpoint,
        verbose: matches.get_flag("verbose"),
        filters: NnsNodeListFilters {
            subnet: typed_option(&matches, SUBNET_FILTER_ARG),
            subnet_kind: typed_option(&matches, SUBNET_KIND_FILTER_ARG),
            data_center: typed_option(&matches, DATA_CENTER_FILTER_ARG),
            node_provider: typed_option(&matches, NODE_PROVIDER_FILTER_ARG),
            node_operator: typed_option(&matches, NODE_OPERATOR_FILTER_ARG),
        },
    })
}

pub(in crate::nns) fn node_info_options<I>(args: I) -> Result<NnsLeafInfoOptions, NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    NnsLeafInfoOptions::parse(args, &NODE_SPEC, DEFAULT_NNS_NODE_SOURCE_ENDPOINT)
}

pub(in crate::nns) fn node_refresh_options<I>(
    args: I,
) -> Result<NnsLeafRefreshOptions, NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    NnsLeafRefreshOptions::parse(args, &NODE_SPEC, DEFAULT_NNS_NODE_SOURCE_ENDPOINT)
}
