mod remote;
mod rig;

use std::sync::atomic::Ordering;
use std::time::Duration;

use crate::store::Store;
use rig::{Rig, end_watch};

async fn cell(store: &Store, sql: &str) -> String {
    let sql = sql.to_string();
    store
        .call(move |conn| conn.query_row(&sql, [], |row| row.get(0)))
        .await
        .unwrap()
}

#[tokio::test(start_paused = true)]
async fn publish_finishes_during_an_agent_run() {
    let rig = Rig::open().await;
    let deps = rig.deps(true);
    let mut signals = crate::review::signals::Signals::new().unwrap();
    let mut job = std::pin::pin!(super::run::jobs(
        &rig.config,
        &rig.store,
        &deps,
        &mut signals
    ));
    let mut seen = rig.seen.clone();
    tokio::select! {
        biased;
        end = &mut job => panic!("review ended before the agent: {end:?}"),
        result = seen.wait_for(|inside| *inside) => result.unwrap(),
    };
    let sha = "h".repeat(40);
    rig.store
        .confirm_publication(1, "COMMENT", &sha, &rig.config.process)
        .await
        .unwrap();
    let mut submitted = false;
    for _ in 0..5 {
        tokio::select! {
            biased;
            end = &mut job => panic!("watch ended during publish: {end:?}"),
            () = tokio::time::sleep(super::WATCH_LOCK_POLL) => {}
        }
        let state = cell(&rig.store, "SELECT state FROM publications").await;
        if state == "submitted" {
            submitted = true;
            break;
        }
    }
    assert!(submitted);
    assert_eq!(rig.posts.load(Ordering::Relaxed), 1);
    let review = format!("SELECT state FROM jobs WHERE id = {}", rig.review);
    assert_eq!(cell(&rig.store, &review).await, "running");
    end_watch();
    tokio::select! {
        biased;
        end = &mut job => end.unwrap(),
        () = tokio::time::sleep(Duration::from_secs(5)) => panic!("review did not finish"),
    };
    assert_ne!(cell(&rig.store, &review).await, "running");
}

#[tokio::test(start_paused = true)]
async fn a_stalled_publish_still_stops_the_agent() {
    let rig = Rig::open().await;
    let deps = rig.deps_hanging();
    let mut signals = crate::review::signals::Signals::new().unwrap();
    let mut job = std::pin::pin!(super::run::jobs(
        &rig.config,
        &rig.store,
        &deps,
        &mut signals
    ));
    let mut seen = rig.seen.clone();
    tokio::select! {
        biased;
        end = &mut job => panic!("review ended before the agent: {end:?}"),
        result = seen.wait_for(|inside| *inside) => result.unwrap(),
    };
    let sha = "h".repeat(40);
    rig.store
        .confirm_publication(1, "COMMENT", &sha, &rig.config.process)
        .await
        .unwrap();
    let mut started = false;
    for _ in 0..5 {
        tokio::select! {
            biased;
            end = &mut job => panic!("watch ended before the publish stalled: {end:?}"),
            () = tokio::time::sleep(super::WATCH_LOCK_POLL) => {}
        }
        if rig.posts.load(Ordering::Relaxed) >= 1 {
            started = true;
            break;
        }
    }
    assert!(started);
    end_watch();
    tokio::select! {
        biased;
        end = &mut job => end.unwrap(),
        () = tokio::time::sleep(Duration::from_secs(5)) => {
            panic!("shutdown did not stop the review")
        }
    };
    let review = format!("SELECT state FROM jobs WHERE id = {}", rig.review);
    assert_ne!(cell(&rig.store, &review).await, "running");
}

#[tokio::test(start_paused = true)]
async fn a_review_completes_without_a_service_job() {
    let rig = Rig::open().await;
    let deps = rig.deps(false);
    let mut signals = crate::review::signals::Signals::new().unwrap();
    let mut job = std::pin::pin!(super::run::jobs(
        &rig.config,
        &rig.store,
        &deps,
        &mut signals
    ));
    let review = format!("SELECT state FROM jobs WHERE id = {}", rig.review);
    let mut finished = false;
    for _ in 0..1000 {
        if cell(&rig.store, &review).await == "succeeded" {
            finished = true;
            break;
        }
        tokio::select! {
            biased;
            end = &mut job => panic!("watch ended before the review: {end:?}"),
            () = tokio::task::yield_now() => {}
        }
    }
    assert!(finished);
    assert_eq!(rig.posts.load(Ordering::Relaxed), 0);
    let publications = "SELECT CAST(count(*) AS TEXT) FROM publications";
    assert_eq!(cell(&rig.store, publications).await, "0");
    end_watch();
    job.await.unwrap();
}
