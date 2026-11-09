mod fixture;
mod held;
mod stop;
use fixture::Fixture;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]
    #[test]
    fn enqueue_preserves_active_job_and_origin_invariants(
        operations in prop::collection::vec((0u8..4, any::<bool>(), any::<bool>()), 1..80),
        tracked in any::<bool>(),
    ) {
        tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
            let fixture = Fixture::new(tracked).await;
            for (operation, force, automatic) in operations {
                let before = fixture.rows().await;
                if operation < 2 {
                    let queued = before.iter().find(|row| row.state == "queued");
                    let running = before.iter().find(|row| row.state == "running");
                    let matching = queued.or_else(|| running.filter(|row| !force || row.force));
                    let result = fixture.enqueue(force, automatic).await;
                    let after = fixture.rows().await;
                    assert!(after.iter().filter(|row| row.state == "queued").count() <= 1);
                    if let Some(previous) = matching {
                        assert!(result.attached);
                        assert_eq!(result.job.job_id, previous.id);
                        assert_eq!(after.len(), before.len());
                        let row = after.iter().find(|row| row.id == previous.id).unwrap();
                        assert_eq!(row.force, previous.force || force);
                    } else {
                        assert!(!result.attached);
                        assert_eq!(after.len(), before.len() + 1);
                        assert!(queued.is_none());
                        assert!(running.is_none_or(|row| force && !row.force));
                        let row = after.last().unwrap();
                        assert_eq!(row.force, force);
                        assert_eq!(row.origin, if automatic { "automatic" } else { "manual" });
                    }
                    for old in &before {
                        let row = after.iter().find(|row| row.id == old.id).unwrap();
                        assert_eq!(row.origin, old.origin);
                    }
                    assert_eq!(fixture.tracked().await, tracked);
                } else if operation == 2 {
                    if !before.iter().any(|row| row.state == "running")
                        && let Some(row) = before.iter().find(|row| row.state == "queued")
                    {
                        fixture.store.start_job(row.id).await.unwrap();
                    }
                } else if let Some(row) = before.iter().find(|row| row.state == "running") {
                    fixture.store.fail_job(row.id, "finished".into()).await.unwrap();
                }
            }
        });
    }
}
