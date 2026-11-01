use super::{Deps, ReviewResult, execute_review_job};
use crate::agent::claude::ClaudeCli;
use crate::app::ProcessInfo;
use crate::cancel::StopReason;
use crate::gh::GhCli;
use crate::git::GitCli;
use crate::paths::Paths;
use crate::pr_url::PrRef;
use crate::procenv::InheritedEnv;
use crate::store::Store;
use tempfile::TempDir;
use tokio::sync::watch;

#[tokio::test]
async fn a_cancel_before_the_run_exists_fails_the_job_and_creates_no_run() {
    cancelled_before_run(None).await;
}

#[tokio::test]
async fn shutdown_before_the_run_exists_keeps_its_cause_and_creates_no_run() {
    cancelled_before_run(Some(StopReason::Shutdown)).await;
}

#[tokio::test]
async fn user_cancel_before_the_run_exists_keeps_its_cause_and_creates_no_run() {
    cancelled_before_run(Some(StopReason::UserCancel)).await;
}

async fn cancelled_before_run(stop: Option<StopReason>) {
    let tmp = TempDir::new().unwrap();
    let paths = Paths::new(tmp.path().join("state"), tmp.path().join("cache"));
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
    let job = store.enqueue_review(&pr, &process).await.unwrap();
    if let Some(reason) = stop {
        assert!(store.set_stop_reason(job.job_id, reason).await.unwrap());
    }
    let env = InheritedEnv::from_pairs(vec![("PATH".into(), std::env::var_os("PATH").unwrap())]);
    let deps = Deps {
        git: GitCli::new(&env),
        gh: GhCli::new(&env),
        agent: ClaudeCli::new(&env).unwrap(),
    };
    let (_stop, cancel) = watch::channel(true);

    let result = execute_review_job(&paths, &store, &deps, &pr, &job, &cancel).await;

    assert!(matches!(result, ReviewResult::Cancelled { run_id: None }));
    assert!(!paths.runs_dir().exists());
    assert!(!paths.repos_dir().exists());
    let sql = "SELECT state, failure_reason, (SELECT count(*) FROM runs) FROM jobs";
    let row = store
        .call(move |conn| {
            conn.query_row(sql, [], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            })
        })
        .await
        .unwrap();
    let reason = "interrupted before the run started";
    let reason = stop.map_or_else(
        || reason.to_string(),
        |stop| format!("{reason} ({})", stop.as_str()),
    );
    assert_eq!(row, ("failed".to_string(), reason, 0));
}
