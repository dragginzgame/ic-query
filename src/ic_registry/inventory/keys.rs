const NODE_RECORD_KEY_PREFIX: &str = "node_record_";
const NODE_OPERATOR_RECORD_KEY_PREFIX: &str = "node_operator_record_";
const DATA_CENTER_RECORD_KEY_PREFIX: &str = "data_center_record_";

pub(super) fn node_record_key(node_principal: &str) -> String {
    format!("{NODE_RECORD_KEY_PREFIX}{node_principal}")
}

pub(super) fn node_operator_record_key(node_operator_principal: &str) -> String {
    format!("{NODE_OPERATOR_RECORD_KEY_PREFIX}{node_operator_principal}")
}

pub(super) fn data_center_record_key(data_center_id: &str) -> String {
    format!("{DATA_CENTER_RECORD_KEY_PREFIX}{data_center_id}")
}
