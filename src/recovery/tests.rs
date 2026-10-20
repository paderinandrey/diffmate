use super::fake::FakeGit;
use super::run as recover;
use crate::app::ProcessInfo;
use crate::paths::Paths;
use crate::pr_url::PrRef;
use crate::store::Store;
use crate::store::records::Revision;
use std::fs;
use tempfile::TempDir;

fn revision() -> Revision {
    let sha = "a".repeat(40);
    Revision {
        head_sha: sha.clone(),
        merge_base_sha: sha.clone(),
        base_tip_sha: sha,
        base_ref: "main".into(),
    }
}

#[tokio::test]
async fn a_running_run_and_job_are_failed_and_their_leftovers_removed() {
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
    store.start_job(job.job_id).await.unwrap();
    let run = store.insert_running_run(&job, &revision()).await.unwrap();
    let clone = paths.repo_dir("o", "r");
    fs::create_dir_all(&clone).unwrap();
    fs::create_dir_all(paths.worktree(run)).unwrap();
    fs::create_dir_all(paths.run_dir(77)).unwrap();
    fs::create_dir_all(paths.runs_dir().join("notes")).unwrap();
    let git = FakeGit::default();

    let report = recover(&paths, &store, &git).await.unwrap();

    assert_eq!(report.interrupted_runs, [run]);
    assert_eq!(report.failed_jobs, [job.job_id]);
    assert_eq!(report.removed_run_dirs, [run, 77]);
    assert_eq!(report.warnings.len(), 1);
    assert!(report.warnings[0].contains("notes"));
    assert!(!paths.run_dir(run).exists() && !paths.run_dir(77).exists());
    assert!(paths.runs_dir().join("notes").exists());
    let worktree = paths.worktree(run);
    let expected = [
        format!("remove {} {}", clone.display(), worktree.display()),
        format!("prune {}", clone.display()),
    ];
    assert_eq!(*git.calls.lock().unwrap(), expected);
    let sql = "SELECT r.state, r.failure_kind, r.interrupted_cause, j.state, j.failure_reason
               FROM runs r JOIN jobs j ON j.id = r.job_id";
    let row = store
        .call(move |conn| {
            conn.query_row(sql, [], |r| (0..5).map(|i| r.get::<_, String>(i)).collect())
        })
        .await
        .unwrap();
    let row: Vec<String> = row;
    assert_eq!(
        row,
        [
            "failed",
            "interrupted",
            "crash",
            "failed",
            "interrupted: crash"
        ]
    );
}
