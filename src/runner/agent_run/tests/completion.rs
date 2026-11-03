use super::*;

#[tokio::test(start_paused = true)]
async fn an_agent_that_finishes_a_second_early_is_kept() {
    let mut fixture = Fixture::new(JobOrigin::Manual).await;
    fixture.deps.agent = FakeAgent::FinishAfter(Duration::from_secs(59));
    let (_tx, cancel) = watch::channel(false);
    let started = Instant::now();
    assert!(matches!(
        fixture.run(&cancel).await,
        ReviewResult::Succeeded { .. }
    ));
    assert_eq!(started.elapsed(), Duration::from_secs(59));
    assert_eq!(
        fixture.store.stop_reason(fixture.job.job_id).await.unwrap(),
        None
    );
    let drafts = fixture
        .store
        .call(|conn| {
            conn.query_row("SELECT COUNT(*) FROM drafts", [], |row| {
                row.get::<_, i64>(0)
            })
        })
        .await
        .unwrap();
    assert_eq!(drafts, 1);
}

#[tokio::test(start_paused = true)]
async fn default_timeout_is_fifteen_minutes() {
    let mut fixture = Fixture::new(JobOrigin::Manual).await;
    fixture.deps.settings = crate::config::Settings::default();
    let (_tx, cancel) = watch::channel(false);
    let started = Instant::now();
    assert!(matches!(
        fixture.run(&cancel).await,
        ReviewResult::RunFailed {
            kind: FailureKind::Timeout,
            ..
        }
    ));
    assert_eq!(started.elapsed(), Duration::from_mins(15));
}
