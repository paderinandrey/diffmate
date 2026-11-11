use crate::store::Store;
use crate::tui::run_scripted;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tempfile::TempDir;

pub(super) const HEAD: &str = "0123456789abcdef0123456789abcdef01234567";

pub(super) async fn seed(store: &Store, number: i64) {
    let now = crate::store::now_ms();
    store
        .call(move |conn| {
            conn.execute_batch(&format!(
                "INSERT INTO targets (id, owner, repo, number, tracking, created_at, head_sha, head_checked_at)
                 VALUES ({number}, 'o', 'r', {number}, 'tracked', {number}, '{HEAD}', {now});
                 INSERT INTO jobs (id, kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at)
                 VALUES ({number}, 'review', {number}, 'succeeded', 1, 0, 0, 0);
                 INSERT INTO runs (id, target_id, job_id, state, result, head_sha, merge_base_sha, base_tip_sha, base_ref, created_at)
                 VALUES ({number}, {number}, {number}, 'succeeded', 'findings', '{HEAD}', 'base', 'base', 'main', 0);
                 INSERT INTO drafts (id, run_id, original_output, created_at) VALUES ({number}, {number}, '{{}}', 0);
                 INSERT INTO findings (id, run_id, draft_id, ordinal, severity, path, line, side, title, body)
                 VALUES ('{number}', {number}, {number}, 1, 'high', 'src/a.rs', 4, 'RIGHT', 'Off', 'body');"
            ))
        })
        .await
        .unwrap();
}

pub(super) fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

pub(super) fn text_of(backend: &TestBackend) -> String {
    backend.to_string()
}

pub(super) async fn open() -> (TempDir, Store) {
    let tmp = TempDir::new().unwrap();
    let paths = crate::paths::Paths::new(tmp.path().join("state"), tmp.path().join("cache"));
    (tmp, Store::open(&paths).await.unwrap())
}

pub(super) async fn count(store: &Store, sql: &'static str) -> i64 {
    store
        .call(move |conn| conn.query_row(sql, [], |row| row.get(0)))
        .await
        .unwrap()
}

#[tokio::test]
async fn queue_shows_the_target_and_quit_writes_nothing() {
    let (_tmp, store) = open().await;
    seed(&store, 1).await;
    let jobs_before = count(&store, "SELECT count(*) FROM jobs").await;
    let drawn = run_scripted(&store, &[key(KeyCode::Char('q'))], TestBackend::new(160, 6))
        .await
        .unwrap();
    let text = text_of(&drawn);
    for part in [
        "o/r#1",
        "tracked",
        "succeeded (findings: 1)",
        "current: head 0123456789ab, checked just now",
        "none",
    ] {
        assert!(text.contains(part), "{part} missing from {text}");
    }
    assert_eq!(store.queue_rows().await.unwrap()[0].target_id, 1);
    assert_eq!(count(&store, "SELECT count(*) FROM publications").await, 0);
    assert_eq!(
        count(&store, "SELECT count(*) FROM jobs").await,
        jobs_before
    );
}

#[tokio::test]
async fn queue_is_newest_first_and_arrows_move_the_selection() {
    let (_tmp, store) = open().await;
    seed(&store, 1).await;
    seed(&store, 2).await;
    let drawn = run_scripted(&store, &[key(KeyCode::Down)], TestBackend::new(160, 6))
        .await
        .unwrap();
    let lines: Vec<_> = text_of(&drawn)
        .lines()
        .map(|line| line.trim_matches('"').trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();
    assert!(
        lines[0].contains("o/r#2") && !lines[0].starts_with('>'),
        "{lines:?}"
    );
    assert!(
        lines[1].starts_with('>') && lines[1].contains("o/r#1"),
        "{lines:?}"
    );
}

#[tokio::test]
async fn queue_frame_keeps_the_review_run_after_sync() {
    let (_tmp, store) = open().await;
    seed(&store, 1).await;
    seed(&store, 2).await;
    let sql = format!(
        "INSERT INTO publications (id, target_id, draft_id, run_id, state, event, confirmed_sha, snapshot, created_at, updated_at)
         VALUES ('0123456789abcdef0123456789abcdef', 1, 1, 1, 'submitted', 'COMMENT', '{HEAD}', '{{}}', 1, 1);
         INSERT INTO jobs (id, kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at, publication_id)
         VALUES (3, 'publish', 1, 'succeeded', 1, 0, 1, 1, '0123456789abcdef0123456789abcdef');
         INSERT INTO jobs (id, kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at, publication_id)
         VALUES (4, 'sync', 1, 'queued', 1, 0, 2, 2, '0123456789abcdef0123456789abcdef');"
    );
    store
        .call(move |conn| conn.execute_batch(&sql))
        .await
        .unwrap();
    let jobs_before = count(&store, "SELECT count(*) FROM jobs").await;
    let drawn = run_scripted(&store, &[key(KeyCode::Char('q'))], TestBackend::new(160, 6))
        .await
        .unwrap();
    let lines: Vec<_> = text_of(&drawn)
        .lines()
        .map(|line| line.trim_matches('"').trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();
    let two = lines.iter().position(|line| line.contains("o/r#2"));
    let one = lines.iter().position(|line| line.contains("o/r#1"));
    assert!(
        matches!((two, one), (Some(newer), Some(older)) if newer < older),
        "{lines:?}"
    );
    let (two, one) = (two.unwrap(), one.unwrap());
    assert!(
        lines[one].contains("succeeded (findings: 1)") && lines[one].contains("submitted"),
        "{lines:?}"
    );
    assert!(
        lines[two].contains("succeeded (findings: 1)") && lines[two].contains("none"),
        "{lines:?}"
    );
    assert_eq!(
        count(&store, "SELECT count(*) FROM jobs").await,
        jobs_before
    );
}
