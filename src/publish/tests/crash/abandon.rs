use super::harness::{self, planted};
use ratatui::crossterm::event::KeyCode;

async fn shown(state: &str, keys: &[KeyCode]) -> (crate::store::Store, String, usize) {
    let (tmp, store, remote) = planted(state).await;
    let events: Vec<_> = keys
        .iter()
        .copied()
        .map(|code| {
            ratatui::crossterm::event::KeyEvent::new(
                code,
                ratatui::crossterm::event::KeyModifiers::NONE,
            )
        })
        .collect();
    let git = crate::git::GitCli::new(&crate::procenv::InheritedEnv::from_pairs(Vec::new()));
    let drawn = crate::tui::run_scripted_publish(
        &store,
        &events,
        ratatui::backend::TestBackend::new(120, 8),
        &harness::paths(&tmp),
        remote.as_ref(),
        &git,
        &crate::config::Settings::default(),
    )
    .await
    .unwrap();
    (store, drawn.to_string(), remote.posts())
}

#[tokio::test]
async fn b_safe_choice_leaves_unknown() {
    let (store, text, _posts) = shown("unknown", &[KeyCode::Char('b')]).await;
    assert!(text.contains("[Leave unknown]"), "{text}");
    assert!(!text.contains("[Abandon]"), "{text}");
    assert_eq!(
        harness::text(&store, "SELECT state FROM publications").await,
        "unknown"
    );
    let (store, _, posts) = shown("unknown", &[KeyCode::Char('b'), KeyCode::Enter]).await;
    assert_eq!(
        harness::text(&store, "SELECT state FROM publications").await,
        "unknown"
    );
    assert_eq!(posts, 0);
}

#[tokio::test]
async fn b_abandon_sets_confirmed_by_me() {
    let (store, _, posts) = shown(
        "unknown",
        &[KeyCode::Char('b'), KeyCode::Right, KeyCode::Enter],
    )
    .await;
    assert_eq!(
        harness::text(&store, "SELECT state FROM publications").await,
        "abandoned"
    );
    assert_eq!(
        harness::text(&store, "SELECT reason FROM publications").await,
        "confirmed by me"
    );
    assert_eq!(posts, 0);
}

#[tokio::test]
async fn b_on_other_state_does_not_ask() {
    let (store, text, posts) = shown("submitted", &[KeyCode::Char('b')]).await;
    assert!(!text.contains("Leave unknown"), "{text}");
    assert_eq!(
        harness::text(&store, "SELECT state FROM publications").await,
        "submitted"
    );
    assert_eq!(posts, 0);
}

#[tokio::test]
async fn b_on_the_open_draft_abandons() {
    let (store, _, posts) = shown(
        "unknown",
        &[
            KeyCode::Enter,
            KeyCode::Char('b'),
            KeyCode::Right,
            KeyCode::Enter,
        ],
    )
    .await;
    assert_eq!(
        harness::text(&store, "SELECT state FROM publications").await,
        "abandoned"
    );
    assert_eq!(
        harness::text(&store, "SELECT reason FROM publications").await,
        "confirmed by me"
    );
    assert_eq!(posts, 0);
}
