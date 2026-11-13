use super::queue::{self, count, open, seed};
use crate::store::Store;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use std::fs::OpenOptions;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

const SENTENCE: &str = "diffmate review or diffmate watch must run it";

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

async fn shown_actions(
    store: &Store,
    keys: &[KeyCode],
    lock: Option<&Path>,
    opener: Option<&str>,
) -> String {
    let events: Vec<_> = keys.iter().copied().map(queue::key).collect();
    let drawn =
        crate::tui::run_scripted_actions(store, &events, TestBackend::new(100, 8), lock, opener)
            .await
            .unwrap();
    queue::text_of(&drawn)
}

fn recorder(dir: &Path, log: &Path) -> String {
    let program = dir.join("record-open");
    let script = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$#\" \"$1\" > '{}'\n",
        log.display()
    );
    std::fs::write(&program, script).unwrap();
    let mode = std::fs::Permissions::from_mode(0o700);
    std::fs::set_permissions(&program, mode).unwrap();
    program.display().to_string()
}

#[tokio::test]
async fn o_spawns_open_with_only_the_pr_url() {
    let (tmp, store) = open().await;
    seed(&store, 1).await;
    let log = tmp.path().join("opened");
    let program = recorder(tmp.path(), &log);
    shown_actions(&store, &[KeyCode::Char('o')], None, Some(&program)).await;
    let recorded = std::fs::read_to_string(&log).unwrap_or_default();
    assert_eq!(recorded, "1\nhttps://github.com/o/r/pull/1\n");
    assert_eq!(count(&store, "SELECT count(*) FROM jobs").await, 1);
}

#[tokio::test]
async fn o_draws_a_spawn_error_without_panicking() {
    let (_tmp, store) = open().await;
    seed(&store, 1).await;
    let text = shown_actions(
        &store,
        &[KeyCode::Char('o')],
        None,
        Some("missing-open-bin"),
    )
    .await;
    assert!(text.contains("could not open the pull request"), "{text}");
    assert_eq!(count(&store, "SELECT count(*) FROM jobs").await, 1);
}

#[tokio::test]
async fn n_without_watch_enqueues_a_manual_review_and_explains() {
    let (_tmp, store) = open().await;
    seed(&store, 1).await;
    let queued = "SELECT count(*) FROM jobs WHERE state = 'queued' AND origin = 'manual' AND force = 0 AND kind = 'review'";
    let text = shown_actions(&store, &[KeyCode::Char('n')], None, None).await;
    assert!(text.contains(SENTENCE), "{text}");
    assert_eq!(latch(&store).await, "tracked0");
    assert_eq!(count(&store, queued).await, 1);
    assert_eq!(
        count(
            &store,
            "SELECT count(*) FROM jobs WHERE origin = 'automatic'"
        )
        .await,
        0
    );
    assert_eq!(count(&store, "SELECT count(*) FROM runs").await, 1);
    let again = shown_actions(&store, &[KeyCode::Char('n')], None, None).await;
    assert!(again.contains(SENTENCE), "{again}");
    assert_eq!(count(&store, queued).await, 1);
}

#[tokio::test]
async fn n_while_watch_holds_the_lock_enqueues_without_the_explanation() {
    let (tmp, store) = open().await;
    seed(&store, 1).await;
    let path = tmp.path().join("state").join("runner.lock");
    std::fs::write(&path, "watch\n").unwrap();
    let held = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
        .unwrap();
    held.try_lock().unwrap();
    let text = shown_actions(&store, &[KeyCode::Char('n')], Some(&path), None).await;
    assert!(!text.contains(SENTENCE), "{text}");
    assert_eq!(latch(&store).await, "tracked0");
    let queued = "SELECT count(*) FROM jobs WHERE state = 'queued' AND origin = 'manual' AND force = 0 AND kind = 'review'";
    assert_eq!(count(&store, queued).await, 1);
}
