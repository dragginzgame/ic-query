use super::spec::DATA_CENTER_SPEC;
use crate::nns::{NnsCommandError, leaf};
use ic_query::nns::data_center::DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT;

impl_leaf_test_helpers!(
    data_center_list_options,
    data_center_info_options,
    data_center_refresh_options,
    data_center_usage,
    data_center_list_usage,
    data_center_info_usage,
    data_center_refresh_usage,
    DATA_CENTER_SPEC,
    DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT
);
