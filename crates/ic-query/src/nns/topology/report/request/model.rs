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

    #[must_use]
    pub const fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }
}
