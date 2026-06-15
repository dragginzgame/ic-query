use super::super::source::{MainnetSns, MainnetSnsList, SnsFetchRequest};

pub(in crate::sns::report) struct SnsLookup {
    pub(in crate::sns::report) fetch_request: SnsFetchRequest,
    pub(in crate::sns::report) list: MainnetSnsList,
    pub(in crate::sns::report) id: usize,
    pub(in crate::sns::report) sns: MainnetSns,
}
