mod seed;
use super::{gh::FakeGh, git::FakeGit};
use crate::agent::{Agent, AgentOutcome, AgentRequest};
use crate::paths::Paths;
use crate::pr_url::PrRef;
use crate::runner::{Deps, ReviewResult, execute_review_job};
use crate::store::{Store, jobs::EnqueuedJob};

pub(super) struct UncalledAgent;

#[allow(refining_impl_trait)]
impl Agent for UncalledAgent {
    fn run(&self, _: &AgentRequest) -> std::future::Ready<AgentOutcome> {
        unreachable!("no agent should start")
    }
}

#[tokio::test]
async fn dedup_uses_the_pinned_retry_head_instead_of_the_current_github_head() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("state"), tmp.path().join("cache"));
    let store = Store::open(&paths).await.unwrap();
    seed::runs(&store).await;
    let pr = PrRef {
        owner: "o".into(),
        repo: "r".into(),
        number: 1,
    };
    let job = EnqueuedJob {
        job_id: 1,
        target_id: 1,
    };
    let deps = Deps {
        git: FakeGit::default(),
        gh: FakeGh::default(),
        agent: UncalledAgent,
        settings: crate::config::Settings::default(),
    };
    let (_stop, cancel) = tokio::sync::watch::channel(false);
    let result = execute_review_job(&paths, &store, &deps, &pr, &job, &cancel).await;
    let ReviewResult::Succeeded {
        run_id,
        head_sha,
        known_head,
        reused,
        ..
    } = result
    else {
        panic!("the retry must reuse the successful old-head run");
    };
    assert_eq!(run_id, 2);
    assert_eq!(head_sha, "old-head");
    assert_eq!(known_head.as_deref(), Some("new-head"));
    assert!(reused);
    assert_eq!(*deps.git.0.lock().unwrap(), ["clone"]);
    assert!(!paths.runs_dir().exists());
    let stored = store.stored_result(job.job_id).await.unwrap();
    assert_eq!(stored.state, "succeeded");
    assert!(stored.reused);
}

#[tokio::test]
async fn manual_review_still_resolves_a_draft_or_closed_pr() {
    for (state, draft) in [("OPEN", true), ("CLOSED", false)] {
        let tmp = tempfile::tempdir().unwrap();
        let paths = Paths::new(tmp.path().join("state"), tmp.path().join("cache"));
        let store = Store::open(&paths).await.unwrap();
        seed::runs(&store).await;
        store
            .call(|conn| {
                conn.execute("UPDATE jobs SET origin = 'manual' WHERE id = 1", [])
                    .map(drop)
            })
            .await
            .unwrap();
        let pr = crate::pr_url::parse("https://github.com/o/r/pull/1").unwrap();
        let deps = Deps {
            git: FakeGit::default(),
            gh: FakeGh {
                state,
                draft,
                ..FakeGh::default()
            },
            agent: UncalledAgent,
            settings: crate::config::Settings::default(),
        };
        let (_tx, cancel) = tokio::sync::watch::channel(false);
        let job = EnqueuedJob {
            job_id: 1,
            target_id: 1,
        };
        let result = execute_review_job(&paths, &store, &deps, &pr, &job, &cancel).await;
        assert!(matches!(
            result,
            ReviewResult::Succeeded {
                reused: true,
                run_id: 3,
                ..
            }
        ));
    }
}
