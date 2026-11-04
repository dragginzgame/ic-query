use crate::nns::NnsInventoryListRequest;

///
/// NnsTopologyRefreshRequest
///
/// Request accepted by the complete NNS topology refresh builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsTopologyRefreshRequest {
    /// Network, endpoint, clock and cache scope shared with topology reads.
    pub read: NnsInventoryListRequest,

    pub dry_run: bool,
}

impl NnsTopologyRefreshRequest {
    #[must_use]
    pub const fn new(read: NnsInventoryListRequest) -> Self {
        Self {
            read,

            dry_run: false,
        }
    }
}
