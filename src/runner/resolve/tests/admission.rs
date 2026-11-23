use super::{gh::FakeGh, git::FakeGit, reuse::UncalledAgent};
use crate::app::ProcessInfo;
use crate::paths::Paths;
use crate::pr_url::parse;
use crate::runner::{Deps, ReviewResult, execute_review_job};
use crate::store::{Store, now_ms, poll::SeenRequest};

async fn attempt(gh: FakeGh) -> (ReviewResult, Store, FakeGit, tempfile::TempDir) {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("state"), tmp.path().join("cache"));
    let store = Store::open(&paths).await.unwrap();
    let pr = parse("https://github.com/o/r/pull/1").unwrap();
    let seen = SeenRequest {
        pr: pr.clone(),
        head_sha: Some("polled-head".into()),
        is_draft: false,
    };
    let process = ProcessInfo {
        pid: 1,
        started_at_ms: 0,
    };
    store
        .apply_poll(vec![seen], vec![], 1, 60_000, &process)
        .await
        .unwrap();
    let job = store.next_runnable_job(60_001).await.unwrap().unwrap().job;
    let deps = Deps {
        gh,
        git: FakeGit::default(),
        agent: UncalledAgent,
        settings: crate::config::Settings::default(),
    };
    let (_tx, cancel) = tokio::sync::watch::channel(false);
    let result = execute_review_job(&paths, &store, &deps, &pr, &job, &cancel).await;
    assert!(!paths.runs_dir().exists());
    (result, store, deps.git, tmp)
}

#[tokio::test]
async fn a_new_draft_is_held_and_a_closed_or_merged_pr_is_cancelled_before_git() {
    for (state, draft) in [("OPEN", true), ("CLOSED", false), ("MERGED", false)] {
        let gh = FakeGh {
            state,
            draft,
            ..FakeGh::default()
        };
        let (result, store, git, _tmp) = attempt(gh).await;
        assert!(git.0.lock().unwrap().is_empty());
        assert!(store.next_runnable_job(i64::MAX).await.unwrap().is_none());
        let stored = store.stored_result(1).await.unwrap();
        assert!(stored.run.is_none());
        if draft {
            assert!(matches!(result, ReviewResult::Deferred));
            assert_eq!(stored.state, "queued");
        } else {
            assert!(matches!(result, ReviewResult::JobCancelled { .. }));
            assert_eq!(stored.state, "cancelled");
        }
    }
}

#[tokio::test(start_paused = true)]
async fn the_fetched_head_gets_its_own_debounce_even_if_github_still_reports_the_polled_head() {
    let start = now_ms();
    let gh = FakeGh {
        head: "polled-head",
        ..FakeGh::default()
    };
    let (result, store, git, _tmp) = attempt(gh).await;
    assert!(matches!(result, ReviewResult::Deferred));
    assert_eq!(*git.0.lock().unwrap(), ["clone", "fetch"]);
    assert!(
        store
            .next_runnable_job(start + 59_999)
            .await
            .unwrap()
            .is_none()
    );
    let queued = store
        .next_runnable_job(now_ms() + 60_000)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(queued.job.job_id, 1);
    let stored = store.stored_result(1).await.unwrap();
    assert_eq!(stored.known_head.as_deref(), Some("new-head"));
    assert!(stored.run.is_none());
}
