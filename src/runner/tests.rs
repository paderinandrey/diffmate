use super::{Deps, ReviewResult, execute_review_job};
use crate::agent::claude::ClaudeCli;
use crate::app::ProcessInfo;
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
    let env = InheritedEnv::from_pairs(vec![("PATH".into(), std::env::var_os("PATH").unwrap())]);
    let deps = Deps {
        git: GitCli::new(&env),
        gh: GhCli::new(&env),
        agent: ClaudeCli::new(&env).unwrap(),
    };
    let (_stop, cancel) = watch::channel(true);

    let result = execute_review_job(&paths, &store, &deps, &pr, &job, &cancel).await;

    assert!(matches!(result, ReviewResult::Cancelled { run_id: None }));
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
    assert_eq!(row, ("failed".to_string(), reason.to_string(), 0));
}
