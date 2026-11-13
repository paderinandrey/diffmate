use super::fake::Fake;
use super::publish::{base, batch, hold, paths, press, seed};
use super::queue::{self, HEAD, open};
use ratatui::crossterm::event::KeyCode;

#[tokio::test]
async fn undecided_blocks_until_that_finding_is_kept() {
    let (tmp, store) = open().await;
    let now = crate::store::now_ms();
    let finding = "INSERT INTO findings (id, run_id, draft_id, ordinal, severity, path, line, side, title, body, decision) VALUES ('f1', 1, 1, 1, 'high', 'src/a.rs', 4, 'RIGHT', 'Title', 'Body', 'kept'), ('f2', 1, 1, 2, 'low', 'src/b.rs', 5, 'RIGHT', 'Other', 'Body', 'undecided');";
    batch(&store, base(now, finding)).await;
    let root = paths(&tmp);
    let gh = Fake::new(HEAD);
    let blocked = press(&store, &root, &gh, &[KeyCode::Enter, KeyCode::Char('c')]).await;
    assert!(blocked.contains("undecided findings: f2"), "{blocked}");
    assert_eq!(
        queue::count(&store, "SELECT count(*) FROM publications").await,
        0
    );
    let keys = [
        KeyCode::Enter,
        KeyCode::Down,
        KeyCode::Char('k'),
        KeyCode::Char('c'),
    ];
    press(&store, &root, &gh, &keys).await;
    assert_eq!(
        queue::count(&store, "SELECT count(*) FROM publications").await,
        1
    );
    assert_eq!(gh.seen().len(), 1);
}

#[tokio::test]
async fn second_c_while_pending_posts_once() {
    let (tmp, store) = open().await;
    seed(&store).await;
    let root = paths(&tmp);
    let held = hold(&root.lock());
    let gh = Fake::new(HEAD);
    let text = press(
        &store,
        &root,
        &gh,
        &[KeyCode::Enter, KeyCode::Char('c'), KeyCode::Char('c')],
    )
    .await;
    assert!(
        text.contains("a publication is already pending, sending, or unknown"),
        "{text}"
    );
    drop(held);
    super::super::runner::serve(&store, &gh, 2).await.unwrap();
    assert_eq!(gh.seen().len(), 1);
    let submitted = "SELECT count(*) FROM publications WHERE state = 'submitted'";
    assert_eq!(queue::count(&store, submitted).await, 1);
}

#[tokio::test]
async fn submitted_draft_does_not_post_again() {
    let (tmp, store) = open().await;
    seed(&store).await;
    let gh = Fake::new(HEAD);
    let text = press(
        &store,
        &paths(&tmp),
        &gh,
        &[KeyCode::Enter, KeyCode::Char('c'), KeyCode::Char('c')],
    )
    .await;
    assert!(text.contains("this draft is already submitted"), "{text}");
    assert_eq!(gh.seen().len(), 1);
}
