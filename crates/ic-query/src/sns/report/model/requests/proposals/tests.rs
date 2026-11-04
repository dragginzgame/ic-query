use super::*;

#[test]
fn decided_filter_requires_an_explicit_complete_cache_route() {
    let lookup = SnsLookupRequest::new("ic", "https://icp-api.io", 0, "aaaaa-aa").unwrap();
    let live = SnsProposalsRequest::new(lookup.clone(), 10);
    assert!(matches!(live.collection(), SnsProposalCollection::Live));
    assert_eq!(
        live.with_status(SnsProposalStatusFilter::Decided),
        Err(SnsProposalViewError)
    );
    let cached = SnsProposalsRequest::new(lookup, 10)
        .with_cache_root("cache")
        .with_status(SnsProposalStatusFilter::Decided)
        .unwrap();
    assert!(
        matches!(cached.collection(), SnsProposalCollection::Cached { cache_root } if cache_root == Path::new("cache"))
    );
    assert_eq!(cached.status(), SnsProposalStatusFilter::Decided);
}
