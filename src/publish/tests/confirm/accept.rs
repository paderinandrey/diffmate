use super::super::support::cell;
use super::harness::{open_gh, pair};
use super::{LIVE, RUN, press};
use ratatui::crossterm::event::KeyCode;

#[tokio::test]
async fn accepting_the_moved_head_posts_the_run() {
    let (_tmp, store, paths, gh) = open_gh(RUN).await;
    gh.then(LIVE);
    press(
        &store,
        &paths,
        &gh,
        &[
            KeyCode::Enter,
            KeyCode::Char('c'),
            KeyCode::Right,
            KeyCode::Enter,
        ],
    )
    .await;
    assert_eq!(pair(&store).await, ("submitted".into(), "succeeded".into()));
    assert_eq!(
        cell::<String>(&store, "SELECT confirmed_sha FROM publications").await,
        LIVE
    );
    let seen = gh.seen();
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].commit_id, RUN);
}
