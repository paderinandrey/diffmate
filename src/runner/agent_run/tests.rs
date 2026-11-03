mod completion;
mod edges;
mod fixture;
mod persistence;
mod persistence_run;

use super::fakes::{FakeAgent, FakeGh, FakeGit};
use crate::agent::FailureKind;
use crate::runner::{JobEnd, ReviewResult};
use crate::store::records::JobOrigin;
use fixture::Fixture;
use std::time::Duration;
use tokio::sync::watch;
use tokio::time::Instant;

#[tokio::test(start_paused = true)]
async fn a_run_over_the_timeout_is_failed_timeout() {
    let fixture = Fixture::new(JobOrigin::Manual).await;
    let (_tx, cancel) = watch::channel(false);
    let started = Instant::now();
    let result = tokio::time::timeout(Duration::from_secs(61), fixture.run(&cancel)).await;
    assert!(result.is_ok(), "run must finish at the 60-second timeout");
    assert!(matches!(
        result.unwrap(),
        ReviewResult::RunFailed {
            kind: FailureKind::Timeout,
            job_end: JobEnd::Failed,
            ..
        }
    ));
    assert_eq!(started.elapsed(), Duration::from_secs(60));
    fixture
        .store
        .call(|conn| {
            let actual: (String, String, Option<String>, String, String) = conn.query_row(
                "SELECT r.state, r.failure_kind, r.interrupted_cause, j.stop_reason, j.state
             FROM runs r JOIN jobs j ON j.id = r.job_id",
                [],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )?;
            assert_eq!(
                actual,
                (
                    "failed".into(),
                    "timeout".into(),
                    None,
                    "timeout".into(),
                    "failed".into()
                )
            );
            Ok(())
        })
        .await
        .unwrap();
    assert!(!fixture.paths.run_dir(1).exists());
    assert!(!fixture.paths.worktree(1).exists());
}
