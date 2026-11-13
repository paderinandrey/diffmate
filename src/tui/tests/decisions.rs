use super::draft::frame;
use super::queue::{HEAD, open};
use crate::store::Store;
use ratatui::crossterm::event::KeyCode;

async fn cell(store: &Store, sql: &str) -> String {
    let sql = sql.to_string();
    store
        .call(move |conn| conn.query_row(&sql, [], |row| row.get(0)))
        .await
        .unwrap()
}

async fn decisions(store: &Store) -> String {
    cell(
        store,
        "SELECT a.decision || b.decision || c.decision || p.decision
         FROM findings a, findings b, findings c, findings p
         WHERE a.id = 'a' AND b.id = 'b' AND c.id = 'c' AND p.id = 'prev'",
    )
    .await
}

async fn seed(store: &Store) {
    let now = crate::store::now_ms();
    let sql = format!(
        "INSERT INTO targets (id, owner, repo, number, tracking, created_at, head_sha, head_checked_at) VALUES (1, 'o', 'r', 1, 'tracked', 1, '{HEAD}', {now});
         INSERT INTO jobs (id, kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at) VALUES (1, 'review', 1, 'succeeded', 1, 0, 0, 0);
         INSERT INTO runs (id, target_id, job_id, state, result, head_sha, merge_base_sha, base_tip_sha, base_ref, created_at) VALUES (1, 1, 1, 'succeeded', 'findings', '{HEAD}', 'base', 'base', 'main', 0), (2, 1, 1, 'succeeded', 'findings', '{HEAD}', 'base', 'base', 'main', 0);
         INSERT INTO drafts (id, run_id, original_output, created_at) VALUES (1, 1, 'OLD', 0), (2, 2, 'ORIGINAL', 0);
         INSERT INTO findings (id, run_id, draft_id, ordinal, severity, path, line, side, title, body, decision) VALUES ('prev', 1, 1, 1, 'low', 'src/a.rs', 4, 'RIGHT', 'Old', 'old', 'dropped');
         INSERT INTO findings (id, run_id, draft_id, ordinal, severity, title, body, decision) VALUES ('c', 2, 2, 3, 'low', 'Three', 'body c', 'dropped');
         INSERT INTO findings (id, run_id, draft_id, ordinal, severity, path, line, side, title, body, previous_finding_id) VALUES ('a', 2, 2, 1, 'high', 'src/a.rs', 4, 'RIGHT', 'One', 'body a', 'prev');
         INSERT INTO findings (id, run_id, draft_id, ordinal, severity, path, line, side, title, body) VALUES ('b', 2, 2, 2, 'high', 'src/a.rs', 4, 'RIGHT', 'Two', 'body b');"
    );
    store
        .call(move |conn| conn.execute_batch(&sql))
        .await
        .unwrap();
}

#[tokio::test]
async fn k_keeps_one_finding_and_shift_k_keeps_only_the_rest() {
    let (_tmp, store) = open().await;
    seed(&store).await;
    let (text, rows) = frame(&store, &[KeyCode::Enter, KeyCode::Char('k')], 16).await;
    let anchors: Vec<_> = rows
        .iter()
        .filter(|line| line.contains("src/a.rs:4"))
        .collect();
    assert_eq!(anchors.len(), 2, "{rows:?}");
    assert!(
        anchors[0].contains("kept") && anchors[0].contains("hint: dropped"),
        "{anchors:?}"
    );
    assert!(
        anchors[1].contains("undecided") && !anchors[1].contains("hint:"),
        "{anchors:?}"
    );
    let (one, two, three) = (
        text.find("One").unwrap(),
        text.find("Two").unwrap(),
        text.find("Three").unwrap(),
    );
    assert!(one < two && two < three, "{text}");
    assert_eq!(decisions(&store).await, "keptundecideddroppeddropped");
    assert_eq!(
        cell(&store, "SELECT original_output FROM drafts WHERE id = 2").await,
        "ORIGINAL"
    );
    let (_, kept) = frame(&store, &[KeyCode::Enter, KeyCode::Char('K')], 16).await;
    let still_two = kept.iter().filter(|line| line.contains("src/a.rs:4"));
    assert!(
        still_two.into_iter().all(|line| line.contains("kept")),
        "{kept:?}"
    );
    let third = kept
        .iter()
        .any(|line| line.contains("Three") && line.contains("dropped"));
    assert!(third, "{kept:?}");
    assert_eq!(decisions(&store).await, "keptkeptdroppeddropped");
    assert_eq!(
        cell(&store, "SELECT original_output FROM drafts WHERE id = 2").await,
        "ORIGINAL"
    );
}

#[tokio::test]
async fn d_drops_only_the_selected_finding() {
    let (_tmp, store) = open().await;
    seed(&store).await;
    frame(&store, &[KeyCode::Enter, KeyCode::Char('d')], 16).await;
    assert_eq!(decisions(&store).await, "droppedundecideddroppeddropped");
}
