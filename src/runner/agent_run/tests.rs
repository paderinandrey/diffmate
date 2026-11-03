use super::fakes::{FakeAgent, FakeGh, FakeGit};
use crate::agent::FailureKind;
use crate::app::ProcessInfo;
use crate::config;
use crate::paths::Paths;
use crate::pr_url::PrRef;
use crate::runner::{Deps, JobEnd, ReviewResult, execute_review_job};
use crate::store::Store;
use crate::store::jobs::EnqueueRequest;
use crate::store::records::JobOrigin;
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::watch;
use tokio::time::Instant;

#[tokio::test(start_paused = true)]
async fn a_run_over_the_timeout_is_failed_timeout() {
    let dir = TempDir::new().unwrap();
    let paths = Paths::from_home(dir.path());
    let store = Store::open(&paths).await.unwrap();
    let pr = PrRef {
        owner: "o".into(),
        repo: "r".into(),
        number: 1,
    };
    let process = ProcessInfo {
        pid: 1,
        started_at_ms: 0,
    };
    let request = EnqueueRequest {
        origin: JobOrigin::Manual,
        force: false,
    };
    let job = store
        .enqueue_review(&pr, &process, request)
        .await
        .unwrap()
        .job;
    let settings = config::parse("run_timeout_minutes = 1", Path::new("config.toml")).unwrap();
    let deps = Deps {
        git: FakeGit,
        gh: FakeGh,
        agent: FakeAgent,
        settings,
    };
    let (_tx, cancel) = watch::channel(false);
    let started = Instant::now();
    let work = execute_review_job(&paths, &store, &deps, &pr, &job, &cancel);
    let result = tokio::time::timeout(Duration::from_secs(61), work).await;
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
    store
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
    assert!(!paths.run_dir(1).exists());
    assert!(!paths.worktree(1).exists());
}
