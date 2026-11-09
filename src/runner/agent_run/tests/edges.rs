use super::*;
use crate::cancel::StopReason;

#[tokio::test(start_paused = true)]
async fn a_signal_first_keeps_its_reason() {
    let fixture = Fixture::new(JobOrigin::Manual).await;
    let (stop, cancel) = watch::channel(false);
    let signal = async {
        tokio::time::sleep(Duration::from_secs(30)).await;
        fixture
            .store
            .set_stop_reason(fixture.job.job_id, StopReason::UserCancel)
            .await
            .unwrap();
        stop.send_replace(true);
    };
    let started = Instant::now();
    let (result, ()) = tokio::join!(fixture.run(&cancel), signal);
    assert!(matches!(
        result,
        ReviewResult::Cancelled {
            job_end: JobEnd::Failed,
            ..
        }
    ));
    assert_eq!(started.elapsed(), Duration::from_secs(30));
    tokio::time::advance(Duration::from_secs(60)).await;
    assert_eq!(
        fixture.store.stop_reason(fixture.job.job_id).await.unwrap(),
        Some(StopReason::UserCancel)
    );
    let row = fixture
        .store
        .call(|conn| {
            conn.query_row(
                "SELECT failure_kind, interrupted_cause FROM runs",
                [],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
        })
        .await
        .unwrap();
    assert_eq!(row, ("interrupted".into(), "user_cancel".into()));
}

#[tokio::test(start_paused = true)]
async fn an_automatic_job_that_times_out_is_requeued() {
    let fixture = Fixture::new(JobOrigin::Automatic).await;
    let (_tx, cancel) = watch::channel(false);
    let before = crate::store::now_ms();
    let result = fixture.run(&cancel).await;
    let after = crate::store::now_ms();
    assert!(matches!(
        result,
        ReviewResult::RunFailed {
            kind: FailureKind::Timeout,
            job_end: JobEnd::Requeued {
                retry: Some((1, 2)),
                next_attempt_at: Some(_)
            },
            ..
        }
    ));
    fixture
        .store
        .call(move |conn| {
            let actual: (String, u32, u32, i64, i64) = conn.query_row(
                "SELECT j.state, j.retry_count, j.max_retries, j.next_attempt_at, r.finished_at
             FROM jobs j JOIN runs r ON r.job_id = j.id",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )?;
            assert_eq!((&*actual.0, actual.1, actual.2), ("queued", 1, 2));
            assert!((before + 60_000..=after + 60_000).contains(&actual.3));
            assert!(actual.3 > actual.4);
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test(start_paused = true)]
async fn preparation_and_agent_share_one_timeout() {
    let mut fixture = Fixture::new(JobOrigin::Manual).await;
    fixture.deps.git.preparation = Duration::from_secs(40);
    fixture.deps.agent = FakeAgent::FinishAfter(Duration::from_secs(30));
    let (_tx, cancel) = watch::channel(false);
    let started = Instant::now();
    assert!(matches!(
        fixture.run(&cancel).await,
        ReviewResult::RunFailed {
            kind: FailureKind::Timeout,
            ..
        }
    ));
    assert_eq!(started.elapsed(), Duration::from_secs(60));
    assert!(!fixture.paths.run_dir(1).exists());
}
