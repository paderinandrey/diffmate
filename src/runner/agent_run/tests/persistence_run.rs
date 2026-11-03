use super::*;
use crate::cancel::StopReason;

#[tokio::test(start_paused = true)]
async fn a_run_keeps_the_signal_persisted_during_a_failed_timeout_write() {
    let fixture = Fixture::new(JobOrigin::Manual).await;
    fixture
        .store
        .call(|conn| {
            conn.execute_batch(
                "CREATE TRIGGER reject_stop BEFORE UPDATE OF stop_reason ON jobs
         WHEN NEW.stop_reason = 'timeout'
         BEGIN SELECT RAISE(ABORT, 'injected timeout write failure'); END;",
            )
        })
        .await
        .unwrap();
    let (stop, cancel) = watch::channel(false);
    let signal = async {
        tokio::time::sleep(Duration::from_secs(62)).await;
        assert!(!*cancel.borrow());
        fixture
            .store
            .set_stop_reason(fixture.job.job_id, StopReason::UserCancel)
            .await
            .unwrap();
        stop.send_replace(true);
    };
    let (result, ()) = tokio::join!(fixture.run(&cancel), signal);
    assert!(matches!(
        result,
        ReviewResult::Cancelled {
            job_end: JobEnd::Failed,
            ..
        }
    ));
    let cause = fixture
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
    assert_eq!(cause, ("interrupted".into(), "user_cancel".into()));
    assert!(!fixture.paths.run_dir(1).exists());
}

#[tokio::test(start_paused = true)]
async fn timing_out_during_preparation_removes_the_worktree() {
    let mut fixture = Fixture::new(JobOrigin::Manual).await;
    fixture.deps.git.preparation = Duration::from_secs(70);
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
