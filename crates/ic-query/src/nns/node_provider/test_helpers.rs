use super::spec::NODE_PROVIDER_SPEC;
use crate::nns::{NnsCommandError, leaf, node_provider::report::DEFAULT_NNS_SOURCE_ENDPOINT};

impl_leaf_test_helpers!(
    node_provider_list_options,
    node_provider_info_options,
    node_provider_refresh_options,
    node_provider_usage,
    node_provider_list_usage,
    node_provider_info_usage,
    node_provider_refresh_usage,
    NODE_PROVIDER_SPEC,
    DEFAULT_NNS_SOURCE_ENDPOINT
);
