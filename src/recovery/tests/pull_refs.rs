use super::{fixture::Fixture, recover};
use crate::config::Settings;
use crate::pr_url::PrRef;
use crate::store::Store;
use std::sync::atomic::Ordering;

#[tokio::test]
async fn failed_pull_ref_cleanup_survives_restart_and_never_touches_a_discarded_open_target() {
    let fixture = Fixture::new().await;
    let repo = fixture.paths.repo_dir("o", "r");
    std::fs::create_dir_all(&repo).unwrap();
    let open = PrRef {
        owner: "o".into(),
        repo: "r".into(),
        number: 10,
    };
    fixture.store.call(|conn| conn.execute(
        "INSERT INTO targets (owner,repo,number,tracking,created_at) VALUES ('o','r',10,'tracked',0)", [],
    )).await.unwrap();
    fixture.store.discard(&open).await.unwrap();
    fixture
        .store
        .observe_review_head(fixture.job.target_id, "closed-head", true)
        .await
        .unwrap();
    fixture.git.fail_ref_cleanup.store(true, Ordering::Relaxed);
    let failed = fixture.recover().await;
    assert!(
        failed
            .warnings
            .iter()
            .any(|w| w.contains("o/r#1") && w.contains("locked"))
    );
    let store = Store::open(&fixture.paths).await.unwrap();
    fixture.git.fail_ref_cleanup.store(false, Ordering::Relaxed);
    let retry = recover(&fixture.paths, &store, &fixture.git, &Settings::default())
        .await
        .unwrap();
    assert_eq!(retry.warnings, Vec::<String>::new());
    assert_eq!(store.pending_ref_cleanup().await.unwrap(), []);
    fixture.recover().await;
    let deletion = format!("delete {} pull/1", repo.display());
    let calls = fixture.git.calls.lock().unwrap();
    let deletions: Vec<_> = calls
        .iter()
        .filter(|call| call.starts_with("delete "))
        .collect();
    assert_eq!(deletions, [&deletion, &deletion]);
}

#[tokio::test]
async fn observing_a_reopened_pr_cancels_its_old_ref_cleanup() {
    let fixture = Fixture::new().await;
    fixture
        .store
        .observe_review_head(fixture.job.target_id, "closed-head", true)
        .await
        .unwrap();
    fixture
        .store
        .observe_review_head(fixture.job.target_id, "new", false)
        .await
        .unwrap();
    fixture.recover().await;
    assert!(fixture.git.calls.lock().unwrap().is_empty());
}
