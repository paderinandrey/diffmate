use super::queue::{self, HEAD, key, open};
use crate::store::Store;
use crate::tui::run_scripted;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;

const PUBLICATION: &str = "0123456789abcdef0123456789abcdef";

async fn cell(store: &Store, sql: &str) -> String {
    let sql = sql.to_string();
    store
        .call(move |conn| conn.query_row(&sql, [], |row| row.get(0)))
        .await
        .unwrap()
}

async fn seed(store: &Store, publication: &str, job: &str) {
    let sql = format!(
        "INSERT INTO targets (id, owner, repo, number, tracking, created_at, head_sha) VALUES (1, 'o', 'r', 1, 'tracked', 1, '{HEAD}');
         INSERT INTO jobs (id, kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at) VALUES (1, 'review', 1, 'succeeded', 1, 0, 0, 0);
         INSERT INTO runs (id, target_id, job_id, state, result, head_sha, merge_base_sha, base_tip_sha, base_ref, created_at) VALUES (1, 1, 1, 'succeeded', 'findings', '{HEAD}', 'base', 'base', 'main', 0);
         INSERT INTO drafts (id, run_id, original_output, created_at) VALUES (1, 1, '{{}}', 0);
         INSERT INTO publications (id, target_id, draft_id, run_id, state, event, confirmed_sha, snapshot, created_at, updated_at) VALUES ('{PUBLICATION}', 1, 1, 1, '{publication}', 'COMMENT', '{HEAD}', '{{}}', 1, 1);
         INSERT INTO jobs (id, kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at, publication_id) VALUES (2, 'publish', 1, '{job}', 1, 0, 1, 1, '{PUBLICATION}');"
    );
    store
        .call(move |conn| conn.execute_batch(&sql))
        .await
        .unwrap();
}

async fn shown(store: &Store, keys: &[KeyCode]) -> String {
    let events: Vec<_> = keys.iter().copied().map(key).collect();
    let drawn = run_scripted(store, &events, TestBackend::new(80, 8))
        .await
        .unwrap();
    queue::text_of(&drawn)
}

async fn states(store: &Store) -> String {
    let publication = cell(store, "SELECT state FROM publications").await;
    let job = cell(store, "SELECT state FROM jobs WHERE id = 2").await;
    format!("{publication}/{job}")
}

#[tokio::test]
async fn keep_queued_is_highlighted_and_enter_writes_nothing() {
    let (_tmp, store) = open().await;
    seed(&store, "pending", "queued").await;
    let text = shown(&store, &[KeyCode::Char('u')]).await;
    assert!(text.contains("[Keep queued]"), "{text}");
    assert!(!text.contains("[Cancel publication]"), "{text}");
    shown(&store, &[KeyCode::Char('u'), KeyCode::Enter]).await;
    assert_eq!(states(&store).await, "pending/queued");
}

#[tokio::test]
async fn the_other_choice_cancels_the_queued_publication() {
    let (_tmp, store) = open().await;
    seed(&store, "pending", "queued").await;
    shown(
        &store,
        &[KeyCode::Char('u'), KeyCode::Right, KeyCode::Enter],
    )
    .await;
    assert_eq!(states(&store).await, "abandoned/cancelled");
    assert_eq!(
        cell(&store, "SELECT reason FROM publications").await,
        "cancelled by me"
    );
}

#[tokio::test]
async fn started_publication_is_not_cancelled() {
    for (publication, job) in [("sending", "running"), ("pending", "running")] {
        let (_tmp, store) = open().await;
        seed(&store, publication, job).await;
        let text = shown(&store, &[KeyCode::Char('u')]).await;
        assert!(
            text.contains("already started"),
            "{publication}/{job}: {text}"
        );
        assert_eq!(states(&store).await, format!("{publication}/{job}"));
    }
}

#[tokio::test]
async fn quit_leaves_a_queued_publication_pending() {
    let (_tmp, store) = open().await;
    seed(&store, "pending", "queued").await;
    run_scripted(&store, &[key(KeyCode::Char('q'))], TestBackend::new(80, 6))
        .await
        .unwrap();
    assert_eq!(states(&store).await, "pending/queued");
}
