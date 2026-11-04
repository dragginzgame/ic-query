use super::*;
use std::cell::Cell;

#[tokio::test]
async fn enrichment_bounds_concurrency_preserves_order_and_finishes_before_error() {
    let count = SNS_METADATA_CONCURRENCY * 2;
    for fail_first in [false, true] {
        let active = Cell::new(0);
        let peak = Cell::new(0);
        let completed = Cell::new(0);
        let result = collect_enrichment_rows((0..count).map(|index| {
            let active = &active;
            let peak = &peak;
            let completed = &completed;
            async move {
                active.set(active.get() + 1);
                peak.set(peak.get().max(active.get()));
                tokio::task::yield_now().await;
                active.set(active.get() - 1);
                completed.set(completed.get() + 1);
                if fail_first && index == 0 {
                    Err(SnsHostError::InvalidSourceData {
                        capability: "fixture",
                        reason: "first row failed".to_string(),
                    })
                } else {
                    Ok(index)
                }
            }
        }))
        .await;
        assert_eq!(completed.get(), count);
        assert_eq!(peak.get(), SNS_METADATA_CONCURRENCY);
        assert_eq!(active.get(), 0);
        if fail_first {
            assert!(matches!(
                result,
                Err(SnsHostError::InvalidSourceData {
                    capability: "fixture",
                    ..
                })
            ));
        } else {
            assert_eq!(result.unwrap(), (0..count).collect::<Vec<_>>());
        }
    }
}
