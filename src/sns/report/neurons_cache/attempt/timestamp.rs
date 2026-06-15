use crate::subnet_catalog::format_utc_timestamp_secs;
use std::time::{SystemTime, UNIX_EPOCH};

pub(in crate::sns::report::neurons_cache::attempt) fn current_timestamp_text(
    fallback: &str,
) -> String {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or_else(
        |_| fallback.to_string(),
        |duration| format_utc_timestamp_secs(duration.as_secs()),
    )
}
