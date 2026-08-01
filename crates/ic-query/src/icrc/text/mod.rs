//! Module: icrc::text
//!
//! Responsibility: expose generic ICRC text renderers through one internal facade.
//! Does not own: live source reads, JSON output, or command parsing.
//! Boundary: keeps account, ledger-history, and ledger-evidence rendering separate.

mod account;
mod history;
mod ledger;

use crate::table::ColumnAlign;

pub use account::{
    icrc_account_transaction_cache_status_report_text, icrc_account_transaction_list_report_text,
    icrc_account_transaction_page_report_text, icrc_account_transaction_refresh_report_text,
    icrc_allowance_report_text, icrc_balance_report_text,
};
pub use history::{
    icrc_archives_report_text, icrc_block_types_report_text, icrc_transactions_report_text,
};
pub use ledger::{
    icrc_capabilities_report_text, icrc_index_report_text, icrc_tip_certificate_report_text,
    icrc_token_report_text,
};

const LEFT_2_ALIGNMENTS: [ColumnAlign; 2] = [ColumnAlign::Left, ColumnAlign::Left];

fn push_table_section<T>(lines: &mut Vec<String>, rows: &[T], render: impl FnOnce(&[T]) -> String) {
    if rows.is_empty() {
        return;
    }
    lines.push(String::new());
    lines.push(render(rows));
}
