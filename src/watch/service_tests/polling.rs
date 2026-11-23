use super::{Rig, cell, end_watch};
use crate::gh::DirectRequest;
use crate::git::Git;
use crate::pr_url::PrRef;
use crate::review::signals::Signals;
use std::time::Duration;

#[tokio::test(start_paused = true)]
async fn polls_discover_requests_and_defer_ref_deletion_until_the_review_finishes() {
    let rig = Rig::open().await;
    let deps = rig.deps_pausing();
    let mut signals = Signals::new().unwrap();
    let mut watch = std::pin::pin!(crate::watch::run::jobs(
        &rig.config,
        &rig.store,
        &deps,
        &mut signals
    ));
    let mut agent = rig.seen.clone();
    tokio::select! {
        result = &mut watch => panic!("watch ended before review: {result:?}"),
        result = agent.wait_for(|inside| *inside) => result.unwrap(),
    };
    let active = PrRef {
        owner: "o".into(),
        repo: "r".into(),
        number: 1,
    };
    let other = PrRef {
        number: 2,
        ..active.clone()
    };
    deps.gh.requests.lock().unwrap().push(DirectRequest {
        pr: other,
        is_draft: false,
    });
    *deps.gh.closed.lock().unwrap() = Some(active);
    tokio::time::advance(rig.config.settings.poll_interval()).await;
    let repo = rig.config.paths.repo_dir("o", "r");
    let head = "refs/diffmate/pull/1/head";
    let observed = async {
        for _ in 0..30 {
            if cell(&rig.store, "SELECT CAST(count(*) AS TEXT) FROM targets WHERE number = 2 AND tracking = 'tracked'").await == "1" {
                return;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        panic!("new request not discovered during the first review");
    };
    tokio::select! {
        result = &mut watch => panic!("watch stopped while polling: {result:?}"),
        () = observed => {},
    }
    let review_state = format!("SELECT state FROM jobs WHERE id = {}", rig.review);
    assert_eq!(cell(&rig.store, &review_state).await, "running");
    assert_eq!(rig.store.pending_ref_cleanup().await.unwrap().len(), 1);
    assert!(deps.git.rev_parse(&repo, head).await.is_ok());
    rig.agent_go.send_replace(true);
    tokio::select! {
        result = &mut watch => panic!("watch ended after review: {result:?}"),
        () = async {
            for _ in 0..30 {
                if rig.store.pending_ref_cleanup().await.unwrap().is_empty() {
                    return;
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            panic!("closed ref not cleaned after review");
        } => {},
    }
    assert!(deps.git.rev_parse(&repo, head).await.is_err());
    end_watch();
    watch.await.unwrap();
}
