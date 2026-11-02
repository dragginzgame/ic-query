//! Module: icrc
//!
//! Responsibility: expose generic ICRC text renderers through one internal facade.
//! Does not own: live source reads, JSON output, or command parsing.
//! Boundary: keeps account, ledger-history, and ledger-evidence rendering separate.

mod account;
mod history;
mod ledger;

use ic_host_text::escape::terminal_text;
use ic_host_text::table::Alignment;

pub use crate::icrc::account::icrc_account_transaction_cache_status_report_text;
pub use crate::icrc::account::icrc_account_transaction_list_report_text;
pub use crate::icrc::account::icrc_account_transaction_page_report_text;
pub use crate::icrc::account::icrc_account_transaction_refresh_report_text;
pub use crate::icrc::account::icrc_allowance_report_text;
pub use crate::icrc::account::icrc_balance_report_text;
pub use crate::icrc::history::icrc_archives_report_text;
pub use crate::icrc::history::icrc_block_types_report_text;
pub use crate::icrc::history::icrc_transactions_report_text;
pub use crate::icrc::ledger::icrc_capabilities_report_text;
pub use crate::icrc::ledger::icrc_index_report_text;
pub use crate::icrc::ledger::icrc_tip_certificate_report_text;
pub use crate::icrc::ledger::icrc_token_report_text;

const LEFT_2_ALIGNMENTS: [Alignment; 2] = [Alignment::Left, Alignment::Left];

fn push_table_section<T>(lines: &mut Vec<String>, rows: &[T], render: impl FnOnce(&[T]) -> String) {
    if rows.is_empty() {
        return;
    }
    lines.push(String::new());
    lines.push(render(rows));
}

fn acquisition_lines(provenance: &ic_query::icrc::IcrcReportProvenance) -> [String; 2] {
    [
        format!("fetched_at: {}", terminal_text(&provenance.fetched_at)),
        format!(
            "source_endpoint: {}",
            terminal_text(&provenance.source_endpoint)
        ),
    ]
}
