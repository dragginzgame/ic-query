//! Module: sns::report::text::neurons
//!
//! Responsibility: group SNS neuron text report renderers.
//! Does not own: neuron report construction, cache loading, or JSON output.
//! Boundary: renders neuron list, refresh, and cache status DTOs for humans.

mod cache_list;
mod cache_status;
mod list;
mod refresh;

pub use cache_list::sns_neurons_cache_list_report_text;
pub use cache_status::sns_neurons_cache_status_report_text;
pub use list::sns_neurons_report_text;
pub use refresh::sns_neurons_refresh_report_text;
