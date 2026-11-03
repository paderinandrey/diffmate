mod gh;
mod git;
mod reuse;
use super::*;
use crate::app::ProcessInfo;

#[tokio::test]
async fn automatic_retries_keep_the_stored_revision_when_the_pr_head_moves() {
    let tmp = tempfile::TempDir::new().unwrap();
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
    let job = store
        .enqueue_review(
            &pr,
            &process,
            crate::store::jobs::EnqueueRequest {
                origin: crate::store::records::JobOrigin::Manual,
                force: false,
            },
        )
        .await
        .unwrap()
        .job;
    store
        .call(|conn| {
            conn.execute("UPDATE jobs SET origin = 'automatic'", [])
                .map(drop)
        })
        .await
        .unwrap();
    let revision = Revision {
        head_sha: "old-head".into(),
        merge_base_sha: "old-base".into(),
        base_tip_sha: "old-tip".into(),
        base_ref: "old-main".into(),
    };
    store.insert_running_run(&job, &revision).await.unwrap();
    let deps = Deps {
        git: git::FakeGit::default(),
        gh: gh::FakeGh,
        agent: (),
        settings: crate::config::Settings::default(),
    };
    let (meta, actual) = resolve(&paths, &store, &deps, &pr, &job).await.unwrap();
    assert_eq!(meta.head_sha, "new-head");
    assert_eq!(actual.head_sha, revision.head_sha);
    assert_eq!(actual.merge_base_sha, revision.merge_base_sha);
    assert_eq!(actual.base_tip_sha, revision.base_tip_sha);
    assert_eq!(actual.base_ref, revision.base_ref);
    assert_eq!(*deps.git.0.lock().unwrap(), ["clone"]);
    store
        .call(|conn| {
            conn.execute("UPDATE jobs SET origin = 'manual'", [])
                .map(drop)
        })
        .await
        .unwrap();
    let (_, current) = resolve(&paths, &store, &deps, &pr, &job).await.unwrap();
    assert_eq!(current.head_sha, "new-head");
    assert_eq!(current.base_ref, "new-main");
    store
        .fail_job(job.job_id, "finished fixture".into())
        .await
        .unwrap();
    let fresh = store
        .enqueue_review(
            &pr,
            &process,
            crate::store::jobs::EnqueueRequest {
                origin: crate::store::records::JobOrigin::Manual,
                force: false,
            },
        )
        .await
        .unwrap()
        .job;
    store
        .call(|conn| {
            conn.execute("UPDATE jobs SET origin = 'automatic' WHERE id = 2", [])
                .map(drop)
        })
        .await
        .unwrap();
    let (_, current) = resolve(&paths, &store, &deps, &pr, &fresh).await.unwrap();
    assert_eq!(current.head_sha, "new-head");
    assert_eq!(
        *deps.git.0.lock().unwrap(),
        ["clone", "clone", "fetch", "clone", "fetch"]
    );
}
