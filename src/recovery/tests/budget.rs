use super::fixture::Fixture;
use super::*;
use crate::store::now_ms;

#[tokio::test]
async fn an_automatic_job_that_crashes_every_time_is_retried_within_the_budget() {
    let mut fixture = Fixture::new().await;
    for attempt in 1..=3 {
        let run = fixture.start().await;
        let before = now_ms();
        let report = recover(
            &fixture.paths,
            &fixture.store,
            &fixture.git,
            &crate::config::Settings::default(),
        )
        .await
        .unwrap();
        let after = now_ms();
        let row = fixture.row().await;
        assert!(report.lines().contains(&format!(
            "recovery: run {run} marked failed (interrupted: crash)"
        )));
        if attempt <= 2 {
            let delay = if attempt == 1 { 60 } else { 300 };
            assert_eq!((row.0.as_str(), row.1, row.2), ("queued", attempt, Some(2)));
            assert!((before + delay * 1000..=after + delay * 1000).contains(&row.3.unwrap()));
            assert!(report.lines().contains(&format!(
                "recovery: job 1 requeued: retry {attempt} of 2 after {delay}s"
            )));
        } else {
            assert_eq!((row.0.as_str(), row.1, row.2), ("failed", 2, Some(2)));
            assert_eq!(
                row.4.as_deref(),
                Some("interrupted: crash; no retries left (2 of 2 used)")
            );
        }
        drop(fixture.store);
        fixture.store = Store::open(&fixture.paths).await.unwrap();
    }
    let count: i64 = fixture.store.call(|conn| conn.query_row(
        "SELECT count(*) FROM runs WHERE state = 'failed' AND failure_kind = 'interrupted' AND interrupted_cause = 'crash'",
        [], |row| row.get(0),
    )).await.unwrap();
    assert_eq!(count, 3);
}
