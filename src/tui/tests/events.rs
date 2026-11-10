use super::fake::Fake;
use super::publish::{base, batch, paths, press, seed};
use super::queue::{HEAD, open};
use ratatui::crossterm::event::KeyCode;

#[tokio::test]
async fn approve_and_request_changes_set_the_event() {
    for (key, event) in [
        (KeyCode::Char('a'), "APPROVE"),
        (KeyCode::Char('r'), "REQUEST_CHANGES"),
    ] {
        let (tmp, store) = open().await;
        seed(&store).await;
        let gh = Fake::new(HEAD);
        press(&store, &paths(&tmp), &gh, &[KeyCode::Enter, key]).await;
        assert_eq!(gh.seen()[0].event, event);
    }
}

#[tokio::test]
async fn empty_draft_body_is_only_the_marker() {
    let (tmp, store) = open().await;
    batch(&store, base(crate::store::now_ms(), "")).await;
    let gh = Fake::new(HEAD);
    press(
        &store,
        &paths(&tmp),
        &gh,
        &[KeyCode::Enter, KeyCode::Char('c')],
    )
    .await;
    let body = gh.seen()[0].body.clone();
    assert!(body.starts_with("<!-- diffmate:publication="), "{body}");
    assert!(body.ends_with(" -->"), "{body}");
    assert_eq!(body.lines().count(), 1, "{body}");
    assert_eq!(gh.seen()[0].comments, Vec::new());
}
