use super::harness::{open_gh, pair, review};
use super::{RUN, press};
use ratatui::crossterm::event::KeyCode;

#[tokio::test]
async fn pending_review_of_mine_posts_nothing() {
    let (_tmp, store, paths, gh) = open_gh(RUN).await;
    let listed = vec![review("me", "PENDING")];
    gh.set_reviews(listed.clone());
    let text = press(&store, &paths, &gh, &[KeyCode::Enter, KeyCode::Char('c')]).await;
    assert!(
        text.contains("you already have a pending review on this pull request"),
        "{text}"
    );
    assert_eq!(pair(&store).await, ("abandoned".into(), "failed".into()));
    assert_eq!(gh.seen(), Vec::new());
    assert_eq!(gh.reviews(), listed);
}

#[tokio::test]
async fn empty_review_list_does_not_block() {
    let (_tmp, store, paths, gh) = open_gh(RUN).await;
    press(&store, &paths, &gh, &[KeyCode::Enter, KeyCode::Char('c')]).await;
    assert_eq!(gh.seen().len(), 1);
    assert_eq!(pair(&store).await, ("submitted".into(), "succeeded".into()));
}

#[tokio::test]
async fn lowercase_pending_does_not_match() {
    let (_tmp, store, paths, gh) = open_gh(RUN).await;
    gh.set_reviews(vec![review("me", "pending")]);
    press(&store, &paths, &gh, &[KeyCode::Enter, KeyCode::Char('c')]).await;
    assert_eq!(gh.seen().len(), 1);
}

#[tokio::test]
async fn other_author_and_approved_do_not_block() {
    for row in [review("other", "PENDING"), review("me", "APPROVED")] {
        let (_tmp, store, paths, gh) = open_gh(RUN).await;
        gh.set_reviews(vec![row]);
        press(&store, &paths, &gh, &[KeyCode::Enter, KeyCode::Char('c')]).await;
        assert_eq!(gh.seen().len(), 1);
    }
}

#[tokio::test]
async fn padded_login_and_state_do_not_match() {
    for (author, state) in [
        (" me", "PENDING"),
        ("ME", "PENDING"),
        ("me ", "PENDING"),
        ("me", "PENDING "),
    ] {
        let (_tmp, store, paths, gh) = open_gh(RUN).await;
        gh.set_reviews(vec![review(author, state)]);
        press(&store, &paths, &gh, &[KeyCode::Enter, KeyCode::Char('c')]).await;
        assert_eq!(gh.seen().len(), 1, "{author} {state}");
    }
}
