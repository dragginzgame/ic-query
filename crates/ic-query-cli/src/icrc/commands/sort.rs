//! Typed CLI sort values for ICRC analytics and cached account history.

use crate::cli::value_enum_with_conversion;
use ic_query::{
    ic::{IcIcrcAccountSort, IcIcrcHolderSort},
    icrc::IcrcAccountTransactionSort,
};

value_enum_with_conversion! {
    ///
    /// IcrcAccountSortArg
    ///
    /// Official account-index sort values accepted by the CLI.
    ///

    pub(super) enum IcrcAccountSortArg => IcIcrcAccountSort {
        #[default]
        Id,
        #[value(name = "-id")]
        IdDescending,
        Balance,
        #[value(name = "-balance")]
        BalanceDescending,
        #[value(name = "total_transactions")]
        TotalTransactions,
        #[value(name = "-total_transactions")]
        TotalTransactionsDescending,
        #[value(name = "created_timestamp")]
        CreatedTimestamp,
        #[value(name = "-created_timestamp")]
        CreatedTimestampDescending,
        Owner,
        #[value(name = "-owner")]
        OwnerDescending,
    }
}

value_enum_with_conversion! {
    ///
    /// IcrcHolderSortArg
    ///
    /// Official holder-index sort values accepted by the CLI.
    ///

    pub(super) enum IcrcHolderSortArg => IcIcrcHolderSort {
        Balance,
        #[value(name = "-balance")]
        BalanceDescending,
        #[value(name = "total_transactions")]
        TotalTransactions,
        #[value(name = "-total_transactions")]
        TotalTransactionsDescending,
        #[value(name = "created_timestamp")]
        CreatedTimestamp,
        #[value(name = "-created_timestamp")]
        CreatedTimestampDescending,
        #[default]
        Principal,
        #[value(name = "-principal")]
        PrincipalDescending,
    }
}

value_enum_with_conversion! {
    ///
    /// IcrcAccountTransactionSortArg
    ///
    /// Ordering of rows from a complete cached account history.
    ///

    pub(super) enum IcrcAccountTransactionSortArg => IcrcAccountTransactionSort {
        #[default]
        Newest,
        Oldest,
    }
}
