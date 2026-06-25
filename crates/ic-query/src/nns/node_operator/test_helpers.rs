use super::spec::NODE_OPERATOR_SPEC;
use crate::nns::{
    NnsCommandError, leaf, node_operator::report::DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
};

impl_leaf_test_helpers!(
    node_operator_list_options,
    node_operator_info_options,
    node_operator_refresh_options,
    node_operator_usage,
    node_operator_list_usage,
    node_operator_info_usage,
    node_operator_refresh_usage,
    NODE_OPERATOR_SPEC,
    DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT
);
