mod admission;
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
    let enqueue = |origin| {
        store.enqueue_review(
            &pr,
            &process,
            crate::store::jobs::EnqueueRequest {
                origin,
                force: false,
            },
        )
    };
    let job = enqueue(crate::store::records::JobOrigin::Automatic)
        .await
        .unwrap()
        .job;
    let revision = Revision {
        head_sha: "old-head".into(),
        merge_base_sha: "old-base".into(),
        base_tip_sha: "old-tip".into(),
        base_ref: "old-main".into(),
    };
    store.insert_running_run(&job, &revision).await.unwrap();
    let deps = Deps {
        git: git::FakeGit::default(),
        gh: gh::FakeGh::default(),
        agent: (),
        settings: crate::config::Settings::default(),
    };
    let automatic = store.start_job(job.job_id).await.unwrap().unwrap();
    let (meta, actual) = resolve(&paths, &store, &deps, &pr, &job, &automatic)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(meta.head_sha, "new-head");
    assert_eq!(actual.head_sha, revision.head_sha);
    assert_eq!(actual.merge_base_sha, revision.merge_base_sha);
    assert_eq!(actual.base_tip_sha, revision.base_tip_sha);
    assert_eq!(actual.base_ref, revision.base_ref);
    assert_eq!(*deps.git.0.lock().unwrap(), ["clone"]);
    store
        .call(|conn| {
            conn.execute("UPDATE jobs SET origin = 'manual', state = 'queued'", [])
                .map(drop)
        })
        .await
        .unwrap();
    let manual = store.start_job(job.job_id).await.unwrap().unwrap();
    let (_, current) = resolve(&paths, &store, &deps, &pr, &job, &manual)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(current.head_sha, "new-head");
    assert_eq!(current.base_ref, "new-main");
    store
        .fail_job(job.job_id, "finished fixture".into())
        .await
        .unwrap();
    let fresh = enqueue(crate::store::records::JobOrigin::Automatic)
        .await
        .unwrap()
        .job;
    let started = store.start_job(fresh.job_id).await.unwrap().unwrap();
    let (_, current) = resolve(&paths, &store, &deps, &pr, &fresh, &started)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(current.head_sha, "new-head");
    assert_eq!(
        *deps.git.0.lock().unwrap(),
        ["clone", "clone", "fetch", "clone", "fetch"]
    );
}
