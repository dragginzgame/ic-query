use super::{
    model::{SnsNeuronsAttemptContext, SnsNeuronsAttemptProgress},
    read::read_sns_neurons_attempt,
    write::write_sns_neurons_attempt_status,
};
use crate::sns::report::SnsHostError;

pub(in crate::sns::report::neurons_cache) fn write_failed_sns_neurons_attempt(
    context: SnsNeuronsAttemptContext<'_>,
    err: &SnsHostError,
) -> Result<(), SnsHostError> {
    let latest = read_sns_neurons_attempt(context.path);
    let progress = SnsNeuronsAttemptProgress::new(
        latest.as_ref().map_or(0, |attempt| attempt.pages_fetched),
        latest.as_ref().map_or(0, |attempt| attempt.rows_fetched),
        latest.and_then(|attempt| attempt.last_cursor),
    );
    write_sns_neurons_attempt_status(context, "failed", progress, Some(err.to_string()))
}
