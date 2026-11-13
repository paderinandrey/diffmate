use super::super::support::cell;
use super::harness::{open_gh, pair};
use super::{LIVE, RUN, press};
use ratatui::crossterm::event::KeyCode;

fn retries() -> u32 {
    crate::config::Settings::default().max_retries()
}

#[tokio::test]
async fn head_moved_needs_confirmation() {
    let (_tmp, store, paths, gh) = open_gh(RUN).await;
    gh.then(LIVE);
    let text = press(&store, &paths, &gh, &[KeyCode::Enter, KeyCode::Char('c')]).await;
    assert!(text.contains("[Don't send]"), "{text}");
    assert!(text.contains("stale: head bbbbbbbbbbbb"), "{text}");
    assert_eq!(gh.seen(), Vec::new());
    assert_eq!(
        pair(&store).await,
        ("pending".into(), "needs_confirmation".into())
    );
    let before = cell::<String>(&store, "SELECT snapshot FROM publications").await;
    let id = cell::<String>(&store, "SELECT id FROM publications").await;
    store.reconfirm(&id, LIVE).await.unwrap();
    assert_eq!(pair(&store).await, ("pending".into(), "queued".into()));
    assert_eq!(
        cell::<String>(&store, "SELECT confirmed_sha FROM publications").await,
        LIVE
    );
    assert_eq!(
        cell::<String>(&store, "SELECT snapshot FROM publications").await,
        before
    );
    let job = cell::<i64>(&store, "SELECT id FROM jobs WHERE kind = 'publish'").await;
    crate::publish::execute_publish(&store, &gh, job, retries())
        .await
        .unwrap();
    let seen = gh.seen();
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].commit_id, RUN);
}

#[tokio::test]
async fn decline_cancels_and_abandons() {
    let (_tmp, store, paths, gh) = open_gh(RUN).await;
    gh.then(LIVE);
    press(
        &store,
        &paths,
        &gh,
        &[KeyCode::Enter, KeyCode::Char('c'), KeyCode::Enter],
    )
    .await;
    assert_eq!(pair(&store).await, ("abandoned".into(), "cancelled".into()));
    assert_eq!(
        cell::<String>(&store, "SELECT reason FROM publications").await,
        "declined"
    );
    assert_eq!(gh.seen(), Vec::new());
}

#[tokio::test]
async fn redelivered_job_stays_abandoned() {
    let (_tmp, store, paths, gh) = open_gh(RUN).await;
    gh.then(LIVE);
    press(
        &store,
        &paths,
        &gh,
        &[KeyCode::Enter, KeyCode::Char('c'), KeyCode::Enter],
    )
    .await;
    let job = cell::<i64>(&store, "SELECT id FROM jobs WHERE kind = 'publish'").await;
    crate::publish::execute_publish(&store, &gh, job, retries())
        .await
        .unwrap();
    assert_eq!(pair(&store).await, ("abandoned".into(), "failed".into()));
    assert_eq!(
        cell::<String>(&store, "SELECT reason FROM publications").await,
        "declined"
    );
    assert_eq!(
        cell::<i64>(&store, "SELECT count(*) FROM publications").await,
        1
    );
    assert_eq!(gh.seen(), Vec::new());
}
