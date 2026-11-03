use super::fixture::Fixture;
use crate::cancel::StopReason;

#[tokio::test]
async fn jobs_without_runs_follow_the_same_rules() {
    let fixture = Fixture::new().await;
    fixture.store.start_job(fixture.job.job_id).await.unwrap();
    fixture
        .store
        .set_stop_reason(fixture.job.job_id, StopReason::Shutdown)
        .await
        .unwrap();
    let report = fixture.recover().await;
    assert_eq!(report.runs, Vec::new());
    assert_eq!(report.jobs[0].1, "requeued (interrupted: shutdown)");
    let row = fixture.row().await;
    assert_eq!(
        (row.0.as_str(), row.1, row.2, row.3),
        ("queued", 0, None, None)
    );
    fixture
        .store
        .call(|conn| {
            conn.execute("UPDATE jobs SET origin = 'manual', state = 'running'", [])
                .map(drop)
        })
        .await
        .unwrap();
    assert_eq!(
        fixture.recover().await.jobs[0].1,
        "failed (interrupted: crash)"
    );
    let before = fixture.snapshot().await;
    assert_eq!(fixture.recover().await.lines(), Vec::<String>::new());
    assert_eq!(fixture.snapshot().await, before);
}

#[tokio::test]
async fn a_running_run_without_a_running_job_is_a_crash() {
    let fixture = Fixture::new().await;
    fixture.start().await;
    fixture
        .store
        .call(|conn| {
            conn.execute(
                "UPDATE jobs SET state = 'failed', stop_reason = 'shutdown'",
                [],
            )
            .map(drop)
        })
        .await
        .unwrap();
    let report = fixture.recover().await;
    assert_eq!(report.jobs, Vec::new());
    assert_eq!(report.runs[0].1, "interrupted: crash");
    let before = fixture.snapshot().await;
    assert_eq!(fixture.recover().await.lines(), Vec::<String>::new());
    assert_eq!(fixture.snapshot().await, before);
}
