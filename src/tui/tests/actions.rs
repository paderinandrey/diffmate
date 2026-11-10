use super::queue::{self, count, open, seed};
use crate::store::Store;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;

async fn cell(store: &Store, sql: &str) -> String {
    let sql = sql.to_string();
    store
        .call(move |conn| conn.query_row(&sql, [], |row| row.get(0)))
        .await
        .unwrap()
}

async fn shown(store: &Store, keys: &[KeyCode]) -> String {
    let events: Vec<_> = keys.iter().copied().map(queue::key).collect();
    let drawn = crate::tui::run_scripted(store, &events, TestBackend::new(80, 8)).await;
    queue::text_of(&drawn.unwrap())
}

async fn latch(store: &Store) -> String {
    cell(
        store,
        "SELECT tracking || request_seen FROM targets WHERE id = 1",
    )
    .await
}

#[tokio::test]
async fn enter_on_keep_leaves_tracking_and_request_seen() {
    let (_tmp, store) = open().await;
    seed(&store, 1).await;
    let opened = shown(&store, &[KeyCode::Char('x')]).await;
    assert!(opened.contains("will be discarded"), "{opened}");
    assert!(opened.contains("[Keep]"), "{opened}");
    assert!(!opened.contains("[Discard]"), "{opened}");
    let closed = shown(&store, &[KeyCode::Char('x'), KeyCode::Enter]).await;
    assert!(!closed.contains("will be discarded"), "{closed}");
    assert_eq!(latch(&store).await, "tracked0");
}

#[tokio::test]
async fn enter_on_discard_uses_the_store_latch() {
    let (_tmp, store) = open().await;
    seed(&store, 1).await;
    shown(
        &store,
        &[KeyCode::Char('x'), KeyCode::Right, KeyCode::Enter],
    )
    .await;
    assert_eq!(latch(&store).await, "untracked1");
    let (_tmp, store) = open().await;
    seed(&store, 1).await;
    shown(
        &store,
        &[
            KeyCode::Enter,
            KeyCode::Char('x'),
            KeyCode::Down,
            KeyCode::Enter,
        ],
    )
    .await;
    assert_eq!(latch(&store).await, "untracked1");
}

#[tokio::test]
async fn quit_or_esc_during_confirm_writes_nothing() {
    let (_tmp, store) = open().await;
    seed(&store, 1).await;
    let publication = "0123456789abcdef0123456789abcdef";
    let sql = format!(
        "INSERT INTO publications (id, target_id, draft_id, run_id, state, event, confirmed_sha, snapshot, created_at, updated_at)
         VALUES ('{publication}', 1, 1, 1, 'pending', 'COMMENT', '{HEAD}', '{{}}', 1, 1);
         INSERT INTO jobs (kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at, publication_id)
         VALUES ('publish', 1, 'queued', 1, 0, 1, 1, '{publication}');",
        HEAD = queue::HEAD
    );
    store
        .call(move |conn| conn.execute_batch(&sql))
        .await
        .unwrap();
    let jobs = count(&store, "SELECT count(*) FROM jobs").await;
    shown(&store, &[KeyCode::Char('x'), KeyCode::Char('q')]).await;
    assert_eq!(latch(&store).await, "tracked0");
    assert_eq!(count(&store, "SELECT count(*) FROM jobs").await, jobs);
    assert_eq!(
        cell(&store, "SELECT state FROM publications").await,
        "pending"
    );
    assert_eq!(
        cell(&store, "SELECT state FROM jobs WHERE kind = 'publish'").await,
        "queued"
    );
    let closed = shown(&store, &[KeyCode::Char('x'), KeyCode::Esc]).await;
    assert!(!closed.contains("will be discarded"), "{closed}");
    assert_eq!(latch(&store).await, "tracked0");
}
