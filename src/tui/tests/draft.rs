use super::queue::{HEAD, key, open, seed, text_of};
use crate::store::Store;
use crate::tui::run_scripted;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use std::fmt::Write;

pub(super) async fn seed_draft(store: &Store) {
    let now = crate::store::now_ms();
    store
        .call(move |conn| {
            conn.execute_batch(&format!(
                "INSERT INTO targets (id, owner, repo, number, tracking, created_at, head_sha, head_checked_at) VALUES (2, 'o', 'r', 2, 'untracked', 2, '{HEAD}', {now}); INSERT INTO jobs (id, kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at) VALUES (2, 'review', 2, 'succeeded', 1, 0, 0, 0); INSERT INTO runs (id, target_id, job_id, state, result, head_sha, merge_base_sha, base_tip_sha, base_ref, created_at) VALUES (2, 2, 2, 'succeeded', 'findings', '{HEAD}', 'base', 'base', 'main', 0); INSERT INTO drafts (id, run_id, original_output, limitation, future_improvements, created_at) VALUES (2, 2, '{{}}', 'missing CI', '[\"note one\",\"note two\"]', 0); INSERT INTO findings (id, run_id, draft_id, ordinal, severity, title, body) VALUES ('loose', 2, 2, 1, 'low', 'Loose', 'loose body'); INSERT INTO findings (id, run_id, draft_id, ordinal, severity, path, line, side, title, body) VALUES ('pinned', 2, 2, 2, 'high', 'src/a.rs', 4, 'RIGHT', 'Pinned', 'anchored body'); INSERT INTO publications (id, target_id, draft_id, run_id, state, event, confirmed_sha, snapshot, created_at, updated_at) VALUES ('0123456789abcdef0123456789abcdef', 2, 2, 2, 'submitted', 'COMMENT', '{HEAD}', '{{}}', 1, 1);"
            ))
        })
        .await
        .unwrap();
}

fn rows_of(backend: &TestBackend) -> Vec<String> {
    text_of(backend)
        .lines()
        .map(|line| line.trim_matches('"').trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

pub(super) fn marked(rows: &[String], part: &str) -> bool {
    rows.iter()
        .any(|line| line.starts_with('>') && line.contains(part))
}

pub(super) async fn frame(store: &Store, keys: &[KeyCode], height: u16) -> (String, Vec<String>) {
    let events: Vec<_> = keys.iter().copied().map(key).collect();
    let drawn = run_scripted(store, &events, TestBackend::new(160, height))
        .await
        .unwrap();
    let text = text_of(&drawn);
    let rows = rows_of(&drawn);
    (text, rows)
}

#[tokio::test]
async fn enter_opens_the_draft_and_esc_restores_the_queue() {
    let (_tmp, store) = open().await;
    seed(&store, 1).await;
    seed_draft(&store).await;
    let (text, rows) = frame(&store, &[KeyCode::Enter], 12).await;
    let anchored = text.find("src/a.rs:4").unwrap();
    let loose = text.find("(no file)").unwrap();
    assert!(anchored < loose, "{text}");
    assert!(text.contains("anchored body"), "{text}");
    assert!(text.contains("Limitation: missing CI"), "{text}");
    assert!(
        text.contains("note one") && text.contains("note two"),
        "{text}"
    );
    assert!(marked(&rows, "src/a.rs:4"), "{rows:?}");
    let notes = rows.iter().filter(|line| line.contains("note "));
    assert!(
        notes.into_iter().all(|line| !line.contains("undecided")),
        "{rows:?}"
    );
    assert!(!marked(&rows, "Limitation"), "{rows:?}");

    let back_keys = [KeyCode::Down, KeyCode::Enter, KeyCode::Esc];
    let (back, back_rows) = frame(&store, &back_keys, 8).await;
    assert!(
        back_rows[0].contains("o/r#2") && !back_rows[0].starts_with('>'),
        "{back_rows:?}"
    );
    assert!(
        back_rows[1].starts_with('>') && back_rows[1].contains("o/r#1"),
        "{back_rows:?}"
    );
    assert!(
        !back.contains("anchored body") && !back.contains("loose body"),
        "{back}"
    );
    let published = back_rows
        .iter()
        .any(|line| line.contains("untracked") && line.contains("submitted"));
    assert!(published, "{back_rows:?}");
}

#[tokio::test]
async fn the_selected_explanation_stays_on_a_short_screen() {
    let (_tmp, store) = open().await;
    let now = crate::store::now_ms();
    let mut sql = format!(
        "INSERT INTO targets (id, owner, repo, number, tracking, created_at, head_sha, head_checked_at) VALUES (1, 'o', 'r', 1, 'tracked', 1, '{HEAD}', {now});
         INSERT INTO jobs (id, kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at) VALUES (1, 'review', 1, 'succeeded', 1, 0, 0, 0);
         INSERT INTO runs (id, target_id, job_id, state, result, head_sha, merge_base_sha, base_tip_sha, base_ref, created_at) VALUES (1, 1, 1, 'succeeded', 'findings', '{HEAD}', 'base', 'base', 'main', 0);
         INSERT INTO drafts (id, run_id, original_output, created_at) VALUES (1, 1, '{{}}', 0);"
    );
    for ordinal in 1..=6 {
        let body = if ordinal == 6 {
            "VISIBLE-BODY"
        } else {
            "short"
        };
        let _ = write!(
            sql,
            "INSERT INTO findings (id, run_id, draft_id, ordinal, severity, path, line, side, title, body) VALUES ('f{ordinal}', 1, 1, {ordinal}, 'high', 'src/a.rs', {ordinal}, 'RIGHT', 'Title{ordinal}', '{body}');"
        );
    }
    store
        .call(move |conn| conn.execute_batch(&sql))
        .await
        .unwrap();
    let keys = [
        KeyCode::Enter,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
    ];
    let events: Vec<_> = keys.into_iter().map(key).collect();
    let drawn = run_scripted(&store, &events, TestBackend::new(80, 4))
        .await
        .unwrap();
    let text = text_of(&drawn);
    assert!(text.contains("VISIBLE-BODY"), "{text}");
    assert!(text.contains("Title6"), "{text}");
    assert!(!text.contains("Title1"), "{text}");
}
